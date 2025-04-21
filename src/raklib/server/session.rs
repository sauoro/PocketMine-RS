use super::listener::ServerEventListener;
use crate::raklib::generic::session::{Session, SessionMeta};
use crate::raklib::protocol::*; // Import protocol items
use crate::raklib::server::Server; // Need reference or interface to Server
use crate::raklib::utils::InternetAddress;
use crate::utils::binary::BinaryStream;
use std::net::SocketAddr;
use std::sync::Arc; // For sharing listener
use tracing::Span;

pub struct ServerSession {
    // Embed the generic Session for state and reliability layers
    inner: Session,
    // Add server-specific fields
    listener: Arc<dyn ServerEventListener>, // Share listener across sessions
    // Need a way to interact back with the server (e.g., send packets)
    // Option 1: Weak reference to Server (needs careful lifetime mgmt)
    // Option 2: Channel (mpsc) to send commands/packets to the server task
    // Option 3: Pass necessary server interfaces/callbacks during creation
    // Let's assume Option 3 for now (callbacks passed during Session::new)
}

impl ServerSession {
    pub fn new(
        internal_id: u64,
        address: SocketAddr,
        client_id: u64,
        mtu_size: u16,
        listener: Arc<dyn ServerEventListener>,
        // TODO: Pass callbacks for sending packets, ACKs, handling disconnects etc.
        //       These will be passed down to the generic Session constructor.
    ) -> Self {
        let span = tracing::span!(tracing::Level::INFO, "Session", session_id = internal_id, addr = %address);
        let meta = SessionMeta { addr: address, client_id, mtu_size, span };

        // TODO: Create the actual callbacks that interact with the Server's socket/state
        let on_recv_callback = Box::new(move |packet: EncapsulatedPacket| {
            // Handle internal packets or pass user packets to listener
            // ... implementation needed ...
        });
        let send_ack_nack_callback = Box::new(|_ack_nack| {
            // Send ACK/NACK via the Server's socket
            // ... implementation needed ...
        });
        let send_datagram_callback = Box::new(|_datagram: Datagram| {
            // Send Datagram via the Server's socket
            // ... implementation needed ...
        });
        let on_ack_callback = Box::new(move |ack_id: u32| {
            // Notify listener about packet ack
            listener.on_packet_ack(internal_id, ack_id);
        });


        let inner_session = Session::new(meta /*, callbacks */); // Pass constructed callbacks here

        Self {
            inner: inner_session,
            listener,
        }
    }

    // --- Delegate methods to inner Session or implement server specifics ---

    pub fn get_span(&self) -> &Span {
        self.inner.get_span()
    }

    pub fn update(&mut self, now: std::time::Instant) -> crate::raklib::error::Result<()> {
        self.inner.update(now)
        // TODO: Add server-specific update logic if needed
    }

    // ... handle_datagram, handle_ack, handle_nack, initiate_disconnect, etc. ...

    // Example: Handle connection packets specific to server session
    fn handle_raknet_connection_packet(&mut self, packet_data: &[u8]) {
        let packet_id = packet_data.get(0).copied();
        match packet_id {
            Some(MessageIdentifiers::ID_CONNECTION_REQUEST) => {
                let mut stream = BinaryStream::from_slice(packet_data);
                match ConnectionRequest::decode(&mut stream) {
                    Ok(pk) => {
                        // Send ConnectionRequestAccepted
                        // TODO: Construct and send the reply packet
                        let reply = ConnectionRequestAccepted::create(
                            InternetAddress::from_socket_addr(self.inner.meta.addr), // Client address
                            vec![], // System addresses (usually empty for reply)
                            pk.send_ping_time,
                            0, // TODO: Get current RakNet time
                        );
                        // self.inner.queue_connected_packet(reply, PacketReliability::RELIABLE, 0, true);
                    }
                    Err(e) => tracing::error!(parent: self.get_span(), "Failed to decode ConnectionRequest: {}", e),
                }
            }
            Some(MessageIdentifiers::ID_NEW_INCOMING_CONNECTION) => {
                // Client confirms connection
                let mut stream = BinaryStream::from_slice(packet_data);
                match NewIncomingConnection::decode(&mut stream) {
                    Ok(_pk) => {
                        if self.inner.get_state() == crate::raklib::generic::session::SessionState::Connecting {
                            // Mark as connected and notify listener
                            self.inner.state = crate::raklib::generic::session::SessionState::Connected;
                            self.listener.on_client_connect(
                                0, // TODO: Pass actual internal_id
                                self.inner.meta.addr,
                                self.inner.meta.client_id
                            );
                            // TODO: Handle ping/pong times if needed (_pk.send_ping_time etc.)
                            // TODO: Initiate first reliable ping maybe?
                        }
                    }
                    Err(e) => tracing::error!(parent: self.get_span(), "Failed to decode NewIncomingConnection: {}", e),
                }
            }
            Some(MessageIdentifiers::ID_DISCONNECTION_NOTIFICATION) => {
                self.inner.force_disconnect(crate::raklib::error::DisconnectReason::ClientDisconnect); // Client initiated disconnect
            }

            // Handle other connection-related packets if necessary
            _ => {}
        }
    }

    // Handle user packets
    fn on_packet_receive(&self, packet_data: Vec<u8>) {
        self.listener.on_packet_receive(0 /* TODO: internal_id */, packet_data);
    }

    // Handle ping measurement
    fn on_ping_measure(&self, ping_ms: u32) {
        self.listener.on_ping_measure(0 /* TODO: internal_id */, ping_ms);
    }

}
