// src/raklib/protocol/mod.rs
pub mod ack;
pub mod consts;
pub mod frame;
pub mod packets;

// Re-export common items
pub use consts::{MessageIdentifiers, PacketReliability};
pub use frame::{Datagram, EncapsulatedPacket, SplitPacketInfo};
pub use ack::{AckNackRecord, AcknowledgePacket, ACK, NACK}; // <<< Keep AcknowledgePacket private if only used internally

use crate::utils::binary::{BinaryStream, Result as BinaryResult, BinaryUtilError};

/// Common trait for RakNet packets that can be encoded and decoded.
pub trait Packet: Sized {
    /// Returns the Packet ID for this packet type.
    fn id() -> u8;

    /// Encodes the packet header and payload into the stream.
    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()>;

    /// Decodes the packet *payload* from the stream.
    /// Assumes the ID byte has *already been consumed* by the caller.
    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self>;

    /// Encodes just the header (usually just the ID).
    fn encode_header(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        stream.put_u8(Self::id())
    }

    /// Encodes just the payload (called by `encode`).
    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()>;

    /// Decodes just the payload (called by `decode`).
    /// Assumes ID byte was consumed before calling decode().
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()>;
}


/// Marker trait for packets sent before a connection is fully established.
/// Includes logic for RakNet magic bytes.
pub trait OfflinePacket: Packet {
    fn magic() -> &'static [u8; 16] {
        b"\x00\xff\xff\x00\xfe\xfe\xfe\xfe\xfd\xfd\xfd\xfd\x12\x34\x56\x78"
    }

    fn write_magic(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        stream.put(Self::magic())
    }

    /// Reads the magic bytes. Should be called *after* the ID byte.
    fn read_magic(stream: &mut BinaryStream) -> BinaryResult<[u8; 16]> {
        let bytes_vec = stream.get(16)?; // Read into a Vec
        bytes_vec.try_into().map_err(|v: Vec<u8>| BinaryUtilError::NotEnoughData { needed: 16, have: v.len() }) // Use map_err after try_into
    }

    fn check_magic(read_magic: &[u8; 16]) -> bool {
        read_magic == Self::magic()
    }
}


/// Marker trait for packets sent after a connection is established (inside an EncapsulatedPacket).
pub trait ConnectedPacket: Packet {}