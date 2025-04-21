use super::{PacketReliability, Packet};
use crate::utils::binary::{BinaryStream, Result as BinaryResult, BinaryUtilError};
use byteorder::{ByteOrder, LittleEndian};
use std::convert::TryInto;

// --- SplitPacketInfo ---
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitPacketInfo {
    pub id: u16,
    pub part_index: u32,
    pub total_part_count: u32,
}

impl SplitPacketInfo {
    pub fn read_from(stream: &mut BinaryStream) -> BinaryResult<Self> {
        Ok(SplitPacketInfo {
            total_part_count: stream.get_u32_be()?, // RakNet uses BigEndian here
            id: stream.get_u16_be()?,             // BigEndian
            part_index: stream.get_u32_be()?,         // BigEndian
        })
    }

    pub fn write_to(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        stream.put_u32_be(self.total_part_count)?;
        stream.put_u16_be(self.id)?;
        stream.put_u32_be(self.part_index)
    }

    pub const ENCODED_LENGTH: usize = 4 + 2 + 4;
}

// --- EncapsulatedPacket ---
#[derive(Debug, Clone)]
pub struct EncapsulatedPacket {
    pub reliability: u8,
    pub message_index: Option<u32>, // u24
    pub sequence_index: Option<u32>, // u24
    pub order_index: Option<u32>, // u24
    pub order_channel: Option<u8>,
    pub split_info: Option<SplitPacketInfo>,
    pub buffer: Vec<u8>, // The actual payload
    /// Internal use: identifier for ACK tracking
    pub identifier_ack: Option<u32>,
}

impl EncapsulatedPacket {
    const RELIABILITY_SHIFT: u8 = 5;
    const RELIABILITY_FLAGS: u8 = 0b111 << Self::RELIABILITY_SHIFT;
    const SPLIT_FLAG: u8 = 0b00010000;

    pub fn new() -> Self {
        EncapsulatedPacket {
            reliability: PacketReliability::UNRELIABLE,
            message_index: None,
            sequence_index: None,
            order_index: None,
            order_channel: None,
            split_info: None,
            buffer: Vec::new(),
            identifier_ack: None,
        }
    }

    pub fn from_binary(stream: &mut BinaryStream) -> BinaryResult<Self> {
        let flags = stream.get_u8()?;
        let reliability = (flags & Self::RELIABILITY_FLAGS) >> Self::RELIABILITY_SHIFT;
        let has_split = (flags & Self::SPLIT_FLAG) != 0;

        // Length is in bits, convert to bytes (ceil division)
        let bit_length = stream.get_u16_be()?;
        let byte_length = (bit_length as usize + 7) / 8;

        if byte_length == 0 {
            return Err(BinaryUtilError::InvalidData("Encapsulated payload length cannot be zero".to_string()));
        }

        let message_index = if PacketReliability::is_reliable(reliability) {
            Some(stream.get_u24_le()?)
        } else {
            None
        };

        let sequence_index = if PacketReliability::is_sequenced(reliability) {
            Some(stream.get_u24_le()?)
        } else {
            None
        };

        let (order_index, order_channel) = if PacketReliability::is_sequenced_or_ordered(reliability) {
            (Some(stream.get_u24_le()?), Some(stream.get_u8()?))
        } else {
            (None, None)
        };

        let split_info = if has_split {
            Some(SplitPacketInfo::read_from(stream)?)
        } else {
            None
        };

        let buffer = stream.get(byte_length)?;

        Ok(EncapsulatedPacket {
            reliability,
            message_index,
            sequence_index,
            order_index,
            order_channel,
            split_info,
            buffer,
            identifier_ack: None, // This is populated internally, not from network
        })
    }

    pub fn to_binary(&self) -> BinaryResult<Vec<u8>> {
        let mut stream = BinaryStream::new();
        self.write_to(&mut stream)?;
        Ok(stream.into_inner())
    }

    pub fn write_to(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        let flags = (self.reliability << Self::RELIABILITY_SHIFT)
            | if self.split_info.is_some() { Self::SPLIT_FLAG } else { 0 };
        stream.put_u8(flags)?;

        let bit_length = (self.buffer.len() * 8) as u16;
        stream.put_u16_be(bit_length)?;

        if let Some(index) = self.message_index {
            if PacketReliability::is_reliable(self.reliability) {
                stream.put_u24_le(index)?;
            }
        }
        if let Some(index) = self.sequence_index {
            if PacketReliability::is_sequenced(self.reliability) {
                stream.put_u24_le(index)?;
            }
        }
        if let (Some(index), Some(channel)) = (self.order_index, self.order_channel) {
            if PacketReliability::is_sequenced_or_ordered(self.reliability) {
                stream.put_u24_le(index)?;
                stream.put_u8(channel)?;
            }
        }
        if let Some(ref info) = self.split_info {
            info.write_to(stream)?;
        }

        stream.put(&self.buffer)?;
        Ok(())
    }

    pub fn header_length(&self) -> usize {
        1 // Flags
            + 2 // Length
            + if PacketReliability::is_reliable(self.reliability) { 3 } else { 0 } // messageIndex
            + if PacketReliability::is_sequenced(self.reliability) { 3 } else { 0 } // sequenceIndex
            + if PacketReliability::is_sequenced_or_ordered(self.reliability) { 3 + 1 } else { 0 } // orderIndex + orderChannel
            + if self.split_info.is_some() { SplitPacketInfo::ENCODED_LENGTH } else { 0 } // split info
    }

    pub fn total_length(&self) -> usize {
        self.header_length() + self.buffer.len()
    }
}

impl Default for EncapsulatedPacket {
    fn default() -> Self {
        Self::new()
    }
}


// --- Datagram ---
#[derive(Debug, Clone)]
pub struct Datagram {
    /// Includes flags like ACK, NACK etc. but NOT the VALID flag.
    pub header_flags: u8,
    pub seq_number: u32, // u24
    pub packets: Vec<EncapsulatedPacket>,
}

impl Datagram {
    pub const BITFLAG_VALID: u8 = 0x80;
    pub const BITFLAG_ACK: u8 = 0x40;
    pub const BITFLAG_NAK: u8 = 0x20;
    // Other flags are less common / internal
    pub const HEADER_SIZE: usize = 1 + 3; // flags + seq number

    pub fn new() -> Self {
        Datagram {
            header_flags: 0,
            seq_number: 0,
            packets: Vec::new(),
        }
    }

    // Datagrams themselves don't use the Packet trait directly usually,
    // they are the containers. We provide encode/decode like Packet though.

    pub fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        stream.put_u8(self.header_flags | Self::BITFLAG_VALID)?;
        stream.put_u24_le(self.seq_number)?;
        for packet in &self.packets {
            packet.write_to(stream)?;
        }
        Ok(())
    }

    pub fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        let flags = stream.get_u8()?;
        if (flags & Self::BITFLAG_VALID) == 0 {
            return Err(BinaryUtilError::InvalidData("Datagram VALID bit not set".to_string()));
        }
        let header_flags = flags & !Self::BITFLAG_VALID; // Store flags without VALID bit
        let seq_number = stream.get_u24_le()?;

        let mut packets = Vec::new();
        while !stream.feof() {
            // Use a temporary stream or check remaining bytes carefully
            // to avoid errors if decoding fails mid-way.
            // For simplicity now, we assume valid structure.
            match EncapsulatedPacket::from_binary(stream) {
                Ok(pk) => packets.push(pk),
                Err(BinaryUtilError::NotEnoughData { .. }) if stream.feof() => break, // Normal EOF
                Err(e) => return Err(e), // Propagate other errors
            }
        }

        Ok(Datagram {
            header_flags,
            seq_number,
            packets,
        })
    }

    pub fn length(&self) -> usize {
        Self::HEADER_SIZE + self.packets.iter().map(|p| p.total_length()).sum::<usize>()
    }

    pub fn is_ack(&self) -> bool {
        (self.header_flags & Self::BITFLAG_ACK) != 0
    }

    pub fn is_nack(&self) -> bool {
        (self.header_flags & Self::BITFLAG_NAK) != 0
    }
}

impl Default for Datagram {
    fn default() -> Self {
        Self::new()
    }
}