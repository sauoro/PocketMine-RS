// src/raknet/protocol/unconnected_pong.rs
#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{Result as BinaryResult};

#[derive(Debug, Clone)]
pub struct UnconnectedPong {
    pub send_ping_time: u64,
    pub server_id: u64, // Server GUID
    pub server_name: String,
}

impl UnconnectedPong {
    pub const ID: u8 = MessageIdentifiers::ID_UNCONNECTED_PONG;

    pub fn create(send_ping_time: u64, server_id: u64, server_name: String) -> Self {
        Self { send_ping_time, server_id, server_name }
    }
}

impl Packet for UnconnectedPong {
    fn get_id() -> u8 { Self::ID }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        stream.put_long(self.send_ping_time)?;
        stream.put_long(self.server_id)?;
        OfflineMessage::write_magic(stream);
        stream.put_string(&self.server_name);
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        self.send_ping_time = stream.get_long()?;
        self.server_id = stream.get_long()?;
        OfflineMessage::read_magic(stream)?;
        self.server_name = stream.get_string()?;
        Ok(())
    }
}

impl OfflineMessage for UnconnectedPong {}