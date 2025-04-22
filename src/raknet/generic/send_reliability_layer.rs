// src/raknet/generic/send_reliability_layer.rs
#![allow(dead_code)]

use crate::raknet::generic::reliable_cache_entry::ReliableCacheEntry;
use crate::raknet::protocol::ack::Ack;
use crate::raknet::protocol::datagram::Datagram;
use crate::raknet::protocol::encapsulated_packet::{EncapsulatedPacket, SplitPacketInfo};
use crate::raknet::protocol::nack::Nack;
use crate::raknet::protocol::packet_reliability::PacketReliability;
use crate::raknet::generic::session::Session; // For MIN_MTU_SIZE
use std::collections::{HashMap, VecDeque, BTreeMap}; // Use BTreeMap for reliableCache to iterate timely
use std::time::{Duration, Instant};

const DATAGRAM_MTU_OVERHEAD: usize = 36 + Datagram::HEADER_SIZE;
const MIN_POSSIBLE_PACKET_SIZE_LIMIT: usize = Session::MIN_MTU_SIZE as usize - DATAGRAM_MTU_OVERHEAD;
const UNACKED_RETRANSMIT_DELAY: Duration = Duration::from_secs(2);
const DEFAULT_RELIABLE_WINDOW_SIZE: u32 = 512;

pub struct SendReliabilityLayer {
    mtu_size: u16,
    send_datagram_callback: Box<dyn Fn(Datagram) + Send + Sync>,
    on_ack: Box<dyn Fn(u32) + Send + Sync>, // identifier_ack
    reliable_window_size: u32,

    send_queue: Vec<EncapsulatedPacket>,
    split_id: u16,
    send_seq_number: u32,
    message_index: u32,

    reliable_window_start: u32,
    reliable_window_end: u32,
    reliable_window: HashMap<u32, bool>, // message_index -> acked (true if acked)

    send_ordered_index: Vec<u32>, // order_channel -> index
    send_sequenced_index: Vec<u32>, // order_channel -> index

    reliable_backlog: BTreeMap<u32, EncapsulatedPacket>, // message_index -> packet (Sorted for processing)
    resend_queue: VecDeque<EncapsulatedPacket>,

    reliable_cache: BTreeMap<u32, ReliableCacheEntry>, // seq_number -> entry (Sorted for timely check)

    need_ack: HashMap<u32, HashMap<u32, u32>>, // identifier_ack -> { message_index -> message_index }

    max_datagram_payload_size: usize,
}

impl SendReliabilityLayer {
    pub fn new(
        mtu_size: u16,
        send_datagram_callback: Box<dyn Fn(Datagram) + Send + Sync>,
        on_ack: Box<dyn Fn(u32) + Send + Sync>,
        reliable_window_size: Option<u32>,
    ) -> Self {
        let window_size = reliable_window_size.unwrap_or(DEFAULT_RELIABLE_WINDOW_SIZE);
        let max_payload = (mtu_size as usize).saturating_sub(DATAGRAM_MTU_OVERHEAD);
        if max_payload < MIN_POSSIBLE_PACKET_SIZE_LIMIT {
            panic!("MTU size {} is too small, minimum possible payload size is {}", mtu_size, MIN_POSSIBLE_PACKET_SIZE_LIMIT);
        }

        Self {
            mtu_size,
            send_datagram_callback,
            on_ack,
            reliable_window_size: window_size,
            send_queue: Vec::new(),
            split_id: 0,
            send_seq_number: 0,
            message_index: 0,
            reliable_window_start: 0,
            reliable_window_end: window_size,
            reliable_window: HashMap::new(),
            send_ordered_index: vec![0; PacketReliability::MAX_ORDER_CHANNELS],
            send_sequenced_index: vec![0; PacketReliability::MAX_ORDER_CHANNELS],
            reliable_backlog: BTreeMap::new(),
            resend_queue: VecDeque::new(),
            reliable_cache: BTreeMap::new(),
            need_ack: HashMap::new(),
            max_datagram_payload_size: max_payload,
        }
    }

    fn send_datagram(&mut self, packets: Vec<EncapsulatedPacket>) {
        if packets.is_empty() {
            return;
        }
        let mut datagram = Datagram::new();
        datagram.seq_number = self.send_seq_number;
        self.send_seq_number = self.send_seq_number.wrapping_add(1);
        datagram.packets = packets;

        let mut resendable = Vec::new();
        for pk in &datagram.packets {
            if PacketReliability::is_reliable(pk.reliability) {
                resendable.push(pk.clone()); // Clone needed for cache
            }
        }
        if !resendable.is_empty() {
            self.reliable_cache.insert(datagram.seq_number, ReliableCacheEntry::new(resendable));
        }

        (self.send_datagram_callback)(datagram);
    }

    fn send_queued(&mut self) {
        if !self.send_queue.is_empty() {
            // Take ownership of the queue to avoid borrow checker issues
            let queue = std::mem::take(&mut self.send_queue);
            self.send_datagram(queue);
        }
    }

    fn add_to_queue(&mut self, mut pk: EncapsulatedPacket, immediate: bool) {
        if PacketReliability::is_reliable(pk.reliability) {
            let msg_idx = pk.message_index.expect("Reliable packet must have message index");
            if msg_idx < self.reliable_window_start {
                // This shouldn't happen if logic is correct, but log if it does
                eprintln!("Attempted to send reliable packet {} which is outside window start {}", msg_idx, self.reliable_window_start);
                return;
            }
            if msg_idx >= self.reliable_window_end {
                // Packet is outside the current reliable window, backlog it
                self.reliable_backlog.insert(msg_idx, pk);
                return;
            }
            self.reliable_window.insert(msg_idx, false); // Mark as sent but not acked
        }

        if let Some(ack_id) = pk.identifier_ack {
            if pk.message_index.is_some() { // Only track ACK for reliable packets
                self.need_ack.entry(ack_id).or_default().insert(pk.message_index.unwrap(), pk.message_index.unwrap());
            }
        }

        let mut current_queue_len = 0;
        for queued in &self.send_queue {
            current_queue_len += queued.get_total_length();
        }

        if current_queue_len + pk.get_total_length() > self.max_datagram_payload_size && !self.send_queue.is_empty() {
            self.send_queued();
        }

        // Add the packet (clone if ACK needed, otherwise move)
        if pk.identifier_ack.is_some() {
            let ack_packet = pk.clone(); // Clone the packet to keep ack info
            self.send_queue.push(ack_packet);
            pk.identifier_ack = None; // Remove ack from the original if it was moved from backlog/resend
        } else {
            self.send_queue.push(pk);
        }


        if immediate {
            self.send_queued();
        }
    }

    pub fn add_encapsulated_to_queue(&mut self, mut packet: EncapsulatedPacket, immediate: bool) {
        if let Some(ack_id) = packet.identifier_ack {
            self.need_ack.entry(ack_id).or_default(); // Ensure entry exists
        }

        let order_channel = packet.order_channel.unwrap_or(0) as usize;
        if order_channel >= PacketReliability::MAX_ORDER_CHANNELS {
            eprintln!("Invalid order channel {}", order_channel);
            return; // Or handle error appropriately
        }

        if PacketReliability::is_ordered(packet.reliability) {
            packet.order_index = Some(self.send_ordered_index[order_channel]);
            self.send_ordered_index[order_channel] = self.send_ordered_index[order_channel].wrapping_add(1);
        } else if PacketReliability::is_sequenced(packet.reliability) {
            packet.order_index = Some(self.send_ordered_index[order_channel]); // Use current ordered index
            packet.sequence_index = Some(self.send_sequenced_index[order_channel]);
            self.send_sequenced_index[order_channel] = self.send_sequenced_index[order_channel].wrapping_add(1);
        }

        // Max size for payload within the encapsulated packet itself
        let max_buffer_size = self.max_datagram_payload_size.saturating_sub(packet.get_header_length());

        if packet.buffer.len() > max_buffer_size {
            // Need to split
            let buffers = packet.buffer.chunks(max_buffer_size.saturating_sub(EncapsulatedPacket::SPLIT_INFO_LENGTH));
            let buffer_count = buffers.len() as u32; // Cast might truncate if > u32::MAX chunks

            if buffer_count == 0 { return; } // Should not happen if len > max_buffer_size

            self.split_id = self.split_id.wrapping_add(1);
            let current_split_id = self.split_id;

            for (count, buffer_chunk) in buffers.enumerate() {
                let mut pk = EncapsulatedPacket::new();
                pk.split_info = Some(SplitPacketInfo::new(current_split_id, count as u32, buffer_count));
                pk.reliability = packet.reliability;
                pk.buffer = bytes::Bytes::copy_from_slice(buffer_chunk);
                pk.identifier_ack = packet.identifier_ack; // Propagate ACK request

                if PacketReliability::is_reliable(pk.reliability) {
                    pk.message_index = Some(self.message_index);
                    self.message_index = self.message_index.wrapping_add(1);
                }

                pk.sequence_index = packet.sequence_index; // Keep original sequence index for split parts
                pk.order_channel = packet.order_channel;
                pk.order_index = packet.order_index; // Keep original order index for split parts

                // Send split parts immediately if original was immediate? RakLib PHP uses true here.
                // This ensures parts aren't delayed relative to each other.
                self.add_to_queue(pk, true);
            }
        } else {
            // No split needed
            if PacketReliability::is_reliable(packet.reliability) {
                packet.message_index = Some(self.message_index);
                self.message_index = self.message_index.wrapping_add(1);
            }
            self.add_to_queue(packet, immediate);
        }
    }

    fn update_reliable_window(&mut self) {
        while self.reliable_window.get(&self.reliable_window_start) == Some(&true) {
            self.reliable_window.remove(&self.reliable_window_start);
            self.reliable_window_start = self.reliable_window_start.wrapping_add(1);
            self.reliable_window_end = self.reliable_window_end.wrapping_add(1);
        }
    }

    pub fn on_ack(&mut self, packet: Ack) {
        for seq in packet.packets {
            if let Some(entry) = self.reliable_cache.remove(&seq) {
                for pk in entry.get_packets() {
                    if let Some(msg_idx) = pk.message_index {
                        if msg_idx >= self.reliable_window_start && msg_idx < self.reliable_window_end {
                            if self.reliable_window.insert(msg_idx, true).is_some() {
                                self.update_reliable_window(); // Update window after marking as acked

                                if let Some(ack_id) = pk.identifier_ack {
                                    if let Some(ack_map) = self.need_ack.get_mut(&ack_id) {
                                        ack_map.remove(&msg_idx);
                                        if ack_map.is_empty() {
                                            self.need_ack.remove(&ack_id);
                                            (self.on_ack)(ack_id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn on_nack(&mut self, packet: Nack) {
        for seq in packet.packets {
            if let Some(entry) = self.reliable_cache.remove(&seq) {
                for pk in entry.get_packets() {
                    // Re-queue immediately if NACK received
                    self.resend_queue.push_back(pk.clone());
                }
            }
        }
    }

    pub fn needs_update(&self) -> bool {
        !self.send_queue.is_empty()
            || !self.reliable_backlog.is_empty()
            || !self.resend_queue.is_empty()
            || !self.reliable_cache.is_empty() // Need to check cache for timeouts
    }

    pub fn update(&mut self, current_time: Instant) {
        let retransmit_older_than = current_time.checked_sub(UNACKED_RETRANSMIT_DELAY);

        if let Some(threshold) = retransmit_older_than {
            let mut timed_out_seqs = Vec::new();
            for (seq, entry) in self.reliable_cache.iter() {
                if entry.get_timestamp() < threshold {
                    timed_out_seqs.push(*seq);
                } else {
                    break; // BTreeMap is sorted by seq, assume timestamps are roughly ordered too
                }
            }

            for seq in timed_out_seqs {
                if let Some(entry) = self.reliable_cache.remove(&seq) {
                    for pk in entry.get_packets() {
                        self.resend_queue.push_back(pk.clone());
                    }
                }
            }
        }


        // Process resend queue first
        while let Some(pk) = self.resend_queue.pop_front() {
            self.add_to_queue(pk, false);
        }

        // Process reliable backlog if window allows
        let backlog_keys: Vec<u32> = self.reliable_backlog.keys().cloned().collect();
        for k in backlog_keys {
            if k >= self.reliable_window_end {
                // Still outside window, stop processing backlog for now
                break;
            }
            if let Some(pk) = self.reliable_backlog.remove(&k) {
                self.add_to_queue(pk, false);
            }
        }

        // Send anything remaining in the immediate queue
        self.send_queued();
    }
}