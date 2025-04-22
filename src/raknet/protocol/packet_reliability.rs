// src/raknet/protocol/packet_reliability.rs
#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PacketReliability {
    Unreliable = 0,
    UnreliableSequenced = 1,
    Reliable = 2,
    ReliableOrdered = 3,
    ReliableSequenced = 4,
    // Internal use, not sent over wire
    UnreliableWithAckReceipt = 5,
    ReliableWithAckReceipt = 6,
    ReliableOrderedWithAckReceipt = 7,
}

impl PacketReliability {
    pub const UNRELIABLE: u8 = 0;
    pub const UNRELIABLE_SEQUENCED: u8 = 1;
    pub const RELIABLE: u8 = 2;
    pub const RELIABLE_ORDERED: u8 = 3;
    pub const RELIABLE_SEQUENCED: u8 = 4;
    pub const UNRELIABLE_WITH_ACK_RECEIPT: u8 = 5;
    pub const RELIABLE_WITH_ACK_RECEIPT: u8 = 6;
    pub const RELIABLE_ORDERED_WITH_ACK_RECEIPT: u8 = 7;

    pub const MAX_ORDER_CHANNELS: usize = 32;

    pub fn is_reliable(reliability: u8) -> bool {
        matches!(reliability,
            Self::RELIABLE |
            Self::RELIABLE_ORDERED |
            Self::RELIABLE_SEQUENCED |
            Self::RELIABLE_WITH_ACK_RECEIPT |
            Self::RELIABLE_ORDERED_WITH_ACK_RECEIPT
        )
    }

    pub fn is_sequenced(reliability: u8) -> bool {
        matches!(reliability,
            Self::UNRELIABLE_SEQUENCED |
            Self::RELIABLE_SEQUENCED
        )
    }

    pub fn is_ordered(reliability: u8) -> bool {
        matches!(reliability,
            Self::RELIABLE_ORDERED |
            Self::RELIABLE_ORDERED_WITH_ACK_RECEIPT
        )
    }

    pub fn is_sequenced_or_ordered(reliability: u8) -> bool {
        matches!(reliability,
            Self::UNRELIABLE_SEQUENCED |
            Self::RELIABLE_ORDERED |
            Self::RELIABLE_SEQUENCED |
            Self::RELIABLE_ORDERED_WITH_ACK_RECEIPT
        )
    }

    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(PacketReliability::Unreliable),
            1 => Some(PacketReliability::UnreliableSequenced),
            2 => Some(PacketReliability::Reliable),
            3 => Some(PacketReliability::ReliableOrdered),
            4 => Some(PacketReliability::ReliableSequenced),
            5 => Some(PacketReliability::UnreliableWithAckReceipt),
            6 => Some(PacketReliability::ReliableWithAckReceipt),
            7 => Some(PacketReliability::ReliableOrderedWithAckReceipt),
            _ => None,
        }
    }
}