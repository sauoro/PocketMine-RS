// src/utils/binary.rs
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    io::{self, Cursor, Read, Seek, SeekFrom, Write}, // Added Seek, SeekFrom
    mem, // For size_of
};
use thiserror::Error;

// --- Error Type (Replaces BinaryDataException) ---
#[derive(Error, Debug)]
pub enum BinaryUtilError {
    #[error("Not enough bytes in buffer: needed {needed}, have {have}")]
    NotEnoughData { needed: usize, have: usize },

    #[error("VarInt did not terminate after 5 bytes")]
    VarIntTooLong,

    #[error("VarLong did not terminate after 10 bytes")]
    VarLongTooLong,

    #[error("Value too large to be encoded as VarInt: {0}")]
    VarIntTooLarge(u32),

    #[error("Value too large to be encoded as VarLong: {0}")]
    VarLongTooLarge(u64),

    #[error("Invalid UTF-8 sequence: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Invalid data: {0}")] // <<< Added Variant
    InvalidData(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error), // Catch errors from Read/Write operations
}

// Define a public Result type alias for convenience <<< Made Public
pub type Result<T> = std::result::Result<T, BinaryUtilError>;

// --- Standalone Binary Read/Write Functions (Replaces static Binary methods) ---
pub const SIZEOF_BYTE: usize = mem::size_of::<u8>();
pub const SIZEOF_SHORT: usize = mem::size_of::<u16>();
pub const SIZEOF_INT: usize = mem::size_of::<u32>();
pub const SIZEOF_LONG: usize = mem::size_of::<u64>();
pub const SIZEOF_FLOAT: usize = mem::size_of::<f32>();
pub const SIZEOF_DOUBLE: usize = mem::size_of::<f64>();

// Boolean
pub fn read_bool(reader: &mut impl Read) -> Result<bool> { Ok(reader.read_u8()? != 0) }
pub fn write_bool(writer: &mut impl Write, val: bool) -> Result<()> { writer.write_u8(if val { 1 } else { 0 })?; Ok(()) }
// Bytes (u8 / i8)
pub fn read_u8(reader: &mut impl Read) -> Result<u8> { Ok(reader.read_u8()?) }
pub fn read_i8(reader: &mut impl Read) -> Result<i8> { Ok(reader.read_i8()?) }
pub fn write_u8(writer: &mut impl Write, val: u8) -> Result<()> { writer.write_u8(val)?; Ok(()) }
pub fn write_i8(writer: &mut impl Write, val: i8) -> Result<()> { writer.write_i8(val)?; Ok(()) }
// Shorts (u16 / i16) - BE
pub fn read_u16_be(reader: &mut impl Read) -> Result<u16> { Ok(reader.read_u16::<BigEndian>()?) }
pub fn read_i16_be(reader: &mut impl Read) -> Result<i16> { Ok(reader.read_i16::<BigEndian>()?) }
pub fn write_u16_be(writer: &mut impl Write, val: u16) -> Result<()> { writer.write_u16::<BigEndian>(val)?; Ok(()) }
pub fn write_i16_be(writer: &mut impl Write, val: i16) -> Result<()> { writer.write_i16::<BigEndian>(val)?; Ok(()) }
// Shorts (u16 / i16) - LE
pub fn read_u16_le(reader: &mut impl Read) -> Result<u16> { Ok(reader.read_u16::<LittleEndian>()?) }
pub fn read_i16_le(reader: &mut impl Read) -> Result<i16> { Ok(reader.read_i16::<LittleEndian>()?) }
pub fn write_u16_le(writer: &mut impl Write, val: u16) -> Result<()> { writer.write_u16::<LittleEndian>(val)?; Ok(()) }
pub fn write_i16_le(writer: &mut impl Write, val: i16) -> Result<()> { writer.write_i16::<LittleEndian>(val)?; Ok(()) }
// Triads (u24) - BE
pub fn read_u24_be(reader: &mut impl Read) -> Result<u32> { let mut buf = [0u8; 3]; reader.read_exact(&mut buf)?; Ok(u32::from_be_bytes([0, buf[0], buf[1], buf[2]])) }
pub fn write_u24_be(writer: &mut impl Write, val: u32) -> Result<()> { let buf = val.to_be_bytes(); writer.write_all(&buf[1..4])?; Ok(()) }
// Triads (u24) - LE
pub fn read_u24_le(reader: &mut impl Read) -> Result<u32> { let mut buf = [0u8; 3]; reader.read_exact(&mut buf)?; Ok(u32::from_le_bytes([buf[0], buf[1], buf[2], 0])) }
pub fn write_u24_le(writer: &mut impl Write, val: u32) -> Result<()> { let buf = val.to_le_bytes(); writer.write_all(&buf[0..3])?; Ok(()) }
// Ints (u32 / i32) - BE
pub fn read_u32_be(reader: &mut impl Read) -> Result<u32> { Ok(reader.read_u32::<BigEndian>()?) }
pub fn read_i32_be(reader: &mut impl Read) -> Result<i32> { Ok(reader.read_i32::<BigEndian>()?) }
pub fn write_u32_be(writer: &mut impl Write, val: u32) -> Result<()> { writer.write_u32::<BigEndian>(val)?; Ok(()) }
pub fn write_i32_be(writer: &mut impl Write, val: i32) -> Result<()> { writer.write_i32::<BigEndian>(val)?; Ok(()) }
// Ints (u32 / i32) - LE
pub fn read_u32_le(reader: &mut impl Read) -> Result<u32> { Ok(reader.read_u32::<LittleEndian>()?) }
pub fn read_i32_le(reader: &mut impl Read) -> Result<i32> { Ok(reader.read_i32::<LittleEndian>()?) }
pub fn write_u32_le(writer: &mut impl Write, val: u32) -> Result<()> { writer.write_u32::<LittleEndian>(val)?; Ok(()) }
pub fn write_i32_le(writer: &mut impl Write, val: i32) -> Result<()> { writer.write_i32::<LittleEndian>(val)?; Ok(()) }
// Floats (f32) - BE
pub fn read_f32_be(reader: &mut impl Read) -> Result<f32> { Ok(reader.read_f32::<BigEndian>()?) }
pub fn write_f32_be(writer: &mut impl Write, val: f32) -> Result<()> { writer.write_f32::<BigEndian>(val)?; Ok(()) }
// Floats (f32) - LE
pub fn read_f32_le(reader: &mut impl Read) -> Result<f32> { Ok(reader.read_f32::<LittleEndian>()?) }
pub fn write_f32_le(writer: &mut impl Write, val: f32) -> Result<()> { writer.write_f32::<LittleEndian>(val)?; Ok(()) }
// Doubles (f64) - BE
pub fn read_f64_be(reader: &mut impl Read) -> Result<f64> { Ok(reader.read_f64::<BigEndian>()?) }
pub fn write_f64_be(writer: &mut impl Write, val: f64) -> Result<()> { writer.write_f64::<BigEndian>(val)?; Ok(()) }
// Doubles (f64) - LE
pub fn read_f64_le(reader: &mut impl Read) -> Result<f64> { Ok(reader.read_f64::<LittleEndian>()?) }
pub fn write_f64_le(writer: &mut impl Write, val: f64) -> Result<()> { writer.write_f64::<LittleEndian>(val)?; Ok(()) }
// Longs (u64 / i64) - BE
pub fn read_u64_be(reader: &mut impl Read) -> Result<u64> { Ok(reader.read_u64::<BigEndian>()?) }
pub fn read_i64_be(reader: &mut impl Read) -> Result<i64> { Ok(reader.read_i64::<BigEndian>()?) }
pub fn write_u64_be(writer: &mut impl Write, val: u64) -> Result<()> { writer.write_u64::<BigEndian>(val)?; Ok(()) }
pub fn write_i64_be(writer: &mut impl Write, val: i64) -> Result<()> { writer.write_i64::<BigEndian>(val)?; Ok(()) }
// Longs (u64 / i64) - LE
pub fn read_u64_le(reader: &mut impl Read) -> Result<u64> { Ok(reader.read_u64::<LittleEndian>()?) }
pub fn read_i64_le(reader: &mut impl Read) -> Result<i64> { Ok(reader.read_i64::<LittleEndian>()?) }
pub fn write_u64_le(writer: &mut impl Write, val: u64) -> Result<()> { writer.write_u64::<LittleEndian>(val)?; Ok(()) }
pub fn write_i64_le(writer: &mut impl Write, val: i64) -> Result<()> { writer.write_i64::<LittleEndian>(val)?; Ok(()) }

// VarInts / VarLongs
const VARINT_MAX_BYTES: usize = 5;
const VARLONG_MAX_BYTES: usize = 10;

// Unsigned VarInt (u32)
pub fn read_var_u32(reader: &mut impl Read) -> Result<u32> {
    let mut value: u32 = 0;
    let mut shift = 0;
    for _i in 0..VARINT_MAX_BYTES { // Use _i as loop var is unused
        let byte = reader.read_u8()?;
        value |= ((byte & 0x7F) as u32) << shift;
        if (byte & 0x80) == 0 {
            return Ok(value);
        }
        shift += 7;
        if shift >= 32 {
            // Optional check, PMMP doesn't have it. Could error or truncate.
        }
    }
    Err(BinaryUtilError::VarIntTooLong)
}

pub fn write_var_u32(writer: &mut impl Write, mut value: u32) -> Result<()> {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        writer.write_u8(byte)?;
        if value == 0 {
            break;
        }
    }
    Ok(())
}

// Signed VarInt (i32 - ZigZag encoded)
pub fn read_var_i32(reader: &mut impl Read) -> Result<i32> {
    let raw = read_var_u32(reader)?;
    Ok(((raw >> 1) ^ (-((raw & 1) as i32)) as u32) as i32) // Standard ZigZag decode
}

pub fn write_var_i32(writer: &mut impl Write, value: i32) -> Result<()> {
    write_var_u32(writer, ((value << 1) ^ (value >> 31)) as u32) // Standard ZigZag encode
}

// Unsigned VarLong (u64)
pub fn read_var_u64(reader: &mut impl Read) -> Result<u64> {
    let mut value: u64 = 0;
    let mut shift = 0;
    for _i in 0..VARLONG_MAX_BYTES { // Use _i
        let byte = reader.read_u8()?;
        value |= ((byte & 0x7F) as u64) << shift;
        if (byte & 0x80) == 0 {
            return Ok(value);
        }
        shift += 7;
        if shift >= 64 {
            // Optional check
        }
    }
    Err(BinaryUtilError::VarLongTooLong)
}

pub fn write_var_u64(writer: &mut impl Write, mut value: u64) -> Result<()> {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        writer.write_u8(byte)?;
        if value == 0 {
            break;
        }
    }
    Ok(())
}

// Signed VarLong (i64 - ZigZag encoded)
pub fn read_var_i64(reader: &mut impl Read) -> Result<i64> {
    let raw = read_var_u64(reader)?;
    Ok(((raw >> 1) ^ (-((raw & 1) as i64)) as u64) as i64)
}

pub fn write_var_i64(writer: &mut impl Write, value: i64) -> Result<()> {
    write_var_u64(writer, ((value << 1) ^ (value >> 63)) as u64)
}


// Endianness Flipping
pub fn flip_short_endianness(val: u16) -> u16 { val.swap_bytes() }
pub fn flip_int_endianness(val: u32) -> u32 { val.swap_bytes() }
pub fn flip_long_endianness(val: u64) -> u64 { val.swap_bytes() }
// Raw Bytes / String
pub fn read_bytes(reader: &mut impl Read, len: usize) -> Result<Vec<u8>> { let mut buf = vec![0u8; len]; reader.read_exact(&mut buf)?; Ok(buf) }
pub fn write_bytes(writer: &mut impl Write, bytes: &[u8]) -> Result<()> { writer.write_all(bytes)?; Ok(()) }
pub fn read_string(reader: &mut impl Read) -> Result<String> { let len = read_var_u32(reader)? as usize; let bytes = read_bytes(reader, len)?; Ok(String::from_utf8(bytes)?) }
pub fn write_string(writer: &mut impl Write, val: &str) -> Result<()> { let bytes = val.as_bytes(); write_var_u32(writer, bytes.len() as u32)?; write_bytes(writer, bytes)?; Ok(()) }


// --- BinaryStream ---
#[derive(Debug, Default)]
pub struct BinaryStream {
    cursor: Cursor<Vec<u8>>,
}

impl BinaryStream {
    /// Creates a new, empty BinaryStream.
    pub fn new() -> Self {
        Self { cursor: Cursor::new(Vec::new()) }
    }

    /// Creates a BinaryStream wrapping an existing byte vector.
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self { cursor: Cursor::new(vec) }
    }

    /// Creates a read-only BinaryStreamReader wrapping a byte slice. <<< Added Method
    pub fn from_slice(slice: &[u8]) -> BinaryStreamReader {
        BinaryStreamReader { cursor: Cursor::new(slice) }
    }

    // --- Buffer Access and Control ---
    pub fn get_offset(&self) -> u64 { self.cursor.position() }
    pub fn set_offset(&mut self, pos: u64) { self.cursor.set_position(pos); }
    pub fn rewind(&mut self) { self.set_offset(0); }
    pub fn get_buffer(&self) -> &Vec<u8> { self.cursor.get_ref() }
    pub fn into_inner(self) -> Vec<u8> { self.cursor.into_inner() }
    pub fn len(&self) -> usize { self.cursor.get_ref().len() }
    pub fn is_empty(&self) -> bool { self.cursor.get_ref().is_empty() }
    pub fn clear(&mut self) { self.set_offset(0); self.cursor.get_mut().clear(); }
    pub fn feof(&self) -> bool { self.cursor.position() >= (self.len() as u64) }

    pub fn put(&mut self, bytes: &[u8]) -> Result<()> { self.cursor.write_all(bytes)?; Ok(()) }

    pub fn get(&mut self, len: usize) -> Result<Vec<u8>> {
        let current_pos = self.get_offset() as usize;
        let buffer_len = self.len();
        if current_pos.saturating_add(len) > buffer_len {
            return Err(BinaryUtilError::NotEnoughData { needed: len, have: buffer_len.saturating_sub(current_pos) });
        }
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn get_remaining(&mut self) -> Result<Vec<u8>> {
        let current_pos = self.get_offset() as usize;
        let buffer_len = self.len();
        if current_pos >= buffer_len {
            return Err(BinaryUtilError::NotEnoughData { needed: 1, have: 0 });
        }
        let remaining_len = buffer_len - current_pos;
        let mut buf = vec![0u8; remaining_len];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Provides mutable access to the underlying Vec<u8>. <<< Added Method
    pub fn get_mut_vec(&mut self) -> &mut Vec<u8> {
        self.cursor.get_mut()
    }

    // --- Read Methods ---
    pub fn get_bool(&mut self) -> Result<bool> { read_bool(&mut self.cursor) }
    pub fn get_u8(&mut self) -> Result<u8> { read_u8(&mut self.cursor) }
    pub fn get_i8(&mut self) -> Result<i8> { read_i8(&mut self.cursor) }
    pub fn get_u16_be(&mut self) -> Result<u16> { read_u16_be(&mut self.cursor) }
    pub fn get_i16_be(&mut self) -> Result<i16> { read_i16_be(&mut self.cursor) }
    pub fn get_u16_le(&mut self) -> Result<u16> { read_u16_le(&mut self.cursor) }
    pub fn get_i16_le(&mut self) -> Result<i16> { read_i16_le(&mut self.cursor) }
    pub fn get_u24_be(&mut self) -> Result<u32> { read_u24_be(&mut self.cursor) }
    pub fn get_u24_le(&mut self) -> Result<u32> { read_u24_le(&mut self.cursor) }
    pub fn get_u32_be(&mut self) -> Result<u32> { read_u32_be(&mut self.cursor) }
    pub fn get_i32_be(&mut self) -> Result<i32> { read_i32_be(&mut self.cursor) }
    pub fn get_u32_le(&mut self) -> Result<u32> { read_u32_le(&mut self.cursor) }
    pub fn get_i32_le(&mut self) -> Result<i32> { read_i32_le(&mut self.cursor) }
    pub fn get_f32_be(&mut self) -> Result<f32> { read_f32_be(&mut self.cursor) }
    pub fn get_f32_le(&mut self) -> Result<f32> { read_f32_le(&mut self.cursor) }
    pub fn get_f64_be(&mut self) -> Result<f64> { read_f64_be(&mut self.cursor) }
    pub fn get_f64_le(&mut self) -> Result<f64> { read_f64_le(&mut self.cursor) }
    pub fn get_u64_be(&mut self) -> Result<u64> { read_u64_be(&mut self.cursor) }
    pub fn get_i64_be(&mut self) -> Result<i64> { read_i64_be(&mut self.cursor) }
    pub fn get_u64_le(&mut self) -> Result<u64> { read_u64_le(&mut self.cursor) }
    pub fn get_i64_le(&mut self) -> Result<i64> { read_i64_le(&mut self.cursor) }
    pub fn get_var_u32(&mut self) -> Result<u32> { read_var_u32(&mut self.cursor) }
    pub fn get_var_i32(&mut self) -> Result<i32> { read_var_i32(&mut self.cursor) }
    pub fn get_var_u64(&mut self) -> Result<u64> { read_var_u64(&mut self.cursor) }
    pub fn get_var_i64(&mut self) -> Result<i64> { read_var_i64(&mut self.cursor) }
    pub fn get_string(&mut self) -> Result<String> { read_string(&mut self.cursor) }

    // --- Write Methods ---
    pub fn put_bool(&mut self, val: bool) -> Result<()> { write_bool(&mut self.cursor, val) }
    pub fn put_u8(&mut self, val: u8) -> Result<()> { write_u8(&mut self.cursor, val) }
    pub fn put_i8(&mut self, val: i8) -> Result<()> { write_i8(&mut self.cursor, val) }
    pub fn put_u16_be(&mut self, val: u16) -> Result<()> { write_u16_be(&mut self.cursor, val) }
    pub fn put_i16_be(&mut self, val: i16) -> Result<()> { write_i16_be(&mut self.cursor, val) }
    pub fn put_u16_le(&mut self, val: u16) -> Result<()> { write_u16_le(&mut self.cursor, val) }
    pub fn put_i16_le(&mut self, val: i16) -> Result<()> { write_i16_le(&mut self.cursor, val) }
    pub fn put_u24_be(&mut self, val: u32) -> Result<()> { write_u24_be(&mut self.cursor, val) }
    pub fn put_u24_le(&mut self, val: u32) -> Result<()> { write_u24_le(&mut self.cursor, val) }
    pub fn put_u32_be(&mut self, val: u32) -> Result<()> { write_u32_be(&mut self.cursor, val) }
    pub fn put_i32_be(&mut self, val: i32) -> Result<()> { write_i32_be(&mut self.cursor, val) }
    pub fn put_u32_le(&mut self, val: u32) -> Result<()> { write_u32_le(&mut self.cursor, val) }
    pub fn put_i32_le(&mut self, val: i32) -> Result<()> { write_i32_le(&mut self.cursor, val) }
    pub fn put_f32_be(&mut self, val: f32) -> Result<()> { write_f32_be(&mut self.cursor, val) }
    pub fn put_f32_le(&mut self, val: f32) -> Result<()> { write_f32_le(&mut self.cursor, val) }
    pub fn put_f64_be(&mut self, val: f64) -> Result<()> { write_f64_be(&mut self.cursor, val) }
    pub fn put_f64_le(&mut self, val: f64) -> Result<()> { write_f64_le(&mut self.cursor, val) }
    pub fn put_u64_be(&mut self, val: u64) -> Result<()> { write_u64_be(&mut self.cursor, val) }
    pub fn put_i64_be(&mut self, val: i64) -> Result<()> { write_i64_be(&mut self.cursor, val) }
    pub fn put_u64_le(&mut self, val: u64) -> Result<()> { write_u64_le(&mut self.cursor, val) }
    pub fn put_i64_le(&mut self, val: i64) -> Result<()> { write_i64_le(&mut self.cursor, val) }
    pub fn put_var_u32(&mut self, val: u32) -> Result<()> { write_var_u32(&mut self.cursor, val) }
    pub fn put_var_i32(&mut self, val: i32) -> Result<()> { write_var_i32(&mut self.cursor, val) }
    pub fn put_var_u64(&mut self, val: u64) -> Result<()> { write_var_u64(&mut self.cursor, val) }
    pub fn put_var_i64(&mut self, val: i64) -> Result<()> { write_var_i64(&mut self.cursor, val) }
    pub fn put_string(&mut self, val: &str) -> Result<()> { write_string(&mut self.cursor, val) }

    // --- PMMP Aliases ---
    #[inline] pub fn get_byte(&mut self) -> Result<u8> { self.get_u8() }
    #[inline] pub fn get_signed_byte(&mut self) -> Result<i8> { self.get_i8() }
    #[inline] pub fn get_short(&mut self) -> Result<u16> { self.get_u16_be() }
    #[inline] pub fn get_signed_short(&mut self) -> Result<i16> { self.get_i16_be() }
    #[inline] pub fn get_lshort(&mut self) -> Result<u16> { self.get_u16_le() }
    #[inline] pub fn get_signed_lshort(&mut self) -> Result<i16> { self.get_i16_le() }
    #[inline] pub fn get_triad(&mut self) -> Result<u32> { self.get_u24_be() }
    #[inline] pub fn get_ltriad(&mut self) -> Result<u32> { self.get_u24_le() }
    #[inline] pub fn get_int(&mut self) -> Result<i32> { self.get_i32_be() }
    #[inline] pub fn get_lint(&mut self) -> Result<i32> { self.get_i32_le() }
    #[inline] pub fn get_float(&mut self) -> Result<f32> { self.get_f32_be() }
    #[inline] pub fn get_lfloat(&mut self) -> Result<f32> { self.get_f32_le() }
    #[inline] pub fn get_double(&mut self) -> Result<f64> { self.get_f64_be() }
    #[inline] pub fn get_ldouble(&mut self) -> Result<f64> { self.get_f64_le() }
    #[inline] pub fn get_long(&mut self) -> Result<i64> { self.get_i64_be() }
    #[inline] pub fn get_llong(&mut self) -> Result<i64> { self.get_i64_le() }
    #[inline] pub fn get_unsigned_var_int(&mut self) -> Result<u32> { self.get_var_u32() }
    #[inline] pub fn get_var_int(&mut self) -> Result<i32> { self.get_var_i32() }
    #[inline] pub fn get_unsigned_var_long(&mut self) -> Result<u64> { self.get_var_u64() }
    #[inline] pub fn get_var_long(&mut self) -> Result<i64> { self.get_var_i64() }

    #[inline] pub fn put_byte(&mut self, val: u8) -> Result<()> { self.put_u8(val) }
    #[inline] pub fn put_short(&mut self, val: u16) -> Result<()> { self.put_u16_be(val) }
    #[inline] pub fn put_lshort(&mut self, val: u16) -> Result<()> { self.put_u16_le(val) }
    #[inline] pub fn put_triad(&mut self, val: u32) -> Result<()> { self.put_u24_be(val) }
    #[inline] pub fn put_ltriad(&mut self, val: u32) -> Result<()> { self.put_u24_le(val) }
    #[inline] pub fn put_int(&mut self, val: i32) -> Result<()> { self.put_i32_be(val) }
    #[inline] pub fn put_lint(&mut self, val: i32) -> Result<()> { self.put_i32_le(val) }
    #[inline] pub fn put_float(&mut self, val: f32) -> Result<()> { self.put_f32_be(val) }
    #[inline] pub fn put_lfloat(&mut self, val: f32) -> Result<()> { self.put_f32_le(val) }
    #[inline] pub fn put_double(&mut self, val: f64) -> Result<()> { self.put_f64_be(val) }
    #[inline] pub fn put_ldouble(&mut self, val: f64) -> Result<()> { self.put_f64_le(val) }
    #[inline] pub fn put_long(&mut self, val: i64) -> Result<()> { self.put_i64_be(val) }
    #[inline] pub fn put_llong(&mut self, val: i64) -> Result<()> { self.put_i64_le(val) }
    #[inline] pub fn put_unsigned_var_int(&mut self, val: u32) -> Result<()> { self.put_var_u32(val) }
    #[inline] pub fn put_var_int(&mut self, val: i32) -> Result<()> { self.put_var_i32(val) }
    #[inline] pub fn put_unsigned_var_long(&mut self, val: u64) -> Result<()> { self.put_var_u64(val) }
    #[inline] pub fn put_var_long(&mut self, val: i64) -> Result<()> { self.put_var_i64(val) }
}


// --- BinaryStreamReader (for reading from slices without mutation) ---
/// A read-only wrapper around a byte slice using `Cursor`.
#[derive(Debug)]
pub struct BinaryStreamReader<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> BinaryStreamReader<'a> {
    pub fn get_offset(&self) -> u64 { self.cursor.position() }
    pub fn set_offset(&mut self, pos: u64) { self.cursor.set_position(pos); }
    pub fn rewind(&mut self) { self.set_offset(0); }
    pub fn len(&self) -> usize { self.cursor.get_ref().len() }
    pub fn is_empty(&self) -> bool { self.cursor.get_ref().is_empty() }
    pub fn feof(&self) -> bool { self.cursor.position() >= (self.len() as u64) }

    /// Reads exactly `len` bytes from the current offset. Returns a slice.
    pub fn get(&mut self, len: usize) -> Result<&'a [u8]> {
        let current_pos = self.get_offset() as usize;
        let buffer_len = self.len();
        let end_pos = current_pos.saturating_add(len);
        if end_pos > buffer_len {
            return Err(BinaryUtilError::NotEnoughData { needed: len, have: buffer_len.saturating_sub(current_pos) });
        }
        // Get the slice directly from the cursor's inner slice
        let result_slice = &self.cursor.get_ref()[current_pos..end_pos];
        // Advance the cursor position
        self.cursor.seek(SeekFrom::Current(len as i64))?;
        Ok(result_slice)
    }

    /// Reads all bytes from the current offset to the end of the buffer. Returns a slice.
    pub fn get_remaining(&mut self) -> Result<&'a [u8]> {
        let current_pos = self.get_offset() as usize;
        let buffer_len = self.len();
        if current_pos >= buffer_len {
            return Err(BinaryUtilError::NotEnoughData { needed: 1, have: 0 });
        }
        let result_slice = &self.cursor.get_ref()[current_pos..];
        // Advance cursor to end
        self.cursor.seek(SeekFrom::End(0))?;
        Ok(result_slice)
    }

    // --- Read Methods (delegating to standalone functions) ---
    pub fn get_bool(&mut self) -> Result<bool> { read_bool(&mut self.cursor) }
    pub fn get_u8(&mut self) -> Result<u8> { read_u8(&mut self.cursor) }
    pub fn get_i8(&mut self) -> Result<i8> { read_i8(&mut self.cursor) }
    pub fn get_u16_be(&mut self) -> Result<u16> { read_u16_be(&mut self.cursor) }
    pub fn get_i16_be(&mut self) -> Result<i16> { read_i16_be(&mut self.cursor) }
    pub fn get_u16_le(&mut self) -> Result<u16> { read_u16_le(&mut self.cursor) }
    pub fn get_i16_le(&mut self) -> Result<i16> { read_i16_le(&mut self.cursor) }
    pub fn get_u24_be(&mut self) -> Result<u32> { read_u24_be(&mut self.cursor) }
    pub fn get_u24_le(&mut self) -> Result<u32> { read_u24_le(&mut self.cursor) }
    pub fn get_u32_be(&mut self) -> Result<u32> { read_u32_be(&mut self.cursor) }
    pub fn get_i32_be(&mut self) -> Result<i32> { read_i32_be(&mut self.cursor) }
    pub fn get_u32_le(&mut self) -> Result<u32> { read_u32_le(&mut self.cursor) }
    pub fn get_i32_le(&mut self) -> Result<i32> { read_i32_le(&mut self.cursor) }
    pub fn get_f32_be(&mut self) -> Result<f32> { read_f32_be(&mut self.cursor) }
    pub fn get_f32_le(&mut self) -> Result<f32> { read_f32_le(&mut self.cursor) }
    pub fn get_f64_be(&mut self) -> Result<f64> { read_f64_be(&mut self.cursor) }
    pub fn get_f64_le(&mut self) -> Result<f64> { read_f64_le(&mut self.cursor) }
    pub fn get_u64_be(&mut self) -> Result<u64> { read_u64_be(&mut self.cursor) }
    pub fn get_i64_be(&mut self) -> Result<i64> { read_i64_be(&mut self.cursor) }
    pub fn get_u64_le(&mut self) -> Result<u64> { read_u64_le(&mut self.cursor) }
    pub fn get_i64_le(&mut self) -> Result<i64> { read_i64_le(&mut self.cursor) }
    pub fn get_var_u32(&mut self) -> Result<u32> { read_var_u32(&mut self.cursor) }
    pub fn get_var_i32(&mut self) -> Result<i32> { read_var_i32(&mut self.cursor) }
    pub fn get_var_u64(&mut self) -> Result<u64> { read_var_u64(&mut self.cursor) }
    pub fn get_var_i64(&mut self) -> Result<i64> { read_var_i64(&mut self.cursor) }
    pub fn get_string(&mut self) -> Result<String> { read_string(&mut self.cursor) }
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    // ... (tests remain the same) ...
    #[test]
    fn test_bool() {
        let mut stream = BinaryStream::new();
        stream.put_bool(true).unwrap(); stream.put_bool(false).unwrap();
        stream.rewind();
        assert_eq!(stream.get_bool().unwrap(), true); assert_eq!(stream.get_bool().unwrap(), false);
        assert!(stream.feof());
    }
    #[test]
    fn test_bytes() {
        let mut stream = BinaryStream::new();
        stream.put_u8(0x12).unwrap(); stream.put_i8(-5).unwrap();
        stream.rewind();
        assert_eq!(stream.get_u8().unwrap(), 0x12); assert_eq!(stream.get_i8().unwrap(), -5);
    }
    #[test] fn test_shorts_be() { /* ... */ }
    #[test] fn test_shorts_le() { /* ... */ }
    #[test] fn test_triads() { /* ... */ }
    #[test] fn test_var_u32() { /* ... */ }
    #[test] fn test_var_i32() { /* ... */ }
    #[test] fn test_var_u64() { /* ... */ }
    #[test] fn test_var_i64() { /* ... */ }
    #[test] fn test_string() { /* ... */ }
    #[test] fn test_get_remaining() { /* ... */ }
    #[test] fn test_get_not_enough_data() { /* ... */ }
}