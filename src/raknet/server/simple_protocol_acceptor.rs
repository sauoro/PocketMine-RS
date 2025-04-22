// src/raknet/server/simple_protocol_acceptor.rs
#![allow(dead_code)]

use crate::raknet::server::protocol_acceptor::ProtocolAcceptor;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct SimpleProtocolAcceptor {
    protocol_version: u8,
}

impl SimpleProtocolAcceptor {
    pub fn new(protocol_version: u8) -> Self {
        Self { protocol_version }
    }
}

#[async_trait]
impl ProtocolAcceptor for SimpleProtocolAcceptor {
    fn accepts(&self, protocol_version: u8) -> bool {
        self.protocol_version == protocol_version
    }

    fn get_primary_version(&self) -> u8 {
        self.protocol_version
    }
}