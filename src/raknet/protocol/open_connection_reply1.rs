// src/raknet/protocol/open_connection_reply1.rs
#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{Result as BinaryResult};

#[derive(Debug, Clone)]
pub struct OpenConnectionReply1 {
    pub server_id: u64, // Server GUID
    pub server_security: bool,
    pub mtu_size: u16,
}

impl OpenConnectionReply1 {
    pub const ID: u8 = MessageIdentifiers::ID_OPEN_CONNECTION_REPLY_1;

    pub fn create(server_id: u64, server_security: bool, mtu_size: u16) -> Self {
        Self { server_id, server_security, mtu_size }
    }
}

impl Packet for OpenConnectionReply1 {
    fn get_id() -> u8 { Self::ID }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        OfflineMessage::write_magic(stream);
        stream.put_long(self.server_id)?;
        stream.put_byte(if self.server_security { 1 } else { 0 });
        stream.put_short(self.mtu_size)?;
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        OfflineMessage::read_magic(stream)?;
        self.server_id = stream.get_long()?;
        self.server_security = stream.get_byte()? != 0;
        self.mtu_size = stream.get_short()?;
        Ok(())
    }
}

impl OfflineMessage for OpenConnectionReply1 {}