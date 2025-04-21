// src/raklib/generic/session.rs
use super::reliability::{
    AcknowledgePacketWrapper, ReceiveReliabilityLayer, ReliableCacheEntry, SendReliabilityLayer,
};
use crate::raklib::error::{DisconnectReason, RakLibError, Result};
use crate::raklib::protocol::{
    self, // Import the protocol module itself for easier access
    packets::{ConnectedPing, ConnectedPong, DisconnectionNotification}, // Import specific packets
    ConnectedPacket, // Import the trait
    Datagram,
    EncapsulatedPacket, MessageIdentifiers, NACK, ACK, // Specific ACK/NACK types
    Packet,
    PacketReliability,
};
use crate::raklib::utils::InternetAddress; // Although not used directly here, it's part of the context
use crate::utils::binary::{BinaryStream, Result as BinaryResult}; // For packet decoding within session
use std::collections::HashMap; // Need explicit import
use std::net::SocketAddr;
use std::sync::Arc; // Use Arc for callbacks if they need shared access
use std::time::{Duration, Instant};
use tracing::{debug, error, trace, warn, Instrument, Span}; // For logging and tracing spans

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Initial state, performing RakNet handshake (OpenConnectionRequest/Reply 1 & 2).
    ConnectingOffline,
    /// Handshake part 1 done, exchanging ConnectionRequest/Accepted.
    ConnectingOnline,
    /// Connection established, normal operation.
    Connected,
    /// Graceful disconnect initiated, waiting for ACKs before sending notification.
    DisconnectingGraceful,
    /// Disconnect notification sent, waiting for final ACKs/timeout.
    DisconnectingNotified,
    /// Session is closed and ready for removal.
    Disconnected,
}

/// Metadata associated with a session.
#[derive(Debug)] // Removed Clone as Span is not Clone
pub struct SessionMeta {
    /// Unique ID assigned by the server/client for this session instance.
    pub internal_id: u64,
    /// Remote peer's address.
    pub addr: SocketAddr,
    /// RakNet client GUID provided during handshake.
    pub client_id: u64,
    /// Negotiated MTU size for this session.
    pub mtu_size: u16,
    /// Tracing span associated with this session for correlated logging.
    pub span: Span,
}

/// Function type for sending raw datagrams (Datagram, ACK, NACK) over the socket.
/// Changed to use Arc for cloneability needed by layers.
type SendRawPacketFn = Arc<dyn Fn(SocketAddr, Vec<u8>) + Send + Sync>;
/// Function type called when a packet sent with identifier_ack is fully ACKed.
type OnPacketAckFn = Arc<dyn Fn(u32) + Send + Sync>;
/// Function type called when the session disconnects for any reason.
type OnDisconnectFn = Arc<dyn Fn(DisconnectReason) + Send + Sync>;
/// Function type called when a user data packet is received.
type OnPacketReceiveFn = Arc<dyn Fn(Vec<u8>) + Send + Sync>;
/// Function type called when a new ping measurement is available.
type OnPingMeasureFn = Arc<dyn Fn(u32) + Send + Sync>; // Changed from u64 to u32 for ms

/// Represents a RakNet connection session, handling reliability, ordering, and connection state.
/// This is generic and used by both ServerSession and ClientSession.
pub struct Session {
    meta: SessionMeta,
    state: SessionState,
    send_layer: SendReliabilityLayer,
    recv_layer: ReceiveReliabilityLayer,
    last_activity_time: Instant, // Time anything was received from the peer
    last_update_time: Instant,   // Time the session's update() was last called
    last_ping_time: Instant,     // Time the last ping was sent
    disconnection_time: Option<Instant>, // Time graceful disconnect was initiated

    // Ping measurement
    last_ping_measure_milli: u32, // Store ping in milliseconds
    // We need to store ping send times to calculate RTT for pongs
    // Map ping send time (local RakNet time) -> Instant sent (wall clock time)
    ping_send_times: HashMap<u64, Instant>,

    // Callbacks provided by the specific implementation (Server/Client)
    // Stored as Arcs
    send_raw_packet_cb: SendRawPacketFn,
    on_packet_ack_cb: OnPacketAckFn,
    on_disconnect_cb: OnDisconnectFn,
    on_packet_receive_cb: OnPacketReceiveFn,
    on_ping_measure_cb: OnPingMeasureFn,
}

// --- Constants ---
impl Session {
    pub const PING_INTERVAL: Duration = Duration::from_secs(5);
    pub const TIMEOUT_DURATION: Duration = Duration::from_secs(30); // Increased timeout
    pub const DISCONNECT_TIMEOUT: Duration = Duration::from_secs(5); // Reduced graceful disconnect timeout
    pub const MIN_MTU_SIZE: usize = 400; // Copied from reliability layer for visibility
    const MAX_PING_TRACKING: usize = 10; // Limit number of outstanding pings tracked
}

// --- Constructor and Core Logic ---
impl Session {
    /// Creates a new generic Session. Callers must provide callbacks as Arcs.
    #[allow(clippy::too_many_arguments)] // Necessary complexity for callbacks
    pub fn new(
        meta: SessionMeta,
        initial_state: SessionState, // Allow starting in different states (e.g., ConnectingOffline vs ConnectingOnline)
        send_raw_packet_cb: SendRawPacketFn, // Must be Arc
        on_packet_ack_cb: OnPacketAckFn,     // Must be Arc
        on_disconnect_cb: OnDisconnectFn,   // Must be Arc
        on_packet_receive_cb: OnPacketReceiveFn, // Must be Arc
        on_ping_measure_cb: OnPingMeasureFn, // Must be Arc
    ) -> Self {
        let span_guard = meta.span.enter(); // Enter span for initialization logs
        debug!(state = ?initial_state, "Creating new session");

        let addr = meta.addr; // Capture addr for closures

        // --- Callbacks for SendReliabilityLayer ---
        let send_raw_packet_cb_clone = Arc::clone(&send_raw_packet_cb);
        let send_datagram_callback = Box::new(move |datagram: Datagram| {
            let mut stream = BinaryStream::new();
            if let Err(e) = datagram.encode(&mut stream) {
                error!(error=%e, seq=datagram.seq_number, "Failed to encode datagram");
                return;
            }
            trace!(seq=datagram.seq_number, len=stream.len(), packet_count=datagram.packets.len(), "Sending Datagram");
            (send_raw_packet_cb_clone)(addr, stream.into_inner()); // Use cloned Arc
        });

        let on_packet_ack_cb_clone = Arc::clone(&on_packet_ack_cb);
        let on_ack_callback = Box::new(move |ack_id: u32| {
            (on_packet_ack_cb_clone)(ack_id); // Use cloned Arc
        });

        // --- Callbacks for ReceiveReliabilityLayer ---
        let send_raw_packet_cb_clone2 = Arc::clone(&send_raw_packet_cb);
        let send_ack_nack_callback = Box::new(move |ack_nack: AcknowledgePacketWrapper| {
            let mut stream = BinaryStream::new();
            if let Err(e) = ack_nack.encode(&mut stream) {
                error!(error=%e, pk_id=ack_nack.id(), "Failed to encode ACK/NACK packet");
                return;
            }
            trace!(pk_id=ack_nack.id(), len=stream.len(), "Sending ACK/NACK");
            (send_raw_packet_cb_clone2)(addr, stream.into_inner());
        });

        // Placeholder callback for RecvLayer - will be removed when RecvLayer returns packets
        let on_recv_callback_for_layer = Box::new(|_packet: EncapsulatedPacket| {
            // This logic will move into Session::update
            unimplemented!("RecvLayer callback should not be used directly; Session::update processes returned packets.");
        });

        // Initialize reliability layers
        let send_layer = SendReliabilityLayer::new(
            meta.mtu_size,
            512, // Default reliable window size, TODO: make configurable
            send_datagram_callback,
            on_ack_callback,
        );

        // TODO: Update RecvLayer::new signature to remove on_recv_callback_for_layer
        let recv_layer = ReceiveReliabilityLayer::new(
            512, // Default window size
            128, 4, // Default split limits, TODO: make configurable
            on_recv_callback_for_layer, // REMOVE THIS ARGUMENT
            send_ack_nack_callback,
        );

        // Drop the temporary span guard
        drop(span_guard);

        Self {
            meta,
            state: initial_state,
            send_layer,
            recv_layer,
            last_activity_time: Instant::now(),
            last_update_time: Instant::now(),
            last_ping_time: Instant::now(), // Send initial ping soon after connect?
            disconnection_time: None,
            last_ping_measure_milli: 0,
            ping_send_times: HashMap::new(),
            // Store the original Arcs
            send_raw_packet_cb,
            on_packet_ack_cb,
            on_disconnect_cb,
            on_packet_receive_cb,
            on_ping_measure_cb,
        }
    }

    /// Returns the session's tracing span.
    pub fn get_span(&self) -> &Span {
        &self.meta.span
    }

    /// Returns the current state of the session.
    #[inline]
    pub fn get_state(&self) -> SessionState {
        self.state
    }

    /// Returns true if the session is in a state considered *potentially* connected
    /// (Includes handshaking states where packets might be exchanged).
    #[inline]
    pub fn is_potentially_connected(&self) -> bool {
        matches!(
            self.state,
            SessionState::ConnectingOnline | SessionState::Connected
        )
    }

    /// Returns true if the session is fully connected (past handshake).
    #[inline]
    pub fn is_fully_connected(&self) -> bool {
        self.state == SessionState::Connected
    }

    /// Returns true if the session is disconnected and can be removed.
    #[inline]
    pub fn is_fully_disconnected(&self) -> bool {
        self.state == SessionState::Disconnected
    }

    /// Returns the unique internal ID for this session.
    #[inline]
    pub fn internal_id(&self) -> u64 {
        self.meta.internal_id
    }

    /// Returns the remote address of the peer.
    #[inline]
    pub fn address(&self) -> SocketAddr {
        self.meta.addr
    }

    /// Returns the client's RakNet GUID.
    #[inline]
    pub fn client_id(&self) -> u64 {
        self.meta.client_id
    }

    /// Returns the last measured ping in milliseconds.
    #[inline]
    pub fn last_ping_ms(&self) -> u32 {
        self.last_ping_measure_milli
    }

    /// Performs periodic updates: checks timeouts, updates reliability layers, sends pings.
    /// Returns Ok(Vec<EncapsulatedPacket>) containing packets ready for routing, or Err on fatal session error.
    pub fn update(&mut self, now: Instant) -> Result<Vec<EncapsulatedPacket>> {
        let _guard = self.meta.span.enter(); // Enter span for logging within this method
        self.last_update_time = now;

        if self.state == SessionState::Disconnected {
            return Ok(Vec::new()); // No packets to process if disconnected
        }

        // --- Timeout Checks ---
        // Use is_potentially_connected to avoid timeout during offline handshake
        if self.is_potentially_connected() && self.last_activity_time + Self::TIMEOUT_DURATION < now
        {
            debug!(
                "Session timed out due to inactivity ({}s)",
                Self::TIMEOUT_DURATION.as_secs()
            );
            self.force_disconnect(DisconnectReason::PeerTimeout);
            // Return Ok here, the state change signals termination to the owner
            return Ok(Vec::new());
        }

        // --- Graceful Disconnect Logic ---
        if matches!(
            self.state,
            SessionState::DisconnectingGraceful | SessionState::DisconnectingNotified
        ) {
            if let Some(disconnect_start) = self.disconnection_time {
                if disconnect_start + Self::DISCONNECT_TIMEOUT < now {
                    debug!(
                        "Graceful disconnect timed out after {}s, forcing close",
                        Self::DISCONNECT_TIMEOUT.as_secs()
                    );
                    self.force_disconnect(DisconnectReason::PeerTimeout); // Force disconnect if graceful takes too long
                    return Ok(Vec::new()); // State changed, return empty vec
                }
            } else {
                // Should not happen, but set timer if it does
                warn!("Disconnecting state without disconnect timer set!");
                self.disconnection_time = Some(now);
            }

            // Check if reliability layers are idle (all sent packets ACKed)
            if !self.send_layer.needs_update() && !self.recv_layer.needs_update() {
                if self.state == SessionState::DisconnectingGraceful {
                    debug!("All queues flushed, sending disconnect notification");
                    // Send DisconnectNotification packet reliably
                    // Ignore error if queuing fails (e.g., already disconnected somehow)
                    let _ = self.queue_internal_packet(
                        DisconnectionNotification {},
                        PacketReliability::RELIABLE_ORDERED,
                        0,
                        true,
                    );
                    self.state = SessionState::DisconnectingNotified;
                    self.disconnection_time = Some(now); // Reset timer for final ACKs/timeout
                } else if self.state == SessionState::DisconnectingNotified {
                    // We assume the DisconnectNotification was likely ACKed if queues are empty now
                    debug!("Disconnect notification likely ACKed, closing session cleanly");
                    self.state = SessionState::Disconnected;
                    // on_disconnect_cb was already called when initiate_disconnect was invoked
                    return Ok(Vec::new()); // Signal clean disconnect completion
                }
            }
        }

        // --- Update Reliability Layers ---
        // *** Revision Needed Here: ***
        // Assume recv_layer.update() is changed to return Vec<EncapsulatedPacket>
        let mut received_packets = Vec::new(); // Placeholder
        // let received_packets = self.recv_layer.update_and_get_packets()?; // Hypothetical combined method
        self.recv_layer.update(); // Sends ACKs/NACKs
        self.send_layer.update(); // Sends queued packets, handles timeouts/resends

        // --- Route received packets (after RecvLayer revision) ---
        for packet in received_packets.drain(..) {
            // Moved routing logic here
            if let Err(e) = self.handle_encapsulated_packet_route(packet) {
                // Errors during routing might indicate need to disconnect
                error!(error = %e, "Error routing encapsulated packet");
                // Decide if error is fatal
                // self.force_disconnect(DisconnectReason::Unknown(0)); // Example
                // return Err(e); // Propagate fatal error
            }
        }


        // --- Send Periodic Pings ---
        if self.is_fully_connected() && self.last_ping_time + Self::PING_INTERVAL <= now {
            self.send_ping(PacketReliability::UNRELIABLE); // Send unreliable ping
            self.last_ping_time = now;
        }

        // --- Cleanup old ping tracking data ---
        if self.ping_send_times.len() > Self::MAX_PING_TRACKING * 2 {
            // Keep some buffer
            // Remove oldest entries (though HashMap doesn't guarantee order easily)
            // A better approach might be a VecDeque or BTreeMap if strict oldest removal is needed.
            if let Some(oldest_time) = self.ping_send_times.values().min().cloned() {
                // Remove entries older than the second oldest maybe? Or just oldest?
                // For simplicity, just remove the absolute oldest found.
                self.ping_send_times
                    .retain(|_, &mut send_instant| send_instant >= oldest_time);
            }
        }

        Ok(Vec::new()) // Return empty vec until RecvLayer returns packets
    } // end update

    /// Handles a raw datagram received from the network for this session.
    /// Returns packets ready for routing.
    pub fn handle_datagram(&mut self, datagram: Datagram, now: Instant) -> Result<Vec<EncapsulatedPacket>> {
        let _guard = self.meta.span.enter();
        trace!(seq = datagram.seq_number, packet_count = datagram.packets.len(), "Handling Datagram");
        self.last_activity_time = now;

        // --- Revision Needed Here: ---
        // Assume recv_layer.on_datagram now returns Vec<EncapsulatedPacket>
        // Pass to ReceiveReliabilityLayer, errors bubble up
        // let received_packets = self.recv_layer.on_datagram(datagram)?;
        self.recv_layer.on_datagram(datagram)?; // Process sequence number, update queues

        // Process packets immediately instead of waiting for update() ?
        // RakLib PHP processes immediately within onDatagram. Let's mimic that.
        let mut packets_to_route = Vec::new();
        // We need access to the packets *after* on_datagram potentially buffers them
        // This reinforces the need for RecvLayer to return ready packets, or have a method like `drain_ready_packets`.
        // Let's add drain_ready_packets to the design.

        // packets_to_route = self.recv_layer.drain_ready_packets()?; // Hypothetical method

        // Route the packets immediately
        let mut final_packets = Vec::new();
        for packet in packets_to_route.drain(..) {
            // handle_encapsulated_packet_route returns only user packets
            if let Some(user_packet) = self.handle_encapsulated_packet_route(packet)? {
                final_packets.push(user_packet);
            }
        }

        Ok(final_packets) // Return only user packets
    }

    /// Handles an incoming ACK packet.
    pub fn handle_ack(&mut self, ack: ACK, now: Instant) {
        let _guard = self.meta.span.enter();
        self.last_activity_time = now;
        self.send_layer.on_ack(&ack);
    }

    /// Handles an incoming NACK packet.
    pub fn handle_nack(&mut self, nack: NACK, now: Instant) {
        let _guard = self.meta.span.enter();
        self.last_activity_time = now;
        self.send_layer.on_nack(&nack);
    }

    /// Initiates a graceful asynchronous disconnect.
    pub fn initiate_disconnect(&mut self, reason: DisconnectReason, now: Instant) {
        let _guard = self.meta.span.enter();
        // Allow initiating disconnect even if already disconnecting gracefully
        if !matches!(
            self.state,
            SessionState::Disconnected | SessionState::DisconnectingNotified
        ) {
            debug!(?reason, current_state=?self.state, "Initiating graceful disconnect");
            // Only call the callback once when disconnect is *initiated*
            if !matches!(self.state, SessionState::DisconnectingGraceful) {
                (self.on_disconnect_cb)(reason);
            }
            self.state = SessionState::DisconnectingGraceful;
            self.disconnection_time = Some(now);
        }
    }

    /// Forces immediate disconnection and state change.
    pub fn force_disconnect(&mut self, reason: DisconnectReason) {
        let _guard = self.meta.span.enter();
        if self.state != SessionState::Disconnected {
            debug!(?reason, current_state=?self.state, "Forcing disconnect");
            // Only call callback if not already disconnecting gracefully (where it was called before)
            if !matches!(
                self.state,
                SessionState::DisconnectingGraceful | SessionState::DisconnectingNotified
            ) {
                (self.on_disconnect_cb)(reason);
            }
            self.state = SessionState::Disconnected;
        }
    }

    /// Queues a user data packet (Vec<u8>) to be sent.
    pub fn queue_user_packet(
        &mut self,
        payload: Vec<u8>,
        reliability: u8,
        order_channel: u8,
        immediate: bool,
    ) -> Result<()> {
        let _guard = self.meta.span.enter();
        if !self.is_potentially_connected() { // Allow queuing during ConnectingOnline? Check RakNet. Usually only when Connected.
            return Err(RakLibError::SessionError(
                "Session is not connected".to_string(),
            ));
        }
        let mut packet = EncapsulatedPacket::new();
        packet.buffer = payload;
        packet.reliability = reliability;
        packet.order_channel = Some(order_channel);
        // Let SendLayer handle splitting, indexing etc.
        self.send_layer.add_encapsulated_to_queue(packet, immediate);
        Ok(())
    }

    /// Queues a user data packet that requires an ACK notification.
    pub fn queue_user_packet_needs_ack(
        &mut self,
        payload: Vec<u8>,
        reliability: u8, // Must be reliable if ACK is needed
        order_channel: u8,
        immediate: bool,
        ack_identifier: u32,
    ) -> Result<()> {
        let _guard = self.meta.span.enter();
        if !self.is_potentially_connected() {
            return Err(RakLibError::SessionError(
                "Session is not connected".to_string(),
            ));
        }
        if !PacketReliability::is_reliable(reliability) {
            return Err(RakLibError::SessionError(
                "ACK requested for unreliable packet".to_string(),
            ));
        }

        let mut packet = EncapsulatedPacket::new();
        packet.buffer = payload;
        packet.reliability = reliability;
        packet.order_channel = Some(order_channel);
        packet.identifier_ack = Some(ack_identifier); // Set the ACK ID

        self.send_layer.add_encapsulated_to_queue(packet, immediate);
        Ok(())
    }

    /// Queues an *internal* RakNet packet (like Ping, Pong, Disconnect).
    /// Use this for packets defined in `raklib::protocol::packets`.
    fn queue_internal_packet<P: ConnectedPacket + Send + 'static>(
        &mut self,
        packet: P,
        reliability: u8,
        order_channel: u8,
        immediate: bool,
    ) -> Result<()> {
        let _guard = self.meta.span.enter();
        if self.state == SessionState::Disconnected {
            // Don't send internal packets if fully disconnected, except maybe ACK/NACK handled by layer
            return Ok(());
        }
        let mut stream = BinaryStream::new();
        // Use encode(), which includes the header ID byte
        packet.encode(&mut stream)?;

        let mut encapsulated = EncapsulatedPacket::new();
        encapsulated.buffer = stream.into_inner();
        encapsulated.reliability = reliability;
        encapsulated.order_channel = Some(order_channel);

        self.send_layer.add_encapsulated_to_queue(encapsulated, immediate);
        Ok(())
    }

    /// Routes a fully reassembled/ordered EncapsulatedPacket based on its ID byte.
    /// Returns Some(Vec<u8>) containing user packet payload if applicable, None otherwise.
    /// Errors should be propagated for handling (potential disconnect).
    fn handle_encapsulated_packet_route(&mut self, packet: EncapsulatedPacket) -> Result<Option<Vec<u8>>> {
        let _guard = self.meta.span.enter();
        if packet.buffer.is_empty() {
            warn!("Received empty encapsulated packet payload, discarding");
            return Ok(None);
        }
        let packet_id = packet.buffer[0];

        if packet_id >= MessageIdentifiers::ID_USER_PACKET_ENUM {
            // --- User Data Packet ---
            if self.state == SessionState::Connected {
                trace!(packet_id, len = packet.buffer.len(), "Routing user data packet");
                // Instead of calling callback here, return the payload
                return Ok(Some(packet.buffer));
                // (self.on_packet_receive_cb)(packet.buffer); // Old approach
            } else {
                debug!(packet_id, state=?self.state, "Discarding user packet received while not fully connected");
            }
        } else {
            // --- Internal RakNet Packet ---
            trace!(packet_id, "Routing internal RakNet packet");
            match packet_id {
                MessageIdentifiers::ID_CONNECTED_PING => {
                    // Use a temporary stream for decoding internal packets
                    let mut stream = BinaryStream::from_slice(&packet.buffer);
                    if let Ok(pk) = ConnectedPing::decode(&mut stream) {
                        self.handle_connected_ping(pk)?;
                    } else { warn!("Failed to decode ConnectedPing"); }
                }
                MessageIdentifiers::ID_CONNECTED_PONG => {
                    let mut stream = BinaryStream::from_slice(&packet.buffer);
                    if let Ok(pk) = ConnectedPong::decode(&mut stream) {
                        self.handle_connected_pong(pk);
                    } else { warn!("Failed to decode ConnectedPong"); }
                }
                MessageIdentifiers::ID_DISCONNECTION_NOTIFICATION => {
                    debug!("Received DisconnectionNotification from peer");
                    // Client considers connection lost immediately.
                    self.force_disconnect(DisconnectReason::ClientDisconnect);
                }
                MessageIdentifiers::ID_CONNECTION_REQUEST => {
                    // Server should handle this in its ServerSession specialization
                    self.handle_connection_request(packet.buffer)?;
                }
                MessageIdentifiers::ID_CONNECTION_REQUEST_ACCEPTED => {
                    // Client should handle this in its ClientSession specialization
                    self.handle_connection_request_accepted(packet.buffer)?;
                }
                MessageIdentifiers::ID_NEW_INCOMING_CONNECTION => {
                    // Server should handle this in its ServerSession specialization
                    self.handle_new_incoming_connection(packet.buffer)?;
                }
                // Add cases for other internal packets if needed (e.g., ADVERTISE_SYSTEM)
                _ => {
                    debug!(packet_id, "Received unhandled internal RakNet packet");
                }
            }
        }
        Ok(None) // No user packet payload returned
    } // end handle_encapsulated_packet_route

    // --- Internal Packet Handlers ---

    fn send_ping(&mut self, reliability: u8) {
        let _guard = self.meta.span.enter();
        let send_time_raknet = self.get_raknet_time_ms();
        let send_time_instant = Instant::now();

        if self.ping_send_times.len() >= Self::MAX_PING_TRACKING {
            // Evict oldest entry if map gets too large
            if let Some((&oldest_raknet_time, _)) = self.ping_send_times.iter().min_by_key(|(_, &instant)| instant) {
                self.ping_send_times.remove(&oldest_raknet_time);
            }
        }
        self.ping_send_times.insert(send_time_raknet, send_time_instant);

        trace!(ping_time = send_time_raknet, "Sending ConnectedPing");
        if let Err(e) =
            self.queue_internal_packet(ConnectedPing::create(send_time_raknet), reliability, 0, true)
        {
            error!(error=%e, "Failed to queue ConnectedPing");
        }
    }

    fn handle_connected_ping(&mut self, ping_packet: ConnectedPing) -> Result<()> {
        let _guard = self.meta.span.enter();
        trace!(ping_time = ping_packet.send_ping_time, "Handling ConnectedPing, sending Pong");
        let pong = ConnectedPong::create(
            ping_packet.send_ping_time,
            self.get_raknet_time_ms(), // Current time when sending pong
        );
        // Pong should be sent unreliably according to RakNet source/tests
        self.queue_internal_packet(pong, PacketReliability::UNRELIABLE, 0, false)
    }

    fn handle_connected_pong(&mut self, pong_packet: ConnectedPong) {
        let _guard = self.meta.span.enter();
        let now_instant = Instant::now();
        let send_ping_raknet_time = pong_packet.send_ping_time;

        if let Some(ping_sent_instant) = self.ping_send_times.remove(&send_ping_raknet_time) {
            // Calculate RTT
            let rtt = now_instant.saturating_duration_since(ping_sent_instant);
            let ping_ms = rtt.as_millis() as u32; // Convert RTT to milliseconds
            self.last_ping_measure_milli = ping_ms;
            trace!(ping_time=send_ping_raknet_time, pong_time=pong_packet.send_pong_time, rtt_ms = ping_ms, "Handling ConnectedPong");
            // Notify the listener
            (self.on_ping_measure_cb)(ping_ms);
        } else {
            // Pong received for an unknown/expired ping, or duplicate pong
            trace!(ping_time=send_ping_raknet_time, "Received ConnectedPong for untracked ping time");
        }
    }

    /// Gets the current time in milliseconds suitable for RakNet timestamps.
    /// Precision might vary. Consider hrtime equivalent if available and needed.
    fn get_raknet_time_ms(&self) -> u64 {
        // Use Instant relative to an arbitrary start point if possible for monotonicity?
        // Or just use system time millis? Using system time is easier but not monotonic.
        // Let's use Instant relative to the first update time or similar for better monotonicity.
        // For simplicity now, let's just use duration since UNIX_EPOCH (like PHP's microtime).
        // WARNING: This is not monotonic and can jump!
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
        // TODO: Implement a monotonic RakNet time source if needed.
    }

    // --- Methods to be overridden or implemented via callbacks by Server/Client Session ---

    /// Handles ID_CONNECTION_REQUEST (Server responsibility). Default does nothing.
    pub fn handle_connection_request(&mut self, _buffer: Vec<u8>) -> Result<()> {
        warn!("handle_connection_request called on generic Session");
        Ok(())
    }
    /// Handles ID_CONNECTION_REQUEST_ACCEPTED (Client responsibility). Default does nothing.
    pub fn handle_connection_request_accepted(&mut self, _buffer: Vec<u8>) -> Result<()> {
        warn!("handle_connection_request_accepted called on generic Session");
        Ok(())
    }
    /// Handles ID_NEW_INCOMING_CONNECTION (Server responsibility). Default does nothing.
    pub fn handle_new_incoming_connection(&mut self, _buffer: Vec<u8>) -> Result<()> {
        warn!("handle_new_incoming_connection called on generic Session");
        Ok(())
    }
} // end impl Session

// Required for putting Session in HashMaps etc. if needed (e.g. by Server)
impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.meta.internal_id == other.meta.internal_id && self.meta.addr == other.meta.addr
    }
}
impl Eq for Session {}
impl std::hash::Hash for Session {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.meta.internal_id.hash(state);
        self.meta.addr.hash(state);
    }
}