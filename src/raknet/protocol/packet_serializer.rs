// src/raknet/protocol/packet_serializer.rs

#![allow(dead_code)]

use crate::utils::binary_stream::BinaryStream; // Use the existing BinaryStream
use crate::utils::error::{BinaryDataException, Result};
use crate::raknet::utils::internet_address::InternetAddress;
use crate::raknet::protocol::{AF_INET, AF_INET6}; // Import constants
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::convert::TryInto;

// We can either wrap BinaryStream or provide these methods as an extension trait.
// Wrapping is closer to the PHP inheritance structure.
#[derive(Debug, Clone, Default)]
pub struct PacketSerializer {
    stream: BinaryStream,
}

// Delegate methods from BinaryStream or implement Deref/DerefMut
// For simplicity, let's explicitly delegate common methods needed.
// A more robust solution might use Deref/DerefMut or a macro.
impl PacketSerializer {
    pub fn new() -> Self {
        Self { stream: BinaryStream::new() }
    }

    pub fn with_buffer(buffer: Vec<u8>, offset: usize) -> Self {
        Self { stream: BinaryStream::with_buffer(buffer, offset) }
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Self { stream: BinaryStream::from_slice(slice) }
    }

    // --- Delegated methods ---
    pub fn get_buffer(&self) -> &[u8] { self.stream.get_buffer() }
    pub fn get_offset(&self) -> usize { self.stream.get_offset() }
    pub fn get_mut_buffer(&mut self) -> &mut Vec<u8> { self.stream.get_mut_buffer() }
    pub fn put(&mut self, bytes: &[u8]) { self.stream.put(bytes) }
    pub fn get(&mut self, len: usize) -> Result<&[u8]> { self.stream.get(len) }
    pub fn get_remaining(&mut self) -> Result<&[u8]> { self.stream.get_remaining() }
    pub fn get_byte(&mut self) -> Result<u8> { self.stream.get_byte() }
    pub fn put_byte(&mut self, v: u8) { self.stream.put_byte(v) }
    pub fn get_short(&mut self) -> Result<u16> { self.stream.get_short() }
    pub fn put_short(&mut self, v: u16) -> Result<()> { self.stream.put_short(v) }
    pub fn get_lshort(&mut self) -> Result<u16> { self.stream.get_lshort() }
    pub fn put_lshort(&mut self, v: u16) -> Result<()> { self.stream.put_lshort(v) }
    pub fn get_int(&mut self) -> Result<i32> { self.stream.get_int() }
    pub fn put_int(&mut self, v: i32) -> Result<()> { self.stream.put_int(v) }
    pub fn get_long(&mut self) -> Result<i64> { self.stream.get_long() }
    pub fn put_long(&mut self, v: i64) -> Result<()> { self.stream.put_long(v) }
    pub fn get_l_triad(&mut self) -> Result<u32> { self.stream.get_ltriad() } // Added LTriad
    pub fn put_l_triad(&mut self, v: u32) -> Result<()> { self.stream.put_ltriad(v) } // Added LTriad
    pub fn feof(&self) -> bool { self.stream.feof() }
    // Add other needed delegations as required...


    // --- RakNet specific methods ---

    pub fn get_string(&mut self) -> Result<String> {
        let len = self.stream.get_short()? as usize;
        let bytes = self.stream.get(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|e| BinaryDataException::new(e.to_string()))
    }

    pub fn put_string(&mut self, v: &str) -> Result<()> {
        let bytes = v.as_bytes();
        // RakNet strings are u16 length prefixed
        let len: u16 = bytes.len().try_into()
            .map_err(|_| BinaryDataException::from_str("String length exceeds u16 max"))?;
        self.stream.put_short(len)?;
        self.stream.put(bytes);
        Ok(())
    }

    pub fn get_address(&mut self) -> Result<InternetAddress> {
        let version = self.stream.get_byte()?;
        match version {
            AF_INET => { // IPv4
                let b1 = !self.stream.get_byte()?;
                let b2 = !self.stream.get_byte()?;
                let b3 = !self.stream.get_byte()?;
                let b4 = !self.stream.get_byte()?;
                let ip_str = format!("{}.{}.{}.{}", b1, b2, b3, b4);
                let port = self.stream.get_short()?;
                InternetAddress::new(ip_str, port, version)
                    .map_err(BinaryDataException::new)
            }
            AF_INET6 => { // IPv6
                self.stream.get_lshort()?; // Skip AF_INET6 family (2 bytes)
                let port = self.stream.get_short()?;
                self.stream.get_int()?; // Skip flow info (4 bytes)
                let ip_bytes: [u8; 16] = self.stream.get(16)?.try_into()
                    .map_err(|_| BinaryDataException::from_str("Could not read 16 bytes for IPv6 address"))?;
                let ip_addr = Ipv6Addr::from(ip_bytes);
                self.stream.get_int()?; // Skip scope ID (4 bytes)
                InternetAddress::new(ip_addr.to_string(), port, version)
                    .map_err(BinaryDataException::new)
            }
            _ => Err(BinaryDataException::new(format!("Unknown IP address version {}", version)))
        }
    }

    pub fn put_address(&mut self, address: &InternetAddress) -> Result<()> {
        self.stream.put_byte(address.version());
        match address.version() {
            AF_INET => { // IPv4
                let ip_addr: Ipv4Addr = address.ip().parse()
                    .map_err(|e| BinaryDataException::new(format!("Invalid IPv4 address '{}': {}", address.ip(), e)))?;
                for octet in ip_addr.octets() {
                    self.stream.put_byte(!octet); // Bitwise NOT
                }
                self.stream.put_short(address.port())?;
            }
            AF_INET6 => { // IPv6
                // Use std::net parsing and types for robustness
                let ip_addr: Ipv6Addr = address.ip().parse()
                    .map_err(|e| BinaryDataException::new(format!("Invalid IPv6 address '{}': {}", address.ip(), e)))?;

                // Note: RakNet's IPv6 serialization seems weird. PHP uses AF_INET6 constant (10 for glibc, 2 for Windows?)
                // Let's try to match the PHP behavior using a constant Little Endian Short=2
                const AF_INET6_SOCKET_CONST: u16 = 2; // Common value? Needs testing. PHP uses AF_INET6
                self.stream.put_lshort(AF_INET6_SOCKET_CONST)?; // Family
                self.stream.put_short(address.port())?; // Port
                self.stream.put_int(0)?; // Flow Info
                self.stream.put(&ip_addr.octets()); // 16 bytes IP
                self.stream.put_int(0)?; // Scope ID
            }
            _ => return Err(BinaryDataException::new(format!("IP version {} is not supported", address.version())))
        }
        Ok(())
    }

    // --- Helper to convert PacketSerializer to BinaryStream ---
    // Useful if other parts of the code expect a plain BinaryStream
    pub fn into_binary_stream(self) -> BinaryStream {
        self.stream
    }
}

// Allow converting from BinaryStream (e.g., when reading a raw buffer)
impl From<BinaryStream> for PacketSerializer {
    fn from(stream: BinaryStream) -> Self {
        Self { stream }
    }
}