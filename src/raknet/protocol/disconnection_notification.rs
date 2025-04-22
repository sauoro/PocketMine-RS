// src/raknet/protocol/disconnection_notification.rs
#![allow(dead_code)]

use crate::raknet::protocol::connected_packet::ConnectedPacket;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::Result as BinaryResult;

#[derive(Debug, Clone, Default)]
pub struct DisconnectionNotification;

impl DisconnectionNotification {
    pub const ID: u8 = MessageIdentifiers::ID_DISCONNECTION_NOTIFICATION;
}

impl Packet for DisconnectionNotification {
    fn get_id() -> u8 { Self::ID }

    fn encode_payload(&mut self, _stream: &mut PacketSerializer) -> BinaryResult<()> {
        // No payload
        Ok(())
    }

    fn decode_payload(&mut self, _stream: &mut PacketSerializer) -> BinaryResult<()> {
        // No payload
        Ok(())
    }
}

impl ConnectedPacket for DisconnectionNotification {}