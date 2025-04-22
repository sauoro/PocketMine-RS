// src/raknet/protocol/connection_request_accepted.rs

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
pub struct ConnectionRequestAccepted {
    /// The client's address as seen by the server.
    pub client_address: InternetAddress,
    /// RakNet internal system addresses (usually unused/dummy in MCPE).
    /// We will serialize dummy addresses.
    pub system_addresses: [InternetAddress; SYSTEM_ADDRESS_COUNT],
    /// The client's timestamp from the ConnectionRequest.
    pub send_ping_time: i64,
    /// The server's current timestamp.
    pub send_pong_time: i64,
}

impl ConnectionRequestAccepted {
    /// Creates a new acceptance packet.
    /// `system_addresses` is usually ignored and filled with dummies.
    pub fn create(
        client_address: InternetAddress,
        send_ping_time: i64,
        send_pong_time: i64,
    ) -> Result<Self> {
        
        // 1. Create an array of MaybeUninit<InternetAddress>
        let mut addresses_uninit: [MaybeUninit<InternetAddress>; SYSTEM_ADDRESS_COUNT] =
            unsafe { MaybeUninit::uninit().assume_init() }; // Safe because we initialize below

        // 2. Initialize each element
        let dummy_addr_str = "0.0.0.0".to_string();
        for elem in &mut addresses_uninit[..] {
            // Use unwrap here because we know "0.0.0.0" is a valid address
            let addr = InternetAddress::new(dummy_addr_str.clone(), 0, 4).unwrap();
            *elem = MaybeUninit::new(addr);
        }

        // 3. Transmute the initialized array (this is safe because all elements are initialized)
        // This requires careful handling to ensure safety, but is a common pattern for array init.
        // A simpler, though potentially slightly less efficient approach, would be to
        // create a Vec and then try_into() the array, but that involves heap allocation.
        let system_addresses = unsafe {
            // Ensure the transmutation is safe by checking sizes and alignments if necessary,
            // though for simple types like this it's generally fine.
            // Reinterpret the pointer.
            (&addresses_uninit as *const _ as *const [InternetAddress; SYSTEM_ADDRESS_COUNT]).read()
        };

        Ok(Self {
            client_address,
            system_addresses,
            send_ping_time,
            send_pong_time,
        })
    }
}

// Manual implementation needed due to address array and short(0) field.
impl Packet for ConnectionRequestAccepted {
    fn get_id(&self) -> u8 {
        MessageIdentifiers::ID_CONNECTION_REQUEST_ACCEPTED
    }

    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        stream.put_address(&self.client_address)?;
        stream.put_short(0)?; // System index (always 0 according to RakNet source?)

        // Write the system addresses (mostly dummies)
        for addr in &self.system_addresses {
            stream.put_address(addr)?;
        }

        stream.put_long(self.send_ping_time)?;
        stream.put_long(self.send_pong_time)?;
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        self.client_address = stream.get_address()?;
        let _system_index = stream.get_short()?; // Read and ignore system index

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
        
        self.system_addresses = unsafe {
            (&addresses_uninit as *const _ as *const [InternetAddress; SYSTEM_ADDRESS_COUNT]).read()
        };

        self.send_ping_time = stream.get_long()?;
        self.send_pong_time = stream.get_long()?;
        Ok(())
    }
}

// Implement the marker trait
impl ConnectedPacket for ConnectionRequestAccepted {}