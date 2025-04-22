// src/raknet/server/server_event_listener.rs
#![allow(dead_code)]

use crate::raknet::generic::disconnect_reason::DisconnectReason;
use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
pub trait ServerEventListener: Send + Sync {
    async fn on_client_connect(
        &self,
        session_id: u64,
        address: String,
        port: u16,
        client_id: i64,
    );

    async fn on_client_disconnect(&self, session_id: u64, reason: DisconnectReason);

    async fn on_packet_receive(&self, session_id: u64, packet: Bytes);

    async fn on_raw_packet_receive(&self, address: String, port: u16, payload: Bytes);

    async fn on_packet_ack(&self, session_id: u64, identifier_ack: u32);

    async fn on_bandwidth_stats_update(&self, bytes_sent_diff: u64, bytes_received_diff: u64);

    async fn on_ping_measure(&self, session_id: u64, ping_ms: u32);
}