// src/raknet/server/protocol_acceptor.rs
#![allow(dead_code)]

use async_trait::async_trait;

#[async_trait]
pub trait ProtocolAcceptor: Send + Sync {
    fn accepts(&self, protocol_version: u8) -> bool;
    fn get_primary_version(&self) -> u8;
}