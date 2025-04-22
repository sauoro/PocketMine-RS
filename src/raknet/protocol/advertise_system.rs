// src/raknet/protocol/advertise_system.rs

#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::protocol::offline_message::OfflineMessage; // Import trait
use crate::utils::error::{Result, BinaryDataException};

#[derive(Debug, Clone)]
pub struct AdvertiseSystem {
    // Magic bytes handled by OfflineMessage trait
    pub server_name: String,
}

impl AdvertiseSystem {
    pub fn new(server_name: String) -> Self {
        Self { server_name }
    }
}

impl Packet for AdvertiseSystem {
    fn get_id(&self) -> u8 {
        // Note: Although often embedded, it can also be sent standalone with ID 0x1d
        MessageIdentifiers::ID_ADVERTISE_SYSTEM
    }

    // Custom encode_payload to include magic bytes
    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        // According to Wireshark captures and common use (like in UnconnectedPong),
        // AdvertiseSystem *itself* doesn't usually include the magic bytes when sent standalone.
        // The magic bytes are part of the *OfflineMessage* framing.
        // However, the PHP code implements encode/decode *without* magic here.
        // Let's follow the PHP code structure first. If this packet is *only* ever
        // embedded within other OfflineMessages (like UnconnectedPong), this is correct.
        // If it can be sent standalone as an offline message, it would need magic.
        // Sticking to the PHP structure for now: No magic here.
        stream.put_string(&self.server_name)?;
        Ok(())
    }

    // Custom decode_payload
    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        // Follow PHP structure: Assumes magic (if applicable) was handled by the caller/wrapper.
        self.server_name = stream.get_string()?;
        Ok(())
    }
}

// NOTE: We are *not* implementing OfflineMessage for this struct itself,
// because the PHP code doesn't, suggesting it's either embedded in other
// offline messages (which handle magic) or sent as a *connected* packet in some contexts,
// or the PHP code might be slightly inconsistent with standard RakNet offline framing.
// If it needs to be sent standalone *offline*, the sender must wrap it or handle magic.
// For now, we match the provided PHP structure.
// If used embedded in UnconnectedPong, the UnconnectedPong struct handles the magic bytes.

// It might be considered a ConnectedPacket if sent after connection? Unlikely for AdvertiseSystem.
// Let's omit the ConnectedPacket trait for now unless a specific use case requires it.
// impl ConnectedPacket for AdvertiseSystem {}