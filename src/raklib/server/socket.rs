use crate::raklib::error::{Result, SocketError, RakLibError};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::debug;

/// Wraps a Tokio UdpSocket for server use.
#[derive(Debug, Clone)]
pub struct ServerSocket {
    socket: Arc<UdpSocket>, // Use Arc for potential sharing across tasks if needed
    bind_addr: SocketAddr,
}

impl ServerSocket {
    pub async fn bind(addr: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(addr).await.map_err(|e| {
            SocketError::Bind {
                addr,
                source: e,
            }
        })?;
        debug!("Server socket bound to {}", addr);
        // TODO: Set socket options (SO_RCVBUF, SO_SNDBUF) if needed using socket2 crate maybe
        Ok(ServerSocket {
            socket: Arc::new(socket),
            bind_addr: addr,
        })
    }

    pub fn get_bind_address(&self) -> SocketAddr {
        self.bind_addr
    }

    /// Reads a packet from the socket. Returns Ok(None) if no packet is available (non-blocking).
    pub async fn read_packet(&self) -> Result<Option<(Vec<u8>, SocketAddr)>> {
        // Use a buffer sized for typical MTU + headers
        let mut buf = vec![0u8; 2048]; // Adjust size as needed
        match self.socket.try_recv_from(&mut buf) {
            Ok((len, src_addr)) => {
                buf.truncate(len); // Keep only the received bytes
                Ok(Some((buf, src_addr)))
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(None) // No data available right now
            }
            Err(e) => {
                // Handle specific errors like connection reset if applicable on UDP
                if e.kind() == std::io::ErrorKind::ConnectionReset {
                    // ECONNRESET - often ignored on UDP, log and continue maybe?
                    debug!("Received connection reset (ignored): {}", e);
                    Ok(None) // Treat as no data for now
                } else {
                    Err(SocketError::Receive(e).into()) // Wrap other IO errors
                }
            }
        }
    }

    /// Writes a packet to the specified destination address.
    pub async fn write_packet(&self, buffer: &[u8], dest: SocketAddr) -> Result<usize> {
        self.socket
            .send_to(buffer, dest)
            .await
            .map_err(|e| SocketError::Send(e).into())
    }

    // Note: enable/disable broadcast, set buffer sizes might require using the `socket2` crate
    //       for cross-platform compatibility if standard `std::net` or `tokio::net` doesn't suffice.
}