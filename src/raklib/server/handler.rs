use super::{Server, ServerInterface, ProtocolAcceptor}; // Need Server/ServerInterface definition
use crate::raklib::protocol::*; // Import all protocol stuff
use crate::raklib::utils::InternetAddress;
use crate::utils::binary::{BinaryStream, Result as BinaryResult, BinaryUtilError};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, warn};

pub struct UnconnectedMessageHandler {
    server_id: u64, // Server's RakNet GUID
    protocol_acceptor: Arc<dyn ProtocolAcceptor>,
    // Need a way to interact with the server (send packets, create sessions)
    // Using ServerInterface trait object for now
    server_interface_factory: Box<dyn Fn() -> Box<dyn ServerInterface>>, // Factory to get interface
    // Pre-create packet instances for quick cloning (like PHP pool)
    // Or use enum dispatch / match on packet ID
    packet_decoder: PacketDecoder, // Helper struct/enum for decoding offline messages
}

impl UnconnectedMessageHandler {
    pub fn new(
        server_id: u64,
        protocol_acceptor: Arc<dyn ProtocolAcceptor>,
        server_interface_factory: Box<dyn Fn() -> Box<dyn ServerInterface>>,
    ) -> Self {
        Self {
            server_id,
            protocol_acceptor,
            server_interface_factory,
            packet_decoder: PacketDecoder::new(),
        }
    }

    /// Handles a raw incoming UDP packet before a session is established.
    /// Returns Ok(true) if handled, Ok(false) if not an offline message, Err on decode failure.
    pub fn handle_raw(&self, payload: &[u8], addr: SocketAddr) -> crate::raklib::error::Result<bool> {
        if payload.is_empty() { return Ok(false); }

        match self.packet_decoder.decode_offline(payload) {
            Ok(Some(packet_enum)) => {
                // Successfully decoded an offline packet
                self.handle_offline_message(packet_enum, addr)?;
                Ok(true)
            }
            Ok(None) => {
                // Not a known offline packet ID, or not an offline packet at all
                Ok(false)
            }
            Err(e) => {
                // Decoding failed (e.g., bad magic, insufficient data)
                warn!(%addr, error = %e, "Failed to decode potential offline message");
                Err(e.into()) // Convert BinaryUtilError to RakLibError
            }
        }
    }

    fn handle_offline_message(&self, packet: OfflinePacketEnum, addr: SocketAddr) -> crate::raklib::error::Result<()> {
        let mut server = (self.server_interface_factory)(); // Get a server interface instance

        match packet {
            OfflinePacketEnum::UnconnectedPing(pk) => {
                debug!(%addr, ping_time=pk.send_ping_time, "Handling UnconnectedPing");
                let pong = packets::UnconnectedPong::create(pk.send_ping_time, self.server_id, "Rust RakNet Server".to_string() /* TODO: Get actual server name */);
                let mut stream = BinaryStream::new();
                pong.encode(&mut stream)?;
                server.send_raw(addr, stream.into_inner());
            }
            OfflinePacketEnum::UnconnectedPingOpenConnections(pk) => {
                // Respond same as UnconnectedPing for now
                debug!(%addr, ping_time=pk.0.send_ping_time, "Handling UnconnectedPingOpenConnections");
                let pong = packets::UnconnectedPong::create(pk.0.send_ping_time, self.server_id, "Rust RakNet Server".to_string());
                let mut stream = BinaryStream::new();
                pong.encode(&mut stream)?;
                server.send_raw(addr, stream.into_inner());
            }
            OfflinePacketEnum::OpenConnectionRequest1(pk) => {
                debug!(%addr, protocol=pk.protocol, mtu=pk.mtu_size, "Handling OpenConnectionRequest1");
                if !self.protocol_acceptor.accepts(pk.protocol) {
                    warn!(%addr, client_protocol = pk.protocol, server_protocol = self.protocol_acceptor.primary_version(), "Rejecting connection due to incompatible protocol");
                    let reply = packets::IncompatibleProtocolVersion::create(self.protocol_acceptor.primary_version(), self.server_id);
                    let mut stream = BinaryStream::new();
                    reply.encode(&mut stream)?;
                    server.send_raw(addr, stream.into_inner());
                } else {
                    // IP header (20) + UDP header (8) = 28
                    // Server should reply with its MTU size capability
                    let server_mtu = 1492; // TODO: Get actual max MTU from server config/socket
                    let reply_mtu = pk.mtu_size.min(server_mtu as u16); // Use client's request or server max, whichever is smaller
                    let reply = packets::OpenConnectionReply1::create(self.server_id, false /* No security */, reply_mtu);
                    let mut stream = BinaryStream::new();
                    reply.encode(&mut stream)?;
                    server.send_raw(addr, stream.into_inner());
                }
            }
            OfflinePacketEnum::OpenConnectionRequest2(pk) => {
                debug!(%addr, client_id=pk.client_id, mtu=pk.mtu_size, "Handling OpenConnectionRequest2");
                // TODO: Implement port checking from server config
                // TODO: Check if session already exists for addr (needs access to server state)
                // TODO: Check session limits

                let server_mtu = 1492; // TODO: Get actual max MTU from server config/socket
                let session_mtu = pk.mtu_size.min(server_mtu as u16);
                if session_mtu < crate::raklib::generic::session::Session::MIN_MTU_SIZE as u16 {
                    warn!(%addr, client_mtu = pk.mtu_size, session_mtu, "Rejecting connection due to small MTU size");
                    // Maybe send an error packet? RakLib PHP just drops it.
                    return Ok(());
                }

                let reply = packets::OpenConnectionReply2::create(
                    self.server_id,
                    InternetAddress::from_socket_addr(addr), // Client address as seen by server
                    session_mtu,
                    false, // No security
                );
                let mut stream = BinaryStream::new();
                reply.encode(&mut stream)?;
                server.send_raw(addr, stream.into_inner());

                // TODO: Tell the server to actually create the session object!
                // server.create_session(addr, pk.client_id, session_mtu); // Needs method on ServerInterface/Server
            }
        }
        Ok(())
    }
}


// Helper to decode offline packets based on ID
struct PacketDecoder; // Could hold packet pool if using cloning

#[derive(Debug)]
enum OfflinePacketEnum {
    UnconnectedPing(packets::UnconnectedPing),
    UnconnectedPingOpenConnections(packets::UnconnectedPingOpenConnections),
    OpenConnectionRequest1(packets::OpenConnectionRequest1),
    OpenConnectionRequest2(packets::OpenConnectionRequest2),
}

impl PacketDecoder {
    fn new() -> Self { Self }

    fn decode_offline<'a>(&self, payload: &'a [u8]) -> BinaryResult<Option<OfflinePacketEnum>> {
        if payload.is_empty() { return Ok(None); }
        let packet_id = payload[0];

        // Check magic bytes *before* full decode for efficiency
        if !self.check_offline_magic(payload) {
            // Might be a connected packet (ACK/NACK/Data) or garbage
            return Ok(None);
        }


        let mut stream = BinaryStream::from_slice(payload);

        match packet_id {
            MessageIdentifiers::ID_UNCONNECTED_PING => {
                // Decode requires consuming the magic bytes AFTER checking ID
                stream.get_u8()?; // Consume ID
                packets::UnconnectedPing::decode(&mut stream).map(OfflinePacketEnum::UnconnectedPing).map(Some)
            },
            MessageIdentifiers::ID_UNCONNECTED_PING_OPEN_CONNECTIONS => {
                stream.get_u8()?;
                packets::UnconnectedPingOpenConnections::decode(&mut stream).map(OfflinePacketEnum::UnconnectedPingOpenConnections).map(Some)
            },
            MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_1 => {
                // Need original length for MTU decode
                let initial_len = stream.len();
                stream.get_u8()?;
                packets::OpenConnectionRequest1::decode(&mut stream).map(|mut pk| {
                    pk.mtu_size = initial_len as u16; // Set MTU based on size
                    OfflinePacketEnum::OpenConnectionRequest1(pk)
                }).map(Some)
            },
            MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_2 => {
                stream.get_u8()?;
                packets::OpenConnectionRequest2::decode(&mut stream).map(OfflinePacketEnum::OpenConnectionRequest2).map(Some)
            },
            // Add other offline packets like IncompatibleProtocolVersion if needed
            _ => Ok(None), // Not a known offline packet ID we handle here
        }
    }

    // Helper to check magic bytes without full decode
    fn check_offline_magic(&self, payload: &[u8]) -> bool {
        // Offline messages have magic bytes after the ID byte
        const MAGIC_OFFSET: usize = 1;
        const MAGIC_LEN: usize = 16;
        if payload.len() < MAGIC_OFFSET + MAGIC_LEN {
            return false; // Too short to contain magic
        }
        let magic_slice = &payload[MAGIC_OFFSET..MAGIC_OFFSET + MAGIC_LEN];
        magic_slice == packets::UnconnectedPing::magic() // Use any OfflinePacket's magic()
    }
}
