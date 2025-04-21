// src/raklib/utils/address.rs
use crate::utils::binary::{BinaryStream, BinaryUtilError, Result as BinaryResult};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    fmt,
    io::Cursor, // Keep Cursor for byte manipulation
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

// Add libc dependency to Cargo.toml if not already there
// [dependencies]
// libc = "0.2"
extern crate libc; // Use libc for AF_INET6 constant

/// Represents an internet address (IP, port, version).
/// Similar to PHP RakLib's InternetAddress.
/// Note: Using std::net::SocketAddr is often more idiomatic in Rust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InternetAddress {
    ip: IpAddr,
    port: u16,
}

impl InternetAddress {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        InternetAddress { ip, port }
    }

    pub fn from_socket_addr(addr: SocketAddr) -> Self {
        InternetAddress {
            ip: addr.ip(),
            port: addr.port(),
        }
    }

    pub fn ip(&self) -> IpAddr {
        self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn version(&self) -> u8 {
        match self.ip {
            IpAddr::V4(_) => 4,
            IpAddr::V6(_) => 6,
        }
    }

    pub fn to_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }

    // --- Methods to match PHP RakLib's PacketSerializer ---

    /// Reads an InternetAddress from a BinaryStream according to RakNet format.
    /// Directly analogous to PacketSerializer::getAddress()
    pub fn read_from(stream: &mut BinaryStream) -> BinaryResult<Self> {
        let version = stream.get_u8()?;
        match version {
            4 => {
                // Read IPv4: ~b1.~b2.~b3.~b4, PortBE
                let b1 = !stream.get_u8()?;
                let b2 = !stream.get_u8()?;
                let b3 = !stream.get_u8()?;
                let b4 = !stream.get_u8()?;
                let port = stream.get_u16_be()?; // RakNet uses BE for port in this structure
                Ok(InternetAddress::new(
                    IpAddr::V4(Ipv4Addr::new(b1, b2, b3, b4)),
                    port,
                ))
            }
            6 => {
                // Read IPv6: FamilyLE(2), PortBE(2), FlowInfoBE(4), IP(16), ScopeIdBE(4)
                // stream.get(2)?; // Read and ignore family (LE short) - handled below
                let mut family_cursor = Cursor::new(stream.get(2)?);
                let _family = family_cursor.read_u16::<LittleEndian>()?; // Usually AF_INET6 / SOCK_ família número
                // NOTE: Do we need to validate the family? Probably not critical.

                let port = stream.get_u16_be()?; // Port is BE Short

                let _flow_info = stream.get_u32_be()?; // Read and ignore flow info

                let ip_bytes = stream.get(16)?; // 16 bytes for IPv6 address
                let addr_bytes: [u8; 16] = match ip_bytes.try_into() {
                    Ok(b) => b,
                    Err(_) => return Err(BinaryUtilError::NotEnoughData{ needed: 16, have: ip_bytes.len()}),
                };

                let _scope_id = stream.get_u32_be()?; // Read and ignore scope ID

                Ok(InternetAddress::new(
                    IpAddr::V6(Ipv6Addr::from(addr_bytes)),
                    port,
                ))
            }
            _ => Err(BinaryUtilError::InvalidData(format!(
                "Unknown IP address version {}",
                version
            ))),
        }
    }

    /// Writes an InternetAddress to a BinaryStream according to RakNet format.
    /// Directly analogous to PacketSerializer::putAddress()
    pub fn write_to(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        match self.ip {
            IpAddr::V4(ipv4) => {
                stream.put_u8(4)?; // Version 4
                for &byte in &ipv4.octets() {
                    stream.put_u8(!byte)?; // Inverted bytes
                }
                stream.put_u16_be(self.port)?; // Port is Big Endian
            }
            IpAddr::V6(ipv6) => {
                stream.put_u8(6)?; // Version 6

                // Family (AF_INET6) - LE Short
                // Use libc::AF_INET6 if available and reliable, otherwise use a common value.
                // Using a known value directly might be safer if libc isn't standard.
                // Common values: Linux=10, Windows=23, macOS=30. Let's use 23 (SOCK_ família for Windows?).
                // const AF_INET6_VAL: u16 = 23; // Or use libc::AF_INET6 as i32 as u16
                stream.put_u16_le(libc::AF_INET6 as u16)?; // Use LE Short

                stream.put_u16_be(self.port)?; // Port (BE Short)
                stream.put_u32_be(0)?; // Flow Info (BE Int) - Always 0?
                stream.put(&ipv6.octets())?; // IP (16 bytes)
                stream.put_u32_be(0)?; // Scope ID (BE Int) - Usually 0 for global
            }
        }
        Ok(())
    }
}

// Display and From implementations remain the same
impl fmt::Display for InternetAddress { /* ... */ }
impl From<SocketAddr> for InternetAddress { /* ... */ }
impl From<InternetAddress> for SocketAddr { /* ... */ }


// Unit tests remain the same, but ensure they pass with the updated logic/constants
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::binary::BinaryStream;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_ipv4_serialization() {
        let ip = Ipv4Addr::new(192, 168, 1, 100);
        let port = 19132;
        let addr = InternetAddress::new(ip.into(), port);

        let mut stream = BinaryStream::new();
        addr.write_to(&mut stream).unwrap();

        // Expected: 4 | ~192 | ~168 | ~1 | ~100 | PortBE
        // Expected: 04 | 3F   | 97   | FE | 9B   | 4A BC
        let expected = vec![0x04, 0x3F, 0x97, 0xFE, 0x9B, 0x4A, 0xBC];
        assert_eq!(stream.get_buffer(), &expected);

        stream.rewind();
        let decoded_addr = InternetAddress::read_from(&mut stream).unwrap();
        assert_eq!(decoded_addr, addr);
        assert!(stream.feof());
    }

    #[test]
    fn test_ipv6_serialization() {
        let ip = Ipv6Addr::new(0x2001, 0x0db8, 0x85a3, 0x0000, 0x0000, 0x8a2e, 0x0370, 0x7334);
        let port = 19133; // 0x4ABD
        let addr = InternetAddress::new(ip.into(), port);

        let mut stream = BinaryStream::new();
        addr.write_to(&mut stream).unwrap();

        // Expected: 6 | FamilyLE(2) | PortBE(2) | FlowBE(4) | IPBytes(16) | ScopeBE(4)
        let mut expected: Vec<u8> = vec![0x06];
        expected.extend(& (libc::AF_INET6 as u16).to_le_bytes()); // Family
        expected.extend(&port.to_be_bytes());           // Port
        expected.extend(&[0u8; 4]);                      // Flow Info
        expected.extend(&ip.octets());                   // IP Address
        expected.extend(&[0u8; 4]);                      // Scope ID

        assert_eq!(stream.get_buffer().len(), 1 + 2 + 2 + 4 + 16 + 4); // Sanity check length
        assert_eq!(stream.get_buffer(), &expected);

        stream.rewind();
        let decoded_addr = InternetAddress::read_from(&mut stream).unwrap();
        assert_eq!(decoded_addr, addr);
        assert!(stream.feof());
    }
}