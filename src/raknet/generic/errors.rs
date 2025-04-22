// src/raknet/generic/errors.rs

#![allow(dead_code)]

pub(crate) use crate::raknet::generic::disconnect_reason::DisconnectReason;
use thiserror::Error; // Add thiserror for easier error definition

/// Error related to packet handling logic within a session.
#[derive(Error, Debug)]
pub enum PacketHandlingError {
    #[error("{message}")]
    Generic {
        message: String,
        disconnect_reason: DisconnectReason,
    },
    // Add more specific variants as needed later
}

impl PacketHandlingError {
    pub fn new(message: String, disconnect_reason: DisconnectReason) -> Self {
        Self::Generic { message, disconnect_reason }
    }

    pub fn disconnect_reason(&self) -> DisconnectReason {
        match self {
            PacketHandlingError::Generic { disconnect_reason, .. } => *disconnect_reason,
        }
    }
}


/// Error related to underlying socket operations.
/// Often wraps std::io::Error or potentially tokio::io::Error.
#[derive(Error, Debug)]
pub enum SocketError {
    #[error("Socket operation failed: {message} (OS Code: {os_code:?})")]
    OperationFailed {
        message: String,
        os_code: Option<i32>, // Store the OS error code if available
        source: Option<std::io::Error>, // Option to wrap the underlying IO error
    },

    #[error("Failed to bind socket: {message} (OS Code: {os_code:?})")]
    BindFailed {
        message: String,
        os_code: Option<i32>,
        source: Option<std::io::Error>,
    },

    // Add other specific socket errors as needed (e.g., ConnectFailed)

    #[error("Underlying IO error")]
    Io { #[from] source: std::io::Error } // Allow easy conversion from io::Error
}

impl SocketError {
    // Helper constructor
    pub fn new_op_failed(message: String, os_code: Option<i32>, source: Option<std::io::Error>) -> Self {
        Self::OperationFailed { message, os_code, source }
    }
    pub fn new_bind_failed(message: String, os_code: Option<i32>, source: Option<std::io::Error>) -> Self {
        Self::BindFailed { message, os_code, source }
    }
}