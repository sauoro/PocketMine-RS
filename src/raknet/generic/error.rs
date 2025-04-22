// src/raknet/generic/error.rs
#![allow(dead_code)]

use crate::raknet::generic::disconnect_reason::DisconnectReason;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SocketError {
    #[error("Socket IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to bind socket: {0}")]
    BindFailed(String),
    #[error("Failed to connect socket: {0}")]
    ConnectFailed(String),
    #[error("Failed to send packet: {0}")]
    SendFailed(String),
    #[error("Failed to receive packet: {0}")]
    RecvFailed(String),
    #[error("Socket operation failed: {0}")]
    OperationFailed(String),
}

#[derive(Error, Debug, Clone)]
#[error("Packet handling error: {message} (Reason: {reason})")]
pub struct PacketHandlingError {
    pub message: String,
    pub reason: DisconnectReason,
}

impl PacketHandlingError {
    pub fn new(message: String, reason: DisconnectReason) -> Self {
        Self { message, reason }
    }
}

// Allow converting BinaryDataException to PacketHandlingError
impl From<crate::utils::error::BinaryDataException> for PacketHandlingError {
    fn from(e: crate::utils::error::BinaryDataException) -> Self {
        PacketHandlingError::new(e.to_string(), DisconnectReason::BadPacket)
    }
}