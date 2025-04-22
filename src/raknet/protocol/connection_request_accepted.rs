// src/raknet/protocol/connection_request_accepted.rs
#![allow(dead_code)]

use crate::raknet::protocol::connected_packet::ConnectedPacket;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::utils::internet_address::InternetAddress;
use crate::raknet::raknet::SYSTEM_ADDRESS_COUNT;
use crate::utils::error::{BinaryDataException, Result as BinaryResult};
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone)]
pub struct ConnectionRequestAccepted {
    pub address: InternetAddress,
    pub system_addresses: Vec<InternetAddress>,
    pub send_ping_time: u64,
    pub send_pong_time: u64,
}

impl ConnectionRequestAccepted {
    pub const ID: u8 = MessageIdentifiers::ID_CONNECTION_REQUEST_ACCEPTED;

    pub fn create(
        client_address: InternetAddress,
        system_addresses: Vec<InternetAddress>,
        send_ping_time: u64,
        send_pong_time: u64,
    ) -> Self {
        let mut sys_addr = system_addresses;
        // Pad with dummy addresses if needed
        let dummy = InternetAddress::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
        while sys_addr.len() < SYSTEM_ADDRESS_COUNT {
            sys_addr.push(dummy.clone());
        }
        // Truncate if too many (shouldn't happen with proper use)
        sys_addr.truncate(SYSTEM_ADDRESS_COUNT);

        Self {
            address: client_address,
            system_addresses: sys_addr,
            send_ping_time,
            send_pong_time,
        }
    }
}

impl Packet for ConnectionRequestAccepted {
    fn get_id() -> u8 { Self::ID }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        stream.put_address(&self.address)?;
        stream.put_short(0)?; // System index, seems unused?

        for i in 0..SYSTEM_ADDRESS_COUNT {
            if let Some(addr) = self.system_addresses.get(i) {
                stream.put_address(addr)?;
            } else {
                // This case should be prevented by create(), but handle defensively
                let dummy = InternetAddress::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
                stream.put_address(&dummy)?;
            }
        }

        stream.put_long(self.send_ping_time)?;
        stream.put_long(self.send_pong_time)?;
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        self.address = stream.get_address()?;
        stream.get_short()?; // Skip system index

        self.system_addresses.clear();
        let dummy = InternetAddress::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

        for _ in 0..SYSTEM_ADDRESS_COUNT {
            // Heuristic from PHP RakLib: Check if enough bytes remain for an address.
            // A full IPv6 address needs ~28 bytes (version + short + short + int + 16 + int).
            // A full IPv4 address needs 7 bytes (version + 4 + short).
            // Be cautious, this is not foolproof.
            if stream.remaining() < 7 { // Minimum possible size (IPv4)
                self.system_addresses.push(dummy.clone());
            } else {
                // Attempt to read, default to dummy on error or incomplete read
                // This is tricky because get_address() consumes bytes even on error within its scope.
                // A safer approach might involve peeking or checking length more carefully.
                // Let's stick to a similar logic as PHP for now.
                let current_offset = stream.get_offset();
                match stream.get_address() {
                    Ok(addr) => self.system_addresses.push(addr),
                    Err(_) => {
                        // Reset offset if read failed partially? BinaryStream doesn't support this well.
                        // Assume reading the address failed completely if error occurred.
                        stream.set_offset(current_offset); // Try to rewind (might not be perfect)
                        self.system_addresses.push(dummy.clone());
                        // If we can't rewind accurately, subsequent reads might fail.
                        // The PHP code relied on offset checks *before* calling getAddress inside the loop.
                        // Let's refine the remaining check:
                        if stream.remaining() < 16 { // If not enough for ping/pong times, add dummies
                            while self.system_addresses.len() < SYSTEM_ADDRESS_COUNT {
                                self.system_addresses.push(dummy.clone());
                            }
                            break; // Exit loop early
                        }
                    }
                }
            }
        }
        // Ensure correct count if loop exited early due to insufficient bytes
        while self.system_addresses.len() < SYSTEM_ADDRESS_COUNT {
            self.system_addresses.push(dummy.clone());
        }


        if stream.remaining() < 16 {
            return Err(BinaryDataException::from_str("Not enough bytes left for ping/pong times"));
        }
        self.send_ping_time = stream.get_long()?;
        self.send_pong_time = stream.get_long()?;
        Ok(())
    }
}

impl ConnectedPacket for ConnectionRequestAccepted {}