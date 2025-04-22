// src/raknet/protocol/packet.rs
#![allow(dead_code)]

use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{BinaryDataException, Result as BinaryResult};
use bytes::BytesMut;

pub trait Packet: Send + Sync + std::fmt::Debug + Clone {
    fn get_id() -> u8 where Self: Sized;

    fn encode_header(&self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        stream.put_byte(Self::get_id());
        Ok(())
    }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()>;

    fn encode(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        self.encode_header(stream)?;
        self.encode_payload(stream)
    }

    fn decode_header(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        let id = stream.get_byte()?;
        if id != Self::get_id() {
            Err(BinaryDataException::new(format!(
                "Expected packet ID {}, but got {}",
                Self::get_id(),
                id
            )))
        } else {
            Ok(())
        }
    }

    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()>;

    fn decode(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        self.decode_header(stream)?;
        self.decode_payload(stream)
    }

    fn try_decode(stream: &mut PacketSerializer) -> BinaryResult<Self> where Self: Sized + Default {
        let mut packet = Self::default();
        packet.decode(stream)?;
        Ok(packet)
    }

    fn to_binary(&mut self) -> BinaryResult<BytesMut> {
        let mut stream = PacketSerializer::new();
        self.encode(&mut stream)?;
        Ok(stream.into_inner())
    }
}