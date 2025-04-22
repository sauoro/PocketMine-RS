// src/raknet/server/server_socket.rs
#![allow(dead_code)]

use crate::raknet::generic::error::SocketError;
use crate::raknet::utils::internet_address::InternetAddress;
use bytes::Bytes;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct ServerSocket {
    socket: UdpSocket,
    bind_address: InternetAddress,
}

impl ServerSocket {
    pub async fn bind(bind_address: InternetAddress) -> Result<Self, SocketError> {
        let socket_addr = bind_address.to_socket_addr();
        let socket = UdpSocket::bind(socket_addr).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                SocketError::BindFailed(format!(
                    "Something else is already running on {}",
                    bind_address
                ))
            } else {
                SocketError::BindFailed(format!(
                    "Failed to bind to {}: {}",
                    bind_address, e
                ))
            }
        })?;

        // Optional: Set buffer sizes (platform specific behavior)
        // socket.set_send_buffer_size(1024 * 1024 * 8)?;
        // socket.set_recv_buffer_size(1024 * 1024 * 8)?;

        Ok(Self { socket, bind_address })
    }

    pub fn get_bind_address(&self) -> &InternetAddress {
        &self.bind_address
    }

    // Broadcast needs specific platform handling, often requires root/admin. Omitted for simplicity.
    // pub async fn enable_broadcast(&self) -> Result<(), SocketError> { ... }
    // pub async fn disable_broadcast(&self) -> Result<(), SocketError> { ... }

    pub async fn recv_from(&self) -> Result<(Bytes, SocketAddr), SocketError> {
        let mut buf = vec![0u8; 65535]; // Max UDP packet size
        let (len, src) = self.socket.recv_from(&mut buf).await.map_err(SocketError::Io)?;
        Ok((Bytes::copy_from_slice(&buf[..len]), src))
    }

    pub async fn send_to(&self, buffer: &[u8], dest: SocketAddr) -> Result<usize, SocketError> {
        self.socket.send_to(buffer, dest).await.map_err(|e| SocketError::SendFailed(e.to_string()))
    }

    pub fn close(&self) {
        // Tokio sockets close when dropped. Explicit close isn't usually needed.
        // If required for immediate resource release, more complex handling is needed.
    }
}