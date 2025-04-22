// src/raknet/protocol/disconnection_notification.rs

#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::connected_packet::ConnectedPacket; // Import the marker trait
use crate::impl_packet_codec; // Import the macro

// This packet has no payload, only the ID.
#[derive(Debug, Clone, Default)]
pub struct DisconnectionNotification;

// Use the macro variant for packets with no payload
impl_packet_codec!(DisconnectionNotification(MessageIdentifiers::ID_DISCONNECTION_NOTIFICATION) {});

// Implement the marker trait
impl ConnectedPacket for DisconnectionNotification {}