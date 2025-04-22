// src/raknet/protocol/connection_request.rs

#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::protocol::connected_packet::ConnectedPacket; // Import the marker trait
use crate::utils::error::Result;

#[derive(Debug, Clone)]
pub struct ConnectionRequest {
    pub client_id: i64, // Often called client GUID in RakNet docs
    pub send_ping_time: i64, // Timestamp from client
    pub use_security: bool, // Whether client requires/supports DTLS (usually false)
}

impl ConnectionRequest {
    pub fn new(client_id: i64, send_ping_time: i64, use_security: bool) -> Self {
        Self { client_id, send_ping_time, use_security }
    }
}

// Manual implementation because the macro doesn't handle bool -> byte conversion easily
impl Packet for ConnectionRequest {
    fn get_id(&self) -> u8 {
        MessageIdentifiers::ID_CONNECTION_REQUEST
    }

    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        stream.put_long(self.client_id)?;
        stream.put_long(self.send_ping_time)?;
        stream.put_byte(if self.use_security { 1 } else { 0 });
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        self.client_id = stream.get_long()?;
        self.send_ping_time = stream.get_long()?;
        self.use_security = stream.get_byte()? != 0;
        Ok(())
    }
}

// Implement the marker trait
impl ConnectedPacket for ConnectionRequest {}