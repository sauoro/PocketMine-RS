// src/raknet/protocol/nack.rs

#![allow(dead_code)]

use crate::raknet::protocol::acknowledge_packet::AcknowledgePacketData;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::protocol::connected_packet::ConnectedPacket; // Marker trait
use crate::utils::error::Result;

// Define the specific NACK Packet ID
pub const ID_NACK: u8 = 0xa0;

#[derive(Debug, Clone, Default)]
pub struct Nack {
    /// The common data structure holding packet sequence numbers.
    pub data: AcknowledgePacketData,
}

impl Nack {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Packet for Nack {
    fn get_id(&self) -> u8 {
        ID_NACK
    }

    // Delegate payload encoding to the common data structure
    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        // Clone data to allow sorting within the encode method
        let mut data_clone = self.data.clone();
        data_clone.encode_payload(stream)
    }

    // Delegate payload decoding to the common data structure
    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        self.data.decode_payload(stream)
    }
}

// Implement the marker trait
impl ConnectedPacket for Nack {}