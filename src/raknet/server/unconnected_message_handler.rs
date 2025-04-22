// src/raknet/server/unconnected_message_handler.rs
#![allow(dead_code)]

use crate::log::Logger;
use crate::raknet::generic::error::PacketHandlingError;
use crate::raknet::generic::session::Session; // For MIN_MTU_SIZE
use crate::raknet::protocol::incompatible_protocol_version::IncompatibleProtocolVersion;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::offline_message::OfflineMessage;
use crate::raknet::protocol::open_connection_reply1::OpenConnectionReply1;
use crate::raknet::protocol::open_connection_reply2::OpenConnectionReply2;
use crate::raknet::protocol::open_connection_request1::OpenConnectionRequest1;
use crate::raknet::protocol::open_connection_request2::OpenConnectionRequest2;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::protocol::unconnected_ping::UnconnectedPing;
use crate::raknet::protocol::unconnected_ping_open_connections::UnconnectedPingOpenConnections;
use crate::raknet::protocol::unconnected_pong::UnconnectedPong;
use crate::raknet::server::protocol_acceptor::ProtocolAcceptor;
use crate::raknet::server::server_session::ServerSession;
use crate::raknet::server::server_socket::ServerSocket;
use crate::raknet::utils::internet_address::InternetAddress;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex; // Assuming ServerSession needs async locking

type SessionExistsFn = Box<dyn Fn(&InternetAddress) -> bool + Send + Sync>;
type CreateSessionFn = Box<dyn Fn(InternetAddress, i64, u16) -> Arc<Mutex<ServerSession>> + Send + Sync>; // Returns Arc<Mutex<ServerSession>>
type GetPortCheckingFn = Box<dyn Fn() -> bool + Send + Sync>;
type GetMaxMtuFn = Box<dyn Fn() -> u16 + Send + Sync>;
type GetServerNameFn = Box<dyn Fn() -> String + Send + Sync>;
type RawSenderFn = Arc<dyn Fn(InternetAddress, bytes::Bytes) + Send + Sync>;


pub struct UnconnectedMessageHandler {
    protocol_acceptor: Arc<dyn ProtocolAcceptor>,
    server_id: u64,
    logger: Box<dyn Logger>,
    // packet_pool: HashMap<u8, Box<dyn OfflineMessage>>, // Dynamic dispatch is complex for Default/Clone

    // Callbacks to access server state/methods
    session_exists: Option<SessionExistsFn>,
    create_session: Option<CreateSessionFn>,
    get_port_checking: Option<GetPortCheckingFn>,
    get_max_mtu: Option<GetMaxMtuFn>,
    get_server_name: Option<GetServerNameFn>,
    raw_sender: Option<RawSenderFn>,
}

impl UnconnectedMessageHandler {
    pub fn new(
        protocol_acceptor: Arc<dyn ProtocolAcceptor>,
        server_id: u64,
        logger: Box<dyn Logger>,
        _socket: Arc<ServerSocket>, // Socket needed for sending replies directly if not using Server's sender
    ) -> Self {
        Self {
            protocol_acceptor,
            server_id,
            logger,
            // packet_pool: Self::create_packet_pool(),
            session_exists: None,
            create_session: None,
            get_port_checking: None,
            get_max_mtu: None,
            get_server_name: None,
            raw_sender: None,
        }
    }

    // Set server access callbacks after construction
    pub fn set_server_access(
        &mut self,
        session_exists: SessionExistsFn,
        create_session: CreateSessionFn,
        get_port_checking: GetPortCheckingFn,
        get_max_mtu: GetMaxMtuFn,
        get_server_name: GetServerNameFn,
        raw_sender: RawSenderFn,
    ) {
        self.session_exists = Some(session_exists);
        self.create_session = Some(create_session);
        self.get_port_checking = Some(get_port_checking);
        self.get_max_mtu = Some(get_max_mtu);
        self.get_server_name = Some(get_server_name);
        self.raw_sender = Some(raw_sender);
    }

    // Simple packet pool alternative: direct matching
    fn decode_packet(&self, buffer: &[u8]) -> Result<Box<dyn OfflineMessage>, PacketHandlingError> {
        if buffer.is_empty() {
            return Err(PacketHandlingError::new("Empty buffer".to_string(), crate::raknet::generic::disconnect_reason::DisconnectReason::BadPacket));
        }
        let id = buffer[0];
        let mut stream = PacketSerializer::from_bytes(buffer);

        match id {
            MessageIdentifiers::ID_UNCONNECTED_PING => Ok(Box::new(UnconnectedPing::try_decode(&mut stream)?)),
            MessageIdentifiers::ID_UNCONNECTED_PING_OPEN_CONNECTIONS => Ok(Box::new(UnconnectedPingOpenConnections::try_decode(&mut stream)?)),
            MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_1 => Ok(Box::new(OpenConnectionRequest1::try_decode(&mut stream)?)),
            MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_2 => Ok(Box::new(OpenConnectionRequest2::try_decode(&mut stream)?)),
            // Add other offline message IDs here if needed
            _ => Err(PacketHandlingError::new(format!("Unknown offline packet ID 0x{:02X}", id), crate::raknet::generic::disconnect_reason::DisconnectReason::BadPacket)),
        }
    }


    pub async fn handle_raw(&mut self, payload: &[u8], address: InternetAddress) -> Result<bool, PacketHandlingError> {
        if payload.is_empty() {
            return Ok(false); // Ignore empty
        }

        let packet = match self.decode_packet(payload) {
            Ok(p) => p,
            Err(_) => return Ok(false), // Could not decode as known offline packet
        };

        // Check magic bytes if the packet type requires it (should be done in decode)
        // Assuming decode handles magic validation for relevant types.

        self.handle(packet, address).await
    }

    async fn handle(&mut self, packet: Box<dyn OfflineMessage>, address: InternetAddress) -> Result<bool, PacketHandlingError> {
        // We need to downcast the trait object to access specific fields
        let packet_id = packet.get_buffer().get(0).copied().unwrap_or(0xff); // Get ID from buffer

        // Get necessary server state via callbacks
        let raw_sender = self.raw_sender.as_ref().ok_or_else(|| PacketHandlingError::new("Raw sender not configured".to_string(), crate::raknet::generic::disconnect_reason::DisconnectReason::ServerShutdown))?.clone();

        match packet_id {
            MessageIdentifiers::ID_UNCONNECTED_PING | MessageIdentifiers::ID_UNCONNECTED_PING_OPEN_CONNECTIONS => {
                if let Some(ping) = packet.as_any().downcast_ref::<UnconnectedPing>() {
                    let get_name_fn = self.get_server_name.as_ref().unwrap(); // Assume set
                    let server_name = (get_name_fn)();
                    let pong = UnconnectedPong::create(ping.send_ping_time, self.server_id, server_name);
                    self.send_packet(pong, address, raw_sender).await?;
                } else if let Some(ping_oc) = packet.as_any().downcast_ref::<UnconnectedPingOpenConnections>() {
                    let get_name_fn = self.get_server_name.as_ref().unwrap(); // Assume set
                    let server_name = (get_name_fn)();
                    let pong = UnconnectedPong::create(ping_oc.send_ping_time, self.server_id, server_name);
                    self.send_packet(pong, address, raw_sender).await?;
                } else {
                    return Err(PacketHandlingError::new("Failed to downcast UnconnectedPing".to_string(), crate::raknet::generic::disconnect_reason::DisconnectReason::BadPacket));
                }
            }
            MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_1 => {
                if let Some(request1) = packet.as_any().downcast_ref::<OpenConnectionRequest1>() {
                    if !self.protocol_acceptor.accepts(request1.protocol) {
                        let incompatible = IncompatibleProtocolVersion::create(self.protocol_acceptor.get_primary_version(), self.server_id);
                        self.send_packet(incompatible, address.clone(), raw_sender).await?;
                        self.logger.notice(&format!("Refused connection from {} due to incompatible RakNet protocol version (version {})", address, request1.protocol));
                    } else {
                        // Adjust MTU based on request and server max
                        let get_max_mtu_fn = self.get_max_mtu.as_ref().unwrap(); // Assume set
                        let max_mtu = (get_max_mtu_fn)();
                        // The MTU in request1 is the *sender's* MTU. The reply tells them *our* MTU.
                        // We should use our max_mtu_size, adjusted for headers.
                        let reply_mtu = max_mtu; // Send our max MTU

                        let reply1 = OpenConnectionReply1::create(self.server_id, false, reply_mtu); // security=false for now
                        self.send_packet(reply1, address, raw_sender).await?;
                    }
                } else {
                    return Err(PacketHandlingError::new("Failed to downcast OpenConnectionRequest1".to_string(), crate::raknet::generic::disconnect_reason::DisconnectReason::BadPacket));
                }
            }
            MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_2 => {
                if let Some(request2) = packet.as_any().downcast_ref::<OpenConnectionRequest2>() {
                    let get_port_checking_fn = self.get_port_checking.as_ref().unwrap(); // Assume set
                    let port_checking_enabled = (get_port_checking_fn)();

                    // Port checking needs the server's actual listening port.
                    // This requires access to the ServerSocket's bind address.
                    // For now, assume port check passes or is disabled.
                    // let server_port = self.socket.get_bind_address().port();
                    // if port_checking_enabled && request2.server_address.port() != server_port {
                    //     self.logger.debug(&format!("Not creating session for {} due to mismatched port", address));
                    //     return Ok(true); // Handled (ignored)
                    // }

                    if request2.mtu_size < Session::MIN_MTU_SIZE {
                        self.logger.debug(&format!("Not creating session for {} due to bad MTU size {}", address, request2.mtu_size));
                        return Ok(true); // Handled (ignored)
                    }

                    let get_max_mtu_fn = self.get_max_mtu.as_ref().unwrap(); // Assume set
                    let max_mtu = (get_max_mtu_fn)();
                    let final_mtu = request2.mtu_size.min(max_mtu);

                    let session_exists_fn = self.session_exists.as_ref().unwrap(); // Assume set
                    if (session_exists_fn)(&address) {
                        // Server::handle_raw_packet checks this already, but double check here.
                        self.logger.debug(&format!("Not creating session for {} due to session already opened", address));
                        return Ok(true); // Handled (ignored)
                    }

                    let reply2 = OpenConnectionReply2::create(self.server_id, address.clone(), final_mtu, false); // security=false
                    self.send_packet(reply2, address.clone(), raw_sender).await?;

                    // Create the session
                    let create_session_fn = self.create_session.as_ref().unwrap(); // Assume set
                    let _session = (create_session_fn)(address, request2.client_id, final_mtu);
                    // Session is now managed by the Server

                } else {
                    return Err(PacketHandlingError::new("Failed to downcast OpenConnectionRequest2".to_string(), crate::raknet::generic::disconnect_reason::DisconnectReason::BadPacket));
                }
            }
            _ => return Ok(false), // Not handled by this handler
        }

        Ok(true) // Packet was handled
    }

    async fn send_packet<P: Packet + OfflineMessage + Send + 'static>(
        &self,
        mut packet: P,
        address: InternetAddress,
        raw_sender: RawSenderFn,
    ) -> Result<(), PacketHandlingError> {
        let mut stream = PacketSerializer::new();
        packet.encode(&mut stream)?; // encode requires mutable reference
        (raw_sender)(address, stream.into_inner().freeze());
        Ok(())
    }

    // Helper to get a mutable reference to a trait object if needed
    // fn get_packet_mut<'a>(&'a mut self, id: u8) -> Option<&'a mut (dyn OfflineMessage + 'static)> {
    //     self.packet_pool.get_mut(&id).map(|b| b.as_mut())
    // }
}

// Implement Any for trait objects used in downcasting
use std::any::Any;
impl dyn OfflineMessage + Send + Sync {
    fn as_any(&self) -> &dyn Any {
        // This requires modifying the OfflineMessage trait or using a crate like `downcast-rs`
        // For simplicity, we might avoid this pattern and use direct matching in handle()
        panic!("as_any not implemented for dyn OfflineMessage");
    }
}

// We need to implement the Packet trait for Box<dyn OfflineMessage> to use it in `handle` easily
// This is complex. It's easier to match on the ID and decode directly.