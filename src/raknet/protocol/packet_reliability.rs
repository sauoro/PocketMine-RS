// src/raknet/protocol/packet_reliability.rs

#![allow(dead_code, non_upper_case_globals)] // Allow constants not strictly uppercase

// From https://github.com/OculusVR/RakNet/blob/master/Source/PacketPriority.h
// Direct mapping of the PHP constants and helper functions

pub struct PacketReliability;

impl PacketReliability {
    pub const UNRELIABLE: u8 = 0;
    pub const UNRELIABLE_SEQUENCED: u8 = 1;
    pub const RELIABLE: u8 = 2;
    pub const RELIABLE_ORDERED: u8 = 3;
    pub const RELIABLE_SEQUENCED: u8 = 4;

    // Not sent on the wire, used internally by RakNet/RakLib
    pub const UNRELIABLE_WITH_ACK_RECEIPT: u8 = 5;
    pub const RELIABLE_WITH_ACK_RECEIPT: u8 = 6;
    pub const RELIABLE_ORDERED_WITH_ACK_RECEIPT: u8 = 7;

    pub const MAX_ORDER_CHANNELS: usize = 32;

    #[inline]
    pub const fn is_reliable(reliability: u8) -> bool {
        matches!(reliability,
            Self::RELIABLE |
            Self::RELIABLE_ORDERED |
            Self::RELIABLE_SEQUENCED |
            Self::RELIABLE_WITH_ACK_RECEIPT | // Include internal types for completeness
            Self::RELIABLE_ORDERED_WITH_ACK_RECEIPT
        )
    }

    #[inline]
    pub const fn is_sequenced(reliability: u8) -> bool {
        matches!(reliability,
            Self::UNRELIABLE_SEQUENCED |
            Self::RELIABLE_SEQUENCED
        )
    }

    #[inline]
    pub const fn is_ordered(reliability: u8) -> bool {
        matches!(reliability,
            Self::RELIABLE_ORDERED |
            Self::RELIABLE_ORDERED_WITH_ACK_RECEIPT // Include internal type
        )
    }

    #[inline]
    pub const fn is_sequenced_or_ordered(reliability: u8) -> bool {
        // Combine the checks, avoiding redundant matches
        matches!(reliability,
            Self::UNRELIABLE_SEQUENCED |
            Self::RELIABLE_ORDERED |
            Self::RELIABLE_SEQUENCED |
            Self::RELIABLE_ORDERED_WITH_ACK_RECEIPT // Include internal type
        )
        // Or simply: Self::is_sequenced(reliability) || Self::is_ordered(reliability)
    }
}