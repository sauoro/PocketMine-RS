// src/raknet/protocol/unconnected_ping_open_connections.rs

#![allow(dead_code)]

use crate::raknet::protocol::packet::{Packet};
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{Result, BinaryDataException};

// This packet has the exact same structure as UnconnectedPing, just a different ID.
// We define it as a separate struct for type safety and clarity.
#[derive(Debug, Clone)]
pub struct UnconnectedPingOpenConnections {
    pub send_ping_time: i64,
    // Magic is handled by the OfflineMessage trait
    pub client_id: i64,
}

impl UnconnectedPingOpenConnections {
    // Optional: Provide a constructor if needed, though fields are public
    pub fn new(send_ping_time: i64, client_id: i64) -> Self {
        Self { send_ping_time, client_id }
    }
}

// Implement the base Packet trait
impl Packet for UnconnectedPingOpenConnections {
    fn get_id(&self) -> u8 {
        MessageIdentifiers::ID_UNCONNECTED_PING_OPEN_CONNECTIONS
    }

    // Use the same payload encoding as UnconnectedPing
    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        stream.put_long(self.send_ping_time)?;
        self.write_magic(stream)?; // Use OfflineMessage trait method
        stream.put_long(self.client_id)?;
        Ok(())
    }

    // Use the same payload decoding as UnconnectedPing
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

// Implement the OfflineMessage trait
impl OfflineMessage for UnconnectedPingOpenConnections {}

// Optional: Implement conversion from UnconnectedPing if useful
// impl From<UnconnectedPing> for UnconnectedPingOpenConnections {
//     fn from(ping: UnconnectedPing) -> Self {
//         Self {
//             send_ping_time: ping.send_ping_time,
//             client_id: ping.client_id,
//         }
//     }
// }