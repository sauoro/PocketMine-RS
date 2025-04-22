// src/raknet/protocol/packet_serializer.rs
#![allow(dead_code)]

use crate::raknet::utils::internet_address::InternetAddress;
use crate::utils::binary_stream::BinaryStream;
use crate::utils::error::{BinaryDataException, Result as BinaryResult};
use bytes::{BytesMut, BufMut}; // Use BytesMut for underlying buffer

// Wrapper around BinaryStream or directly use BytesMut with helper methods
pub struct PacketSerializer {
    stream: BinaryStream, // Reuse existing BinaryStream logic
}

impl PacketSerializer {
    pub fn new() -> Self {
        Self { stream: BinaryStream::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { stream: BinaryStream::with_capacity(capacity) }
    }

    pub fn from_bytes(buffer: &[u8]) -> Self {
        Self { stream: BinaryStream::from_slice(buffer) }
    }

    // --- RakNet specific methods ---

    pub fn get_string(&mut self) -> BinaryResult<String> {
        let len = self.stream.get_short()?;
        if len == 0 {
            return Ok(String::new());
        }
        let bytes = self.stream.get(len as usize)?;
        String::from_utf8(bytes.to_vec())
            .map_err(|e| BinaryDataException::new(format!("Failed to decode UTF-8 string: {}", e)))
    }

    pub fn put_string(&mut self, v: &str) {
        let bytes = v.as_bytes();
        // RakNet strings use u16 length prefix
        self.stream.put_short(bytes.len() as u16).unwrap(); // TODO: Handle error better
        self.stream.put(bytes);
    }

    pub fn get_address(&mut self) -> BinaryResult<InternetAddress> {
        InternetAddress::read(&mut self.stream)
    }

    pub fn put_address(&mut self, address: &InternetAddress) -> BinaryResult<()> {
        address.write(&mut self.stream)
    }

    // --- Expose underlying BinaryStream methods ---
    pub fn get_buffer(&self) -> &[u8] { self.stream.get_buffer() }
    pub fn get_mut_buffer(&mut self) -> &mut Vec<u8> { self.stream.get_mut_buffer() }
    pub fn put(&mut self, bytes: &[u8]) { self.stream.put(bytes) }
    pub fn put_slice(&mut self, bytes: &[u8]) { self.stream.put(bytes) } // Alias
    pub fn get(&mut self, len: usize) -> BinaryResult<&[u8]> { self.stream.get(len) }
    pub fn get_slice(&mut self, len: usize) -> BinaryResult<&[u8]> { self.stream.get(len) } // Alias
    pub fn get_remaining(&mut self) -> BinaryResult<&[u8]> { self.stream.get_remaining() }
    pub fn get_byte(&mut self) -> BinaryResult<u8> { self.stream.get_byte() }
    pub fn put_byte(&mut self, v: u8) { self.stream.put_byte(v) }
    pub fn get_short(&mut self) -> BinaryResult<u16> { self.stream.get_short() }
    pub fn put_short(&mut self, v: u16) -> BinaryResult<()> { self.stream.put_short(v) }
    pub fn get_lshort(&mut self) -> BinaryResult<u16> { self.stream.get_lshort() }
    pub fn put_lshort(&mut self, v: u16) -> BinaryResult<()> { self.stream.put_lshort(v) }
    pub fn get_triad(&mut self) -> BinaryResult<u32> { self.stream.get_triad() }
    pub fn put_triad(&mut self, v: u32) -> BinaryResult<()> { self.stream.put_triad(v) }
    pub fn get_ltriad(&mut self) -> BinaryResult<u32> { self.stream.get_ltriad() }
    pub fn put_ltriad(&mut self, v: u32) -> BinaryResult<()> { self.stream.put_ltriad(v) }
    pub fn get_int(&mut self) -> BinaryResult<i32> { self.stream.get_int() }
    pub fn put_int(&mut self, v: i32) -> BinaryResult<()> { self.stream.put_int(v) }
    pub fn get_lint(&mut self) -> BinaryResult<i32> { self.stream.get_lint() }
    pub fn put_lint(&mut self, v: i32) -> BinaryResult<()> { self.stream.put_lint(v) }
    pub fn get_long(&mut self) -> BinaryResult<u64> { self.stream.get_unsigned_long() } // RakNet often uses u64 for time/GUIDs
    pub fn put_long(&mut self, v: u64) -> BinaryResult<()> { self.stream.put_unsigned_long(v) }
    pub fn get_llong(&mut self) -> BinaryResult<u64> { self.stream.get_unsigned_llong() }
    pub fn put_llong(&mut self, v: u64) -> BinaryResult<()> { self.stream.put_unsigned_llong(v) }
    pub fn skip(&mut self, len: usize) { self.stream.set_offset(self.stream.get_offset() + len); } // Basic skip
    pub fn feof(&self) -> bool { self.stream.feof() }
    pub fn len(&self) -> usize { self.stream.get_buffer().len() }
    pub fn remaining(&self) -> usize { self.stream.get_buffer().len().saturating_sub(self.stream.get_offset()) }
    pub fn get_offset(&self) -> usize { self.stream.get_offset() }
    pub fn set_offset(&mut self, offset: usize) { self.stream.set_offset(offset) }
    pub fn into_inner(self) -> BytesMut { BytesMut::from(self.stream.get_buffer().to_vec()) } // Convert internal Vec<u8> to BytesMut
}

impl Default for PacketSerializer {
    fn default() -> Self {
        Self::new()
    }
}