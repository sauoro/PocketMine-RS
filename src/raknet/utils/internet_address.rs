// src/raknet/utils/internet_address.rs
#![allow(dead_code)]

use crate::utils::binary;
use crate::utils::error::BinaryDataException;
use crate::utils::BinaryStream;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InternetAddress {
    V4(SocketAddrV4),
    V6(SocketAddrV6),
}

impl InternetAddress {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        match ip {
            IpAddr::V4(ipv4) => InternetAddress::V4(SocketAddrV4::new(ipv4, port)),
            IpAddr::V6(ipv6) => InternetAddress::V6(SocketAddrV6::new(ipv6, port, 0, 0)),
        }
    }

    pub fn from_string(ip_str: &str, port: u16) -> Result<Self, std::net::AddrParseError> {
        let ip = IpAddr::from_str(ip_str)?;
        Ok(Self::new(ip, port))
    }

    pub fn from_socket_addr(addr: SocketAddr) -> Self {
        match addr {
            SocketAddr::V4(v4) => InternetAddress::V4(v4),
            SocketAddr::V6(v6) => InternetAddress::V6(v6),
        }
    }

    pub fn ip(&self) -> IpAddr {
        match self {
            InternetAddress::V4(addr) => IpAddr::V4(*addr.ip()),
            InternetAddress::V6(addr) => IpAddr::V6(*addr.ip()),
        }
    }

    pub fn port(&self) -> u16 {
        match self {
            InternetAddress::V4(addr) => addr.port(),
            InternetAddress::V6(addr) => addr.port(),
        }
    }

    pub fn version(&self) -> u8 {
        match self {
            InternetAddress::V4(_) => 4,
            InternetAddress::V6(_) => 6,
        }
    }

    pub fn to_socket_addr(&self) -> SocketAddr {
        match *self {
            InternetAddress::V4(addr) => SocketAddr::V4(addr),
            InternetAddress::V6(addr) => SocketAddr::V6(addr),
        }
    }

    pub fn equals(&self, other: &InternetAddress) -> bool {
        self == other
    }

    pub fn to_string_addr_only(&self) -> String {
        self.ip().to_string()
    }

    pub fn read(stream: &mut BinaryStream) -> Result<Self, BinaryDataException> {
        let version = stream.get_byte()?;
        match version {
            4 => {
                let b1 = !stream.get_byte()?;
                let b2 = !stream.get_byte()?;
                let b3 = !stream.get_byte()?;
                let b4 = !stream.get_byte()?;
                let port = stream.get_short()?;
                let ip = Ipv4Addr::new(b1, b2, b3, b4);
                Ok(InternetAddress::V4(SocketAddrV4::new(ip, port)))
            }
            6 => {
                stream.get_lshort()?; // Skip family (assumed AF_INET6)
                let port = stream.get_short()?;
                stream.get_int()?; // Skip flow info
                let segments: [u16; 8] = [
                    stream.get_short()?,
                    stream.get_short()?,
                    stream.get_short()?,
                    stream.get_short()?,
                    stream.get_short()?,
                    stream.get_short()?,
                    stream.get_short()?,
                    stream.get_short()?,
                ];
                stream.get_int()?; // Skip scope ID
                let ip = Ipv6Addr::new(
                    segments[0], segments[1], segments[2], segments[3],
                    segments[4], segments[5], segments[6], segments[7],
                );
                Ok(InternetAddress::V6(SocketAddrV6::new(ip, port, 0, 0)))
            }
            _ => Err(BinaryDataException::new(format!("Unknown IP address version {}", version))),
        }
    }

    pub fn write(&self, stream: &mut BinaryStream) -> Result<(), BinaryDataException> {
        stream.put_byte(self.version());
        match self {
            InternetAddress::V4(addr) => {
                for octet in addr.ip().octets().iter() {
                    stream.put_byte(!*octet);
                }
                stream.put_short(addr.port())?;
            }
            InternetAddress::V6(addr) => {
                const AF_INET6_LITTLE_ENDIAN: u16 = 23; // Platform dependent, but RakNet seems to use this
                stream.put_lshort(AF_INET6_LITTLE_ENDIAN)?;
                stream.put_short(addr.port())?;
                stream.put_int(0)?; // Flow info
                for segment in addr.ip().segments().iter() {
                    stream.put_short(*segment)?; // Network byte order (Big Endian)
                }
                stream.put_int(0)?; // Scope ID
            }
        }
        Ok(())
    }
}

impl fmt::Display for InternetAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternetAddress::V4(addr) => write!(f, "{}:{}", addr.ip(), addr.port()),
            InternetAddress::V6(addr) => write!(f, "[{}]:{}", addr.ip(), addr.port()),
        }
    }
}