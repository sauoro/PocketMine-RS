use super::{Packet, MessageIdentifiers};
use crate::utils::binary::{BinaryStream, Result as BinaryResult, BinaryUtilError};

const RECORD_TYPE_RANGE: u8 = 0;
const RECORD_TYPE_SINGLE: u8 = 1;

/// Represents a single record within an ACK or NACK packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckNackRecord {
    Single(u32), // u24
    Range(u32, u32), // u24, u24
}

/// Base structure and logic for ACK/NACK packets.
#[derive(Debug, Clone)]
pub struct AcknowledgePacket {
    /// Packet sequence numbers being acknowledged or negative-acknowledged.
    /// Note: The PHP code stores these directly as `int[]`, then sorts and encodes ranges.
    /// Here, we might store `AckNackRecord` directly, or process `u32` Vec on encode/decode.
    /// Let's stick closer to PHP logic first: store sequence numbers, process later.
    pub packets: Vec<u32>, // Vec of seq_numbers (u24)
}

impl AcknowledgePacket {
    // Encodes the packet list into range/single records
    pub fn encode_records(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        let mut packets = self.packets.clone();
        packets.sort_unstable(); // Sort numerically

        let mut records: Vec<AckNackRecord> = Vec::new();
        let count = packets.len();

        if count > 0 {
            let mut ptr = 1;
            let mut start = packets[0];
            let mut last = packets[0];

            while ptr < count {
                let current = packets[ptr];
                ptr += 1;
                let diff = current.wrapping_sub(last); // Use wrapping_sub for differences

                if diff == 1 {
                    last = current;
                } else if diff > 1 {
                    // End previous record
                    if start == last {
                        records.push(AckNackRecord::Single(start));
                    } else {
                        records.push(AckNackRecord::Range(start, last));
                    }
                    // Start new record
                    start = current;
                    last = current;
                }
                // Ignore diff == 0 (duplicates) or diff < 0 (shouldn't happen after sort)
            }

            // Final record
            if start == last {
                records.push(AckNackRecord::Single(start));
            } else {
                records.push(AckNackRecord::Range(start, last));
            }
        }

        stream.put_u16_be(records.len() as u16)?; // Record count is Big Endian

        for record in records {
            match record {
                AckNackRecord::Single(seq) => {
                    stream.put_u8(RECORD_TYPE_SINGLE)?;
                    stream.put_u24_le(seq)?;
                }
                AckNackRecord::Range(start, end) => {
                    stream.put_u8(RECORD_TYPE_RANGE)?;
                    stream.put_u24_le(start)?;
                    stream.put_u24_le(end)?;
                }
            }
        }
        Ok(())
    }

    // Decodes range/single records into a list of packet sequence numbers
    pub fn decode_records(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> {
        let record_count = stream.get_u16_be()?; // Record count is Big Endian
        self.packets.clear();
        // Protect against malicious large record counts, limit total packets decoded
        let mut packet_count = 0;
        const MAX_PACKETS_PER_ACK: usize = 8192; // Generous limit

        for _ in 0..record_count {
            if stream.feof() || packet_count >= MAX_PACKETS_PER_ACK { break; }

            let record_type = stream.get_u8()?;
            if record_type == RECORD_TYPE_SINGLE {
                let seq = stream.get_u24_le()?;
                if packet_count < MAX_PACKETS_PER_ACK {
                    self.packets.push(seq);
                    packet_count += 1;
                }
            } else if record_type == RECORD_TYPE_RANGE {
                let start = stream.get_u24_le()?;
                let end = stream.get_u24_le()?;
                // Protect against huge ranges
                if end < start { continue; } // Invalid range
                let range_count = end - start + 1;
                // Check intermediate overflow and overall limit
                if range_count > MAX_PACKETS_PER_ACK as u32 || packet_count + (range_count as usize) > MAX_PACKETS_PER_ACK {
                    // Limit the range to avoid exceeding max packets, or error out
                    // For now, just truncate the range
                    let allowed = MAX_PACKETS_PER_ACK - packet_count;
                    for i in 0..allowed {
                        self.packets.push(start + i as u32);
                    }
                    packet_count = MAX_PACKETS_PER_ACK;
                    break; // Stop processing further records
                } else {
                    for seq in start..=end {
                        self.packets.push(seq);
                    }
                    packet_count += range_count as usize;
                }

            } else {
                // Invalid record type, maybe stop processing?
                return Err(BinaryUtilError::InvalidData(format!("Invalid ACK/NACK record type {}", record_type)));
            }
        }
        Ok(())
    }
}

// --- ACK Packet ---
#[derive(Debug, Clone)]
pub struct ACK(pub AcknowledgePacket); // Tuple struct wrapping the base

impl Packet for ACK {
    fn id() -> u8 { MessageIdentifiers::ID_ACK }

    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.encode_header(stream)?;
        self.encode_payload(stream)
    }

    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        Self::decode_header(stream)?;
        let mut ack_base = AcknowledgePacket { packets: Vec::new() };
        ack_base.decode_records(stream)?;
        Ok(ACK(ack_base))
    }

    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.0.encode_records(stream)
    }

    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.0.decode_records(stream)
    }
}

// --- NACK Packet ---
#[derive(Debug, Clone)]
pub struct NACK(pub AcknowledgePacket); // Tuple struct wrapping the base

impl Packet for NACK {
    fn id() -> u8 { MessageIdentifiers::ID_NACK }

    fn encode(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.encode_header(stream)?;
        self.encode_payload(stream)
    }

    fn decode(stream: &mut BinaryStream) -> BinaryResult<Self> {
        Self::decode_header(stream)?;
        let mut nack_base = AcknowledgePacket { packets: Vec::new() };
        nack_base.decode_records(stream)?;
        Ok(NACK(nack_base))
    }

    fn encode_payload(&self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.0.encode_records(stream)
    }

    fn decode_payload(&mut self, stream: &mut BinaryStream) -> BinaryResult<()> {
        self.0.decode_records(stream)
    }
}