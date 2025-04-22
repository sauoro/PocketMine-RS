// src/raknet/generic/disconnect_reason.rs
#![allow(dead_code)]

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
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
    BadPacket = 9, // Added for generic packet errors
    Unknown = 255,
}

impl DisconnectReason {
    pub fn to_string(&self) -> &'static str {
        match self {
            DisconnectReason::ClientDisconnect => "client disconnect",
            DisconnectReason::ServerDisconnect => "server disconnect",
            DisconnectReason::PeerTimeout => "timeout",
            DisconnectReason::ClientReconnect => "new session established on same address and port",
            DisconnectReason::ServerShutdown => "server shutdown",
            DisconnectReason::SplitPacketTooLarge => "received packet split into more parts than allowed",
            DisconnectReason::SplitPacketTooManyConcurrent => "too many received split packets being reassembled at once",
            DisconnectReason::SplitPacketInvalidPartIndex => "invalid split packet part index",
            DisconnectReason::SplitPacketInconsistentHeader => "received split packet header inconsistent with previous fragments",
            DisconnectReason::BadPacket => "bad packet received",
            DisconnectReason::Unknown => "unknown reason",
        }
    }

    pub fn from_u8(reason: u8) -> Self {
        match reason {
            0 => Self::ClientDisconnect,
            1 => Self::ServerDisconnect,
            2 => Self::PeerTimeout,
            3 => Self::ClientReconnect,
            4 => Self::ServerShutdown,
            5 => Self::SplitPacketTooLarge,
            6 => Self::SplitPacketTooManyConcurrent,
            7 => Self::SplitPacketInvalidPartIndex,
            8 => Self::SplitPacketInconsistentHeader,
            9 => Self::BadPacket,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Display for DisconnectReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}