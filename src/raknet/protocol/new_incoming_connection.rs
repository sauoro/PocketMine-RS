// src/raknet/protocol/new_incoming_connection.rs
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
pub struct NewIncomingConnection {
    pub address: InternetAddress,
    pub system_addresses: Vec<InternetAddress>,
    pub send_ping_time: u64,
    pub send_pong_time: u64,
}

impl NewIncomingConnection {
    pub const ID: u8 = MessageIdentifiers::ID_NEW_INCOMING_CONNECTION;
}

impl Packet for NewIncomingConnection {
    fn get_id() -> u8 { Self::ID }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        stream.put_address(&self.address)?;
        for addr in &self.system_addresses {
            stream.put_address(addr)?;
        }
        stream.put_long(self.send_ping_time)?;
        stream.put_long(self.send_pong_time)?;
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        self.address = stream.get_address()?;

        let stop_offset = stream.len().saturating_sub(16); // buffer length - sizeof(ping) - sizeof(pong)
        self.system_addresses.clear();
        let dummy = InternetAddress::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

        for _ in 0..SYSTEM_ADDRESS_COUNT {
            if stream.get_offset() >= stop_offset {
                self.system_addresses.push(dummy.clone());
            } else {
                // Heuristic from PHP RakLib: Check if enough bytes remain for an address.
                if stream.remaining() < 7 { // Minimum possible size (IPv4)
                    self.system_addresses.push(dummy.clone());
                } else {
                    // Attempt to read, default to dummy on error or incomplete read
                    let current_offset = stream.get_offset();
                    match stream.get_address() {
                        Ok(addr) => self.system_addresses.push(addr),
                        Err(_) => {
                            stream.set_offset(current_offset); // Try to rewind
                            self.system_addresses.push(dummy.clone());
                            // If we can't rewind accurately, assume not enough space left
                            if stream.remaining() < 16 { // Not enough for ping/pong
                                while self.system_addresses.len() < SYSTEM_ADDRESS_COUNT {
                                    self.system_addresses.push(dummy.clone());
                                }
                                break; // Exit loop early
                            }
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
            return Err(BinaryDataException::from_str("Not enough bytes left for ping/pong times in NewIncomingConnection"));
        }
        self.send_ping_time = stream.get_long()?;
        self.send_pong_time = stream.get_long()?;
        Ok(())
    }
}

impl ConnectedPacket for NewIncomingConnection {}