// src/raknet/protocol/open_connection_reply_2.rs

#![allow(dead_code)]

use crate::raknet::protocol::packet::{Packet};
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::utils::internet_address::InternetAddress;
use crate::utils::error::{Result, BinaryDataException};

#[derive(Debug, Clone)]
pub struct OpenConnectionReply2 {
    // Magic handled by OfflineMessage trait
    pub server_id: i64,
    pub client_address: InternetAddress, // The address of the client connecting
    pub mtu_size: u16,
    pub server_security: bool, // Encryption enabled flag (usually false for MCPE)
}

impl OpenConnectionReply2 {
    /// Creates a new reply packet.
    pub fn create(server_id: i64, client_address: InternetAddress, mtu_size: u16, server_security: bool) -> Self {
        Self {
            server_id,
            client_address,
            mtu_size,
            server_security,
        }
    }
}

impl Packet for OpenConnectionReply2 {
    fn get_id(&self) -> u8 {
        MessageIdentifiers::ID_OPEN_CONNECTION_REPLY_2
    }

    // Custom payload encoding
    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        self.write_magic(stream)?;
        stream.put_long(self.server_id)?;
        stream.put_address(&self.client_address)?;
        stream.put_short(self.mtu_size)?;
        stream.put_byte(if self.server_security { 1 } else { 0 });
        Ok(())
    }

    // Custom payload decoding
    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        let magic = self.read_magic(stream)?;
        if !self.is_valid_magic(&magic) {
            return Err(BinaryDataException::from_str("Invalid magic bytes"));
        }
        self.server_id = stream.get_long()?;
        self.client_address = stream.get_address()?;
        self.mtu_size = stream.get_short()?;
        self.server_security = stream.get_byte()? != 0;
        Ok(())
    }
}

// Implement the OfflineMessage trait
impl OfflineMessage for OpenConnectionReply2 {}