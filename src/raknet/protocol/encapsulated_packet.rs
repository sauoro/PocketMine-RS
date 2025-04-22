// src/raknet/protocol/encapsulated_packet.rs

#![allow(dead_code)]

use crate::raknet::protocol::packet_reliability::PacketReliability;
use crate::raknet::protocol::split_packet_info::SplitPacketInfo;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{Result, BinaryDataException};
use crate::utils::binary; // For write_ltriad, read_ltriad
use std::convert::TryInto;
use std::fmt;
use std::io::Cursor; // Using Cursor for easier buffer management within methods

// Constants from EncapsulatedPacket.php
const RELIABILITY_SHIFT: u8 = 5;
const RELIABILITY_FLAGS: u8 = 0b111 << RELIABILITY_SHIFT;
const SPLIT_FLAG: u8 = 0b00010000;

// Constants for header lengths (min/max possible based on flags)
// Min: flags(1) + length(2) = 3
// Max: flags(1) + length(2) + msgIdx(3) + seqIdx(3) + ordIdx(3) + ordCh(1) + split(10) = 26
const MIN_HEADER_LENGTH: usize = 3;
pub const SPLIT_INFO_LENGTH: usize = 4 + 2 + 4; // split count(4) + split ID(2) + split index(4)

#[derive(Clone)] // Removed Debug derive initially, will add custom Debug impl
pub struct EncapsulatedPacket {
    /// Reliability setting for this packet.
    pub reliability: u8,
    /// Message index, used for reliability. Only present if reliable.
    pub message_index: Option<u32>, // u24
    /// Sequence index, used for sequencing. Only present if sequenced.
    pub sequence_index: Option<u32>, // u24
    /// Order index, used for ordering. Only present if ordered or sequenced.
    pub order_index: Option<u32>, // u24
    /// Order channel, used for ordering. Only present if ordered or sequenced.
    pub order_channel: Option<u8>,
    /// Split packet info. Only present if this is a fragment.
    pub split_info: Option<SplitPacketInfo>,
    /// The actual payload of the packet.
    pub buffer: Vec<u8>,
    /// Optional ACK identifier attached by the sender.
    pub identifier_ack: Option<u32>, // RakLib uses u32 for this internally? Check Session.php usage. Let's use u32 for now.
}

impl EncapsulatedPacket {
    /// Creates a new empty encapsulated packet.
    pub fn new() -> Self {
        Self {
            reliability: PacketReliability::RELIABLE_ORDERED, // Default to common value
            message_index: None,
            sequence_index: None,
            order_index: None,
            order_channel: None,
            split_info: None,
            buffer: Vec::new(),
            identifier_ack: None,
        }
    }

    /// Deserializes an EncapsulatedPacket from a BinaryStream/PacketSerializer.
    /// Assumes the stream cursor is positioned at the start of the packet's data.
    pub fn from_binary(stream: &mut PacketSerializer) -> Result<Self> {
        let flags = stream.get_byte()?;
        let reliability = (flags & RELIABILITY_FLAGS) >> RELIABILITY_SHIFT;
        let has_split = (flags & SPLIT_FLAG) != 0;

        // Length is in BITS, convert to bytes, rounding up.
        let length_bits = stream.get_short()?;
        let length_bytes = ((length_bits + 7) / 8) as usize; // Equivalent to ceil(len / 8)

        if length_bytes == 0 {
            return Err(BinaryDataException::from_str(
                "Encapsulated payload length cannot be zero",
            ));
        }

        let message_index = if PacketReliability::is_reliable(reliability) {
            Some(stream.get_l_triad()?)
        } else {
            None
        };

        let sequence_index = if PacketReliability::is_sequenced(reliability) {
            Some(stream.get_l_triad()?)
        } else {
            None
        };

        let (order_index, order_channel) = if PacketReliability::is_sequenced_or_ordered(reliability) {
            let ord_idx = Some(stream.get_l_triad()?);
            let ord_ch = Some(stream.get_byte()?);
            (ord_idx, ord_ch)
        } else {
            (None, None)
        };

        let split_info = if has_split {
            let split_count = stream.get_int()? as u32; // Read as i32, cast to u32
            let split_id = stream.get_short()?;
            let split_index = stream.get_int()? as u32; // Read as i32, cast to u32
            // Basic validation
            if split_index >= split_count {
                return Err(BinaryDataException::new(format!(
                    "Invalid split packet index {} >= count {}", split_index, split_count
                )));
            }
            Some(SplitPacketInfo::new(split_id, split_index, split_count))
        } else {
            None
        };

        let buffer = stream.get(length_bytes)?.to_vec();

        Ok(Self {
            reliability,
            message_index,
            sequence_index,
            order_index,
            order_channel,
            split_info,
            buffer,
            identifier_ack: None, // ACK ID is not part of the serialized format
        })
    }

    /// Serializes the EncapsulatedPacket into a byte vector.
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.get_header_length() + self.buffer.len());
        // Use a Cursor for easier writing into the buffer
        let mut stream = PacketSerializer::from_slice(&[]); // Dummy stream for methods

        // Flags
        let flags = (self.reliability << RELIABILITY_SHIFT) | if self.split_info.is_some() { SPLIT_FLAG } else { 0 };
        buf.push(flags);

        // Length (in bits)
        let length_bits: u16 = (self.buffer.len() * 8).try_into().map_err(|_| {
            BinaryDataException::from_str("Packet buffer length exceeds maximum representable bit length")
        })?;
        buf.extend_from_slice(&binary::write_short(length_bits)?);

        // Optional fields based on reliability
        if let Some(index) = self.message_index {
            if !PacketReliability::is_reliable(self.reliability) {
                return Err(BinaryDataException::from_str("message_index should only be set for reliable packets"));
            }
            buf.extend_from_slice(&binary::write_ltriad(index)?);
        } else if PacketReliability::is_reliable(self.reliability) {
            return Err(BinaryDataException::from_str("message_index is required for reliable packets"));
        }


        if let Some(index) = self.sequence_index {
            if !PacketReliability::is_sequenced(self.reliability) {
                return Err(BinaryDataException::from_str("sequence_index should only be set for sequenced packets"));
            }
            buf.extend_from_slice(&binary::write_ltriad(index)?);
        } else if PacketReliability::is_sequenced(self.reliability) {
            return Err(BinaryDataException::from_str("sequence_index is required for sequenced packets"));
        }


        if let Some(index) = self.order_index {
            if !PacketReliability::is_sequenced_or_ordered(self.reliability) {
                return Err(BinaryDataException::from_str("order_index should only be set for ordered/sequenced packets"));
            }
            let channel = self.order_channel.ok_or_else(|| {
                BinaryDataException::from_str("order_channel is required when order_index is set")
            })?;
            buf.extend_from_slice(&binary::write_ltriad(index)?);
            buf.push(channel);
        } else if PacketReliability::is_sequenced_or_ordered(self.reliability) {
            return Err(BinaryDataException::from_str("order_index and order_channel are required for ordered/sequenced packets"));
        }

        // Split info
        if let Some(info) = &self.split_info {
            buf.extend_from_slice(&binary::write_int(info.total_part_count() as i32)?);
            buf.extend_from_slice(&binary::write_short(info.id())?);
            buf.extend_from_slice(&binary::write_int(info.part_index() as i32)?);
        }

        // Payload buffer
        buf.extend_from_slice(&self.buffer);

        Ok(buf)
    }

    /// Calculates the length of the header part of the packet based on its flags/reliability.
    pub fn get_header_length(&self) -> usize {
        MIN_HEADER_LENGTH // flags(1) + length(2)
            + if self.message_index.is_some() { 3 } else { 0 }
            + if self.sequence_index.is_some() { 3 } else { 0 }
            + if self.order_index.is_some() { 3 + 1 } else { 0 } // order index + order channel
            + if self.split_info.is_some() { SPLIT_INFO_LENGTH } else { 0 }
    }

    /// Calculates the total serialized length of the packet (header + payload).
    pub fn get_total_length(&self) -> usize {
        self.get_header_length() + self.buffer.len()
    }
}

// Custom Debug implementation to avoid printing the potentially large buffer
impl fmt::Debug for EncapsulatedPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncapsulatedPacket")
            .field("reliability", &self.reliability)
            .field("message_index", &self.message_index)
            .field("sequence_index", &self.sequence_index)
            .field("order_index", &self.order_index)
            .field("order_channel", &self.order_channel)
            .field("split_info", &self.split_info)
            .field("buffer_len", &self.buffer.len()) // Show buffer length instead of content
            .field("identifier_ack", &self.identifier_ack)
            .finish()
    }
}