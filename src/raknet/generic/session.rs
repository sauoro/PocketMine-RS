// src/raknet/generic/session.rs
#![allow(dead_code)]

use crate::log::{Logger, PrefixedLogger};
use crate::raknet::generic::disconnect_reason::DisconnectReason;
use crate::raknet::generic::error::PacketHandlingError;
use crate::raknet::generic::receive_reliability_layer::ReceiveReliabilityLayer;
use crate::raknet::generic::send_reliability_layer::SendReliabilityLayer;
use crate::raknet::protocol::acknowledge_packet::AcknowledgePacket;
use crate::raknet::protocol::ack::Ack;
use crate::raknet::protocol::connected_packet::ConnectedPacket;
use crate::raknet::protocol::connected_ping::ConnectedPing;
use crate::raknet::protocol::connected_pong::ConnectedPong;
use crate::raknet::protocol::datagram::Datagram;
use crate::raknet::protocol::disconnection_notification::DisconnectionNotification;
use crate::raknet::protocol::encapsulated_packet::EncapsulatedPacket;
use crate::raknet::protocol::message_identifiers::MessageIdentifiers;
use crate::raknet::protocol::nack::Nack;
use crate::raknet::protocol::packet::{Packet}; // Import Packet trait
use crate::raknet::protocol::packet_reliability::PacketReliability;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::utils::internet_address::InternetAddress;
use std::sync::{Arc}; // Removed Mutex for now
use std::time::{Duration, Instant};
use tokio::time::timeout; // For async timeouts

const DEFAULT_MAX_SPLIT_PACKET_PART_COUNT: u32 = 128;
const DEFAULT_MAX_CONCURRENT_SPLIT_COUNT: usize = 4;
const SESSION_TIMEOUT: Duration = Duration::from_secs(10);
const PING_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Connecting,
    Connected,
    DisconnectPending, // Graceful disconnect initiated
    DisconnectNotified, // DisconnectNotification sent, waiting for ACK/timeout
    Disconnected,
}

pub struct Session {
    logger: Box<dyn Logger>,
    address: InternetAddress,
    id: i64, // Client ID / GUID
    state: SessionState,

    last_update: Instant,
    disconnection_time: Option<Instant>, // Time disconnect was initiated
    is_active: bool, // Flag to track if any packet received in the last tick

    last_ping_time: Instant,
    last_ping_measure: Duration,

    recv_layer: ReceiveReliabilityLayer,
    send_layer: SendReliabilityLayer,

    // Callback for sending raw datagrams (usually set by Server/Client)
    raw_sender: Arc<dyn Fn(InternetAddress, bytes::Bytes) + Send + Sync>,

    // Callbacks for session events (usually set by Server/Client)
    on_packet_ack_callback: Arc<dyn Fn(u32) + Send + Sync>, // identifier_ack
    on_disconnect_callback: Arc<dyn Fn(DisconnectReason) + Send + Sync>,
    on_packet_receive_callback: Arc<dyn Fn(bytes::Bytes) + Send + Sync>, // User packet buffer
    on_ping_measure_callback: Arc<dyn Fn(u32) + Send + Sync>, // ping ms
    handle_raknet_connection_packet_callback: Arc<dyn Fn(bytes::Bytes) + Send + Sync>,
}

impl Session {
    pub const MIN_MTU_SIZE: u16 = 400;

    pub fn new(
        base_logger: Box<dyn Logger>,
        address: InternetAddress,
        client_id: i64,
        mtu_size: u16,
        raw_sender: Arc<dyn Fn(InternetAddress, bytes::Bytes) + Send + Sync>,
        on_packet_ack_callback: Arc<dyn Fn(u32) + Send + Sync>,
        on_disconnect_callback: Arc<dyn Fn(DisconnectReason) + Send + Sync>,
        on_packet_receive_callback: Arc<dyn Fn(bytes::Bytes) + Send + Sync>,
        on_ping_measure_callback: Arc<dyn Fn(u32) + Send + Sync>,
        handle_raknet_connection_packet_callback: Arc<dyn Fn(bytes::Bytes) + Send + Sync>,
        recv_max_split_parts: Option<u32>,
        recv_max_concurrent_splits: Option<usize>,
    ) -> Result<Self, String> {
        if mtu_size < Self::MIN_MTU_SIZE {
            return Err(format!("MTU size must be at least {}, got {}", Self::MIN_MTU_SIZE, mtu_size));
        }

        let logger = Box::new(PrefixedLogger::new(base_logger, format!("Session: {}", address)));

        let logger_clone_recv = logger.clone_boxed(); // Clone logger for recv layer
        let on_recv_closure = Box::new({
            let logger_clone = logger.clone_boxed();
            let packet_receive_cb = on_packet_receive_callback.clone();
            let connection_packet_cb = handle_raknet_connection_packet_callback.clone();
            let state_ref = Arc::new(std::sync::atomic::AtomicU8::new(SessionState::Connecting as u8)); // Share state atomically
            let state_ref_clone = state_ref.clone();

            move |pk: EncapsulatedPacket| {
                let current_state_val = state_ref_clone.load(std::sync::atomic::Ordering::Relaxed);
                let current_state = unsafe { std::mem::transmute::<u8, SessionState>(current_state_val) }; // Safe if only SessionState values are stored

                let id = pk.buffer.get(0).copied().unwrap_or(0xff); // Get packet ID
                if id < MessageIdentifiers::ID_USER_PACKET_ENUM {
                    if current_state == SessionState::Connecting {
                        (connection_packet_cb)(pk.buffer);
                    } else if id == MessageIdentifiers::ID_DISCONNECTION_NOTIFICATION {
                        // Handled separately in handle_remote_disconnect, called from handle_packet
                        logger_clone.debug("Received DisconnectionNotification");
                    } else if id == MessageIdentifiers::ID_CONNECTED_PING {
                        // Handled directly in handle_encapsulated_packet_route
                    } else if id == MessageIdentifiers::ID_CONNECTED_PONG {
                        // Handled directly in handle_encapsulated_packet_route
                    }
                } else if current_state == SessionState::Connected {
                    (packet_receive_cb)(pk.buffer);
                } else {
                    logger_clone.debug(&format!("Ignoring user packet 0x{:02X} in state {:?}", id, current_state));
                }
            }
        });


        let send_packet_closure = Box::new({
            let raw_sender_clone = raw_sender.clone();
            let address_clone = address.clone();
            move |packet: Box<dyn AcknowledgePacket>| {
                let mut serializer = PacketSerializer::new();
                // Create a mutable reference to the boxed trait object
                let mut packet_mut = packet; // Shadow the original binding
                let packet_ref = packet_mut.as_mut(); // Get a mutable ref to the trait object

                // Pass the mutable reference to the encode method
                if packet_ref.encode(&mut serializer).is_ok() {
                    (raw_sender_clone)(address_clone.clone(), serializer.into_inner().freeze());
                }
            }
        });

        let logger_clone_send = logger.clone_boxed(); // Clone logger for send layer
        let on_ack_closure = Box::new({
            let cb = on_packet_ack_callback.clone();
            move |id_ack| (cb)(id_ack)
        });


        let recv_layer = ReceiveReliabilityLayer::new(
            logger_clone_recv,
            on_recv_closure,
            send_packet_closure,
            recv_max_split_parts,
            recv_max_concurrent_splits,
        );

        let send_layer = SendReliabilityLayer::new(
            mtu_size,
            Box::new({
                let raw_sender_clone = raw_sender.clone();
                let address_clone = address.clone();
                move |datagram: Datagram| {
                    let mut serializer = PacketSerializer::new();
                    if datagram.encode(&mut serializer).is_ok() {
                        (raw_sender_clone)(address_clone.clone(), serializer.into_inner().freeze());
                    }
                }
            }),
            on_ack_closure,
            None, // Use default window size
        );


        Ok(Self {
            logger,
            address,
            id: client_id,
            state: SessionState::Connecting,
            last_update: Instant::now(),
            disconnection_time: None,
            is_active: false,
            last_ping_time: Instant::now(), // Initialize to now
            last_ping_measure: Duration::from_millis(1), // Default ping
            recv_layer,
            send_layer,
            raw_sender, // Store the Arc
            on_packet_ack_callback,
            on_disconnect_callback,
            on_packet_receive_callback,
            on_ping_measure_callback,
            handle_raknet_connection_packet_callback,
        })
    }

    // Helper to clone the logger without needing Clone trait on Logger itself
    pub fn clone_logger(&self) -> Box<dyn Logger> {
        self.logger.clone_boxed()
    }

    pub fn get_logger(&self) -> &dyn Logger {
        &*self.logger
    }

    pub fn get_address(&self) -> &InternetAddress {
        &self.address
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn get_state(&self) -> SessionState {
        self.state
    }

    pub fn set_state(&mut self, new_state: SessionState) {
        self.state = new_state;
        // Potentially update shared state if using Arc<AtomicU8> approach
    }

    pub fn is_temporary(&self) -> bool {
        self.state == SessionState::Connecting
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.state, SessionState::Connecting | SessionState::Connected)
    }

    pub fn update(&mut self, current_time: Instant) {
        if !self.is_active && self.last_update.elapsed() > SESSION_TIMEOUT {
            self.forcibly_disconnect(DisconnectReason::PeerTimeout);
            return;
        }

        if self.state == SessionState::DisconnectPending || self.state == SessionState::DisconnectNotified {
            if !self.send_layer.needs_update() && !self.recv_layer.needs_update() {
                if self.state == SessionState::DisconnectPending {
                    self.queue_connected_packet(DisconnectionNotification::default(), PacketReliability::RELIABLE_ORDERED, 0, true);
                    self.state = SessionState::DisconnectNotified;
                    self.logger.debug("All pending traffic flushed, sent disconnect notification");
                } else { // DisconnectNotified
                    self.state = SessionState::Disconnected;
                    self.logger.debug("Client cleanly disconnected, marking session for destruction");
                    return;
                }
            } else if let Some(disconnect_start) = self.disconnection_time {
                if disconnect_start.elapsed() > SESSION_TIMEOUT {
                    self.state = SessionState::Disconnected;
                    self.logger.debug("Timeout during graceful disconnect, forcibly closing session");
                    return;
                }
            }
        }


        self.is_active = false; // Reset activity flag for this tick

        self.recv_layer.update();
        self.send_layer.update(current_time);

        if self.last_ping_time.elapsed() > PING_INTERVAL {
            self.send_ping(PacketReliability::UNRELIABLE);
            self.last_ping_time = current_time;
        }

        self.last_update = current_time; // Update last active time
    }

    pub fn queue_connected_packet<P: ConnectedPacket + Default + 'static>(
        &mut self,
        mut packet: P,
        reliability: u8,
        order_channel: u8,
        immediate: bool,
    ) {
        if !self.is_connected() { return } // Don't queue if not connected/connecting

        let mut serializer = PacketSerializer::new();
        if packet.encode(&mut serializer).is_err() {
            self.logger.error(&format!("Failed to encode packet ID {}", P::get_id()));
            return;
        }

        let mut encapsulated = EncapsulatedPacket::new();
        encapsulated.reliability = reliability;
        encapsulated.order_channel = Some(order_channel);
        encapsulated.buffer = serializer.into_inner().freeze(); // Get Bytes

        self.add_encapsulated_to_queue(encapsulated, immediate);
    }

    pub fn add_encapsulated_to_queue(&mut self, packet: EncapsulatedPacket, immediate: bool) {
        if !self.is_connected() { return } // Don't queue if not connected/connecting
        self.send_layer.add_encapsulated_to_queue(packet, immediate);
    }

    pub fn send_ping(&mut self, reliability: u8) {
        let ping_time = self.get_raknet_time_ms();
        self.queue_connected_packet(ConnectedPing::create(ping_time), reliability, 0, true);
    }

    fn handle_encapsulated_packet_route(&mut self, packet: EncapsulatedPacket) {
        let id = packet.buffer.get(0).copied().unwrap_or(0xff);
        if id < MessageIdentifiers::ID_USER_PACKET_ENUM {
            if self.state == SessionState::Connecting {
                (self.handle_raknet_connection_packet_callback)(packet.buffer);
            } else if id == MessageIdentifiers::ID_DISCONNECTION_NOTIFICATION {
                self.handle_remote_disconnect();
            } else if id == MessageIdentifiers::ID_CONNECTED_PING {
                let mut serializer = PacketSerializer::from_bytes(&packet.buffer);
                match ConnectedPing::default().decode(&mut serializer) {
                    Ok(mut ping_packet) => {
                        let pong_time = self.get_raknet_time_ms();
                        self.queue_connected_packet(ConnectedPong::create(ping_packet.send_ping_time, pong_time), PacketReliability::UNRELIABLE, 0, true);
                    }
                    Err(e) => self.logger.error(&format!("Error decoding ConnectedPing: {}", e)),
                }
            } else if id == MessageIdentifiers::ID_CONNECTED_PONG {
                let mut serializer = PacketSerializer::from_bytes(&packet.buffer);
                match ConnectedPong::default().decode(&mut serializer) {
                    Ok(pong_packet) => {
                        self.handle_pong(pong_packet.send_ping_time, pong_packet.send_pong_time);
                    }
                    Err(e) => self.logger.error(&format!("Error decoding ConnectedPong: {}", e)),
                }
            }
        } else if self.state == SessionState::Connected {
            (self.on_packet_receive_callback)(packet.buffer);
        } else {
            self.logger.debug(&format!("Ignoring user packet 0x{:02X} in state {:?}", id, self.state));
        }
    }


    fn handle_pong(&mut self, send_ping_time: u64, _send_pong_time: u64) {
        // TODO: Handle clock differential (_send_pong_time)
        let current_time = self.get_raknet_time_ms();
        if current_time < send_ping_time {
            self.logger.debug(&format!("Received invalid pong: timestamp {} is in the future by {} ms", send_ping_time, send_ping_time - current_time));
        } else {
            let ping_ms = (current_time - send_ping_time) as u32; // Cast to u32 for callback
            self.last_ping_measure = Duration::from_millis(ping_ms as u64);
            (self.on_ping_measure_callback)(ping_ms);
        }
    }

    pub fn handle_packet(&mut self, packet_bytes: bytes::Bytes) -> Result<(), PacketHandlingError> {
        self.is_active = true;
        // self.last_update = Instant::now(); // update() sets this

        let packet_id = packet_bytes.get(0).copied().unwrap_or(0xff);

        // Attempt to decode based on ID
        let mut serializer = PacketSerializer::from_bytes(&packet_bytes);

        match packet_id {
            // Datagram (most common case)
            id if (id & Datagram::BITFLAG_VALID) != 0 => {
                // Check if it's ACK or NACK first based on flags in the first byte
                if (id & Datagram::BITFLAG_ACK) != 0 {
                    let mut ack = Ack::new();
                    ack.decode(&mut serializer)?;
                    self.send_layer.on_ack(ack);
                } else if (id & Datagram::BITFLAG_NAK) != 0 {
                    let mut nack = Nack::new();
                    nack.decode(&mut serializer)?;
                    self.send_layer.on_nack(nack);
                } else {
                    // Regular datagram
                    // Datagram::decode consumes the serializer, create a new one for it
                    let mut datagram_serializer = PacketSerializer::from_bytes(&packet_bytes);
                    let datagram = Datagram::decode(&mut datagram_serializer)?;
                    self.recv_layer.on_datagram(datagram)?;
                }
            }
            // Standalone ACK/NACK (less common, but possible)
            MessageIdentifiers::ID_ACK => {
                let mut ack = Ack::new();
                ack.decode(&mut serializer)?;
                self.send_layer.on_ack(ack);
            }
            MessageIdentifiers::ID_NACK => {
                let mut nack = Nack::new();
                nack.decode(&mut serializer)?;
                self.send_layer.on_nack(nack);
            }
            // Handle other potential top-level packets if needed
            _ => {
                self.logger.debug(&format!("Received unhandled top-level packet ID 0x{:02X}", packet_id));
            }
        }

        Ok(())
    }

    pub fn initiate_disconnect(&mut self, reason: DisconnectReason) {
        if self.is_connected() {
            self.state = SessionState::DisconnectPending;
            self.disconnection_time = Some(Instant::now());
            (self.on_disconnect_callback)(reason);
            self.logger.debug(&format!("Requesting graceful disconnect because \"{}\"", reason));
        }
    }

    pub fn forcibly_disconnect(&mut self, reason: DisconnectReason) {
        if self.state != SessionState::Disconnected {
            self.state = SessionState::Disconnected;
            (self.on_disconnect_callback)(reason);
            self.logger.debug(&format!("Forcibly disconnecting session due to {}", reason));
        }
    }

    fn handle_remote_disconnect(&mut self) {
        // The client sent DisconnectionNotification
        // Ensure ACKs for it are processed by ticking layers one last time?
        // Or just assume it's closing.
        self.recv_layer.update(); // Try to send ACK for the notification

        if self.is_connected() {
            // Avoid double-calling on_disconnect if we initiated it first
            (self.on_disconnect_callback)(DisconnectReason::ClientDisconnect);
        }
        self.state = SessionState::Disconnected;
        self.logger.debug("Terminating session due to client disconnect");
    }

    pub fn is_fully_disconnected(&self) -> bool {
        self.state == SessionState::Disconnected
    }

    // RakNet time calculation (milliseconds since arbitrary start)
    fn get_raknet_time_ms(&self) -> u64 {
        // Using Instant::now() is fine for relative pings, but doesn't match RakNet's epoch.
        // For absolute time synchronization (which RakNet does), a proper epoch sync is needed.
        // For now, elapsed time since start is sufficient for ping.
        Instant::now().elapsed().as_millis() as u64 // Simple elapsed time for ping
    }
}

// Implement Clone manually if needed, requires cloning Arcs and potentially state.
// Usually, sessions aren't cloned but managed by ID.

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("address", &self.address)
            .field("id", &self.id)
            .field("state", &self.state)
            .field("last_update", &self.last_update)
            .field("last_ping_measure", &self.last_ping_measure)
            .finish_non_exhaustive() // Don't print layers/callbacks
    }
}