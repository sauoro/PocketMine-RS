// src/raknet/protocol/unconnected_ping.rs
#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{Result as BinaryResult};

#[derive(Debug, Clone)]
pub struct UnconnectedPing {
    pub send_ping_time: u64,
    pub client_id: i64, // Client GUID
}

impl UnconnectedPing {
    pub const ID: u8 = MessageIdentifiers::ID_UNCONNECTED_PING;
}

impl Packet for UnconnectedPing {
    fn get_id() -> u8 { Self::ID }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        stream.put_long(self.send_ping_time)?;
        OfflineMessage::write_magic(stream);
        stream.put_long(self.client_id as u64)?; // Cast needed
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        self.send_ping_time = stream.get_long()?;
        OfflineMessage::read_magic(stream)?;
        self.client_id = stream.get_long()? as i64; // Cast needed
        Ok(())
    }
}

impl OfflineMessage for UnconnectedPing {}