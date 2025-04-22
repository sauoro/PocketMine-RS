// src/raknet/protocol/acknowledge_packet.rs

#![allow(dead_code)]

use crate::raknet::protocol::packet_serializer::PacketSerializer;
use crate::utils::error::{Result, BinaryDataException};
use crate::utils::binary; // For write_ltriad, read_ltriad if PacketSerializer doesn't have them
use std::convert::TryInto;

// Enum to represent record types within the payload
const RECORD_TYPE_RANGE: u8 = 0;
const RECORD_TYPE_SINGLE: u8 = 1;

// Maximum number of ACK ranges RakNet will try to parse in one packet.
// Prevents buffer overruns/DoS. Value from RakNet source.
const MAX_ACK_RANGES: u16 = 4096;

/// Holds the common data and serialization logic for ACK/NACK packets.
#[derive(Debug, Clone, Default)]
pub struct AcknowledgePacketData {
    /// Sequence numbers of the packets being acknowledged or negatively acknowledged.
    pub packets: Vec<u32>, // Packet sequence numbers are u24, fit in u32
}

impl AcknowledgePacketData {
    /// Encodes the packet sequence numbers into the compact ACK/NACK format.
    pub fn encode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        // Sort packets to allow for range compression
        self.packets.sort_unstable();
        self.packets.dedup(); // Remove duplicates

        let mut records: u16 = 0;
        let mut buffer = Vec::new(); // Temporary buffer for payload construction
        let count = self.packets.len();

        if count > 0 {
            let mut pointer = 1;
            let mut start = self.packets[0];
            let mut last = self.packets[0];

            while pointer < count {
                let current = self.packets[pointer];
                pointer += 1;
                let diff = current.wrapping_sub(last); // Use wrapping sub for sequence numbers

                if diff == 1 {
                    last = current;
                } else if diff > 1 {
                    // End of a range or single packet
                    if start == last {
                        // Single packet record
                        buffer.push(RECORD_TYPE_SINGLE);
                        // Use helper from utils::binary or implement directly
                        buffer.extend_from_slice(&binary::write_ltriad(start)?);
                    } else {
                        // Range record
                        buffer.push(RECORD_TYPE_RANGE);
                        buffer.extend_from_slice(&binary::write_ltriad(start)?);
                        buffer.extend_from_slice(&binary::write_ltriad(last)?);
                    }
                    records += 1;
                    start = current;
                    last = current;
                }
                // Ignore diff == 0 (shouldn't happen after dedup) or diff < 0 (shouldn't happen after sort)
            }

            // Write the last record
            if start == last {
                buffer.push(RECORD_TYPE_SINGLE);
                buffer.extend_from_slice(&binary::write_ltriad(start)?);
            } else {
                buffer.push(RECORD_TYPE_RANGE);
                buffer.extend_from_slice(&binary::write_ltriad(start)?);
                buffer.extend_from_slice(&binary::write_ltriad(last)?);
            }
            records += 1;
        }

        stream.put_short(records)?;
        stream.put(&buffer);
        Ok(())
    }

    /// Decodes the compact ACK/NACK format into a list of packet sequence numbers.
    pub fn decode_payload(&mut self, stream: &mut PacketSerializer) -> Result<()> {
        let record_count = stream.get_short()?;
        self.packets.clear();

        if record_count > MAX_ACK_RANGES {
            return Err(BinaryDataException::new(format!(
                "ACK/NACK packet decode: Too many records ({}) > MAX_ACK_RANGES ({})",
                record_count, MAX_ACK_RANGES
            )));
        }

        self.packets.reserve(record_count as usize); // Approximate reservation

        for _ in 0..record_count {
            if stream.feof() {
                return Err(BinaryDataException::from_str(
                    "ACK/NACK packet decode: Unexpected EOF while reading records",
                ));
            }
            let record_type = stream.get_byte()?;

            match record_type {
                RECORD_TYPE_SINGLE => {
                    if stream.get_offset() + 3 > stream.get_buffer().len() {
                        return Err(BinaryDataException::from_str("ACK/NACK packet decode: Unexpected EOF reading single record value"));
                    }
                    let seq = stream.get_l_triad()?; // Read LittleEndian Triad
                    self.packets.push(seq);
                }
                RECORD_TYPE_RANGE => {
                    if stream.get_offset() + 6 > stream.get_buffer().len() {
                        return Err(BinaryDataException::from_str("ACK/NACK packet decode: Unexpected EOF reading range record values"));
                    }
                    let start = stream.get_l_triad()?;
                    let end = stream.get_l_triad()?;

                    // Prevent DoS from huge ranges. RakNet uses 8192, but 512 seems safer based on PHP.
                    // Let's use a slightly larger but still reasonable limit.
                    const MAX_RANGE_SIZE: u32 = 2048;
                    if end < start {
                        return Err(BinaryDataException::new(format!(
                            "ACK/NACK packet decode: Invalid range end < start ({} < {})", end, start
                        )));
                    }
                    let range_size = end.wrapping_sub(start).wrapping_add(1); // Calculate size carefully
                    if range_size > MAX_RANGE_SIZE {
                        return Err(BinaryDataException::new(format!(
                            "ACK/NACK packet decode: Range size {} exceeds limit {}", range_size, MAX_RANGE_SIZE
                        )));
                    }

                    // Reserve space if possible, check against overall packet limit.
                    // (Could check against MAX_ACK_RANGES * reasonable_avg_size here if needed)
                    self.packets.reserve(range_size as usize);


                    for c in start..=end {
                        self.packets.push(c);
                        // Optional: Check if self.packets exceeds a total maximum size limit
                        // if self.packets.len() > SOME_ABSOLUTE_MAX_PACKETS { return Err(...) }
                    }
                }
                _ => {
                    return Err(BinaryDataException::new(format!(
                        "ACK/NACK packet decode: Unknown record type {}", record_type
                    )));
                }
            }
        }
        Ok(())
    }
}