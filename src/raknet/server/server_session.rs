// src/raknet/server/server_session.rs
#![allow(dead_code)]

use crate::log::Logger;
use crate::raknet::generic::disconnect_reason::DisconnectReason;
use crate::raknet::generic::session::{Session, SessionState};
use crate::raknet::protocol::connection_request::ConnectionRequest;
use crate::raknet::protocol::connection_request_accepted::ConnectionRequestAccepted;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::new_incoming_connection::NewIncomingConnection;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_reliability::PacketReliability;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::server::server_event_listener::ServerEventListener;
use crate::raknet::utils::internet_address::InternetAddress;
use bytes::Bytes;
use std::sync::Arc;
use std::time::Duration;

pub struct ServerSession {
    server_id: u64,
    internal_id: u64, // Unique ID within this server instance
    base_session: Session,
    event_listener: Arc<dyn ServerEventListener>,
}

impl ServerSession {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        server_id: u64,
        internal_id: u64,
        logger: Box<dyn Logger>,
        address: InternetAddress,
        client_id: i64,
        mtu_size: u16,
        raw_sender: Arc<dyn Fn(InternetAddress, bytes::Bytes) + Send + Sync>,
        event_listener: Arc<dyn ServerEventListener>,
        recv_max_split_parts: Option<u32>,
        recv_max_concurrent_splits: Option<usize>,
    ) -> Result<Self, String> {
        let event_listener_clone = event_listener.clone();
        let internal_id_clone = internal_id;
        let address_clone = address.clone();
        let client_id_clone = client_id;
        let server_id_clone = server_id; // Clone server_id

        let base_session = Session::new(
            logger,
            address.clone(),
            client_id,
            mtu_size,
            raw_sender,
            // on_packet_ack
            Arc::new({
                let listener = event_listener_clone.clone();
                let id = internal_id_clone;
                move |ack_id| {
                    let listener_clone = listener.clone(); // Clone Arc for the async block
                    tokio::spawn(async move {
                        listener_clone.on_packet_ack(id, ack_id).await;
                    });
                }
            }),
            // on_disconnect
            Arc::new({
                let listener = event_listener_clone.clone();
                let id = internal_id_clone;
                move |reason| {
                    let listener_clone = listener.clone(); // Clone Arc for the async block
                    tokio::spawn(async move {
                        listener_clone.on_client_disconnect(id, reason).await;
                    });
                }
            }),
            // on_packet_receive (User packets)
            Arc::new({
                let listener = event_listener_clone.clone();
                let id = internal_id_clone;
                move |packet_bytes| {
                    let listener_clone = listener.clone(); // Clone Arc for the async block
                    tokio::spawn(async move {
                        listener_clone.on_packet_receive(id, packet_bytes).await;
                    });
                }
            }),
            // on_ping_measure
            Arc::new({
                let listener = event_listener_clone.clone();
                let id = internal_id_clone;
                move |ping_ms| {
                    let listener_clone = listener.clone(); // Clone Arc for the async block
                    tokio::spawn(async move {
                        listener_clone.on_ping_measure(id, ping_ms).await;
                    });
                }
            }),
            // handle_raknet_connection_packet
            Arc::new({
                // Need mutable access to self here, tricky with Arc closure.
                // Pass necessary methods/state instead.
                // This closure is called from within the Session's receive layer.
                // We need a way to call methods on `self` (or `base_session`)
                // from here.
                // This might require refactoring Session to take `Arc<Mutex<Self>>`
                // or using channels.

                // Placeholder: Log the packet for now. Proper handling needs Session refactor.
                move |packet_bytes: bytes::Bytes| {
                    let packet_id = packet_bytes.get(0).copied().unwrap_or(0xff);
                    eprintln!("ServerSession received RakNet connection packet: ID 0x{:02X}", packet_id);
                    // Call self.handle_raknet_connection_packet(packet_bytes) here somehow
                }
            }),
            recv_max_split_parts,
            recv_max_concurrent_splits,
        )?;

        Ok(Self {
            server_id,
            internal_id,
            base_session,
            event_listener,
        })
    }

    pub fn get_internal_id(&self) -> u64 {
        self.internal_id
    }

    // Delegate methods to base_session
    pub fn get_address(&self) -> &InternetAddress { self.base_session.get_address() }
    pub fn get_id(&self) -> i64 { self.base_session.get_id() }
    pub fn get_state(&self) -> SessionState { self.base_session.get_state() }
    pub fn is_temporary(&self) -> bool { self.base_session.is_temporary() }
    pub fn is_connected(&self) -> bool { self.base_session.is_connected() }
    pub fn update(&mut self, current_time: std::time::Instant) { self.base_session.update(current_time) }
    pub fn add_encapsulated_to_queue(&mut self, packet: EncapsulatedPacket, immediate: bool) { self.base_session.add_encapsulated_to_queue(packet, immediate) }
    pub fn handle_packet(&mut self, packet_bytes: Bytes) -> Result<(), crate::raknet::generic::error::PacketHandlingError> { self.base_session.handle_packet(packet_bytes) }
    pub fn initiate_disconnect(&mut self, reason: DisconnectReason) { self.base_session.initiate_disconnect(reason) }
    pub fn forcibly_disconnect(&mut self, reason: DisconnectReason) { self.base_session.forcibly_disconnect(reason) }
    pub fn is_fully_disconnected(&self) -> bool { self.base_session.is_fully_disconnected() }
    pub fn get_logger(&self) -> &dyn Logger { self.base_session.get_logger() }

    // Server specific connection logic (called from base_session's callback)
    pub fn handle_raknet_connection_packet(&mut self, packet_bytes: Bytes) {
        let packet_id = packet_bytes.get(0).copied().unwrap_or(0xff);
        let mut serializer = PacketSerializer::from_bytes(&packet_bytes);

        match packet_id {
            MessageIdentifiers::ID_CONNECTION_REQUEST => {
                let mut request = ConnectionRequest { client_id: 0, send_ping_time: 0, use_security: false }; // Default values
                if request.decode(&mut serializer).is_ok() {
                    // TODO: Security checks?
                    let reply = ConnectionRequestAccepted::create(
                        self.get_address().clone(),
                        vec![], // System addresses (usually empty for server -> client)
                        request.send_ping_time,
                        self.base_session.get_raknet_time_ms(), // Use base_session's time method
                    );
                    self.base_session.queue_connected_packet(reply, PacketReliability::UNRELIABLE, 0, true);
                } else {
                    self.get_logger().error("Failed to decode ConnectionRequest");
                }
            }
            MessageIdentifiers::ID_NEW_INCOMING_CONNECTION => {
                let mut connect_packet = NewIncomingConnection {
                    address: self.get_address().clone(), // Placeholder
                    system_addresses: vec![],
                    send_ping_time: 0,
                    send_pong_time: 0,
                };
                if connect_packet.decode(&mut serializer).is_ok() {
                    // TODO: Port checking? Compare connect_packet.address with self.server_id/port?
                    // The PHP code compares connect_packet.address.port with server.getPort()
                    // Need access to server port here. For now, assume port check passed or is disabled.

                    if self.base_session.get_state() == SessionState::Connecting {
                        self.base_session.set_state(SessionState::Connected); // Mark as connected
                        // Trigger event
                        let listener = self.event_listener.clone();
                        let session_id = self.internal_id;
                        let addr = self.get_address().clone();
                        let client_id = self.get_id();
                        tokio::spawn(async move {
                            listener.on_client_connect(session_id, addr.to_string_addr_only(), addr.port(), client_id).await;
                        });

                        self.base_session.send_ping(PacketReliability::UNRELIABLE); // Send initial ping
                    } else {
                        self.get_logger().debug("Received NewIncomingConnection in non-connecting state");
                    }
                } else {
                    self.get_logger().error("Failed to decode NewIncomingConnection");
                }
            }
            _ => {
                self.get_logger().debug(&format!("Unhandled RakNet connection packet ID 0x{:02X}", packet_id));
            }
        }
    }
}

impl std::fmt::Debug for ServerSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerSession")
            .field("internal_id", &self.internal_id)
            .field("base_session", &self.base_session)
            .finish_non_exhaustive()
    }
}