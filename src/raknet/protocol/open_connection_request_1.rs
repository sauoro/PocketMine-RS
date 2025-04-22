// src/raknet/protocol/open_connection_request_1.rs

#![allow(dead_code)]

use crate::raknet::protocol::packet::{Packet};
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{Result, BinaryDataException};
use crate::raknet::DEFAULT_PROTOCOL_VERSION; // Import default protocol version
use std::convert::TryInto; // For usize -> u16 conversion checking

#[derive(Debug, Clone)]
pub struct OpenConnectionRequest1 {
    // Magic handled by OfflineMessage trait
    pub protocol: u8,
    /// The MTU size is determined by the length of the padding added.
    /// It's not directly serialized but calculated based on the packet length.
    /// We store it after decoding or set it before encoding to control padding.
    pub mtu_size: u16,
}

impl OpenConnectionRequest1 {
    /// Creates a new request with the specified MTU size.
    pub fn new(mtu_size: u16) -> Self {
        Self {
            protocol: DEFAULT_PROTOCOL_VERSION,
            mtu_size,
        }
    }
}

impl Packet for OpenConnectionRequest1 {
    fn get_id(&self) -> u8 {
        MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_1
    }

    // Custom payload encoding to include magic, protocol, and padding
    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        self.write_magic(stream)?;
        stream.put_byte(self.protocol);

        // Calculate current length (ID + Magic + Protocol Byte)
        let current_len = 1 + 16 + 1;
        let target_len: usize = self.mtu_size.into();

        if target_len <= current_len {
            // MTU size is too small to even fit the header, magic, and protocol.
            // Or, no padding needed if exactly equal.
            // PHP RakLib allows this, let's allow it too but maybe log a warning elsewhere.
            // If target_len < current_len, this might be an error state.
            if target_len < current_len {
                return Err(BinaryDataException::new(format!(
                    "Target MTU size {} is smaller than required header size {}",
                    target_len, current_len
                )));
            }
            // If target_len == current_len, no padding needed.
        } else {
            let padding_len = target_len - current_len;
            // Create a padding buffer. Avoid large allocations if padding_len is huge.
            // Consider adding a reasonable upper limit check on mtu_size / padding_len.
            let padding = vec![0u8; padding_len];
            stream.put(&padding);
        }
        Ok(())
    }

    // Custom payload decoding to read magic, protocol, and infer MTU size
    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        // Get initial buffer length *before* reading anything from payload
        // The total length read by the socket includes the packet ID (1 byte)
        // We need the total size of the UDP datagram payload received.
        // Assuming the PacketSerializer was created `from_slice` with the raw UDP payload:
        let total_payload_len = stream.get_buffer().len(); // Get length of the underlying buffer

        let magic = self.read_magic(stream)?;
        if !self.is_valid_magic(&magic) {
            return Err(BinaryDataException::from_str("Invalid magic bytes"));
        }
        self.protocol = stream.get_byte()?;

        // The rest of the buffer is padding. The total length determines the client's MTU guess.
        // The MTU size is the total length of the packet (ID byte + payload).
        // Since the ID was already read by the caller (typically), the MTU is total_payload_len + 1
        // However, the PHP code calculates it based on the length of the *entire stream buffer*
        // which implies it's the length of the datagram received.
        self.mtu_size = total_payload_len.try_into().map_err(|_| {
            BinaryDataException::from_str("Packet length exceeds u16 max, cannot determine MTU size")
        })?;

        // Consume the remaining padding bytes to mark the stream as fully read
        stream.get_remaining()?;

        Ok(())
    }
}

// Implement the OfflineMessage trait
impl OfflineMessage for OpenConnectionRequest1 {}