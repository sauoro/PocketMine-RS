// src/raknet/protocol/open_connection_request_2.rs

#![allow(dead_code)]

use crate::raknet::protocol::packet::{Packet};
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::utils::internet_address::InternetAddress;
use crate::utils::error::{Result, BinaryDataException};

#[derive(Debug, Clone)]
pub struct OpenConnectionRequest2 {
    // Magic handled by OfflineMessage trait
    pub server_address: InternetAddress,
    pub mtu_size: u16,
    pub client_id: i64,
}

impl OpenConnectionRequest2 {
    /// Creates a new request packet.
    pub fn new(server_address: InternetAddress, mtu_size: u16, client_id: i64) -> Self {
        Self {
            server_address,
            mtu_size,
            client_id,
        }
    }
}

impl Packet for OpenConnectionRequest2 {
    fn get_id(&self) -> u8 {
        MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_2
    }

    // Custom payload encoding
    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        self.write_magic(stream)?;
        stream.put_address(&self.server_address)?;
        stream.put_short(self.mtu_size)?;
        stream.put_long(self.client_id)?;
        Ok(())
    }

    // Custom payload decoding
    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        let magic = self.read_magic(stream)?;
        if !self.is_valid_magic(&magic) {
            return Err(BinaryDataException::from_str("Invalid magic bytes"));
        }
        self.server_address = stream.get_address()?;
        self.mtu_size = stream.get_short()?;
        self.client_id = stream.get_long()?;
        Ok(())
    }
}

// Implement the OfflineMessage trait
impl OfflineMessage for OpenConnectionRequest2 {}