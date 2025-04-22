// src/raknet/utils/internet_address.rs

#![allow(dead_code)]

use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct InternetAddress {
    ip: String,
    port: u16,
    version: u8,
}

impl InternetAddress {
    pub fn new(ip: String, port: u16, version: u8) -> Result<Self, String> {
        Ok(Self { ip, port, version })
    }

    pub fn ip(&self) -> &str {
        &self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn equals(&self, other: &InternetAddress) -> bool {
        self.ip == other.ip && self.port == other.port && self.version == other.version
    }
}

impl PartialEq for InternetAddress {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

impl Eq for InternetAddress {}

impl Hash for InternetAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ip.hash(state);
        self.port.hash(state);
        self.version.hash(state);
    }
}

impl fmt::Display for InternetAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.ip, self.port)
    }
}