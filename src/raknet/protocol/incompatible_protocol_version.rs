// src/raknet/protocol/incompatible_protocol_version.rs
#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{Result as BinaryResult};

#[derive(Debug, Clone)]
pub struct IncompatibleProtocolVersion {
    pub protocol_version: u8,
    pub server_id: u64, // This is the server GUID in RakNet, represented as long
}

impl IncompatibleProtocolVersion {
    pub const ID: u8 = MessageIdentifiers::ID_INCOMPATIBLE_PROTOCOL_VERSION;

    pub fn create(protocol_version: u8, server_id: u64) -> Self {
        Self { protocol_version, server_id }
    }
}

impl Packet for IncompatibleProtocolVersion {
    fn get_id() -> u8 { Self::ID }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        stream.put_byte(self.protocol_version);
        OfflineMessage::write_magic(stream);
        stream.put_long(self.server_id)?;
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        self.protocol_version = stream.get_byte()?;
        OfflineMessage::read_magic(stream)?;
        self.server_id = stream.get_long()?;
        Ok(())
    }
}

impl OfflineMessage for IncompatibleProtocolVersion {}