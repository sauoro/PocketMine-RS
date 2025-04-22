// src/raknet/protocol/acknowledge_packet.rs
#![allow(dead_code)]

use crate::raknet::protocol::packet::Packet;
use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::binary;
use crate::utils::error::{BinaryDataException, Result as BinaryResult};
use std::convert::TryInto;

const RECORD_TYPE_RANGE: u8 = 0;
const RECORD_TYPE_SINGLE: u8 = 1;

pub trait AcknowledgePacket: Packet {
    fn get_packets(&self) -> &[u32];
    fn get_packets_mut(&mut self) -> &mut Vec<u32>;

    fn encode_payload_impl(&self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        let mut packets = self.get_packets().to_vec();
        packets.sort_unstable();
        packets.dedup(); // Ensure no duplicates

        let mut payload_buf = Vec::new();
        let count = packets.len();
        let mut records: u16 = 0;

        if count > 0 {
            let mut pointer = 1;
            let mut start = packets[0];
            let mut last = packets[0];

            while pointer < count {
                let current = packets[pointer];
                pointer += 1;
                let diff = current.wrapping_sub(last);
                if diff == 1 {
                    last = current;
                } else if diff > 1 {
                    // End of sequence or single packet
                    if start == last {
                        payload_buf.push(RECORD_TYPE_SINGLE);
                        payload_buf.extend_from_slice(&binary::write_ltriad(start)?);
                        start = current;
                        last = current;
                    } else {
                        payload_buf.push(RECORD_TYPE_RANGE);
                        payload_buf.extend_from_slice(&binary::write_ltriad(start)?);
                        payload_buf.extend_from_slice(&binary::write_ltriad(last)?);
                        start = current;
                        last = current;
                    }
                    records += 1;
                } // Ignore duplicates (diff == 0)
            }

            // Write the last record
            if start == last {
                payload_buf.push(RECORD_TYPE_SINGLE);
                payload_buf.extend_from_slice(&binary::write_ltriad(start)?);
            } else {
                payload_buf.push(RECORD_TYPE_RANGE);
                payload_buf.extend_from_slice(&binary::write_ltriad(start)?);
                payload_buf.extend_from_slice(&binary::write_ltriad(last)?);
            }
            records += 1;
        }

        stream.put_short(records)?;
        stream.put_slice(&payload_buf);
        Ok(())
    }

    fn decode_payload_impl(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        let count = stream.get_short()?;
        self.get_packets_mut().clear();
        let mut cnt = 0;
        for _ in 0..count {
            if stream.feof() || cnt >= 4096 {
                break;
            }
            if stream.get_byte()? == RECORD_TYPE_RANGE {
                let start = stream.get_ltriad()?;
                let end = stream.get_ltriad()?;
                if end < start {
                    return Err(BinaryDataException::from_str("Invalid ACK/NACK range record: end < start"));
                }
                // Limit the number of packets added in a single range to prevent OOM
                let range_count = (end - start).saturating_add(1);
                if range_count > 512 {
                    return Err(BinaryDataException::from_str("ACK/NACK range record exceeded limit of 512 packets"));
                }

                if cnt + (range_count as usize) > 4096 {
                    return Err(BinaryDataException::from_str("ACK/NACK packet limit exceeded"));
                }
                for c in start..=end {
                    self.get_packets_mut().push(c);
                    cnt += 1;
                }
            } else {
                if cnt >= 4096 {
                    return Err(BinaryDataException::from_str("ACK/NACK packet limit exceeded"));
                }
                self.get_packets_mut().push(stream.get_ltriad()?);
                cnt += 1;
            }
        }
        Ok(())
    }
}

// Implement Packet for Box<dyn AcknowledgePacket> or specific types
impl<T: AcknowledgePacket> Packet for T {
    fn get_id() -> u8 where Self: Sized { T::get_id() }

    fn encode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        self.encode_payload_impl(stream)
    }
    fn decode_payload(&mut self, stream: &mut PacketSerializer) -> BinaryResult<()> {
        self.decode_payload_impl(stream)
    }
}