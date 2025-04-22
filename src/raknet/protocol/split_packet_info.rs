// src/raknet/protocol/split_packet_info.rs

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitPacketInfo {
    /// The ID identifying which split packet this fragment belongs to.
    id: u16, // split ID is u16 in RakNet
    /// The index of this fragment within the split packet (0-based).
    part_index: u32, // split index is u32
    /// The total number of fragments this packet is split into.
    total_part_count: u32, // split count is u32
}

impl SplitPacketInfo {
    pub fn new(id: u16, part_index: u32, total_part_count: u32) -> Self {
        // Basic validation can be added here if needed
        // e.g., part_index < total_part_count
        Self { id, part_index, total_part_count }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn part_index(&self) -> u32 {
        self.part_index
    }

    pub fn total_part_count(&self) -> u32 {
        self.total_part_count
    }
}