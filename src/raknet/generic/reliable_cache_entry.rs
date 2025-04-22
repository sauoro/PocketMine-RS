// src/raknet/generic/reliable_cache_entry.rs
#![allow(dead_code)]

use crate::raknet::protocol::encapsulated_packet::EncapsulatedPacket;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ReliableCacheEntry {
    packets: Vec<EncapsulatedPacket>, // Store clones
    timestamp: Instant,
}

impl ReliableCacheEntry {
    pub fn new(packets: Vec<EncapsulatedPacket>) -> Self {
        Self {
            packets,
            timestamp: Instant::now(),
        }
    }

    pub fn get_packets(&self) -> &[EncapsulatedPacket] {
        &self.packets
    }

    pub fn get_timestamp(&self) -> Instant {
        self.timestamp
    }
}