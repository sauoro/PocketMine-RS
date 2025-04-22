// src/raknet/protocol/unconnected_ping_open_connections.rs
#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::protocol::unconnected_ping::UnconnectedPing;
use crate::utils::error::{Result as BinaryResult};

#[derive(Debug, Clone)]
pub struct UnconnectedPingOpenConnections {
    // Inherits fields from UnconnectedPing
    pub send_ping_time: u64,
    pub client_id: i64,
}

impl UnconnectedPingOpenConnections {
    pub const ID: u8 = MessageIdentifiers::ID_UNCONNECTED_PING_OPEN_CONNECTIONS;
}

// Delegate encoding/decoding to UnconnectedPing
impl Packet for UnconnectedPingOpenConnections {
    fn get_id() -> u8 { Self::ID }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        let mut base = UnconnectedPing {
            send_ping_time: self.send_ping_time,
            client_id: self.client_id,
        };
        base.encode_payload(stream)
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        let mut base = UnconnectedPing {
            send_ping_time: 0,
            client_id: 0,
        };
        base.decode_payload(stream)?;
        self.send_ping_time = base.send_ping_time;
        self.client_id = base.client_id;
        Ok(())
    }
}

impl OfflineMessage for UnconnectedPingOpenConnections {}
