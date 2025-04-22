// src/raknet/protocol/datagram.rs

#![allow(dead_code)]

use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::protocol::encapsulated_packet::EncapsulatedPacket;
use crate::utils::error::{Result, BinaryDataException};

// Datagram flags combined with Packet ID (first byte)
pub const BITFLAG_VALID: u8 = 0x80;
pub const BITFLAG_ACK: u8 = 0x40;
pub const BITFLAG_NAK: u8 = 0x20;
// Unused RakNet flags (for reference)
// const BITFLAG_PACKET_PAIR: u8 = 0x10;
// const BITFLAG_CONTINUOUS_SEND: u8 = 0x08;
// const BITFLAG_NEEDS_B_AND_AS: u8 = 0x04;

// Header size: Flags (1 byte) + Sequence Number (3 bytes LE Triad)
pub const HEADER_SIZE: usize = 1 + 3;

#[derive(Debug, Clone, Default)]
pub struct Datagram {
    /// Flags indicating if this datagram is an ACK or NACK (used only by ACK/NACK packets).
    /// For regular datagrams, this should be 0.
    pub header_flags: u8,
    /// Sequence number for reliability layer.
    pub seq_number: u32, // u24, fits in u32
    /// The encapsulated packets contained within this datagram.
    pub packets: Vec<EncapsulatedPacket>,
}

impl Datagram {
    /// Calculates the total serialized length of the datagram (header + all encapsulated packets).
    pub fn get_length(&self) -> usize {
        HEADER_SIZE
            + self
            .packets
            .iter()
            .map(|p| p.get_total_length())
            .sum::<usize>()
    }
}

// Datagram doesn't use the standard Packet trait directly for encode/decode
// because its first byte isn't a fixed ID but contains flags.
// The Session/ReliabilityLayer handles encoding/decoding based on these flags.
// We provide encode/decode methods but they won't implement the Packet trait's default header handling.

impl Datagram {
    /// Encodes the Datagram into a PacketSerializer.
    pub fn encode(&self, stream: &mut PacketSerializer) -> Result<()> {
        stream.put_byte(BITFLAG_VALID | self.header_flags); // Always set VALID flag
        stream.put_l_triad(self.seq_number)?;
        for packet in &self.packets {
            // EncapsulatedPacket::to_binary returns a Vec<u8> Result
            let encoded_packet = packet.to_binary()?;
            stream.put(&encoded_packet);
        }
        Ok(())
    }

    /// Decodes a Datagram from a PacketSerializer.
    /// Assumes the first byte (flags) has already been read by the caller to determine packet type.
    pub fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        // header_flags are assumed to be set by the caller based on the first byte read.
        self.seq_number = stream.get_l_triad()?;
        self.packets.clear();
        while !stream.feof() {
            match EncapsulatedPacket::from_binary(stream) {
                Ok(packet) => self.packets.push(packet),
                Err(e) => {
                    // If decoding an encapsulated packet fails mid-datagram,
                    // it might indicate corruption or truncation.
                    // Log the error and potentially stop processing the rest?
                    // For now, return the error to let the caller decide.
                    eprintln!("Error decoding EncapsulatedPacket within Datagram: {}", e); // Simple stderr log
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}

// Note: We might later add From implementations or helper functions if needed
// for converting between Datagram and ACK/NACK types if they share parts of the logic.
// For now, ACK/NACK have their own structs and Packet impls.