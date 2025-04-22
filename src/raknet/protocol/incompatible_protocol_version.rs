// src/raknet/protocol/incompatible_protocol_version.rs

#![allow(dead_code)]

use crate::raknet::protocol::packet::{Packet};
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{Result, BinaryDataException};

#[derive(Debug, Clone)]
pub struct IncompatibleProtocolVersion {
    pub protocol_version: u8, // The protocol version the client requested
    // Magic handled by OfflineMessage trait
    pub server_id: i64, // The server's unique ID
}

impl IncompatibleProtocolVersion {
    /// Creates a new rejection packet.
    pub fn create(protocol_version: u8, server_id: i64) -> Self {
        Self {
            protocol_version,
            server_id,
        }
    }
}

impl Packet for IncompatibleProtocolVersion {
    fn get_id(&self) -> u8 {
        MessageIdentifiers::ID_INCOMPATIBLE_PROTOCOL_VERSION
    }

    // Custom payload encoding
    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        stream.put_byte(self.protocol_version);
        self.write_magic(stream)?;
        stream.put_long(self.server_id)?;
        Ok(())
    }

    // Custom payload decoding
    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        self.protocol_version = stream.get_byte()?;
        let magic = self.read_magic(stream)?;
        if !self.is_valid_magic(&magic) {
            return Err(BinaryDataException::from_str("Invalid magic bytes"));
        }
        self.server_id = stream.get_long()?;
        Ok(())
    }
}

// Implement the OfflineMessage trait
impl OfflineMessage for IncompatibleProtocolVersion {}