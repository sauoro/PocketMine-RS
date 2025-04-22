// src/raknet/protocol/connected_packet.rs

#![allow(dead_code)]

use crate::raknet::protocol::packet::Packet;
use std::fmt::Debug; // Import Debug

/// A marker trait for packets that are sent/received *after* a RakNet connection
/// is established (i.e., they are typically wrapped in an EncapsulatedPacket).
/// It inherits from the base `Packet` trait.
pub trait ConnectedPacket: Packet + Debug + Send + Sync {}

// Allow Box<dyn ConnectedPacket> to be treated as a ConnectedPacket reference
// This might be useful for dynamic dispatch specific to connected packets.
impl<T: ConnectedPacket + ?Sized> ConnectedPacket for Box<T> {}