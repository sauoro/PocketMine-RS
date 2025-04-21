// src/raklib/utils/address.rs
extern crate libc; // <<< Added extern crate declaration
use crate::utils::binary::{BinaryStream, BinaryUtilError, Result as BinaryResult};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    fmt, // <<< Added import
    io::Cursor,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6}, // <<< Removed unused SocketAddrV4/V6
};

/// Represents an internet address (IP, port, version).
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
        InternetAddress { ip: addr.ip(), port: addr.port() }
    }
    pub fn ip(&self) -> IpAddr { self.ip }
    pub fn port(&self) -> u16 { self.port }
    pub fn version(&self) -> u8 { match self.ip { IpAddr::V4(_) => 4, IpAddr::V6(_) => 6, } }
    pub fn to_socket_addr(&self) -> SocketAddr { SocketAddr::new(self.ip, self.port) }

    // --- Methods to match PHP RakLib's PacketSerializer ---
    pub fn read_from(stream: &mut BinaryStream) -> BinaryResult<Self> {
        let version = stream.get_u8()?;
        match version {
            4 => {
                let b1 = !stream.get_u8()?; let b2 = !stream.get_u8()?; let b3 = !stream.get_u8()?; let b4 = !stream.get_u8()?;
                let port = stream.get_u16_be()?;
                Ok(InternetAddress::new(IpAddr::V4(Ipv4Addr::new(b1, b2, b3, b4)), port))
            }
            6 => {
                let mut family_cursor = Cursor::new(stream.get(2)?);
                let _family = family_cursor.read_u16::<LittleEndian>()?;
                let port = stream.get_u16_be()?;
                let _flow_info = stream.get_u32_be()?;
                let ip_bytes = stream.get(16)?;
                let addr_bytes: [u8; 16] = match ip_bytes.try_into() {
                    Ok(b) => b, Err(_) => return Err(BinaryUtilError::NotEnoughData{ needed: 16, have: ip_bytes.len()}),
                };
                let _scope_id = stream.get_u32_be()?;
                Ok(InternetAddress::new(IpAddr::V6(Ipv6Addr::from(addr_bytes)), port))
            }
            _ => Err(BinaryUtilError::InvalidData(format!("Unknown IP address version {}", version))),
        }
    }
    pub fn write_to(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        match self.ip {
            IpAddr::V4(ipv4) => {
                stream.put_u8(4)?;
                for &byte in &ipv4.octets() { stream.put_u8(!byte)?; }
                stream.put_u16_be(self.port)?;
            }
            IpAddr::V6(ipv6) => {
                stream.put_u8(6)?;
                stream.put_u16_le(libc::AF_INET6 as u16)?; // Use LE Short for family
                stream.put_u16_be(self.port)?;
                stream.put_u32_be(0)?; // Flow Info
                stream.put(&ipv6.octets())?;
                stream.put_u32_be(0)?; // Scope ID
            }
        }
        Ok(())
    }
}

// Implement Display, From<SocketAddr>, From<InternetAddress> <<< Implemented
impl fmt::Display for InternetAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.ip, self.port)
    }
}
impl From<SocketAddr> for InternetAddress {
    fn from(addr: SocketAddr) -> Self {
        Self::from_socket_addr(addr)
    }
}
impl From<InternetAddress> for SocketAddr {
    fn from(addr: InternetAddress) -> Self {
        addr.to_socket_addr()
    }
}

// --- Unit tests ---
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
        let mut expected: Vec<u8> = vec![0x06];
        expected.extend(&(libc::AF_INET6 as u16).to_le_bytes());
        expected.extend(&port.to_be_bytes());
        expected.extend(&[0u8; 4]); // Flow Info
        expected.extend(&ip.octets()); // IP Address
        expected.extend(&[0u8; 4]); // Scope ID
        assert_eq!(stream.get_buffer().len(), 1 + 2 + 2 + 4 + 16 + 4);
        assert_eq!(stream.get_buffer(), &expected);
        stream.rewind();
        let decoded_addr = InternetAddress::read_from(&mut stream).unwrap();
        assert_eq!(decoded_addr, addr);
        assert!(stream.feof());
    }
}