// src/raknet/protocol/open_connection_request1.rs
#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::raknet::DEFAULT_PROTOCOL_VERSION;
use crate::utils::error::{Result as BinaryResult};
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct OpenConnectionRequest1 {
    pub protocol: u8,
    pub mtu_size: u16, // Size of the entire UDP packet received
}

impl OpenConnectionRequest1 {
    pub const ID: u8 = MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_1;

    pub fn create(mtu_size: u16) -> Self {
        Self {
            protocol: DEFAULT_PROTOCOL_VERSION,
            mtu_size,
        }
    }
}

impl Packet for OpenConnectionRequest1 {
    fn get_id() -> u8 { Self::ID }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        OfflineMessage::write_magic(stream);
        stream.put_byte(self.protocol);
        // Pad with null bytes to match MTU size
        let current_len = stream.get_buffer().len();
        let padding_len = self.mtu_size.saturating_sub(current_len.try_into().unwrap_or(u16::MAX)) as usize;
        if padding_len > 0 {
            stream.put_slice(&vec![0u8; padding_len]);
        }
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        // Store the original buffer length before reading payload
        // Note: The MTU size is implicitly the size of the received datagram.
        // The padding is usually ignored on decode.
        self.mtu_size = stream.len().try_into().unwrap_or(u16::MAX); // Approximate MTU from packet size
        OfflineMessage::read_magic(stream)?;
        self.protocol = stream.get_byte()?;
        stream.skip(stream.remaining()); // Consume any padding
        Ok(())
    }
}

impl OfflineMessage for OpenConnectionRequest1 {}
