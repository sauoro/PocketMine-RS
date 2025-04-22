// src/raknet/protocol/offline_message.rs

#![allow(dead_code)]

use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{BinaryDataException, Result};
use std::fmt::Debug;

// Magic bytes used to distinguish offline messages
const MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

// Trait for common offline message behavior (magic handling)
// Using a trait allows different offline message structs to share this logic.
pub trait OfflineMessage: Debug + Send + Sync {
    fn read_magic(&self, stream: &mut PacketSerializer) -> Result<[u8; 16]> {
        let magic = stream.get(16)?;
        magic.try_into().map_err(|_| BinaryDataException::from_str("Failed to read 16 bytes for magic"))
    }

    fn write_magic(&self, stream: &mut PacketSerializer) -> Result<()> {
        stream.put(&MAGIC);
        Ok(())
    }

    fn is_valid_magic(&self, read_magic: &[u8; 16]) -> bool {
        read_magic == &MAGIC
    }
}