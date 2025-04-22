// src/raknet/protocol/unconnected_ping.rs

#![allow(dead_code)]

use crate::raknet::protocol::packet::{Packet};
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{Result, BinaryDataException}; // Import Result and specific error

#[derive(Debug, Clone)]
pub struct UnconnectedPing {
    pub send_ping_time: i64,
    // Magic is handled by the OfflineMessage trait, not stored here
    pub client_id: i64,
}

impl UnconnectedPing {
    pub fn new(send_ping_time: i64, client_id: i64) -> Self {
        Self { send_ping_time, client_id }
    }
}

// Implement the base Packet trait
impl Packet for UnconnectedPing {
    fn get_id(&self) -> u8 {
        MessageIdentifiers::ID_UNCONNECTED_PING
    }

    // Custom payload encoding to include magic bytes
    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        stream.put_long(self.send_ping_time)?;
        self.write_magic(stream)?; // Use OfflineMessage trait method
        stream.put_long(self.client_id)?;
        Ok(())
    }

    // Custom payload decoding to read and validate magic bytes
    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        self.send_ping_time = stream.get_long()?;
        let magic = self.read_magic(stream)?; // Use OfflineMessage trait method
        if !self.is_valid_magic(&magic) {
            return Err(BinaryDataException::from_str("Invalid magic bytes"));
        }
        self.client_id = stream.get_long()?;
        Ok(())
    }
}

// Implement the OfflineMessage trait to provide magic handling methods
impl OfflineMessage for UnconnectedPing {}