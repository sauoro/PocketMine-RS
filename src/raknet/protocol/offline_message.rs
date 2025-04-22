// src/raknet/protocol/offline_message.rs
#![allow(dead_code)]

use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{BinaryDataException, Result as BinaryResult};

const MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

pub trait OfflineMessage: Packet {
    fn read_magic(stream: &mut PacketSerializer) -> BinaryResult<()> {
        let magic_read = stream.get_slice(16)?;
        if magic_read != MAGIC {
            Err(BinaryDataException::from_str("Invalid offline message magic bytes"))
        } else {
            Ok(())
        }
    }

    fn write_magic(stream: &mut PacketSerializer) {
        stream.put_slice(&MAGIC);
    }

    fn is_valid_magic(stream: &PacketSerializer) -> bool {
        stream.get_buffer().len() >= 17 // Packet ID + Magic
            && &stream.get_buffer()[1..17] == MAGIC
    }
}