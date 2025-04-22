// src/raknet/generic/disconnect_reason.rs

#![allow(dead_code)]

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // Use u8 representation for potential serialization/matching
pub enum DisconnectReason {
    ClientDisconnect = 0,
    ServerDisconnect = 1,
    PeerTimeout = 2,
    ClientReconnect = 3,
    ServerShutdown = 4,
    SplitPacketTooLarge = 5,
    SplitPacketTooManyConcurrent = 6,
    SplitPacketInvalidPartIndex = 7,
    SplitPacketInconsistentHeader = 8,
    // Add an Unknown variant for robustness
    Unknown(u8),
}

impl DisconnectReason {
    pub fn to_string(&self) -> String {
        match self {
            DisconnectReason::ClientDisconnect => "client disconnect".to_string(),
            DisconnectReason::ServerDisconnect => "server disconnect".to_string(),
            DisconnectReason::PeerTimeout => "timeout".to_string(),
            DisconnectReason::ClientReconnect => "new session established on same address and port".to_string(),
            DisconnectReason::ServerShutdown => "server shutdown".to_string(),
            DisconnectReason::SplitPacketTooLarge => "received packet split into more parts than allowed".to_string(),
            DisconnectReason::SplitPacketTooManyConcurrent => "too many received split packets being reassembled at once".to_string(),
            DisconnectReason::SplitPacketInvalidPartIndex => "invalid split packet part index".to_string(),
            DisconnectReason::SplitPacketInconsistentHeader => "received split packet header inconsistent with previous fragments".to_string(),
            DisconnectReason::Unknown(reason) => format!("Unknown reason {}", reason),
        }
    }

    // Optional: Allow conversion from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => DisconnectReason::ClientDisconnect,
            1 => DisconnectReason::ServerDisconnect,
            2 => DisconnectReason::PeerTimeout,
            3 => DisconnectReason::ClientReconnect,
            4 => DisconnectReason::ServerShutdown,
            5 => DisconnectReason::SplitPacketTooLarge,
            6 => DisconnectReason::SplitPacketTooManyConcurrent,
            7 => DisconnectReason::SplitPacketInvalidPartIndex,
            8 => DisconnectReason::SplitPacketInconsistentHeader,
            other => DisconnectReason::Unknown(other),
        }
    }
}

impl fmt::Display for DisconnectReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}