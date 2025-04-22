// src/raknet/protocol/encapsulated_packet.rs
#![allow(dead_code)]

use crate::raknet::protocol::packet_reliability::PacketReliability;
use crate::utils::binary;
use crate::utils::error::{BinaryDataException, Result as BinaryResult};
use crate::utils::BinaryStream;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::convert::TryInto;
use std::fmt;

const RELIABILITY_SHIFT: u8 = 5;
const RELIABILITY_FLAGS: u8 = 0b111 << RELIABILITY_SHIFT;
const SPLIT_FLAG: u8 = 0b00010000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitPacketInfo {
    pub id: u16,
    pub part_index: u32,
    pub total_part_count: u32,
}

impl SplitPacketInfo {
    pub fn new(id: u16, part_index: u32, total_part_count: u32) -> Self {
        Self { id, part_index, total_part_count }
    }

    pub fn get_id(&self) -> u16 { self.id }
    pub fn get_part_index(&self) -> u32 { self.part_index }
    pub fn get_total_part_count(&self) -> u32 { self.total_part_count }
}

#[derive(Clone)]
pub struct EncapsulatedPacket {
    pub reliability: u8,
    pub message_index: Option<u32>,
    pub sequence_index: Option<u32>,
    pub order_index: Option<u32>,
    pub order_channel: Option<u8>,
    pub split_info: Option<SplitPacketInfo>,
    pub buffer: Bytes, // Use Bytes for efficient slicing
    pub identifier_ack: Option<u32>, // Not part of RakNet protocol, used internally for tracking
}

impl EncapsulatedPacket {
    pub const SPLIT_INFO_LENGTH: usize = 4 + 2 + 4; // split count (4) + split ID (2) + split index (4)

    pub fn new() -> Self {
        Self {
            reliability: PacketReliability::UNRELIABLE,
            message_index: None,
            sequence_index: None,
            order_index: None,
            order_channel: None,
            split_info: None,
            buffer: Bytes::new(),
            identifier_ack: None,
        }
    }

    pub fn read(stream: &mut BinaryStream) -> BinaryResult<Self> {
        let flags = stream.get_byte()?;
        let reliability = (flags & RELIABILITY_FLAGS) >> RELIABILITY_SHIFT;
        let has_split = (flags & SPLIT_FLAG) != 0;

        let length_bits = stream.get_short()?;
        let length_bytes = (length_bits as f64 / 8.0).ceil() as usize;
        if length_bytes == 0 {
            return Err(BinaryDataException::from_str("Encapsulated payload length cannot be zero"));
        }

        let mut packet = EncapsulatedPacket::new();
        packet.reliability = reliability;

        if PacketReliability::is_reliable(reliability) {
            packet.message_index = Some(stream.get_ltriad()?);
        }

        if PacketReliability::is_sequenced(reliability) {
            packet.sequence_index = Some(stream.get_ltriad()?);
        }

        if PacketReliability::is_sequenced_or_ordered(reliability) {
            packet.order_index = Some(stream.get_ltriad()?);
            packet.order_channel = Some(stream.get_byte()?);
        }

        if has_split {
            let split_count = stream.get_int()? as u32; // Read as i32, cast to u32
            let split_id = stream.get_short()?;
            let split_index = stream.get_int()? as u32; // Read as i32, cast to u32
            packet.split_info = Some(SplitPacketInfo::new(split_id, split_index, split_count));
        }

        let buffer_slice = stream.get(length_bytes)?;
        packet.buffer = Bytes::copy_from_slice(buffer_slice);

        Ok(packet)
    }

    pub fn write(&self, buf: &mut BytesMut) -> BinaryResult<()> {
        let flags = (self.reliability << RELIABILITY_SHIFT) | (if self.split_info.is_some() { SPLIT_FLAG } else { 0 });
        buf.put_u8(flags);

        let length_bits = (self.buffer.len() * 8) as u16;
        buf.put_u16(length_bits);

        if let Some(index) = self.message_index {
            if PacketReliability::is_reliable(self.reliability) {
                buf.put(&binary::write_ltriad(index)?[..]);
            }
        }
        if let Some(index) = self.sequence_index {
            if PacketReliability::is_sequenced(self.reliability) {
                buf.put(&binary::write_ltriad(index)?[..]);
            }
        }
        if let Some(index) = self.order_index {
            if PacketReliability::is_sequenced_or_ordered(self.reliability) {
                buf.put(&binary::write_ltriad(index)?[..]);
                buf.put_u8(self.order_channel.unwrap_or(0)); // Should always exist if order_index exists
            }
        }

        if let Some(split) = &self.split_info {
            buf.put_i32(split.total_part_count.try_into().map_err(|_| BinaryDataException::from_str("Split count too large"))?);
            buf.put_u16(split.id);
            buf.put_i32(split.part_index.try_into().map_err(|_| BinaryDataException::from_str("Split index too large"))?);
        }

        buf.put(self.buffer.clone());
        Ok(())
    }

    pub fn get_header_length(&self) -> usize {
        1 + // flags
            2 + // length
            (if PacketReliability::is_reliable(self.reliability) { 3 } else { 0 }) + // message index
            (if PacketReliability::is_sequenced(self.reliability) { 3 } else { 0 }) + // sequence index
            (if PacketReliability::is_sequenced_or_ordered(self.reliability) { 3 + 1 } else { 0 }) + // order index (3) + order channel (1)
            (if self.split_info.is_some() { Self::SPLIT_INFO_LENGTH } else { 0 })
    }

    pub fn get_total_length(&self) -> usize {
        self.get_header_length() + self.buffer.len()
    }
}

impl Default for EncapsulatedPacket {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for EncapsulatedPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncapsulatedPacket")
            .field("reliability", &self.reliability)
            .field("message_index", &self.message_index)
            .field("sequence_index", &self.sequence_index)
            .field("order_index", &self.order_index)
            .field("order_channel", &self.order_channel)
            .field("split_info", &self.split_info)
            .field("buffer_len", &self.buffer.len())
            .field("identifier_ack", &self.identifier_ack)
            .finish()
    }
}

// Manual PartialEq because Bytes doesn't derive it directly
impl PartialEq for EncapsulatedPacket {
    fn eq(&self, other: &Self) -> bool {
        self.reliability == other.reliability &&
            self.message_index == other.message_index &&
            self.sequence_index == other.sequence_index &&
            self.order_index == other.order_index &&
            self.order_channel == other.order_channel &&
            self.split_info == other.split_info &&
            self.buffer == other.buffer && // Compares content
            self.identifier_ack == other.identifier_ack
    }
}
impl Eq for EncapsulatedPacket {}