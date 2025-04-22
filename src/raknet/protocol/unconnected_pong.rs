// src/raknet/protocol/unconnected_pong.rs

#![allow(dead_code)]

use crate::raknet::protocol::packet::{Packet};
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{Result, BinaryDataException};

#[derive(Debug, Clone)]
pub struct UnconnectedPong {
    pub send_ping_time: i64,
    pub server_id: i64,
    // Magic is handled by the OfflineMessage trait
    pub server_name: String,
}

impl UnconnectedPong {
    // Equivalent to PHP's static create method
    pub fn create(send_ping_time: i64, server_id: i64, server_name: String) -> Self {
        Self {
            send_ping_time,
            server_id,
            server_name,
        }
    }
}

impl Packet for UnconnectedPong {
    fn get_id(&self) -> u8 {
        MessageIdentifiers::ID_UNCONNECTED_PONG
    }

    // Custom payload encoding
    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        stream.put_long(self.send_ping_time)?;
        stream.put_long(self.server_id)?;
        self.write_magic(stream)?;
        stream.put_string(&self.server_name)?; // Use put_string for length-prefixed string
        Ok(())
    }

    // Custom payload decoding
    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        self.send_ping_time = stream.get_long()?;
        self.server_id = stream.get_long()?;
        let magic = self.read_magic(stream)?;
        if !self.is_valid_magic(&magic) {
            return Err(BinaryDataException::from_str("Invalid magic bytes"));
        }
        self.server_name = stream.get_string()?; // Use get_string for length-prefixed string
        Ok(())
    }
}

// Implement the OfflineMessage trait
impl OfflineMessage for UnconnectedPong {}