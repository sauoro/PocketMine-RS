// src/raknet/generic/receive_reliability_layer.rs
#![allow(dead_code)]

use crate::log::Logger; // Use the project's Logger trait
use crate::raknet::generic::disconnect_reason::DisconnectReason;
use crate::raknet::generic::error::PacketHandlingError;
use crate::raknet::protocol::acknowledge_packet::AcknowledgePacket;
use crate::raknet::protocol::{ack::Ack, datagram::Datagram, encapsulated_packet::EncapsulatedPacket, nack::Nack, packet_reliability::PacketReliability};
use std::collections::{HashMap, VecDeque};

const WINDOW_SIZE: u32 = 2048;
const MAX_CONCURRENT_SPLIT_PACKETS: usize = 4; // Default from PHP
const DEFAULT_MAX_SPLIT_PACKET_PART_COUNT: u32 = 128; // Default from PHP

pub struct ReceiveReliabilityLayer {
    logger: Box<dyn Logger>, // Use dependency injection or global logger
    on_recv: Box<dyn Fn(EncapsulatedPacket) + Send + Sync>,
    send_packet: Box<dyn Fn(Box<dyn AcknowledgePacket>) + Send + Sync>, // Send ACK/NACK
    max_split_packet_part_count: u32,
    max_concurrent_split_packets: usize,

    window_start: u32,
    window_end: u32,
    highest_seq_number: i64, // Use i64 to easily detect first packet (-1)

    ack_queue: HashMap<u32, u32>, // seq -> seq
    nack_queue: HashMap<u32, u32>, // seq -> seq

    reliable_window_start: u32,
    reliable_window_end: u32,
    reliable_window: HashMap<u32, bool>, // message_index -> received

    receive_ordered_index: Vec<u32>, // order_channel -> index
    receive_sequenced_highest_index: Vec<u32>, // order_channel -> index
    receive_ordered_packets: Vec<HashMap<u32, EncapsulatedPacket>>, // order_channel -> order_index -> packet

    split_packets: HashMap<u16, Vec<Option<EncapsulatedPacket>>>, // split_id -> part_index -> packet
}

impl ReceiveReliabilityLayer {
    pub fn new(
        logger: Box<dyn Logger>,
        on_recv: Box<dyn Fn(EncapsulatedPacket) + Send + Sync>,
        send_packet: Box<dyn Fn(Box<dyn AcknowledgePacket>) + Send + Sync>,
        max_split_packet_part_count: Option<u32>,
        max_concurrent_split_packets: Option<usize>,
    ) -> Self {
        let max_split_count = max_split_packet_part_count.unwrap_or(DEFAULT_MAX_SPLIT_PACKET_PART_COUNT);
        let max_concurrent_splits = max_concurrent_split_packets.unwrap_or(MAX_CONCURRENT_SPLIT_PACKETS);

        Self {
            logger,
            on_recv,
            send_packet,
            max_split_packet_part_count: max_split_count,
            max_concurrent_split_packets: max_concurrent_splits,
            window_start: 0,
            window_end: WINDOW_SIZE,
            highest_seq_number: -1,
            ack_queue: HashMap::new(),
            nack_queue: HashMap::new(),
            reliable_window_start: 0,
            reliable_window_end: WINDOW_SIZE,
            reliable_window: HashMap::new(),
            receive_ordered_index: vec![0; PacketReliability::MAX_ORDER_CHANNELS],
            receive_sequenced_highest_index: vec![0; PacketReliability::MAX_ORDER_CHANNELS],
            receive_ordered_packets: vec![HashMap::new(); PacketReliability::MAX_ORDER_CHANNELS],
            split_packets: HashMap::new(),
        }
    }

    fn handle_encapsulated_packet_route(&self, pk: EncapsulatedPacket) {
        (self.on_recv)(pk);
    }

    fn handle_split(&mut self, packet: EncapsulatedPacket) -> Result<Option<EncapsulatedPacket>, PacketHandlingError> {
        let split_info = match packet.split_info {
            Some(ref info) => info,
            None => return Ok(Some(packet)), // Not a split packet
        };

        let total_parts = split_info.get_total_part_count();
        let part_index = split_info.get_part_index();

        if total_parts >= self.max_split_packet_part_count || total_parts == 0 {
            return Err(PacketHandlingError::new(
                format!("Invalid split packet part count ({})", total_parts),
                DisconnectReason::SplitPacketTooLarge,
            ));
        }
        if part_index >= total_parts {
            return Err(PacketHandlingError::new(
                format!("Invalid split packet part index (part index {}, part count {})", part_index, total_parts),
                DisconnectReason::SplitPacketInvalidPartIndex,
            ));
        }

        let split_id = split_info.get_id();

        let entry = self.split_packets.entry(split_id).or_insert_with(|| {
            // Check concurrent limit before inserting new entry
            if self.split_packets.len() >= self.max_concurrent_split_packets {
                // Need to return an error here, but entry API makes it awkward.
                // We'll check again after insertion attempt.
                return vec![]; // Placeholder, error check follows
            }
            vec![None; total_parts as usize]
        });

        if entry.is_empty() && self.split_packets.len() > self.max_concurrent_split_packets {
            // Check if insertion failed due to limit
            self.split_packets.remove(&split_id); // Clean up placeholder
            return Err(PacketHandlingError::new(
                format!("Exceeded concurrent split packet reassembly limit of {}", self.max_concurrent_split_packets),
                DisconnectReason::SplitPacketTooManyConcurrent,
            ));
        }


        if entry.len() != total_parts as usize {
            return Err(PacketHandlingError::new(
                format!("Wrong split count {} for split packet {}, expected {}", total_parts, split_id, entry.len()),
                DisconnectReason::SplitPacketInconsistentHeader,
            ));
        }

        if entry[part_index as usize].is_some() {
            // Duplicate part, ignore
            return Ok(None);
        }
        entry[part_index as usize] = Some(packet.clone()); // Clone needed parts

        // Check if all parts are received
        if entry.iter().all(|p| p.is_some()) {
            let mut reassembled_buffer = Vec::with_capacity(entry.iter().map(|p| p.as_ref().unwrap().buffer.len()).sum());
            for i in 0..total_parts as usize {
                reassembled_buffer.extend_from_slice(&entry[i].as_ref().unwrap().buffer);
            }

            let mut pk = EncapsulatedPacket::new();
            pk.buffer = reassembled_buffer.into(); // Convert Vec<u8> to Bytes
            pk.reliability = packet.reliability;
            pk.message_index = packet.message_index;
            pk.sequence_index = packet.sequence_index;
            pk.order_index = packet.order_index;
            pk.order_channel = packet.order_channel;

            self.split_packets.remove(&split_id);
            Ok(Some(pk))
        } else {
            Ok(None) // Still waiting for parts
        }
    }

    fn handle_encapsulated_packet(&mut self, packet: EncapsulatedPacket) -> Result<(), PacketHandlingError> {
        if let Some(message_index) = packet.message_index {
            if message_index < self.reliable_window_start || message_index >= self.reliable_window_end || self.reliable_window.contains_key(&message_index) {
                // Duplicate or out of range reliable packet
                return Ok(());
            }

            self.reliable_window.insert(message_index, true);

            if message_index == self.reliable_window_start {
                while self.reliable_window.remove(&self.reliable_window_start).is_some() {
                    self.reliable_window_start = self.reliable_window_start.wrapping_add(1);
                    self.reliable_window_end = self.reliable_window_end.wrapping_add(1);
                }
            }
        }

        let reassembled_packet = match self.handle_split(packet)? {
            Some(pk) => pk,
            None => return Ok(()), // Waiting for more split parts
        };

        let order_channel = match reassembled_packet.order_channel {
            Some(ch) => {
                if ch as usize >= PacketReliability::MAX_ORDER_CHANNELS {
                    self.logger.debug(&format!("Invalid packet, bad order channel ({})", ch));
                    return Ok(());
                }
                ch as usize
            }
            None => 0, // Default channel if not ordered/sequenced? Or error? Let's assume 0 for now.
        };

        if PacketReliability::is_sequenced(reassembled_packet.reliability) {
            let sequence_index = reassembled_packet.sequence_index.unwrap_or(0); // Should always exist
            let order_index = reassembled_packet.order_index.unwrap_or(0);

            if sequence_index < self.receive_sequenced_highest_index[order_channel] || order_index < self.receive_ordered_index[order_channel] {
                // Too old sequenced packet, discard it
                return Ok(());
            }

            self.receive_sequenced_highest_index[order_channel] = sequence_index.wrapping_add(1);
            self.handle_encapsulated_packet_route(reassembled_packet);

        } else if PacketReliability::is_ordered(reassembled_packet.reliability) {
            let order_index = reassembled_packet.order_index.unwrap_or(0); // Should always exist

            if order_index == self.receive_ordered_index[order_channel] {
                self.receive_sequenced_highest_index[order_channel] = 0; // Reset sequence index for this channel
                self.receive_ordered_index[order_channel] = order_index.wrapping_add(1);

                self.handle_encapsulated_packet_route(reassembled_packet);

                // Process buffered ordered packets for this channel
                while let Some(buffered_packet) = self.receive_ordered_packets[order_channel].remove(&self.receive_ordered_index[order_channel]) {
                    self.handle_encapsulated_packet_route(buffered_packet);
                    self.receive_ordered_index[order_channel] = self.receive_ordered_index[order_channel].wrapping_add(1);
                }

            } else if order_index > self.receive_ordered_index[order_channel] {
                // Future ordered packet, buffer it
                if self.receive_ordered_packets[order_channel].len() >= WINDOW_SIZE as usize {
                    // Queue overflow, potential issue
                    self.logger.warning(&format!("Ordered packet queue overflow for channel {}", order_channel));
                    return Ok(());
                }
                self.receive_ordered_packets[order_channel].insert(order_index, reassembled_packet);
            } // Else: duplicate/already received ordered packet, ignore

        } else {
            // Unreliable, no ordering or sequencing
            self.handle_encapsulated_packet_route(reassembled_packet);
        }
        Ok(())
    }


    pub fn on_datagram(&mut self, packet: Datagram) -> Result<(), PacketHandlingError> {
        let seq_number = packet.seq_number;

        if seq_number < self.window_start || seq_number >= self.window_end || self.ack_queue.contains_key(&seq_number) {
            self.logger.debug(&format!(
                "Received duplicate or out-of-window packet (sequence number {}, window {}-{})",
                seq_number, self.window_start, self.window_end
            ));
            return Ok(());
        }

        self.nack_queue.remove(&seq_number);
        self.ack_queue.insert(seq_number, seq_number);

        if seq_number as i64 > self.highest_seq_number {
            self.highest_seq_number = seq_number as i64;
        }

        if seq_number == self.window_start {
            while self.ack_queue.remove(&self.window_start).is_some() {
                self.window_start = self.window_start.wrapping_add(1);
                self.window_end = self.window_end.wrapping_add(1);
            }
        } else if seq_number > self.window_start {
            // We got a gap
            for i in self.window_start..seq_number {
                if !self.ack_queue.contains_key(&i) {
                    self.nack_queue.insert(i, i);
                }
            }
        }
        // else: seq_number < self.window_start (already handled by initial check)

        for pk in packet.packets {
            self.handle_encapsulated_packet(pk)?;
        }
        Ok(())
    }

    pub fn update(&mut self) {
        // Check for gaps based on highest received vs window start
        let diff = (self.highest_seq_number + 1) as u32.wrapping_sub(self.window_start);
        if diff > 0 && self.highest_seq_number >= 0 {
            for i in self.window_start..self.highest_seq_number as u32 + 1 {
                if !self.ack_queue.contains_key(&i) && !self.nack_queue.contains_key(&i) {
                    self.nack_queue.insert(i, i);
                }
            }
            self.window_start = (self.highest_seq_number + 1) as u32;
            self.window_end = self.window_start.wrapping_add(WINDOW_SIZE);
        }

        if !self.ack_queue.is_empty() {
            let mut pk = Ack::new();
            pk.packets = self.ack_queue.keys().cloned().collect();
            (self.send_packet)(Box::new(pk));
            self.ack_queue.clear();
        }

        if !self.nack_queue.is_empty() {
            let mut pk = Nack::new();
            pk.packets = self.nack_queue.keys().cloned().collect();
            (self.send_packet)(Box::new(pk));
            self.nack_queue.clear();
        }
    }

    pub fn needs_update(&self) -> bool {
        !self.ack_queue.is_empty() || !self.nack_queue.is_empty()
    }
}
