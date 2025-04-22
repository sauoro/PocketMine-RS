// src/raknet/protocol/open_connection_request2.rs
#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::utils::internet_address::InternetAddress;
use crate::utils::error::{Result as BinaryResult};

#[derive(Debug, Clone)]
pub struct OpenConnectionRequest2 {
    pub server_address: InternetAddress,
    pub mtu_size: u16,
    pub client_id: i64, // Client GUID
}

impl OpenConnectionRequest2 {
    pub const ID: u8 = MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_2;
}

impl Packet for OpenConnectionRequest2 {
    fn get_id() -> u8 { Self::ID }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        OfflineMessage::write_magic(stream);
        stream.put_address(&self.server_address)?;
        stream.put_short(self.mtu_size)?;
        stream.put_long(self.client_id as u64)?; // Cast needed
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        OfflineMessage::read_magic(stream)?;
        self.server_address = stream.get_address()?;
        self.mtu_size = stream.get_short()?;
        self.client_id = stream.get_long()? as i64; // Cast needed
        Ok(())
    }
}

impl OfflineMessage for OpenConnectionRequest2 {}