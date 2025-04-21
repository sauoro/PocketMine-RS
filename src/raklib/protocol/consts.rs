// src/raklib/protocol/consts.rs
#[allow(non_snake_case, non_upper_case_globals)]
pub mod MessageIdentifiers {
    pub const ID_CONNECTED_PING: u8 = 0x00;
    pub const ID_UNCONNECTED_PING: u8 = 0x01;
    pub const ID_UNCONNECTED_PING_OPEN_CONNECTIONS: u8 = 0x02; // <<< Added
    pub const ID_CONNECTED_PONG: u8 = 0x03; // <<< Added
    pub const ID_DETECT_LOST_CONNECTIONS: u8 = 0x04;
    pub const ID_OPEN_CONNECTION_REQUEST_1: u8 = 0x05;
    pub const ID_OPEN_CONNECTION_REPLY_1: u8 = 0x06;
    pub const ID_OPEN_CONNECTION_REQUEST_2: u8 = 0x07;
    pub const ID_OPEN_CONNECTION_REPLY_2: u8 = 0x08;
    pub const ID_CONNECTION_REQUEST: u8 = 0x09;
    pub const ID_REMOTE_SYSTEM_REQUIRES_PUBLIC_KEY: u8 = 0x0a;
    pub const ID_OUR_SYSTEM_REQUIRES_SECURITY: u8 = 0x0b;
    pub const ID_PUBLIC_KEY_MISMATCH: u8 = 0x0c;
    pub const ID_OUT_OF_BAND_INTERNAL: u8 = 0x0d;
    pub const ID_SND_RECEIPT_ACKED: u8 = 0x0e;
    pub const ID_SND_RECEIPT_LOSS: u8 = 0x0f;
    pub const ID_CONNECTION_REQUEST_ACCEPTED: u8 = 0x10;
    pub const ID_CONNECTION_ATTEMPT_FAILED: u8 = 0x11;
    pub const ID_ALREADY_CONNECTED: u8 = 0x12;
    pub const ID_NEW_INCOMING_CONNECTION: u8 = 0x13;
    pub const ID_NO_FREE_INCOMING_CONNECTIONS: u8 = 0x14;
    pub const ID_DISCONNECTION_NOTIFICATION: u8 = 0x15;
    pub const ID_CONNECTION_LOST: u8 = 0x16;
    pub const ID_CONNECTION_BANNED: u8 = 0x17;
    pub const ID_INVALID_PASSWORD: u8 = 0x18;
    pub const ID_INCOMPATIBLE_PROTOCOL_VERSION: u8 = 0x19;
    pub const ID_IP_RECENTLY_CONNECTED: u8 = 0x1a;
    pub const ID_TIMESTAMP: u8 = 0x1b;
    pub const ID_UNCONNECTED_PONG: u8 = 0x1c;
    pub const ID_ADVERTISE_SYSTEM: u8 = 0x1d; // <<< Added
    pub const ID_DOWNLOAD_PROGRESS: u8 = 0x1e;
    // ... (Add ALL other identifiers from PHP interface if needed later) ...
    // User packet IDs start here:
    pub const ID_USER_PACKET_ENUM: u8 = 0x86;
    // ACK/NACK range (often treated specially)
    pub const ID_NACK: u8 = 0xa0; // to 0xaf
    pub const ID_ACK: u8 = 0xc0; // to 0xcf
}

#[allow(non_snake_case, non_upper_case_globals)]
pub mod PacketReliability {
    pub const UNRELIABLE: u8 = 0;
    pub const UNRELIABLE_SEQUENCED: u8 = 1;
    pub const RELIABLE: u8 = 2;
    pub const RELIABLE_ORDERED: u8 = 3;
    pub const RELIABLE_SEQUENCED: u8 = 4;
    pub const UNRELIABLE_WITH_ACK_RECEIPT: u8 = 5; // Internal use usually
    pub const RELIABLE_WITH_ACK_RECEIPT: u8 = 6;
    pub const RELIABLE_ORDERED_WITH_ACK_RECEIPT: u8 = 7;

    pub const MAX_ORDER_CHANNELS: usize = 32;

    #[inline]
    pub fn is_reliable(reliability: u8) -> bool {
        matches!(
            reliability,
            RELIABLE
                | RELIABLE_ORDERED
                | RELIABLE_SEQUENCED
                | RELIABLE_WITH_ACK_RECEIPT
                | RELIABLE_ORDERED_WITH_ACK_RECEIPT
        )
    }

    #[inline]
    pub fn is_sequenced(reliability: u8) -> bool {
        matches!(reliability, UNRELIABLE_SEQUENCED | RELIABLE_SEQUENCED)
    }

    #[inline]
    pub fn is_ordered(reliability: u8) -> bool {
        matches!(reliability, RELIABLE_ORDERED | RELIABLE_ORDERED_WITH_ACK_RECEIPT)
    }

    #[inline]
    pub fn is_sequenced_or_ordered(reliability: u8) -> bool {
        matches!(
             reliability,
             UNRELIABLE_SEQUENCED | RELIABLE_ORDERED | RELIABLE_SEQUENCED | RELIABLE_ORDERED_WITH_ACK_RECEIPT
         )
    }
}