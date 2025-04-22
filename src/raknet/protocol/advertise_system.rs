// src/raknet/protocol/advertise_system.rs
#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{BinaryDataException, Result as BinaryResult};

#[derive(Debug, Clone)]
pub struct AdvertiseSystem {
    pub server_name: String,
}

impl AdvertiseSystem {
    pub const ID: u8 = MessageIdentifiers::ID_ADVERTISE_SYSTEM;
}

impl Packet for AdvertiseSystem {
    fn get_id() -> u8 { Self::ID }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        stream.put_string(&self.server_name);
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        self.server_name = stream.get_string()?;
        Ok(())
    }
}