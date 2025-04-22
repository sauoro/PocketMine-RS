// src/raknet/protocol/connection_request.rs
#![allow(dead_code)]

use crate::raknet::protocol::connected_packet::ConnectedPacket;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{Result as BinaryResult};

#[derive(Debug, Clone)]
pub struct ConnectionRequest {
    pub client_id: i64, // RakNet uses signed long for GUID/ClientID
    pub send_ping_time: u64,
    pub use_security: bool,
}

impl ConnectionRequest {
    pub const ID: u8 = MessageIdentifiers::ID_CONNECTION_REQUEST;
}

impl Packet for ConnectionRequest {
    fn get_id() -> u8 { Self::ID }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        stream.put_long(self.client_id as u64)?; // Cast needed for BinaryStream
        stream.put_long(self.send_ping_time)?;
        stream.put_byte(if self.use_security { 1 } else { 0 });
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        self.client_id = stream.get_long()? as i64; // Cast needed
        self.send_ping_time = stream.get_long()?;
        self.use_security = stream.get_byte()? != 0;
        Ok(())
    }
}

impl ConnectedPacket for ConnectionRequest {}