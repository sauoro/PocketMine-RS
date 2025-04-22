// src/raknet/protocol/open_connection_reply2.rs
#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::utils::internet_address::InternetAddress;
use crate::utils::error::{Result as BinaryResult};

#[derive(Debug, Clone)]
pub struct OpenConnectionReply2 {
    pub server_id: u64, // Server GUID
    pub client_address: InternetAddress,
    pub mtu_size: u16,
    pub server_security: bool,
}

impl OpenConnectionReply2 {
    pub const ID: u8 = MessageIdentifiers::ID_OPEN_CONNECTION_REPLY_2;

    pub fn create(
        server_id: u64,
        client_address: InternetAddress,
        mtu_size: u16,
        server_security: bool,
    ) -> Self {
        Self { server_id, client_address, mtu_size, server_security }
    }
}

impl Packet for OpenConnectionReply2 {
    fn get_id() -> u8 { Self::ID }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        OfflineMessage::write_magic(stream);
        stream.put_long(self.server_id)?;
        stream.put_address(&self.client_address)?;
        stream.put_short(self.mtu_size)?;
        stream.put_byte(if self.server_security { 1 } else { 0 });
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        OfflineMessage::read_magic(stream)?;
        self.server_id = stream.get_long()?;
        self.client_address = stream.get_address()?;
        self.mtu_size = stream.get_short()?;
        self.server_security = stream.get_byte()? != 0;
        Ok(())
    }
}

impl OfflineMessage for OpenConnectionReply2 {}
