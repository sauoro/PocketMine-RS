// src/raknet/protocol/connected_pong.rs
#![allow(dead_code)]

use crate::raknet::protocol::connected_packet::ConnectedPacket;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{Result as BinaryResult};

#[derive(Debug, Clone)]
pub struct ConnectedPong {
    pub send_ping_time: u64,
    pub send_pong_time: u64,
}

impl ConnectedPong {
    pub const ID: u8 = MessageIdentifiers::ID_CONNECTED_PONG;

    pub fn create(send_ping_time: u64, send_pong_time: u64) -> Self {
        Self { send_ping_time, send_pong_time }
    }
}

impl Packet for ConnectedPong {
    fn get_id() -> u8 { Self::ID }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        stream.put_long(self.send_ping_time)?;
        stream.put_long(self.send_pong_time)?;
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        self.send_ping_time = stream.get_long()?;
        self.send_pong_time = stream.get_long()?;
        Ok(())
    }
}

impl ConnectedPacket for ConnectedPong {}