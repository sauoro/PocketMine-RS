// src/raknet/protocol/connected_pong.rs

#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::connected_packet::ConnectedPacket; // Import the marker trait
use crate::impl_packet_codec; // Import the macro

#[derive(Debug, Clone)]
pub struct ConnectedPong {
    pub send_ping_time: i64,
    pub send_pong_time: i64,
}

impl ConnectedPong {
    // Equivalent to PHP's static create method
    pub fn create(send_ping_time: i64, send_pong_time: i64) -> Self {
        Self { send_ping_time, send_pong_time }
    }
}

// Use the macro to generate encode/decode for the two 'long' fields
impl_packet_codec!(ConnectedPong(MessageIdentifiers::ID_CONNECTED_PONG) {
    send_ping_time: long,
    send_pong_time: long
});

// Implement the marker trait
impl ConnectedPacket for ConnectedPong {}