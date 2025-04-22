// src/raknet/server/server.rs
#![allow(dead_code)]

use crate::log::{BufferedLogger, Logger};
use crate::raknet::generic::disconnect_reason::DisconnectReason;
use crate::raknet::generic::error::{PacketHandlingError, SocketError};
use crate::raknet::generic::session::{Session, SessionState};
use crate::raknet::protocol::encapsulated_packet::EncapsulatedPacket;
use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::raknet::server::protocol_acceptor::ProtocolAcceptor;
use crate::raknet::server::server_event_listener::ServerEventListener;
use crate::raknet::server::server_event_source::ServerEventSource;
use crate::raknet::server::server_interface::ServerInterface;
use crate::raknet::server::server_session::ServerSession;
use crate::raknet::server::server_socket::ServerSocket;
use crate::raknet::server::unconnected_message_handler::UnconnectedMessageHandler;
use crate::raknet::utils::internet_address::InternetAddress;
use crate::raknet::raknet; // For SYSTEM_ADDRESS_COUNT
use bytes::Bytes;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex}; // Use tokio Mutex for async locking
use tokio::time;

const RAKLIB_TPS: u64 = 100;
const RAKLIB_TIME_PER_TICK: Duration = Duration::from_millis(1000 / RAKLIB_TPS);

pub struct Server {
    server_id: u64,
    logger: Box<dyn Logger>,
    socket: Arc<ServerSocket>,
    max_mtu_size: u16,
    protocol_acceptor: Arc<dyn ProtocolAcceptor>,
    // event_source: Arc<dyn ServerEventSource>, // Replaced by internal loop/channels
    event_listener: Arc<dyn ServerEventListener>,
    // trace_cleaner: Arc<ExceptionTraceCleaner>, // Omitted for now

    recv_max_split_parts: u32,
    recv_max_concurrent_splits: usize,

    sessions: Arc<Mutex<HashMap<u64, Arc<Mutex<ServerSession>>>>>, // session_internal_id -> Session
    sessions_by_address: Arc<Mutex<HashMap<InternetAddress, u64>>>, // address -> session_internal_id

    unconnected_handler: Arc<Mutex<UnconnectedMessageHandler>>,

    name: Arc<Mutex<String>>,
    packet_limit_per_tick_per_ip: usize,
    port_checking: Arc<AtomicBool>,

    shutdown: Arc<AtomicBool>,
    block_list: Arc<Mutex<HashMap<String, Instant>>>, // ip_string -> unblock_time
    ip_sec_counter: Arc<Mutex<HashMap<String, usize>>>, // ip_string -> count_this_tick

    next_session_id: Arc<AtomicU64>,

    raw_packet_filters: Arc<Mutex<Vec<String>>>, // Regex strings

    // Statistics
    receive_bytes: Arc<AtomicU64>,
    send_bytes: Arc<AtomicU64>,

    // Internal channels/handles
    shutdown_signal: mpsc::Sender<()>, // To signal the run loop to stop
    task_handle: Option<tokio::task::JoinHandle<()>>, // Handle to the main run task
}

impl Server {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        server_id: u64,
        logger: Box<dyn Logger>,
        bind_address: InternetAddress,
        max_mtu_size: u16,
        protocol_acceptor: Arc<dyn ProtocolAcceptor>,
        // event_source: Arc<dyn ServerEventSource>,
        event_listener: Arc<dyn ServerEventListener>,
        // trace_cleaner: Arc<ExceptionTraceCleaner>,
        recv_max_split_parts: Option<u32>,
        recv_max_concurrent_splits: Option<usize>,
    ) -> Result<Self, SocketError> {
        if max_mtu_size < Session::MIN_MTU_SIZE {
            return Err(SocketError::OperationFailed(format!(
                "MTU size must be at least {}, got {}",
                Session::MIN_MTU_SIZE, max_mtu_size
            )));
        }

        let socket = Arc::new(ServerSocket::bind(bind_address).await?);

        let (shutdown_tx, _shutdown_rx) = mpsc::channel(1); // Buffer of 1 is enough

        let unconnected_handler = Arc::new(Mutex::new(UnconnectedMessageHandler::new(
            protocol_acceptor.clone(),
            server_id,
            logger.clone_boxed(),
            socket.clone(),
            // Need to provide access to server state for handler actions
        )));


        let server = Self {
            server_id,
            logger,
            socket: socket.clone(),
            max_mtu_size,
            protocol_acceptor,
            // event_source,
            event_listener,
            // trace_cleaner,
            recv_max_split_parts: recv_max_split_parts.unwrap_or(raknet::DEFAULT_PROTOCOL_VERSION as u32), // Fix default
            recv_max_concurrent_splits: recv_max_concurrent_splits.unwrap_or(4), // Fix default
            sessions: Arc::new(Mutex::new(HashMap::new())),
            sessions_by_address: Arc::new(Mutex::new(HashMap::new())),
            unconnected_handler, // Placeholder, needs proper init
            name: Arc::new(Mutex::new(String::new())),
            packet_limit_per_tick_per_ip: 200, // Default from PHP
            port_checking: Arc::new(AtomicBool::new(false)),
            shutdown: Arc::new(AtomicBool::new(false)),
            block_list: Arc::new(Mutex::new(HashMap::new())),
            ip_sec_counter: Arc::new(Mutex::new(HashMap::new())),
            next_session_id: Arc::new(AtomicU64::new(0)),
            raw_packet_filters: Arc::new(Mutex::new(Vec::new())),
            receive_bytes: Arc::new(AtomicU64::new(0)),
            send_bytes: Arc::new(AtomicU64::new(0)),
            shutdown_signal: shutdown_tx,
            task_handle: None,
        };

        // Initialize unconnected_handler with necessary closures/access
        {
            let mut handler_guard = server.unconnected_handler.lock().await;
            let sessions_clone = server.sessions.clone();
            let sessions_by_addr_clone = server.sessions_by_address.clone();
            let next_session_id_clone = server.next_session_id.clone();
            let logger_clone = server.logger.clone_boxed();
            let event_listener_clone = server.event_listener.clone();
            let port_checking_clone = server.port_checking.clone();
            let max_mtu_clone = server.max_mtu_size;
            let recv_parts_clone = server.recv_max_split_parts;
            let recv_splits_clone = server.recv_max_concurrent_splits;
            let server_id_clone = server.server_id;
            let server_name_clone = server.name.clone();
            let raw_sender_clone = server.get_raw_sender(); // Closure for sending raw packets

            handler_guard.set_server_access(
                Box::new({
                    let sessions = sessions_by_addr_clone.clone();
                    move |addr: &InternetAddress| {
                        let guard = sessions.blocking_lock(); // Use blocking lock if needed from sync context
                        guard.contains_key(addr)
                    }
                }),
                Box::new({
                    let sessions = sessions_clone.clone();
                    let sessions_by_addr = sessions_by_addr_clone.clone();
                    let next_session_id = next_session_id_clone.clone();
                    let logger = logger_clone.clone_boxed();
                    let event_listener = event_listener_clone.clone();
                    let raw_sender_cb = raw_sender_clone.clone(); // Clone the sender closure

                    move |address: InternetAddress, client_id: i64, mtu: u16| {
                        // This needs to be async or run in a blocking thread
                        // For simplicity, use blocking_lock, but beware of deadlocks
                        let mut sessions_guard = sessions.blocking_lock();
                        let mut sessions_by_addr_guard = sessions_by_addr.blocking_lock();

                        if let Some(existing_id) = sessions_by_addr_guard.get(&address) {
                            if let Some(existing_session_lock) = sessions_guard.get(existing_id) {
                                let mut existing_session = existing_session_lock.blocking_lock();
                                existing_session.forcibly_disconnect(DisconnectReason::ClientReconnect);
                                // Removal happens later or in session update
                            }
                            sessions_by_addr_guard.remove(&address); // Remove old mapping
                        }

                        // --- Simplified Session Creation (Full version in ServerSession) ---
                        let session_internal_id = next_session_id.fetch_add(1, Ordering::Relaxed);

                        // Create ServerSession - This is complex and needs its own logic
                        // We'll create a placeholder here
                        let session_logger = logger.clone_boxed(); // Example: clone base logger
                        let session_event_listener = event_listener.clone();

                        let session = Arc::new(Mutex::new(ServerSession::new(
                            server_id_clone,
                            session_internal_id,
                            logger.clone_boxed(), // Pass logger
                            address.clone(),
                            client_id,
                            mtu,
                            raw_sender_cb.clone(), // Pass sender closure
                            session_event_listener, // Pass listener Arc
                            None, // recv_max_split_parts
                            None // recv_max_concurrent_splits
                        ).expect("Failed to create session"))); // Handle error better

                        sessions_by_addr_guard.insert(address.clone(), session_internal_id);
                        sessions_guard.insert(session_internal_id, session.clone());

                        logger.debug(&format!("Created session for {} with MTU size {}", address, mtu));

                        session // Return the created session Arc<Mutex<ServerSession>>
                    }
                }),
                Box::new({
                    let port_check = port_checking_clone.clone();
                    move || port_check.load(Ordering::Relaxed)
                }),
                Box::new(move || max_mtu_clone),
                Box::new({
                    let name_arc = server_name_clone.clone();
                    move || {
                        name_arc.blocking_lock().clone() // Blocking lock for simplicity
                    }
                }),
                raw_sender_clone, // Pass sender for direct replies like Pong
            );
        }


        Ok(server)
    }

    // Method to start the server's run loop
    pub fn start(mut self) {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_signal = shutdown_tx; // Store the sender

        let server_arc = Arc::new(self); // Create Arc for sharing state in the task

        let handle = tokio::spawn({
            let server = server_arc.clone();
            async move {
                let mut interval = time::interval(RAKLIB_TIME_PER_TICK);
                let mut tick_count: u64 = 0;
                server.logger.info("RakNet server thread started");

                loop {
                    tokio::select! {
                         _ = interval.tick() => {
                             if server.shutdown.load(Ordering::Relaxed) {
                                 server.logger.info("Shutdown signal received, stopping server tick loop.");
                                 break;
                             }
                             let start = Instant::now();
                             server.tick().await;
                             tick_count += 1;

                             // Periodic tasks (like bandwidth stats)
                             if tick_count % RAKLIB_TPS == 0 {
                                 server.run_periodic_tasks(tick_count).await;
                             }

                             // Sleep logic is handled by interval.tick()
                         }
                         // Handle incoming packets
                         result = server.socket.recv_from() => {
                             match result {
                                 Ok((buffer, src_addr)) => {
                                     server.receive_bytes.fetch_add(buffer.len() as u64, Ordering::Relaxed);
                                     server.handle_raw_packet(buffer, src_addr).await;
                                 }
                                 Err(e) => {
                                     if e.kind() != std::io::ErrorKind::WouldBlock {
                                         server.logger.error(&format!("Socket recv error: {}", e));
                                         // Decide if error is fatal
                                     }
                                 }
                             }
                         }
                         // Handle shutdown signal
                         _ = shutdown_rx.recv() => {
                            server.logger.info("External shutdown signal received, stopping server run loop.");
                            server.shutdown.store(true, Ordering::Relaxed);
                             // Perform graceful shutdown steps here before breaking
                             server.perform_graceful_shutdown().await;
                             break;
                         }
                     }
                }
                server.logger.info("RakNet server thread finished.");
            }
        });
        // Store the handle back into the server struct (requires Self to be mutable)
        // This approach is tricky. Usually, you'd return the handle or manage it externally.
        // Let's assume the caller manages the handle or doesn't need it stored directly.
        // self.task_handle = Some(handle); // Cannot do this as self is moved into Arc

        // To store the handle, the Server struct needs to be created, then wrapped in Arc,
        // then the task spawned, and *then* the handle stored. This requires careful structuring.
        // For now, we won't store the handle in the struct itself after spawning.
    }


    async fn tick(&self) {
        let current_time = Instant::now();
        let mut sessions = self.sessions.lock().await;
        let mut sessions_by_address = self.sessions_by_address.lock().await;
        let mut sessions_to_remove = Vec::new();

        // Update sessions
        for (id, session_lock) in sessions.iter() {
            let mut session = session_lock.lock().await;
            session.update(current_time);
            if session.is_fully_disconnected() {
                sessions_to_remove.push((*id, session.get_address().clone()));
            }
        }

        // Remove disconnected sessions
        for (id, addr) in sessions_to_remove {
            sessions.remove(&id);
            sessions_by_address.remove(&addr);
            self.logger.debug(&format!("Removed disconnected session {}", addr));
        }

        // Clear IP security counter for the new tick
        self.ip_sec_counter.lock().await.clear();
    }

    async fn run_periodic_tasks(&self, tick_count: u64) {
        // Bandwidth Stats
        let sent = self.send_bytes.swap(0, Ordering::Relaxed);
        let received = self.receive_bytes.swap(0, Ordering::Relaxed);
        if sent > 0 || received > 0 {
            self.event_listener.on_bandwidth_stats_update(sent, received).await;
        }

        // Unblock IPs
        let mut block_list = self.block_list.lock().await;
        let now = Instant::now();
        block_list.retain(|_ip, unblock_time| *unblock_time > now);
    }

    async fn handle_raw_packet(&self, buffer: Bytes, src_addr: SocketAddr) {
        let internet_addr = InternetAddress::from_socket_addr(src_addr);
        let ip_str = internet_addr.to_string_addr_only();

        // --- IP Blocking / Rate Limiting ---
        { // Scoped lock for block_list
            let block_list_guard = self.block_list.lock().await;
            if let Some(unblock_time) = block_list_guard.get(&ip_str) {
                if Instant::now() < *unblock_time {
                    return; // Blocked
                }
                // Expired block, will be removed in periodic task
            }
        } // block_list_guard dropped


        { // Scoped lock for ip_sec_counter
            let mut ip_sec_guard = self.ip_sec_counter.lock().await;
            let count = ip_sec_guard.entry(ip_str.clone()).or_insert(0);
            *count += 1;
            if *count >= self.packet_limit_per_tick_per_ip {
                self.block_address(&ip_str, 300).await; // Default 5 min block
                return;
            }
        } // ip_sec_guard dropped


        if buffer.is_empty() {
            return;
        }

        // --- Session Handling ---
        let session_id_opt = self.sessions_by_address.lock().await.get(&internet_addr).cloned();

        if let Some(session_id) = session_id_opt {
            let session_lock_opt = self.sessions.lock().await.get(&session_id).cloned();
            if let Some(session_lock) = session_lock_opt {
                let mut session = session_lock.lock().await;
                // Don't handle packets for sessions pending graceful disconnect unless required (e.g., ACKs)
                // if session.get_state() == SessionState::DisconnectPending || session.get_state() == SessionState::DisconnectNotified {
                //    // Maybe allow ACK/NACK?
                // }
                match session.handle_packet(buffer) {
                    Ok(_) => {}
                    Err(e) => {
                        self.logger.error(&format!("Error handling packet from {}: {}", internet_addr, e));
                        // Disconnect on packet handling error
                        session.forcibly_disconnect(e.reason);
                    }
                }
                return; // Handled by session
            } else {
                // Session ID exists in address map but not in session map (race condition?)
                self.sessions_by_address.lock().await.remove(&internet_addr);
                self.logger.warning(&format!("Session inconsistency for {}", internet_addr));
            }
        }

        // --- Unconnected Message Handling ---
        if !self.shutdown.load(Ordering::Relaxed) {
            let mut unconnected_handler = self.unconnected_handler.lock().await;

            // Provide necessary context to the handler
            let handled = match unconnected_handler.handle_raw(&buffer, internet_addr.clone()).await {
                Ok(handled) => handled,
                Err(e) => {
                    self.logger.error(&format!("Error handling unconnected packet from {}: {}", internet_addr, e));
                    self.block_address(&ip_str, 5).await; // Short block for bad packets
                    true // Consider it handled (by erroring out)
                }
            };


            if !handled {
                // Check raw packet filters if not handled by unconnected logic
                let filters = self.raw_packet_filters.lock().await;
                let buffer_str_lossy = String::from_utf8_lossy(&buffer); // Potential allocation
                let mut filtered = false;
                for pattern_str in filters.iter() {
                    // Use regex crate if complex patterns are needed
                    // Simple contains check for now:
                    if buffer_str_lossy.contains(pattern_str) { // Basic filter
                        self.event_listener.on_raw_packet_receive(ip_str, internet_addr.port(), buffer).await;
                        filtered = true;
                        break;
                    }
                }

                if !filtered {
                    self.logger.debug(&format!("Ignored packet from {} (no session, 0x{:02X})", internet_addr, buffer.get(0).unwrap_or(0xff)));
                }
            }
        }
    }

    // Method to initiate shutdown
    pub async fn shutdown(&self) {
        self.logger.info("Initiating RakNet server shutdown...");
        self.shutdown.store(true, Ordering::Relaxed);
        // Send shutdown signal to the run loop
        let _ = self.shutdown_signal.send(()).await; // Ignore error if receiver dropped
    }

    // Graceful shutdown logic called from within the run loop
    async fn perform_graceful_shutdown(&self) {
        self.logger.info("Performing graceful shutdown...");
        let sessions = self.sessions.lock().await;
        for session_lock in sessions.values() {
            let mut session = session_lock.lock().await;
            session.initiate_disconnect(DisconnectReason::ServerShutdown);
        }
        drop(sessions); // Release lock

        // Wait for sessions to disconnect (with timeout)
        let shutdown_timeout = Duration::from_secs(5);
        let start = Instant::now();
        loop {
            self.tick().await; // Process disconnects
            let sessions = self.sessions.lock().await;
            if sessions.is_empty() || start.elapsed() > shutdown_timeout {
                if sessions.is_empty() {
                    self.logger.info("All sessions disconnected gracefully.");
                } else {
                    self.logger.warning("Shutdown timeout reached, forcibly closing remaining sessions.");
                    // Forcibly close remaining ones if needed (tick should handle removal)
                }
                break;
            }
            drop(sessions);
            tokio::time::sleep(RAKLIB_TIME_PER_TICK / 2).await; // Small delay
        }

        self.socket.close();
        self.logger.info("Graceful shutdown complete.");
    }

    // Wait for the server task to complete (blocking)
    pub async fn wait_shutdown(self) {
        if let Some(handle) = self.task_handle {
            let _ = handle.await; // Wait for the task to finish
        } else {
            // If start() wasn't called or handle wasn't stored
            self.perform_graceful_shutdown().await;
        }
    }


    // --- ServerInterface Implementation (Helper Methods) ---

    pub async fn get_session(&self, session_id: u64) -> Option<Arc<Mutex<ServerSession>>> {
        self.sessions.lock().await.get(&session_id).cloned()
    }

    pub async fn get_session_by_address(&self, address: &InternetAddress) -> Option<Arc<Mutex<ServerSession>>> {
        let sessions_by_addr = self.sessions_by_address.lock().await;
        if let Some(id) = sessions_by_addr.get(address) {
            drop(sessions_by_addr); // Release address lock before acquiring session lock
            self.sessions.lock().await.get(id).cloned()
        } else {
            None
        }
    }

    pub async fn block_address(&self, address: &str, timeout_secs: u64) {
        let unblock_time = Instant::now() + Duration::from_secs(timeout_secs);
        let mut block_list = self.block_list.lock().await;
        block_list.insert(address.to_string(), unblock_time);
        if timeout_secs > 0 {
            self.logger.notice(&format!("Blocked {} for {} seconds", address, timeout_secs));
        } else {
            self.logger.notice(&format!("Blocked {} permanently", address)); // Or handle 0 as unblock? PHP used INT_MAX
        }

        // Optionally disconnect existing sessions from this IP
        let mut sessions_to_disconnect = Vec::new();
        let sessions_by_addr = self.sessions_by_address.lock().await;
        let sessions = self.sessions.lock().await;
        for (addr, session_id) in sessions_by_addr.iter() {
            if addr.to_string_addr_only() == address {
                if sessions.contains_key(session_id) {
                    sessions_to_disconnect.push(*session_id);
                }
            }
        }
        drop(sessions_by_addr);
        drop(sessions);

        for id in sessions_to_disconnect {
            if let Some(session_lock) = self.sessions.lock().await.get(&id) {
                session_lock.lock().await.forcibly_disconnect(DisconnectReason::ServerDisconnect); // Or specific reason
            }
        }
    }

    pub async fn unblock_address(&self, address: &str) {
        if self.block_list.lock().await.remove(address).is_some() {
            self.logger.debug(&format!("Unblocked {}", address));
        }
    }

    pub fn get_port(&self) -> u16 {
        self.socket.get_bind_address().port()
    }

    pub fn get_max_mtu_size(&self) -> u16 {
        self.max_mtu_size
    }

    pub fn get_id(&self) -> u64 {
        self.server_id
    }

    // Helper to create the raw sender closure
    fn get_raw_sender(&self) -> Arc<dyn Fn(InternetAddress, bytes::Bytes) + Send + Sync> {
        let socket_clone = self.socket.clone();
        let send_bytes_clone = self.send_bytes.clone();
        Arc::new(move |addr, payload| {
            let socket = socket_clone.clone();
            let send_bytes = send_bytes_clone.clone();
            tokio::spawn(async move {
                match socket.send_to(&payload, addr.to_socket_addr()).await {
                    Ok(len) => {
                        send_bytes.fetch_add(len as u64, Ordering::Relaxed);
                    }
                    Err(e) => {
                        // Log error, logger needs to be accessible here too
                        eprintln!("Raw packet send error to {}: {}", addr, e);
                    }
                }
            });
        })
    }
}


#[async_trait]
impl ServerInterface for Server {
    async fn send_encapsulated(
        &self,
        session_id: u64,
        packet: EncapsulatedPacket,
        immediate: bool,
    ) {
        if let Some(session_lock) = self.sessions.lock().await.get(&session_id) {
            session_lock.lock().await.add_encapsulated_to_queue(packet, immediate);
        }
    }

    async fn send_raw(&self, address: String, port: u16, payload: Bytes) {
        match InternetAddress::from_string(&address, port) {
            Ok(addr) => {
                let sender = self.get_raw_sender();
                (sender)(addr, payload);
            }
            Err(e) => self.logger.error(&format!("Invalid address format for send_raw: {}:{} - {}", address, port, e)),
        }
    }

    async fn close_session(&self, session_id: u64, reason: DisconnectReason) {
        if let Some(session_lock) = self.sessions.lock().await.get(&session_id) {
            session_lock.lock().await.initiate_disconnect(reason);
        }
    }

    async fn set_name(&self, name: String) {
        *self.name.lock().await = name;
    }

    async fn set_port_check(&self, value: bool) {
        self.port_checking.store(value, Ordering::Relaxed);
    }

    async fn set_packets_per_tick_limit(&mut self, limit: usize) {
        self.packet_limit_per_tick_per_ip = limit;
    }

    async fn block_address(&self, address: String, timeout_secs: u64) {
        self.block_address(&address, timeout_secs).await; // Call internal helper
    }

    async fn unblock_address(&self, address: String) {
        self.unblock_address(&address).await; // Call internal helper
    }

    async fn add_raw_packet_filter(&self, regex: String) {
        self.raw_packet_filters.lock().await.push(regex);
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        // Ensure shutdown is triggered if the server object is dropped
        if !self.shutdown.load(Ordering::Relaxed) {
            self.shutdown.store(true, Ordering::Relaxed);
            // Best effort signal, ignore error
            let _ = self.shutdown_signal.try_send(());
            // Cannot block here in drop, so can't wait for graceful shutdown fully.
            // The task might continue running for a bit.
            self.logger.warning("Server dropped without explicit shutdown, attempting to signal run loop.");
            self.socket.close(); // Close socket immediately
        }
    }
}