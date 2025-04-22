// src/raknet/protocol/new_incoming_connection.rs

#![allow(dead_code)]

use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::protocol::connected_packet::ConnectedPacket; // Marker trait
use crate::raknet::utils::internet_address::InternetAddress;
use crate::raknet::SYSTEM_ADDRESS_COUNT; // Import constant
use crate::utils::error::{Result, BinaryDataException};
use std::mem::MaybeUninit; // Needed for safe array initialization

#[derive(Debug, Clone)]
pub struct NewIncomingConnection {
    /// The server's address as seen by the client (from ConnectionRequestAccepted).
    pub server_address: InternetAddress,
    /// RakNet internal system addresses (usually received from server, echoed back).
    pub system_addresses: [InternetAddress; SYSTEM_ADDRESS_COUNT],
    /// The server's timestamp from ConnectionRequestAccepted.
    pub send_ping_time: i64,
    /// The client's current timestamp when sending this packet.
    pub send_pong_time: i64,
}

impl NewIncomingConnection {
    /// Creates a new notification packet.
    pub fn new(
        server_address: InternetAddress,
        system_addresses: [InternetAddress; SYSTEM_ADDRESS_COUNT], // Typically copied from ConnectionRequestAccepted
        send_ping_time: i64, // Copied from ConnectionRequestAccepted
        send_pong_time: i64, // Client's current time
    ) -> Self {
        Self {
            server_address,
            system_addresses,
            send_ping_time,
            send_pong_time,
        }
    }
}

// Manual implementation needed due to address array.
impl Packet for NewIncomingConnection {
    fn get_id(&self) -> u8 {
        MessageIdentifiers::ID_NEW_INCOMING_CONNECTION
    }

    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        stream.put_address(&self.server_address)?;
        // Write the system addresses
        for addr in &self.system_addresses {
            stream.put_address(addr)?;
        }
        stream.put_long(self.send_ping_time)?;
        stream.put_long(self.send_pong_time)?;
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        self.server_address = stream.get_address()?;

        // Use MaybeUninit for safe array initialization during read
        let mut addresses_uninit: [MaybeUninit<InternetAddress>; SYSTEM_ADDRESS_COUNT] =
            unsafe { MaybeUninit::uninit().assume_init() };

        // Read the system addresses
        for i in 0..SYSTEM_ADDRESS_COUNT {
            if stream.feof() {
                // Handle EOF or truncated packet
                let dummy_addr_str = "0.0.0.0".to_string();
                // Initialize remaining with dummies
                for j in i..SYSTEM_ADDRESS_COUNT {
                    addresses_uninit[j] = MaybeUninit::new(InternetAddress::new(dummy_addr_str.clone(), 0, 4).unwrap());
                }
                break; // Stop reading further
            } else {
                addresses_uninit[i] = MaybeUninit::new(stream.get_address()?);
            }
        }

        // Assign the safely initialized array back
        self.system_addresses = unsafe {
            (&addresses_uninit as *const _ as *const [InternetAddress; SYSTEM_ADDRESS_COUNT]).read()
        };

        self.send_ping_time = stream.get_long()?;
        self.send_pong_time = stream.get_long()?;
        Ok(())
    }
}

// Implement the marker trait
impl ConnectedPacket for NewIncomingConnection {}