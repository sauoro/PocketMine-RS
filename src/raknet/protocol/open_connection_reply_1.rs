// src/raknet/protocol/open_connection_reply_1.rs

#![allow(dead_code)]

use crate::raknet::protocol::packet::{Packet};
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{Result, BinaryDataException};

#[derive(Debug, Clone)]
pub struct OpenConnectionReply1 {
    // Magic handled by OfflineMessage trait
    pub server_id: i64,
    pub server_security: bool,
    pub mtu_size: u16,
}

impl OpenConnectionReply1 {
    /// Creates a new reply packet.
    pub fn create(server_id: i64, server_security: bool, mtu_size: u16) -> Self {
        Self {
            server_id,
            server_security,
            mtu_size,
        }
    }
}

impl Packet for OpenConnectionReply1 {
    fn get_id(&self) -> u8 {
        MessageIdentifiers::ID_OPEN_CONNECTION_REPLY_1
    }

    // Custom payload encoding for magic, server ID, security flag, and MTU size
    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        self.write_magic(stream)?;
        stream.put_long(self.server_id)?;
        stream.put_byte(if self.server_security { 1 } else { 0 });
        stream.put_short(self.mtu_size)?;
        Ok(())
    }

    // Custom payload decoding
    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        let magic = self.read_magic(stream)?;
        if !self.is_valid_magic(&magic) {
            return Err(BinaryDataException::from_str("Invalid magic bytes"));
        }
        self.server_id = stream.get_long()?;
        self.server_security = stream.get_byte()? != 0;
        self.mtu_size = stream.get_short()?;
        Ok(())
    }
}

// Implement the OfflineMessage trait
impl OfflineMessage for OpenConnectionReply1 {}