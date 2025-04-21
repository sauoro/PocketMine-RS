// src/raklib/protocol/packets.rs

// Allow names like MessageIdentifiers constants
#![allow(non_snake_case)]
// Allow packet struct names to match PHP/RakNet convention
#![allow(clippy::upper_case_acronyms)]

use super::{Packet, OfflinePacket, ConnectedPacket, MessageIdentifiers};
use crate::utils::binary::{BinaryStream, Result as BinaryResult, BinaryUtilError};
use crate::raklib::utils::InternetAddress;
use crate::raklib; // For constants like DEFAULT_PROTOCOL_VERSION, DEFAULT_SYSTEM_ADDRESS_COUNT
use std::net::{IpAddr, Ipv4Addr}; // For dummy address creation

// --- UnconnectedPing ---
#[derive(Debug, Clone)]
pub struct UnconnectedPing {
    pub send_ping_time: u64, // i64 in PHP, u64 more likely for timestamps
    pub client_id: u64,     // i64 in PHP, use u64 (RakNet GUID)
}

impl Packet for UnconnectedPing {
    fn id() -> u8 { MessageIdentifiers::ID_UNCONNECTED_PING }

    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.encode_header(stream)?; // Write ID
        self.encode_payload(stream)
    }
    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        // ID should be checked by caller (e.g., PacketDecoder) before calling this
        let mut pk = Self { send_ping_time: 0, client_id: 0 };
        // OfflinePacket::decode ensures magic is read and checked first usually
        pk.decode_payload(stream)?;
        Ok(pk)
    }

    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        stream.put_i64_be(self.send_ping_time as i64)?; // Use i64_be for RakNet Long
        self.write_magic(stream)?;
        stream.put_i64_be(self.client_id as i64)?;
        Ok(())
    }
    // Assumes ID and Magic have already been read/verified by the caller (like OfflinePacket handling)
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.send_ping_time = stream.get_i64_be()? as u64;
        // Magic is read *between* timestamp and client_id in this packet structure
        let magic = Self::read_magic(stream)?;
        if !Self::check_magic(&magic) {
            return Err(BinaryUtilError::InvalidData("Invalid magic".to_string()));
        }
        self.client_id = stream.get_i64_be()? as u64;
        Ok(())
    }
}
// Implement OfflinePacket to provide magic handling helpers
impl OfflinePacket for UnconnectedPing {}


// --- UnconnectedPingOpenConnections ---
// Structure is identical to UnconnectedPing, only the ID differs.
#[derive(Debug, Clone)]
pub struct UnconnectedPingOpenConnections(pub UnconnectedPing); // Wrap UnconnectedPing

impl Packet for UnconnectedPingOpenConnections {
    fn id() -> u8 { MessageIdentifiers::ID_UNCONNECTED_PING_OPEN_CONNECTIONS }

    // Delegate encode/decode to inner UnconnectedPing, but manage the ID correctly
    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.encode_header(stream)?; // Write the correct ID (0x02)
        self.0.encode_payload(stream) // Write the inner UnconnectedPing payload (time, magic, clientid)
    }

    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        // ID should be checked by caller first
        let mut inner = UnconnectedPing{send_ping_time: 0, client_id: 0};
        // Decode the payload directly into the inner struct
        // Assumes ID byte was already consumed by caller.
        inner.decode_payload(stream)?;
        Ok(Self(inner))
    }

    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.0.encode_payload(stream) // Delegate payload encoding
    }
    // Payload decoding is handled within the main decode method for this wrapper type
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.0.decode_payload(stream) // Delegate payload decoding
    }
}
impl OfflinePacket for UnconnectedPingOpenConnections {}


// --- UnconnectedPong ---
#[derive(Debug, Clone)]
pub struct UnconnectedPong {
    pub send_ping_time: u64,
    pub server_id: u64, // Server GUID
    pub server_name: String, // Server MOTD/name string
}
impl UnconnectedPong {
    pub fn create(send_ping_time: u64, server_id: u64, server_name: String) -> Self {
        Self { send_ping_time, server_id, server_name }
    }
}
impl Packet for UnconnectedPong {
    fn id() -> u8 { MessageIdentifiers::ID_UNCONNECTED_PONG }
    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.encode_header(stream)?;
        self.encode_payload(stream)
    }
    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        // ID checked by caller
        let mut pk = Self { send_ping_time: 0, server_id: 0, server_name: String::new() };
        pk.decode_payload(stream)?;
        Ok(pk)
    }
    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        stream.put_i64_be(self.send_ping_time as i64)?;
        stream.put_i64_be(self.server_id as i64)?;
        self.write_magic(stream)?;
        stream.put_string(&self.server_name)?; // Uses length-prefixed string helper
        Ok(())
    }
    // Assumes ID handled by caller
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.send_ping_time = stream.get_i64_be()? as u64;
        self.server_id = stream.get_i64_be()? as u64;
        // Magic read between server_id and server_name
        let magic = Self::read_magic(stream)?;
        if !Self::check_magic(&magic) {
            return Err(BinaryUtilError::InvalidData("Invalid magic".to_string()));
        }
        self.server_name = stream.get_string()?; // Uses length-prefixed string helper
        Ok(())
    }
}
impl OfflinePacket for UnconnectedPong {}


// --- OpenConnectionRequest1 ---
#[derive(Debug, Clone)]
pub struct OpenConnectionRequest1 {
    pub protocol: u8, // RakNet protocol version
    /// Determined by the total size of the received packet on decode.
    /// Used for padding the packet up to this size on encode.
    pub mtu_size: u16,
}
impl Packet for OpenConnectionRequest1 {
    fn id() -> u8 { MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_1 }

    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.encode_header(stream)?;
        self.encode_payload(stream)
    }

    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        // ID checked by caller
        // The caller (e.g., PacketDecoder) MUST set mtu_size after decoding.
        let mut pk = Self { protocol: 0, mtu_size: 0 };
        pk.decode_payload(stream)?;
        Ok(pk)
    }

    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.write_magic(stream)?;
        stream.put_u8(self.protocol)?;
        // Pad with null bytes up to mtu_size if mtu_size is validly set
        let current_len = stream.len() + 1; // +1 for packet ID byte assumed written by caller
        if self.mtu_size > 0 && (self.mtu_size as usize) > current_len {
            let padding_len = self.mtu_size as usize - current_len;
            if stream.get_ref().capacity() < stream.len() + padding_len {
                // Ensure underlying Vec has enough capacity if padding
                stream.get_mut().reserve(padding_len);
            }
            stream.put(&vec![0u8; padding_len])?;
        } else if self.mtu_size > 0 && (self.mtu_size as usize) < current_len {
            // This indicates an issue - trying to pad smaller than current content. Error or warning?
            return Err(BinaryUtilError::InvalidData(format!(
                "Cannot pad OpenConnectionRequest1 to MTU {} as current size {} is larger",
                self.mtu_size, current_len
            )));
        }
        // If mtu_size is 0, no padding occurs.
        Ok(())
    }

    // Assumes ID handled by caller
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> {
        // Magic read between ID and protocol
        let magic = Self::read_magic(stream)?;
        if !Self::check_magic(&magic) {
            return Err(BinaryUtilError::InvalidData("Invalid magic".to_string()));
        }
        self.protocol = stream.get_u8()?;
        // MTU size is determined from overall packet length by the caller.
        // Consume any remaining bytes (which represent the padding).
        stream.get_remaining()?; // Read and discard all remaining bytes
        Ok(())
    }
}
impl OfflinePacket for OpenConnectionRequest1 {}


// --- OpenConnectionReply1 ---
#[derive(Debug, Clone)]
pub struct OpenConnectionReply1 {
    pub server_id: u64, // server GUID
    pub server_security: bool, // Does server require security? (Usually false for MCBE)
    pub mtu_size: u16, // Max payload size server accepts for the session
}
impl OpenConnectionReply1 {
    pub fn create(server_id: u64, server_security: bool, mtu_size: u16) -> Self {
        Self { server_id, server_security, mtu_size }
    }
}
impl Packet for OpenConnectionReply1 {
    fn id() -> u8 { MessageIdentifiers::ID_OPEN_CONNECTION_REPLY_1 }
    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.encode_header(stream)?;
        self.encode_payload(stream)
    }
    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        // ID checked by caller
        let mut pk = Self { server_id: 0, server_security: false, mtu_size: 0 };
        pk.decode_payload(stream)?;
        Ok(pk)
    }
    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.write_magic(stream)?;
        stream.put_i64_be(self.server_id as i64)?;
        stream.put_u8(if self.server_security { 1 } else { 0 })?;
        stream.put_u16_be(self.mtu_size)?;
        Ok(())
    }
    // Assumes ID handled by caller
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> {
        // Magic read between ID and server_id
        let magic = Self::read_magic(stream)?;
        if !Self::check_magic(&magic) {
            return Err(BinaryUtilError::InvalidData("Invalid magic".to_string()));
        }
        self.server_id = stream.get_i64_be()? as u64;
        self.server_security = stream.get_u8()? != 0;
        self.mtu_size = stream.get_u16_be()?;
        Ok(())
    }
}
impl OfflinePacket for OpenConnectionReply1 {}


// --- OpenConnectionRequest2 ---
#[derive(Debug, Clone)]
pub struct OpenConnectionRequest2 {
    pub server_address: InternetAddress, // Address client thinks it's connecting to (from OpenConnectionReply1)
    pub mtu_size: u16, // Client's desired final MTU for the session
    pub client_id: u64, // client GUID
}
impl Packet for OpenConnectionRequest2 {
    fn id() -> u8 { MessageIdentifiers::ID_OPEN_CONNECTION_REQUEST_2 }
    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.encode_header(stream)?;
        self.encode_payload(stream)
    }
    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        // ID checked by caller
        let dummy_addr = InternetAddress::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        let mut pk = Self { server_address: dummy_addr, mtu_size: 0, client_id: 0 };
        pk.decode_payload(stream)?;
        Ok(pk)
    }
    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.write_magic(stream)?;
        self.server_address.write_to(stream)?;
        stream.put_u16_be(self.mtu_size)?;
        stream.put_i64_be(self.client_id as i64)?;
        Ok(())
    }
    // Assumes ID handled by caller
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> {
        // Magic read between ID and server_address
        let magic = Self::read_magic(stream)?;
        if !Self::check_magic(&magic) {
            return Err(BinaryUtilError::InvalidData("Invalid magic".to_string()));
        }
        self.server_address = InternetAddress::read_from(stream)?;
        self.mtu_size = stream.get_u16_be()?;
        self.client_id = stream.get_i64_be()? as u64;
        Ok(())
    }
}
impl OfflinePacket for OpenConnectionRequest2 {}


// --- OpenConnectionReply2 ---
#[derive(Debug, Clone)]
pub struct OpenConnectionReply2 {
    pub server_id: u64, // server GUID
    pub client_address: InternetAddress, // Client's public address as seen by server
    pub mtu_size: u16, // Final confirmed MTU size for the connection
    pub server_security: bool, // Encryption enabled? Usually false for MCBE
}
impl OpenConnectionReply2 {
    pub fn create(server_id: u64, client_address: InternetAddress, mtu_size: u16, server_security: bool) -> Self {
        Self { server_id, client_address, mtu_size, server_security }
    }
}
impl Packet for OpenConnectionReply2 {
    fn id() -> u8 { MessageIdentifiers::ID_OPEN_CONNECTION_REPLY_2 }
    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.encode_header(stream)?;
        self.encode_payload(stream)
    }
    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        // ID checked by caller
        let dummy_addr = InternetAddress::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        let mut pk = Self { server_id: 0, client_address: dummy_addr, mtu_size: 0, server_security: false };
        pk.decode_payload(stream)?;
        Ok(pk)
    }
    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.write_magic(stream)?;
        stream.put_i64_be(self.server_id as i64)?;
        self.client_address.write_to(stream)?;
        stream.put_u16_be(self.mtu_size)?;
        stream.put_u8(if self.server_security { 1 } else { 0 })?;
        Ok(())
    }
    // Assumes ID handled by caller
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> {
        // Magic read between ID and server_id
        let magic = Self::read_magic(stream)?;
        if !Self::check_magic(&magic) {
            return Err(BinaryUtilError::InvalidData("Invalid magic".to_string()));
        }
        self.server_id = stream.get_i64_be()? as u64;
        self.client_address = InternetAddress::read_from(stream)?;
        self.mtu_size = stream.get_u16_be()?;
        self.server_security = stream.get_u8()? != 0;
        Ok(())
    }
}
impl OfflinePacket for OpenConnectionReply2 {}


// --- ConnectionRequest ---
#[derive(Debug, Clone)]
pub struct ConnectionRequest {
    pub client_id: u64, // Client GUID
    pub send_ping_time: u64, // Timestamp from client when sending request (RakNetTime)
    pub use_security: bool, // Client requests security? Usually false for MCBE
}
impl Packet for ConnectionRequest {
    fn id() -> u8 { MessageIdentifiers::ID_CONNECTION_REQUEST }
    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.encode_header(stream)?;
        self.encode_payload(stream)
    }
    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        Self::decode_header(stream)?; // Consume ID
        let mut pk = Self{client_id:0, send_ping_time:0, use_security:false};
        pk.decode_payload(stream)?;
        Ok(pk)
    }
    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        stream.put_i64_be(self.client_id as i64)?;
        stream.put_i64_be(self.send_ping_time as i64)?;
        stream.put_u8(if self.use_security { 1 } else { 0 })?;
        Ok(())
    }
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.client_id = stream.get_i64_be()? as u64;
        self.send_ping_time = stream.get_i64_be()? as u64;
        self.use_security = stream.get_u8()? != 0;
        Ok(())
    }
}
impl ConnectedPacket for ConnectionRequest {} // This is sent within an EncapsulatedPacket


// --- ConnectionRequestAccepted ---
#[derive(Debug, Clone)]
pub struct ConnectionRequestAccepted {
    pub address: InternetAddress, // Client's address as seen by server
    pub system_addresses: Vec<InternetAddress>, // RakNet internal addresses (must contain DEFAULT_SYSTEM_ADDRESS_COUNT)
    pub send_ping_time: u64, // Timestamp from original ConnectionRequest (RakNetTime)
    pub send_pong_time: u64, // Timestamp from server when sending this reply (RakNetTime)
}
impl ConnectionRequestAccepted {
    // Constructor ensures system_addresses has the correct length
    pub fn create(address: InternetAddress, mut system_addresses: Vec<InternetAddress>, send_ping_time: u64, send_pong_time: u64) -> Self {
        let dummy = InternetAddress::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        system_addresses.resize(raklib::DEFAULT_SYSTEM_ADDRESS_COUNT, dummy); // Pad/truncate
        Self { address, system_addresses, send_ping_time, send_pong_time }
    }
}
impl Packet for ConnectionRequestAccepted {
    fn id() -> u8 { MessageIdentifiers::ID_CONNECTION_REQUEST_ACCEPTED }
    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.encode_header(stream)?;
        self.encode_payload(stream)
    }
    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        Self::decode_header(stream)?; // Consume ID
        let dummy_addr = InternetAddress::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        let mut pk = Self {
            address: dummy_addr.clone(),
            // Initialize with correct capacity
            system_addresses: Vec::with_capacity(raklib::DEFAULT_SYSTEM_ADDRESS_COUNT),
            send_ping_time: 0,
            send_pong_time: 0,
        };
        pk.decode_payload(stream)?;
        Ok(pk)
    }
    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.address.write_to(stream)?;
        stream.put_u16_be(0)?; // System index (legacy, unused?)

        // Ensure exactly DEFAULT_SYSTEM_ADDRESS_COUNT are written
        for i in 0..raklib::DEFAULT_SYSTEM_ADDRESS_COUNT {
            // If self.system_addresses is shorter, this will use the dummy from create()
            // If longer, it will truncate.
            self.system_addresses[i].write_to(stream)?;
        }

        stream.put_i64_be(self.send_ping_time as i64)?;
        stream.put_i64_be(self.send_pong_time as i64)?;
        Ok(())
    }
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.address = InternetAddress::read_from(stream)?;
        let _system_index = stream.get_u16_be()?; // Read and ignore

        // Read exactly DEFAULT_SYSTEM_ADDRESS_COUNT addresses
        self.system_addresses.clear(); // Clear any initial dummies if decode is called multiple times
        for _ in 0..raklib::DEFAULT_SYSTEM_ADDRESS_COUNT {
            // Check sufficient data before attempting read
            let remaining = stream.len().saturating_sub(stream.get_offset() as usize);
            // Rough estimate: Min size for IPv4 (7) + timestamps (16) = 23
            if remaining < 23 {
                return Err(BinaryUtilError::NotEnoughData{ needed: 23, have: remaining });
            }
            self.system_addresses.push(InternetAddress::read_from(stream)?);
        }

        let remaining = stream.len().saturating_sub(stream.get_offset() as usize);
        if remaining < 16 { return Err(BinaryUtilError::NotEnoughData{ needed: 16, have: remaining});}

        self.send_ping_time = stream.get_i64_be()? as u64;
        self.send_pong_time = stream.get_i64_be()? as u64;
        Ok(())
    }
}
impl ConnectedPacket for ConnectionRequestAccepted {}


// --- NewIncomingConnection ---
#[derive(Debug, Clone)]
pub struct NewIncomingConnection {
    // From RakNet source: Server address is the first one, client addresses are the system_addresses
    pub server_address: InternetAddress, // Address of the server (from client's perspective?)
    pub system_addresses: Vec<InternetAddress>, // Client's addresses (loopback, public, etc.)
    pub send_ping_time: u64, // Timestamp from server when sending ConnectionRequestAccepted (RakNetTime)
    pub send_pong_time: u64, // Timestamp from client when sending this packet (RakNetTime)
}
impl Packet for NewIncomingConnection {
    fn id() -> u8 { MessageIdentifiers::ID_NEW_INCOMING_CONNECTION }
    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.encode_header(stream)?;
        self.encode_payload(stream)
    }
    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        Self::decode_header(stream)?; // Consume ID
        let dummy_addr = InternetAddress::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        let mut pk = Self {
            server_address: dummy_addr.clone(),
            system_addresses: Vec::with_capacity(raklib::DEFAULT_SYSTEM_ADDRESS_COUNT),
            send_ping_time: 0,
            send_pong_time: 0,
        };
        pk.decode_payload(stream)?;
        Ok(pk)
    }
    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.server_address.write_to(stream)?;
        // Write exactly DEFAULT_SYSTEM_ADDRESS_COUNT addresses, padding with dummy if needed.
        let dummy = InternetAddress::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        let mut written_count = 0;
        for addr in &self.system_addresses {
            if written_count >= raklib::DEFAULT_SYSTEM_ADDRESS_COUNT { break; }
            addr.write_to(stream)?;
            written_count += 1;
        }
        while written_count < raklib::DEFAULT_SYSTEM_ADDRESS_COUNT {
            dummy.write_to(stream)?;
            written_count += 1;
        }

        stream.put_i64_be(self.send_ping_time as i64)?;
        stream.put_i64_be(self.send_pong_time as i64)?;
        Ok(())
    }
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.server_address = InternetAddress::read_from(stream)?;
        // Decode exactly DEFAULT_SYSTEM_ADDRESS_COUNT addresses
        self.system_addresses.clear();
        for _ in 0..raklib::DEFAULT_SYSTEM_ADDRESS_COUNT {
            let remaining = stream.len().saturating_sub(stream.get_offset() as usize);
            if remaining < 23 { // Min size for IPv4 + timestamps
                return Err(BinaryUtilError::NotEnoughData{ needed: 23, have: remaining });
            }
            self.system_addresses.push(InternetAddress::read_from(stream)?);
        }

        let remaining = stream.len().saturating_sub(stream.get_offset() as usize);
        if remaining < 16 { return Err(BinaryUtilError::NotEnoughData{ needed: 16, have: remaining});}
        self.send_ping_time = stream.get_i64_be()? as u64;
        self.send_pong_time = stream.get_i64_be()? as u64;
        Ok(())
    }
}
impl ConnectedPacket for NewIncomingConnection {}


// --- DisconnectionNotification ---
#[derive(Debug, Clone, Default)]
pub struct DisconnectionNotification {}
impl Packet for DisconnectionNotification {
    fn id() -> u8 { MessageIdentifiers::ID_DISCONNECTION_NOTIFICATION }
    // No payload
    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> { self.encode_header(stream) }
    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> { Self::decode_header(stream).map(|_| Self::default()) }
    fn encode_payload(&self, _stream: &mut BinaryStream) -> BinaryResult<()> { Ok(()) }
    fn decode_payload(&mut self, _stream: &mut BinaryStream) -> BinaryResult<()> { Ok(()) }
}
impl ConnectedPacket for DisconnectionNotification {}


// --- ConnectedPing ---
#[derive(Debug, Clone)]
pub struct ConnectedPing { pub send_ping_time: u64 }
impl ConnectedPing { pub fn create(send_ping_time: u64) -> Self { Self { send_ping_time } } }
impl Packet for ConnectedPing {
    fn id() -> u8 { MessageIdentifiers::ID_CONNECTED_PING }
    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> { self.encode_header(stream)?; self.encode_payload(stream) }
    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> { Self::decode_header(stream)?; let mut pk=Self{send_ping_time:0}; pk.decode_payload(stream)?; Ok(pk)}
    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> { stream.put_i64_be(self.send_ping_time as i64) }
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> { self.send_ping_time = stream.get_i64_be()? as u64; Ok(())}
}
impl ConnectedPacket for ConnectedPing {}


// --- ConnectedPong ---
#[derive(Debug, Clone)]
pub struct ConnectedPong { pub send_ping_time: u64, pub send_pong_time: u64 }
impl ConnectedPong { pub fn create(send_ping_time: u64, send_pong_time: u64) -> Self { Self { send_ping_time, send_pong_time } } }
impl Packet for ConnectedPong {
    fn id() -> u8 { MessageIdentifiers::ID_CONNECTED_PONG }
    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> { self.encode_header(stream)?; self.encode_payload(stream) }
    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> { Self::decode_header(stream)?; let mut pk=Self{send_ping_time:0, send_pong_time:0}; pk.decode_payload(stream)?; Ok(pk)}
    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> { stream.put_i64_be(self.send_ping_time as i64)?; stream.put_i64_be(self.send_pong_time as i64) }
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> { self.send_ping_time = stream.get_i64_be()? as u64; self.send_pong_time = stream.get_i64_be()? as u64; Ok(()) }
}
impl ConnectedPacket for ConnectedPong {}


// --- IncompatibleProtocolVersion ---
#[derive(Debug, Clone)]
pub struct IncompatibleProtocolVersion { pub protocol_version: u8, pub server_id: u64 }
impl IncompatibleProtocolVersion { pub fn create(protocol_version: u8, server_id: u64) -> Self { Self { protocol_version, server_id } } }
impl Packet for IncompatibleProtocolVersion {
    fn id() -> u8 { MessageIdentifiers::ID_INCOMPATIBLE_PROTOCOL_VERSION }
    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> { self.encode_header(stream)?; self.encode_payload(stream) }
    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        // ID checked by caller
        let mut pk=Self{protocol_version:0, server_id:0};
        pk.decode_payload(stream)?;
        Ok(pk)
    }
    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        stream.put_u8(self.protocol_version)?;
        self.write_magic(stream)?;
        stream.put_i64_be(self.server_id as i64)
    }
    // Assumes ID handled by caller
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.protocol_version = stream.get_u8()?;
        // Magic read between protocol and server_id
        let magic = Self::read_magic(stream)?;
        if !Self::check_magic(&magic) {
            return Err(BinaryUtilError::InvalidData("Invalid magic".to_string()));
        }
        self.server_id = stream.get_i64_be()? as u64;
        Ok(())
    }
}
impl OfflinePacket for IncompatibleProtocolVersion {}


// --- AdvertiseSystem ---
#[derive(Debug, Clone)]
pub struct AdvertiseSystem { pub server_name: String }
impl Packet for AdvertiseSystem {
    fn id() -> u8 { MessageIdentifiers::ID_ADVERTISE_SYSTEM }
    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> { self.encode_header(stream)?; self.encode_payload(stream) }
    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        Self::decode_header(stream)?; // Consume ID
        let mut pk=Self{server_name:String::new()};
        pk.decode_payload(stream)?;
        Ok(pk)
    }
    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> { stream.put_string(&self.server_name) }
    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> { self.server_name = stream.get_string()?; Ok(())}
}
// AdvertiseSystem can technically be offline or connected.
// Mark as Connected based on typical RakLib server usage pattern (sent periodically to connected clients).
impl ConnectedPacket for AdvertiseSystem {}