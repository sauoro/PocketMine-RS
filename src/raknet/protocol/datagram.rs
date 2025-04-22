// src/raknet/protocol/datagram.rs
#![allow(dead_code)]

use crate::raknet::protocol::encapsulated_packet::EncapsulatedPacket;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{BinaryDataException, Result as BinaryResult};
use bytes::BytesMut; // Use BytesMut for efficient building

#[derive(Debug, Clone)]
pub struct Datagram {
    pub header_flags: u8,
    pub packets: Vec<EncapsulatedPacket>,
    pub seq_number: u32,
}

impl Datagram {
    pub const BITFLAG_VALID: u8 = 0x80;
    pub const BITFLAG_ACK: u8 = 0x40;
    pub const BITFLAG_NAK: u8 = 0x20;
    pub const BITFLAG_PACKET_PAIR: u8 = 0x10; // Not typically used by receiver
    pub const BITFLAG_CONTINUOUS_SEND: u8 = 0x08; // Not typically used by receiver
    pub const BITFLAG_NEEDS_B_AND_AS: u8 = 0x04; // Not typically used by receiver

    pub const HEADER_SIZE: usize = 1 + 3; // header flags (1) + sequence number (3)

    pub fn new() -> Self {
        Self {
            header_flags: 0,
            packets: Vec::new(),
            seq_number: 0,
        }
    }

    pub fn length(&self) -> usize {
        let mut length = Self::HEADER_SIZE;
        for packet in &self.packets {
            length += packet.get_total_length();
        }
        length
    }

    // Specific encode for Datagram since header is different
    pub fn encode(&self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        stream.put_byte(Self::BITFLAG_VALID | self.header_flags);
        stream.put_ltriad(self.seq_number)?;

        let mut packet_data = BytesMut::new();
        for packet in &self.packets {
            packet.write(&mut packet_data)?;
        }
        stream.put_slice(&packet_data);
        Ok(())
    }

    // Specific decode for Datagram
    pub fn decode(stream: &mut PacketSerializer) -> BinaryResult<Self> {
        let header_flags = stream.get_byte()?;
        let seq_number = stream.get_ltriad()?;

        let mut packets = Vec::new();
        while !stream.feof() {
            packets.push(EncapsulatedPacket::read(stream)?);
        }

        Ok(Self {
            header_flags: header_flags & !Self::BITFLAG_VALID, // Store flags without VALID bit
            packets,
            seq_number,
        })
    }
}

impl Default for Datagram {
    fn default() -> Self {
        Self::new()
    }
}

// Note: Datagram doesn't fit the standard Packet trait perfectly due to header difference
// but we might need common methods later. For now, it stands alone.