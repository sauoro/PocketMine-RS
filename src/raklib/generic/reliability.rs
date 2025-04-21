// src/raklib/generic/reliability.rs

use crate::raklib::protocol::{ack::*, frame::*, consts::*, Packet}; // Import necessary protocol items
use crate::raklib::error::{DisconnectReason, RakLibError, Result}; // Use our RakLib error types
use crate::utils::binary::BinaryStream; // Keep for potential future use, though layers might not serialize directly
use std::collections::{HashMap, VecDeque, BTreeMap, HashSet}; // Use BTreeMap for ordered reliable cache, HashSet for ACK queues
use std::time::{Duration, Instant}; // For timing retransmits
use tracing::{debug, trace, warn}; // Use tracing for logging

// --- ReliableCacheEntry ---
#[derive(Debug, Clone)]
pub struct ReliableCacheEntry {
    /// The original packets sent in the datagram containing reliable payloads.
    /// Cloned for potential resend.
    packets: Vec<EncapsulatedPacket>,
    timestamp: Instant,
}

impl ReliableCacheEntry {
    pub fn new(packets: Vec<EncapsulatedPacket>) -> Self {
        Self { packets, timestamp: Instant::now() }
    }
    pub fn packets(&self) -> &[EncapsulatedPacket] { &self.packets }
    pub fn timestamp(&self) -> Instant { self.timestamp }
}

// --- SendReliabilityLayer ---
#[derive(Debug)]
pub struct SendReliabilityLayer {
    mtu_size: u16,
    max_datagram_payload_size: usize,
    // Callback to send a finalized Datagram over the network
    send_datagram_callback: Box<dyn Fn(Datagram) + Send + Sync>,
    // Callback when a packet sent with identifier_ack is fully ACKed
    on_ack_callback: Box<dyn Fn(u32) + Send + Sync>,

    /// Packets waiting to be bundled into the next Datagram.
    send_queue: Vec<EncapsulatedPacket>,
    /// Reliable packets that need to be resent due to NACK or timeout.
    resend_queue: VecDeque<EncapsulatedPacket>,

    split_id: u16, // Counter for split packet IDs
    send_seq_number: u32, // Sequence number for outgoing Datagrams (u24)
    message_index: u32, // Counter for reliable EncapsulatedPackets (u24)

    reliable_window_start: u32, // The oldest unacknowledged message_index (u24)
    reliable_window_end: u32, // reliable_window_start + reliable_window_size (u24)
    reliable_window_size: u32,
    /// Tracks which message indices within the window have been ACKed.
    /// Map: message_index (u24) -> acked (bool)
    reliable_window: HashMap<u32, bool>, // Optimization: Could use a BitVec or similar
    /// Reliable packets waiting because their message_index is >= reliable_window_end.
    /// Ordered by message_index.
    reliable_backlog: BTreeMap<u32, EncapsulatedPacket>,

    // Per-channel counters for ordered/sequenced packets
    send_ordered_index: [u32; PacketReliability::MAX_ORDER_CHANNELS], // u24
    send_sequenced_index: [u32; PacketReliability::MAX_ORDER_CHANNELS], // u24

    /// Cache of sent Datagrams containing reliable packets, keyed by seq_number (u24).
    /// Used for resending on NACK or timeout. Ordered for efficient timeout checks.
    reliable_cache: BTreeMap<u32, ReliableCacheEntry>,

    /// Tracks which message indices are needed to fulfill an ACK request.
    /// Map: identifier_ack (u32) -> Set of message_indices (u24)
    need_ack: HashMap<u32, HashSet<u32>>,

    unacked_retransmit_delay: Duration,
}

impl SendReliabilityLayer {
    // Constants moved inside impl block
    const DATAGRAM_MTU_OVERHEAD: usize = 36 + Datagram::HEADER_SIZE; // IP(20)+UDP(8)+RakNet(8?) + DatagramHdr(4) = 40? RakLib PHP uses 36. Verify. Let's assume 40 is safer.
    const MIN_MTU_SIZE: usize = 400; // Minimum practical MTU
    const MIN_POSSIBLE_PACKET_SIZE_LIMIT: usize = Self::MIN_MTU_SIZE - Self::DATAGRAM_MTU_OVERHEAD;

    pub fn new(
        mtu_size: u16,
        reliable_window_size: u32,
        send_datagram_callback: Box<dyn Fn(Datagram) + Send + Sync>,
        on_ack_callback: Box<dyn Fn(u32) + Send + Sync>,
    ) -> Self {
        let max_datagram_payload_size = (mtu_size as usize).saturating_sub(Self::DATAGRAM_MTU_OVERHEAD);
        if max_datagram_payload_size < Self::MIN_POSSIBLE_PACKET_SIZE_LIMIT {
            warn!(mtu_size, max_datagram_payload_size, "Calculated max datagram payload size is very small or negative!");
        }

        Self {
            mtu_size,
            max_datagram_payload_size: max_datagram_payload_size.max(Self::MIN_POSSIBLE_PACKET_SIZE_LIMIT),
            send_datagram_callback,
            on_ack_callback,
            send_queue: Vec::new(),
            resend_queue: VecDeque::new(),
            split_id: 0,
            send_seq_number: 0,
            message_index: 0,
            reliable_window_start: 0,
            // Calculate initial window end carefully with wrapping
            reliable_window_end: reliable_window_size.wrapping_sub(1) & 0xFFFFFF, // Window is [start, end], inclusive? Check RakNet. Assume [start, end) for now. Let's make end exclusive: start + size
            reliable_window_end: reliable_window_size & 0xFFFFFF, // End is exclusive: start + size
            reliable_window_size,
            reliable_window: HashMap::new(),
            reliable_backlog: BTreeMap::new(),
            send_ordered_index: [0; PacketReliability::MAX_ORDER_CHANNELS],
            send_sequenced_index: [0; PacketReliability::MAX_ORDER_CHANNELS],
            reliable_cache: BTreeMap::new(),
            need_ack: HashMap::new(),
            unacked_retransmit_delay: Duration::from_secs_f32(2.0), // TODO: Make configurable/dynamic
        }
    }

    /// Adds an EncapsulatedPacket to the send queue, assigning necessary indices and splitting if needed.
    pub fn add_encapsulated_to_queue(&mut self, mut packet: EncapsulatedPacket, immediate: bool) {
        // Track ACK requirement before potential cloning/splitting
        if let Some(ack_id) = packet.identifier_ack {
            self.need_ack.entry(ack_id).or_default(); // Ensure entry exists
        }

        // Assign order/sequence indices
        if PacketReliability::is_ordered(packet.reliability) {
            let channel = packet.order_channel.unwrap_or(0) as usize;
            if channel >= PacketReliability::MAX_ORDER_CHANNELS {
                warn!(channel, "Attempted to send packet on invalid order channel");
                return; // Drop packet with invalid channel
            }
            packet.order_index = Some(self.send_ordered_index[channel]);
            self.send_ordered_index[channel] = self.send_ordered_index[channel].wrapping_add(1) & 0xFFFFFF;
        } else if PacketReliability::is_sequenced(packet.reliability) {
            let channel = packet.order_channel.unwrap_or(0) as usize;
            if channel >= PacketReliability::MAX_ORDER_CHANNELS {
                warn!(channel, "Attempted to send packet on invalid sequence channel");
                return;
            }
            packet.order_index = Some(self.send_ordered_index[channel]); // Uses the *current* order index for sequencing relative to ordered packets
            packet.sequence_index = Some(self.send_sequenced_index[channel]);
            self.send_sequenced_index[channel] = self.send_sequenced_index[channel].wrapping_add(1) & 0xFFFFFF;
        }

        // Check if packet needs splitting
        let max_payload_size = self.max_datagram_payload_size;
        let header_len = packet.header_length(); // Header length *without* split info
        let max_buffer_size = max_payload_size.saturating_sub(header_len);

        if packet.buffer.len() > max_buffer_size {
            // --- Splitting Logic ---
            let split_header_len = header_len + SplitPacketInfo::ENCODED_LENGTH;
            let max_split_payload_size = max_payload_size.saturating_sub(split_header_len);

            if max_split_payload_size == 0 {
                warn!(packet_size = packet.buffer.len(), mtu = self.mtu_size, "Packet too large to split with current MTU settings.");
                // Drop the packet, cannot split it small enough
                return;
            }

            let buffers: Vec<&[u8]> = packet.buffer.chunks(max_split_payload_size).collect();
            let buffer_count = buffers.len() as u32;
            if buffer_count > (1 << 16) { // Check if count fits in u16 split ID? No, count is u32. Check part count limit?
                warn!(parts = buffer_count, "Packet split into too many parts, dropping.");
                return;
            }


            self.split_id = self.split_id.wrapping_add(1); // Increment and wrap split ID u16

            for (count, buffer_slice) in buffers.into_iter().enumerate() {
                let mut pk = EncapsulatedPacket::new(); // Create a new packet for the split part
                pk.split_info = Some(SplitPacketInfo {
                    id: self.split_id,
                    part_index: count as u32,
                    total_part_count: buffer_count,
                });
                pk.reliability = packet.reliability;
                pk.buffer = buffer_slice.to_vec();
                pk.identifier_ack = packet.identifier_ack; // Propagate ACK ID to all parts

                // Assign reliability/ordering info to each part
                if PacketReliability::is_reliable(pk.reliability) {
                    pk.message_index = Some(self.message_index);
                    self.message_index = self.message_index.wrapping_add(1) & 0xFFFFFF;
                }
                pk.sequence_index = packet.sequence_index; // All parts share original sequence index if set
                pk.order_channel = packet.order_channel; // All parts share original channel
                pk.order_index = packet.order_index; // All parts share original order index

                // Use internal add_to_queue logic for window checks etc.
                // Split parts are usually sent immediately to avoid delaying the whole message.
                self.add_to_send_queue_internal(pk, true);
            }
        } else {
            // --- No Splitting Needed ---
            if PacketReliability::is_reliable(packet.reliability) {
                packet.message_index = Some(self.message_index);
                self.message_index = self.message_index.wrapping_add(1) & 0xFFFFFF;
            }
            self.add_to_send_queue_internal(packet, immediate);
        }
    }

    /// Internal helper to manage reliable window, backlog, and the actual send queue.
    fn add_to_send_queue_internal(&mut self, packet: EncapsulatedPacket, immediate: bool) {
        // --- Reliable Window / Backlog Management ---
        if PacketReliability::is_reliable(packet.reliability) {
            let msg_idx = packet.message_index.expect("Reliable packet must have message index");

            // Check if index is behind the window (can happen with resends after rapid ACKs)
            let window_start = self.reliable_window_start;
            let is_behind = if self.reliable_window_end >= window_start { // No wrap-around
                msg_idx < window_start
            } else { // Wrap-around case
                msg_idx < window_start && msg_idx >= self.reliable_window_end // Between end and start is behind
            };
            if is_behind {
                debug!(msg_idx, window_start, "Attempted to queue reliable packet behind window start, dropping.");
                // We shouldn't resend packets that are already implicitly acked by window movement.
                return;
            }

            // Check if index is ahead of the window
            let window_end = self.reliable_window_end;
            let is_ahead = if window_end >= window_start { // No wrap-around
                msg_idx >= window_end
            } else { // Wrap-around case
                msg_idx >= window_end && msg_idx < window_start // Between end and start is ahead
            };
            if is_ahead {
                trace!(msg_idx, window_end, "Queuing packet to reliable backlog (ahead of window).");
                self.reliable_backlog.insert(msg_idx, packet);
                return; // Add to backlog, don't put in send_queue yet
            }

            // Packet is within the current window, mark it for tracking if not already resent/acked
            self.reliable_window.entry(msg_idx).or_insert(false); // false means not acked yet
        }

        // --- ACK Tracking ---
        if let (Some(ack_id), Some(msg_idx)) = (packet.identifier_ack, packet.message_index) {
            if PacketReliability::is_reliable(packet.reliability) {
                // Track that this message_index is needed for this ack_id
                self.need_ack.entry(ack_id).or_default().insert(msg_idx);
            }
        }

        // --- Add to Send Queue and Flush if Needed ---
        let current_queue_size: usize = self.send_queue.iter().map(|p| p.total_length()).sum();
        let packet_total_len = packet.total_length();

        // Flush *before* adding if the new packet would exceed MTU
        if !self.send_queue.is_empty() && (current_queue_size + packet_total_len > self.max_datagram_payload_size) {
            trace!(current_queue_size, packet_total_len, max_payload_size=self.max_datagram_payload_size, "Flushing send queue due to MTU limit");
            self.flush_send_queue();
        }

        // Add the packet (clone if it has ack_id for caching)
        if packet.identifier_ack.is_some() {
            let mut cached_packet = packet.clone();
            cached_packet.identifier_ack = None; // Clear ACK ID for the version added to send queue/cache
            self.send_queue.push(cached_packet);
        } else {
            self.send_queue.push(packet);
        }


        // Flush immediately if requested or if the single added packet fills the MTU
        if immediate || packet_total_len >= self.max_datagram_payload_size {
            self.flush_send_queue();
        }
    }

    /// Bundles packets from send_queue into a Datagram and sends it.
    fn flush_send_queue(&mut self) {
        if self.send_queue.is_empty() { return; }
        trace!(packet_count = self.send_queue.len(), seq = self.send_seq_number, "Flushing send queue");

        let datagram = Datagram {
            header_flags: 0, // Normal datagram
            seq_number: self.send_seq_number,
            packets: std::mem::take(&mut self.send_queue), // Move packets out of queue
        };
        self.send_seq_number = self.send_seq_number.wrapping_add(1) & 0xFFFFFF; // u24 wrap

        // Cache reliable packets from this datagram for potential resend
        let reliable_packets_in_datagram: Vec<_> = datagram.packets.iter()
            .filter(|p| PacketReliability::is_reliable(p.reliability))
            .cloned() // Clone packets for the cache entry
            .collect();

        if !reliable_packets_in_datagram.is_empty() {
            self.reliable_cache.insert(datagram.seq_number, ReliableCacheEntry::new(reliable_packets_in_datagram));
        }

        // Send the datagram via the callback
        (self.send_datagram_callback)(datagram);
    }

    /// Advances the reliable window based on contiguous ACKs at the start.
    fn update_reliable_window(&mut self) {
        let mut window_moved = false;
        // Loop while the packet at the start of the window exists and is marked ACKed
        while self.reliable_window.get(&self.reliable_window_start) == Some(&true) {
            self.reliable_window.remove(&self.reliable_window_start);
            self.reliable_window_start = self.reliable_window_start.wrapping_add(1) & 0xFFFFFF;
            self.reliable_window_end = self.reliable_window_end.wrapping_add(1) & 0xFFFFFF;
            window_moved = true;
        }
        if window_moved {
            trace!(new_start = self.reliable_window_start, new_end = self.reliable_window_end, "Reliable window moved");
        }
    }

    /// Processes an incoming ACK packet.
    pub fn on_ack(&mut self, ack_packet: &ACK) {
        let seq_numbers = &ack_packet.0.packets; // Access inner Vec<u32>
        trace!(ack_count = seq_numbers.len(), "Processing ACK");
        for &seq in seq_numbers {
            // Remove from reliable cache and process ACKs for contained packets
            if let Some(entry) = self.reliable_cache.remove(&seq) {
                trace!(seq, "Datagram ACKed");
                for pk in entry.packets() {
                    if let Some(msg_idx) = pk.message_index {
                        // Check if this message index is still relevant (within or ahead of window start)
                        let window_start = self.reliable_window_start;
                        let is_relevant = if self.reliable_window_end >= window_start { // No wrap
                            msg_idx >= window_start
                        } else { // Wrap
                            msg_idx >= window_start || msg_idx < self.reliable_window_end
                        };

                        if is_relevant {
                            // Mark as ACKed in the window tracking
                            self.reliable_window.insert(msg_idx, true);
                            trace!(msg_idx, "Reliable packet ACKed");

                            // Check if this fulfills an identifier_ack requirement
                            // Use the original packet's ACK ID if it was stored (or reconstruct)
                            // We cleared identifier_ack for the cached version, so we need the original context if possible,
                            // or rely on the fact that need_ack tracks msg_idx -> ack_id.
                            // Let's check need_ack.
                            let mut ack_id_to_notify = None;
                            for (ack_id, needed_indices) in self.need_ack.iter_mut() {
                                if needed_indices.remove(&msg_idx) {
                                    if needed_indices.is_empty() {
                                        ack_id_to_notify = Some(*ack_id); // Mark for notification outside the loop
                                        break; // Found the ack_id, no need to check others for this msg_idx
                                    }
                                }
                            }
                            if let Some(ack_id) = ack_id_to_notify {
                                self.need_ack.remove(&ack_id); // Remove the entry once fulfilled
                                trace!(ack_id, "ACK identifier fulfilled");
                                (self.on_ack_callback)(ack_id);
                            }

                        } else {
                            // Got ACK for packet already implicitly acked by window movement. Normal.
                            trace!(msg_idx, window_start = self.reliable_window_start, "Received ACK for packet behind window start (already implicitly ACKed)");
                        }
                    }
                }
            } else {
                // ACK for a sequence number not in cache (duplicate ACK or ACK for unreliable datagram?)
                trace!(seq, "Received ACK for unknown or already ACKed datagram");
            }
        }
        // After processing all ACKs, update the window start
        self.update_reliable_window();
    }

    /// Processes an incoming NACK packet.
    pub fn on_nack(&mut self, nack_packet: &NACK) {
        let seq_numbers = &nack_packet.0.packets; // Access inner Vec<u32>
        trace!(nack_count = seq_numbers.len(), "Processing NACK");
        for &seq in seq_numbers {
            if let Some(entry) = self.reliable_cache.remove(&seq) {
                debug!(seq, "Received NACK, queuing {} packets for resend", entry.packets().len());
                for pk in entry.packets().iter().cloned() { // Clone for resend queue
                    // Only resend reliable packets
                    if PacketReliability::is_reliable(pk.reliability) {
                        self.resend_queue.push_back(pk);
                    }
                }
            } else {
                // NACK for something not in cache (already resent/acked?)
                trace!(seq, "Received NACK for unknown or already handled datagram");
            }
        }
    }

    /// Returns true if there's pending work (packets to send, ACKs to wait for).
    pub fn needs_update(&self) -> bool {
        !self.send_queue.is_empty()
            || !self.reliable_backlog.is_empty()
            || !self.resend_queue.is_empty()
            || !self.reliable_cache.is_empty() // Need to check cache for timeouts
    }

    /// Performs periodic tasks like checking for timeouts and sending queued packets.
    pub fn update(&mut self) {
        let now = Instant::now();
        let retransmit_older_than = now.checked_sub(self.unacked_retransmit_delay);

        // --- Check for Timeouts ---
        if let Some(threshold) = retransmit_older_than {
            let mut seqs_to_resend = Vec::new();
            // Iterate reliable cache (ordered by seq number/time) to find timed out packets
            for (&seq, entry) in &self.reliable_cache {
                if entry.timestamp() < threshold {
                    seqs_to_resend.push(seq);
                } else {
                    break; // Cache is ordered by time (implicitly via seq number), can stop early
                }
            }

            for seq in seqs_to_resend {
                if let Some(entry) = self.reliable_cache.remove(&seq) {
                    debug!(seq, "Datagram timed out, queuing {} packets for resend", entry.packets().len());
                    for pk in entry.packets().iter().cloned() {
                        if PacketReliability::is_reliable(pk.reliability) {
                            self.resend_queue.push_back(pk);
                        }
                    }
                }
            }
        }

        // --- Process Resend Queue ---
        while let Some(pk) = self.resend_queue.pop_front() {
            // Check if the packet's message index is still relevant (not behind window start)
            if let Some(msg_idx) = pk.message_index {
                let window_start = self.reliable_window_start;
                let is_behind = if self.reliable_window_end >= window_start { // No wrap
                    msg_idx < window_start
                } else { // Wrap
                    msg_idx < window_start && msg_idx >= self.reliable_window_end
                };

                if !is_behind {
                    trace!(msg_idx, "Adding packet from resend queue back to send queue");
                    self.add_to_send_queue_internal(pk, false); // Re-queue for sending (non-immediate)
                } else {
                    debug!(msg_idx, window_start=self.reliable_window_start, "Skipping resend of packet behind window");
                }
            } else {
                warn!("Attempted to resend non-reliable packet from resend queue?");
            }
        }

        // --- Process Reliable Backlog ---
        let mut backlog_ready_to_send = Vec::new();
        // Check which backlog packets are now within the window
        self.reliable_backlog.retain(|&msg_idx, packet| {
            let window_start = self.reliable_window_start;
            let window_end = self.reliable_window_end;
            let is_in_window = if window_end >= window_start { // No wrap
                msg_idx >= window_start && msg_idx < window_end
            } else { // Wrap
                msg_idx >= window_start || msg_idx < window_end
            };

            if is_in_window {
                backlog_ready_to_send.push(packet.clone()); // Clone to send
                false // Remove from backlog
            } else {
                true // Keep in backlog
            }
        });

        for pk in backlog_ready_to_send {
            trace!(msg_idx=pk.message_index.unwrap_or(0), "Moving packet from backlog to send queue");
            self.add_to_send_queue_internal(pk, false); // Add to send queue (non-immediate)
        }

        // --- Flush Send Queue ---
        // Send any packets accumulated from resends or backlog processing
        self.flush_send_queue();
    }
} // end impl SendReliabilityLayer


// --- ReceiveReliabilityLayer ---
#[derive(Debug)]
pub struct ReceiveReliabilityLayer {
    window_start: u32, // Sequence number of the start of the receive window (u24)
    window_end: u32,   // Sequence number of the end of the receive window (exclusive) (u24)
    window_size: u32,
    highest_seq_number: i64, // Highest datagram sequence number received (-1 if none)

    /// Sequence numbers received within the window, waiting to be ACKed.
    ack_queue: HashSet<u32>, // Use HashSet for quick lookups
    /// Sequence numbers *before* window_start that were missed and need NACKing.
    nack_queue: HashSet<u32>,

    // Reliable window for incoming packets (RakNet seems to use seq number based window mainly)
    // Let's simplify and remove reliable_window_start/end/map for now, rely on seq number window.

    // Per-channel state for ordering/sequencing
    recv_ordered_index: [u32; PacketReliability::MAX_ORDER_CHANNELS], // u24
    recv_sequenced_highest_index: [u32; PacketReliability::MAX_ORDER_CHANNELS], // u24
    /// Buffers out-of-order reliable-ordered packets.
    /// Map: order_channel -> order_index -> EncapsulatedPacket
    recv_ordered_packets: [HashMap<u32, EncapsulatedPacket>; PacketReliability::MAX_ORDER_CHANNELS],

    /// Reassembles split packets.
    /// Map: split_id (u16) -> Vec<Option<EncapsulatedPacket>> (index is part_index)
    split_packets: HashMap<u16, Vec<Option<EncapsulatedPacket>>>,

    max_split_packet_part_count: usize,
    max_concurrent_split_packets: usize,

    // Callback when a fully reassembled/ordered packet is ready for processing.
    on_recv_callback: Box<dyn Fn(EncapsulatedPacket) + Send + Sync>,
    // Callback to send an ACK or NACK packet over the network.
    send_ack_nack_callback: Box<dyn Fn(AcknowledgePacketWrapper) + Send + Sync>,
}

impl ReceiveReliabilityLayer {
    pub fn new(
        window_size: u32,
        max_split_packet_part_count: usize,
        max_concurrent_split_packets: usize,
        on_recv_callback: Box<dyn Fn(EncapsulatedPacket) + Send + Sync>,
        send_ack_nack_callback: Box<dyn Fn(AcknowledgePacketWrapper) + Send + Sync>,
    ) -> Self {
        // Initialize the array of HashMaps correctly
        const EMPTY_ORDERED_MAP: HashMap<u32, EncapsulatedPacket> = HashMap::new();
        let recv_ordered_packets = [EMPTY_ORDERED_MAP; PacketReliability::MAX_ORDER_CHANNELS];

        Self {
            window_start: 0,
            window_end: window_size & 0xFFFFFF, // Ensure window size fits u24 if needed
            window_size,
            highest_seq_number: -1,
            ack_queue: HashSet::new(),
            nack_queue: HashSet::new(),
            recv_ordered_index: [0; PacketReliability::MAX_ORDER_CHANNELS],
            recv_sequenced_highest_index: [0; PacketReliability::MAX_ORDER_CHANNELS],
            recv_ordered_packets,
            split_packets: HashMap::new(),
            max_split_packet_part_count,
            max_concurrent_split_packets,
            on_recv_callback,
            send_ack_nack_callback,
        }
    }

    /// Handles an incoming Datagram, processing its sequence number and encapsulated packets.
    pub fn on_datagram(&mut self, datagram: Datagram) -> Result<()> { // Return Result for handling errors
        let seq = datagram.seq_number;

        // --- Sequence Number and Window Check ---
        let window_start = self.window_start;
        let is_out_of_window = if self.window_end >= window_start { // No wrap
            seq < window_start || seq >= self.window_end
        } else { // Wrap
            seq < window_start && seq >= self.window_end // Between end and start is out
        };

        if is_out_of_window || self.ack_queue.contains(&seq) {
            trace!(seq, window_start, window_end=self.window_end, "Received duplicate or out-of-window datagram, ignoring.");
            return Ok(()); // Ignore duplicate or out-of-window packet
        }

        // --- Update ACK/NACK Queues and Window ---
        self.ack_queue.insert(seq); // Mark as received for ACK
        self.nack_queue.remove(&seq); // Remove from NACK if it was previously missing

        // Update highest received sequence number
        if self.highest_seq_number < 0 || seq_greater_than(seq, self.highest_seq_number as u32) {
            self.highest_seq_number = seq as i64;
        }

        // Check for gaps (if seq > window_start) and mark missing packets for NACK
        if seq_greater_than(seq, window_start) {
            let mut i = window_start;
            while i != seq { // Loop carefully handling wrap-around
                if !self.ack_queue.contains(&i) {
                    trace!(missing_seq = i, "Marking missing sequence number for NACK");
                    self.nack_queue.insert(i);
                }
                i = i.wrapping_add(1) & 0xFFFFFF;
            }
        }

        // Advance window start if possible (contiguous packets received)
        let mut current_start = window_start;
        while self.ack_queue.contains(¤t_start) {
            self.ack_queue.remove(¤t_start); // Don't need to explicitly ACK contiguous packets
            current_start = current_start.wrapping_add(1) & 0xFFFFFF;
        }

        if current_start != window_start {
            let diff = current_start.wrapping_sub(window_start) & 0xFFFFFF;
            self.window_start = current_start;
            self.window_end = self.window_end.wrapping_add(diff) & 0xFFFFFF; // Move end forward
            trace!(new_start = self.window_start, new_end = self.window_end, "Receive window advanced");
        }


        // --- Process Encapsulated Packets ---
        for pk in datagram.packets {
            if let Err(e) = self.handle_encapsulated_packet(pk) {
                // If handling an encapsulated packet fails (e.g., split error), propagate the error.
                // The session layer should likely disconnect the peer.
                error!(error = %e, "Error handling encapsulated packet");
                return Err(e);
            }
        }

        Ok(())
    } // end on_datagram

    /// Handles a single EncapsulatedPacket, dealing with splitting, ordering, and sequencing.
    fn handle_encapsulated_packet(&mut self, packet: EncapsulatedPacket) -> Result<()> {
        // --- Handle Splitting ---
        let packet = match self.handle_split(packet)? {
            Some(p) => p, // Reassembly complete or not split
            None => return Ok(()), // Split part received, waiting for more
        };

        // --- Handle Ordering and Sequencing ---
        let reliability = packet.reliability;
        if PacketReliability::is_sequenced_or_ordered(reliability) {
            let channel = packet.order_channel.unwrap_or(0) as usize;
            if channel >= PacketReliability::MAX_ORDER_CHANNELS {
                warn!(channel, "Received packet with invalid order channel, dropping.");
                return Ok(());
            }

            if PacketReliability::is_sequenced(reliability) {
                let seq_idx = packet.sequence_index.unwrap_or(0);
                let order_idx = packet.order_index.unwrap_or(0);

                // Check if older than last processed ordered packet OR older sequence index on this channel
                // Need careful comparison for wrap-around.
                if seq_less_than(order_idx, self.recv_ordered_index[channel]) || seq_less_than(seq_idx, self.recv_sequenced_highest_index[channel]) {
                    trace!(seq_idx, order_idx, channel, current_ordered=self.recv_ordered_index[channel], current_sequenced=self.recv_sequenced_highest_index[channel], "Discarding old sequenced packet");
                    return Ok(());
                }

                // Update highest sequence index for this channel
                self.recv_sequenced_highest_index[channel] = seq_idx.wrapping_add(1) & 0xFFFFFF;
                // Route the packet
                (self.on_recv_callback)(packet);

            } else { // Must be Reliable Ordered
                let order_idx = packet.order_index.unwrap_or(0);

                if order_idx == self.recv_ordered_index[channel] {
                    // Expected packet arrived
                    trace!(order_idx, channel, "Received expected ordered packet");
                    self.recv_ordered_index[channel] = order_idx.wrapping_add(1) & 0xFFFFFF;
                    self.recv_sequenced_highest_index[channel] = 0; // Reset sequence index

                    // Route this packet
                    (self.on_recv_callback)(packet);

                    // Process any buffered ordered packets that are now contiguous
                    let mut current_order_idx = self.recv_ordered_index[channel];
                    loop {
                        match self.recv_ordered_packets[channel].remove(¤t_order_idx) {
                            Some(buffered_packet) => {
                                trace!(order_idx = current_order_idx, channel, "Processing buffered ordered packet");
                                (self.on_recv_callback)(buffered_packet);
                                current_order_idx = current_order_idx.wrapping_add(1) & 0xFFFFFF;
                            }
                            None => break, // No more contiguous packets in buffer
                        }
                    }
                    self.recv_ordered_index[channel] = current_order_idx; // Update index

                } else if seq_greater_than(order_idx, self.recv_ordered_index[channel]) {
                    // Packet arrived out of order (ahead of expected), buffer it
                    if self.recv_ordered_packets[channel].len() >= self.window_size as usize { // Use window size as buffer limit?
                        warn!(channel, order_idx, current_order_idx=self.recv_ordered_index[channel], "Ordered packet receive buffer full, dropping packet.");
                        // Consider disconnecting peer if buffer overflows consistently
                        return Ok(());
                    }
                    trace!(order_idx, channel, current_order_idx=self.recv_ordered_index[channel], "Buffering out-of-order packet");
                    self.recv_ordered_packets[channel].insert(order_idx, packet);
                } else {
                    // Duplicate ordered packet (order_idx < current), ignore
                    trace!(order_idx, channel, current_order_idx=self.recv_ordered_index[channel], "Discarding duplicate ordered packet");
                }
            }

        } else {
            // Unreliable packet, route immediately
            (self.on_recv_callback)(packet);
        }
        Ok(())
    } // end handle_encapsulated_packet

    /// Handles a potential split packet part. Returns Ok(Some(packet)) if reassembly is complete,
    /// Ok(None) if more parts are needed, Err on failure (which should disconnect peer).
    fn handle_split(&mut self, packet: EncapsulatedPacket) -> Result<Option<EncapsulatedPacket>> {
        let split_info = match packet.split_info {
            Some(ref info) => info,
            None => return Ok(Some(packet)), // Not a split packet
        };

        let total_parts = split_info.total_part_count;
        let part_index = split_info.part_index;
        let split_id = split_info.id;

        trace!(split_id, part_index, total_parts, "Handling split packet part");

        // --- Input Validation ---
        if total_parts == 0 || total_parts as usize > self.max_split_packet_part_count {
            warn!(split_id, total_parts, max=self.max_split_packet_part_count, "Invalid split packet part count");
            return Err(RakLibError::PacketHandling {
                message: format!("Invalid split packet part count {}", total_parts),
                source: None,
                disconnect_reason: DisconnectReason::SplitPacketTooLarge
            });
        }
        if part_index >= total_parts {
            warn!(split_id, part_index, total_parts, "Invalid split packet part index");
            return Err(RakLibError::PacketHandling {
                message: format!("Invalid split packet part index {}/{}", part_index, total_parts),
                source: None,
                disconnect_reason: DisconnectReason::SplitPacketInvalidPartIndex
            });
        }

        // --- Get or Create Split Buffer ---
        // Use entry API for cleaner logic
        let is_new_split = !self.split_packets.contains_key(&split_id);
        if is_new_split {
            // Check concurrent limit *before* inserting
            if self.split_packets.len() >= self.max_concurrent_split_packets {
                warn!(limit=self.max_concurrent_split_packets, "Exceeded concurrent split packet limit");
                return Err(RakLibError::PacketHandling {
                    message: format!("Exceeded concurrent split packet limit {}", self.max_concurrent_split_packets),
                    source: None,
                    disconnect_reason: DisconnectReason::SplitPacketTooManyConcurrent
                });
            }
        }

        let split_buffer = self.split_packets.entry(split_id).or_insert_with(|| {
            trace!(split_id, total_parts, "Creating new buffer for split packet");
            vec![None; total_parts as usize]
        });

        // --- Check Consistency and Store Part ---
        if split_buffer.len() != total_parts as usize {
            // This buffer existed before, but the new part claims a different total count
            warn!(split_id, got_total=total_parts, expected_total=split_buffer.len(), "Inconsistent split count for existing split packet");
            // Clear the inconsistent buffer and return error? Or just error? Let's error.
            self.split_packets.remove(&split_id); // Clean up bad state
            return Err(RakLibError::PacketHandling {
                message: format!("Inconsistent split count for ID {}: got {}, expected {}", split_id, total_parts, split_buffer.len()),
                source: None,
                disconnect_reason: DisconnectReason::SplitPacketInconsistentHeader
            });
        }

        if split_buffer[part_index as usize].is_none() {
            trace!(split_id, part_index, "Storing split packet part");
            split_buffer[part_index as usize] = Some(packet);
        } else {
            trace!(split_id, part_index, "Received duplicate split packet part, ignoring");
            // Duplicate part, ignore it but don't error
            return Ok(None);
        }

        // --- Check for Completion ---
        if split_buffer.iter().all(Option::is_some) {
            // --- Reassemble ---
            trace!(split_id, total_parts, "All parts received, reassembling split packet");
            let mut reassembled_buffer = Vec::new();
            let mut base_packet: Option<EncapsulatedPacket> = None;

            // Iterate through the Vec<Option<Packet>>, which we know are all Some
            for part_option in split_buffer.iter() {
                let part = part_option.as_ref().unwrap(); // Safe unwrap due to all() check
                reassembled_buffer.extend_from_slice(&part.buffer);
                if base_packet.is_none() {
                    // Use the first part to copy reliability, ordering, etc.
                    base_packet = Some(part.clone());
                }
            }

            // Clean up the completed entry
            self.split_packets.remove(&split_id);

            let mut final_packet = base_packet.expect("Base packet should exist after loop");
            final_packet.buffer = reassembled_buffer;
            final_packet.split_info = None; // Clear split info as it's now reassembled

            trace!(split_id, final_len=final_packet.buffer.len(), "Split packet reassembled successfully");
            Ok(Some(final_packet))
        } else {
            // Not yet complete, more parts needed
            Ok(None)
        }
    } // end handle_split

    /// Returns true if there are pending ACKs or NACKs to be sent.
    pub fn needs_update(&self) -> bool {
        !self.ack_queue.is_empty() || !self.nack_queue.is_empty()
    }

    /// Generates and sends ACK/NACK packets based on the current queue state.
    pub fn update(&mut self) {
        if !self.ack_queue.is_empty() {
            let ack = ACK(AcknowledgePacket {
                packets: self.ack_queue.iter().cloned().collect(), // Collect seq numbers
            });
            trace!(ack_count = ack.0.packets.len(), "Sending ACK");
            (self.send_ack_nack_callback)(AcknowledgePacketWrapper::Ack(ack));
            self.ack_queue.clear();
        }

        if !self.nack_queue.is_empty() {
            let nack = NACK(AcknowledgePacket {
                packets: self.nack_queue.iter().cloned().collect(),
            });
            debug!(nack_count = nack.0.packets.len(), "Sending NACK for missing packets");
            (self.send_ack_nack_callback)(AcknowledgePacketWrapper::Nack(nack));
            self.nack_queue.clear();
        }
    }
} // end impl ReceiveReliabilityLayer


/// Wrapper enum to allow sending ACK or NACK through a single callback type.
#[derive(Debug)]
pub enum AcknowledgePacketWrapper {
    Ack(ACK),
    Nack(NACK),
}

impl AcknowledgePacketWrapper {
    // Helper to encode the inner packet into a BinaryStream
    pub fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        match self {
            AcknowledgePacketWrapper::Ack(ack) => ack.encode(stream),
            AcknowledgePacketWrapper::Nack(nack) => nack.encode(stream),
        }
    }

    // Helper to get the ID of the inner packet
    pub fn id(&self) -> u8 {
        match self {
            AcknowledgePacketWrapper::Ack(_) => ACK::id(),
            AcknowledgePacketWrapper::Nack(_) => NACK::id(),
        }
    }
}


// --- Sequence Number Comparison Helpers (Handling u24 wrap-around) ---

/// Returns true if s1 is greater than s2, considering wrap-around for u24 numbers.
#[inline]
fn seq_greater_than(s1: u32, s2: u32) -> bool {
    // Assumes MAX_SEQ = 0xFFFFFF
    ((s1 > s2) && (s1 - s2 <= 0x7FFFFF)) || ((s1 < s2) && (s2 - s1 > 0x7FFFFF))
}

/// Returns true if s1 is less than s2, considering wrap-around for u24 numbers.
#[inline]
fn seq_less_than(s1: u32, s2: u32) -> bool {
    // Assumes MAX_SEQ = 0xFFFFFF
    ((s1 < s2) && (s2 - s1 <= 0x7FFFFF)) || ((s1 > s2) && (s1 - s2 > 0x7FFFFF))
}