// src/raknet/protocol/ack.rs
#![allow(dead_code)]

use crate::raknet::protocol::acknowledge_packet::AcknowledgePacket;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;

#[derive(Debug, Clone)]
pub struct Ack {
    pub packets: Vec<u32>,
}

impl Ack {
    pub const ID: u8 = MessageIdentifiers::ID_ACK;

    pub fn new() -> Self {
        Self { packets: Vec::new() }
    }
}

impl AcknowledgePacket for Ack {
    fn get_id() -> u8 { Ack::ID }
    fn get_packets(&self) -> &[u32] { &self.packets }
    fn get_packets_mut(&mut self) -> &mut Vec<u32> { &mut self.packets }
}