// src/raknet/protocol/connected_ping.rs

#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::connected_packet::ConnectedPacket; // Import the marker trait
use crate::impl_packet_codec; // Import the macro

#[derive(Debug, Clone)]
pub struct ConnectedPing {
    pub send_ping_time: i64,
}

impl ConnectedPing {
    // Equivalent to PHP's static create method
    pub fn create(send_ping_time: i64) -> Self {
        Self { send_ping_time }
    }
}

// Use the macro to generate encode/decode for the single 'long' field
impl_packet_codec!(ConnectedPing(MessageIdentifiers::ID_CONNECTED_PING) {
    send_ping_time: long
});

// Implement the marker trait
impl ConnectedPacket for ConnectedPing {}