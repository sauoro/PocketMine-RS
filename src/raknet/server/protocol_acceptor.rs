// src/raknet/server/protocol_acceptor.rs

#![allow(dead_code)]

pub trait ProtocolAcceptor {
    fn accepts(&self, protocol_version: u8) -> bool;
    fn get_primary_version(&self) -> u8;
}