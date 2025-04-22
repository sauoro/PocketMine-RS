// src/raknet/protocol/ack.rs

#![allow(dead_code)]

use crate::raknet::protocol::acknowledge_packet::AcknowledgePacketData;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::protocol::connected_packet::ConnectedPacket; // Marker trait
use crate::utils::error::Result;

// Define the specific ACK Packet ID
// Note: RakNet standard doesn't define specific IDs for ACK/NACK in the MessageIdentifiers enum,
// they are identified by flags in the Datagram header. However, PM/RakLib uses 0xc0 and 0xa0
// as standalone packet IDs sometimes, likely for packets *outside* a Datagram context,
// or perhaps just as internal identifiers? We'll follow the PHP structure assigning specific IDs.
pub const ID_ACK: u8 = 0xc0;

#[derive(Debug, Clone, Default)]
pub struct Ack {
    /// The common data structure holding packet sequence numbers.
    pub data: AcknowledgePacketData,
}

impl Ack {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Packet for Ack {
    fn get_id(&self) -> u8 {
        ID_ACK
    }

    // Delegate payload encoding to the common data structure
    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        // We need a mutable borrow of data here temporarily for sorting,
        // but the Packet trait takes &self. We clone or handle mutability carefully.
        // Cloning is simpler for now. A more optimized version might avoid the clone.
        let mut data_clone = self.data.clone();
        data_clone.encode_payload(stream)
    }

    // Delegate payload decoding to the common data structure
    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        self.data.decode_payload(stream)
    }
}

// Implement the marker trait
impl ConnectedPacket for Ack {}