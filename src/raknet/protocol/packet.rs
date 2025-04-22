// src/raknet/protocol/packet.rs

#![allow(dead_code)]

use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::Result; // Use the Result from utils::error
use std::fmt::Debug; // Require Debug for easier logging/inspection

// Define a macro to implement boilerplate for packets
// This reduces repetition for simple packets.
// It assumes the packet struct has fields matching the encode/decode order.
// More complex packets will need manual `impl Packet` blocks.
#[macro_export]
macro_rules! impl_packet_codec {
    ($struct_name:ident($packet_id:expr) { $( $field_name:ident : $field_type:ident ),* $(,)? }) => {
        impl $crate::raknet::protocol::packet::Packet for $struct_name {
            fn get_id(&self) -> u8 {
                $packet_id
            }

            fn encode_payload(&self, mut stream: &mut $crate::raknet::protocol::packet_serializer::PacketSerializer) -> $crate::utils::error::Result<()> {
                $(
                    // Use paste! or manually handle method names if needed
                    paste::paste! {
                        stream.[<put_ $field_type>](self.$field_name)?;
                    }
                )*
                Ok(())
            }

            fn decode_payload(&mut self, mut stream: &mut $crate::raknet::protocol::packet_serializer::PacketSerializer) -> $crate::utils::error::Result<()> {
                $(
                     paste::paste! {
                        self.$field_name = stream.[<get_ $field_type>]()?;
                    }
                )*
                Ok(())
            }
        }
    };
     // Variant for packets with no payload
    ($struct_name:ident($packet_id:expr) {}) => {
        impl $crate::raknet::protocol::packet::Packet for $struct_name {
             fn get_id(&self) -> u8 {
                $packet_id
            }
            fn encode_payload(&self, _stream: &mut $crate::raknet::protocol::packet_serializer::PacketSerializer) -> $crate::utils::error::Result<()> {
                Ok(()) // No payload to encode
            }
            fn decode_payload(&mut self, _stream: &mut $crate::raknet::protocol::packet_serializer::PacketSerializer) -> $crate::utils::error::Result<()> {
                Ok(()) // No payload to decode
            }
        }
    };
}

// Re-export the macro
pub(crate) use impl_packet_codec;

// Base trait for all packets
pub trait Packet: Debug + Send + Sync {
    /// Returns the specific ID of this packet type.
    fn get_id(&self) -> u8;

    /// Encodes the packet header (usually just the ID) into the stream.
    /// Default implementation writes the result of `get_id()`.
    fn encode_header(&self, stream: &mut PacketSerializer) -> Result<()> {
        stream.put_byte(self.get_id());
        Ok(())
    }

    /// Encodes the packet-specific payload into the stream.
    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()>;

    /// Decodes the packet header from the stream.
    /// Default implementation reads and verifies the packet ID.
    fn decode_header(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        let _id = stream.get_byte()?;
        // Optionally verify ID here if needed, though type dispatch usually handles this.
        // if id != self.get_id() { ... return Err(...) }
        Ok(())
    }

    /// Decodes the packet-specific payload from the stream.
    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()>;

    /// Encodes the entire packet (header + payload) into a stream.
    fn encode(&self, stream: &mut PacketSerializer) -> Result<()> {
        self.encode_header(stream)?;
        self.encode_payload(stream)?;
        Ok(())
    }

    /// Decodes the entire packet (header + payload) from a stream.
    fn decode(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        self.decode_header(stream)?;
        self.decode_payload(stream)?;
        Ok(())
    }
}

// Allow Box<dyn Packet> to be treated as a Packet reference
impl<T: Packet + ?Sized> Packet for Box<T> {
    fn get_id(&self) -> u8 {
        (**self).get_id()
    }
    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        (**self).encode_payload(stream)
    }
    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        (**self).decode_payload(stream)
    }
    // Override encode/decode to avoid double boxing if needed
    fn encode(&self, stream: &mut PacketSerializer) -> Result<()> {
        (**self).encode(stream)
    }
    fn decode(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        (**self).decode(stream)
    }
}

// Implement Packet for Vec<u8> representing a raw packet buffer
// This is useful when you just want to pass raw data around without full decoding.
impl Packet for Vec<u8> {
    fn get_id(&self) -> u8 {
        if self.is_empty() {
            0 // Or perhaps panic/return Option<u8>? Define behavior for empty buffer.
        } else {
            self[0]
        }
    }

    fn encode_payload(&self, stream: &mut PacketSerializer) -> Result<()> {
        if self.len() > 1 {
            stream.put(&self[1..]);
        }
        Ok(())
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        let remaining = stream.get_remaining()?.to_vec();
        self.clear(); // Clear existing content
        // ID is assumed already read by decode_header (which reads [0] by default)
        self.push(stream.get_buffer()[stream.get_offset() - 1]); // Re-add the ID byte read by decode_header
        self.extend_from_slice(&remaining); // Add the rest of the payload
        Ok(())
    }

    // Override encode/decode for raw bytes
    fn encode(&self, stream: &mut PacketSerializer) -> Result<()> {
        stream.put(self);
        Ok(())
    }

    fn decode(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        // Read the entire remaining buffer into self
        let full_buffer = stream.get_remaining()?.to_vec();
        // The ID byte is already consumed by the caller usually,
        // but if decode is called directly on a stream at position 0,
        // we need to handle it.
        // Assuming decode is called *after* the ID has been read
        // to determine the type:
        let initial_offset = stream.get_offset();
        if initial_offset > 0 {
            self.clear();
            self.push(stream.get_buffer()[initial_offset - 1]); // ID byte
            self.extend_from_slice(&full_buffer);
        } else {
            // If called at position 0, maybe error or just copy whole buffer?
            // Let's assume the ID was read.
            return Err(crate::utils::error::BinaryDataException::from_str(
                "Packet::decode for Vec<u8> called on stream at offset 0"
            ));
        }

        Ok(())
    }
}