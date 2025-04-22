// src/raknet/server/server_interface.rs
#![allow(dead_code)]

use crate::raknet::generic::disconnect_reason::DisconnectReason;
use crate::raknet::protocol::encapsulated_packet::EncapsulatedPacket;
use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
pub trait ServerInterface: Send + Sync {
    async fn send_encapsulated(
        &self,
        session_id: u64,
        packet: EncapsulatedPacket,
        immediate: bool,
    );

    async fn send_raw(&self, address: String, port: u16, payload: Bytes);

    async fn close_session(&self, session_id: u64, reason: DisconnectReason);

    async fn set_name(&self, name: String);

    async fn set_port_check(&self, value: bool);

    // Making this mutable requires careful handling of Arc/Mutex state
    async fn set_packets_per_tick_limit(&mut self, limit: usize);

    async fn block_address(&self, address: String, timeout_secs: u64);

    async fn unblock_address(&self, address: String);

    async fn add_raw_packet_filter(&self, regex: String);
}