// src/raknet/protocol/connected_packet.rs
#![allow(dead_code)]

use crate::raknet::protocol::packet::Packet;

pub trait ConnectedPacket: Packet {}