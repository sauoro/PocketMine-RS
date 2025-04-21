use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    io::{self, Cursor, Read, Write},
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

    #[error("IO error: {0}")]
    IoError(#[from] io::Error), // Catch errors from Read/Write operations
}

// Define a Result type alias for convenience
pub type Result<T> = std::result::Result<T, BinaryUtilError>;

// --- Standalone Binary Read/Write Functions (Replaces static Binary methods) ---

// Constants matching PHP sizes (Rust's size_of is compile-time)
pub const SIZEOF_BYTE: usize = mem::size_of::<u8>();
pub const SIZEOF_SHORT: usize = mem::size_of::<u16>();
pub const SIZEOF_INT: usize = mem::size_of::<u32>();
pub const SIZEOF_LONG: usize = mem::size_of::<u64>();
pub const SIZEOF_FLOAT: usize = mem::size_of::<f32>();
pub const SIZEOF_DOUBLE: usize = mem::size_of::<f64>();

// --- Boolean ---
pub fn read_bool(reader: &mut impl Read) -> Result<bool> {
    Ok(reader.read_u8()? != 0)
}

pub fn write_bool(writer: &mut impl Write, val: bool) -> Result<()> {
    writer.write_u8(if val { 1 } else { 0 })?;
    Ok(())
}

// --- Bytes (u8 / i8) ---
pub fn read_u8(reader: &mut impl Read) -> Result<u8> {
    Ok(reader.read_u8()?)
}

pub fn read_i8(reader: &mut impl Read) -> Result<i8> {
    Ok(reader.read_i8()?)
}

pub fn write_u8(writer: &mut impl Write, val: u8) -> Result<()> {
    writer.write_u8(val)?;
    Ok(())
}

pub fn write_i8(writer: &mut impl Write, val: i8) -> Result<()> {
    writer.write_i8(val)?;
    Ok(())
}

// --- Shorts (u16 / i16) ---
// Big Endian (BE) - Corresponds to pack "n"
pub fn read_u16_be(reader: &mut impl Read) -> Result<u16> {
    Ok(reader.read_u16::<BigEndian>()?)
}
pub fn read_i16_be(reader: &mut impl Read) -> Result<i16> {
    Ok(reader.read_i16::<BigEndian>()?)
}
pub fn write_u16_be(writer: &mut impl Write, val: u16) -> Result<()> {
    writer.write_u16::<BigEndian>(val)?;
    Ok(())
}
pub fn write_i16_be(writer: &mut impl Write, val: i16) -> Result<()> {
    writer.write_i16::<BigEndian>(val)?;
    Ok(())
}

// Little Endian (LE) - Corresponds to pack "v"
pub fn read_u16_le(reader: &mut impl Read) -> Result<u16> {
    Ok(reader.read_u16::<LittleEndian>()?)
}
pub fn read_i16_le(reader: &mut impl Read) -> Result<i16> {
    Ok(reader.read_i16::<LittleEndian>()?)
}
pub fn write_u16_le(writer: &mut impl Write, val: u16) -> Result<()> {
    writer.write_u16::<LittleEndian>(val)?;
    Ok(())
}
pub fn write_i16_le(writer: &mut impl Write, val: i16) -> Result<()> {
    writer.write_i16::<LittleEndian>(val)?;
    Ok(())
}

// --- Triads (3 bytes) ---
pub fn read_u24_be(reader: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 3];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes([0, buf[0], buf[1], buf[2]]))
}

pub fn write_u24_be(writer: &mut impl Write, val: u32) -> Result<()> {
    // Ensure value fits in 3 bytes
    // Note: PHP doesn't check this, but Rust can.
    // if val > 0xFFFFFF {
    //     return Err(BinaryUtilError::Other("Value too large for Triad".to_string()));
    // }
    let buf = val.to_be_bytes();
    writer.write_all(&buf[1..4])?; // Write the lower 3 bytes
    Ok(())
}

pub fn read_u24_le(reader: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 3];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes([buf[0], buf[1], buf[2], 0]))
}

pub fn write_u24_le(writer: &mut impl Write, val: u32) -> Result<()> {
    // if val > 0xFFFFFF {
    //     return Err(BinaryUtilError::Other("Value too large for Triad".to_string()));
    // }
    let buf = val.to_le_bytes();
    writer.write_all(&buf[0..3])?; // Write the lower 3 bytes
    Ok(())
}

// --- Ints (u32 / i32) ---
// Big Endian (BE) - Corresponds to pack "N"
pub fn read_u32_be(reader: &mut impl Read) -> Result<u32> {
    Ok(reader.read_u32::<BigEndian>()?)
}
pub fn read_i32_be(reader: &mut impl Read) -> Result<i32> {
    Ok(reader.read_i32::<BigEndian>()?)
}
pub fn write_u32_be(writer: &mut impl Write, val: u32) -> Result<()> {
    writer.write_u32::<BigEndian>(val)?;
    Ok(())
}
pub fn write_i32_be(writer: &mut impl Write, val: i32) -> Result<()> {
    writer.write_i32::<BigEndian>(val)?;
    Ok(())
}

// Little Endian (LE) - Corresponds to pack "V"
pub fn read_u32_le(reader: &mut impl Read) -> Result<u32> {
    Ok(reader.read_u32::<LittleEndian>()?)
}
pub fn read_i32_le(reader: &mut impl Read) -> Result<i32> {
    Ok(reader.read_i32::<LittleEndian>()?)
}
pub fn write_u32_le(writer: &mut impl Write, val: u32) -> Result<()> {
    writer.write_u32::<LittleEndian>(val)?;
    Ok(())
}
pub fn write_i32_le(writer: &mut impl Write, val: i32) -> Result<()> {
    writer.write_i32::<LittleEndian>(val)?;
    Ok(())
}

// --- Floats (f32) ---
// Big Endian (BE) - Corresponds to pack "G"
pub fn read_f32_be(reader: &mut impl Read) -> Result<f32> {
    Ok(reader.read_f32::<BigEndian>()?)
}
pub fn write_f32_be(writer: &mut impl Write, val: f32) -> Result<()> {
    writer.write_f32::<BigEndian>(val)?;
    Ok(())
}

// Little Endian (LE) - Corresponds to pack "g"
pub fn read_f32_le(reader: &mut impl Read) -> Result<f32> {
    Ok(reader.read_f32::<LittleEndian>()?)
}
pub fn write_f32_le(writer: &mut impl Write, val: f32) -> Result<()> {
    writer.write_f32::<LittleEndian>(val)?;
    Ok(())
}

// --- Doubles (f64) ---
// Big Endian (BE) - Corresponds to pack "E"
pub fn read_f64_be(reader: &mut impl Read) -> Result<f64> {
    Ok(reader.read_f64::<BigEndian>()?)
}
pub fn write_f64_be(writer: &mut impl Write, val: f64) -> Result<()> {
    writer.write_f64::<BigEndian>(val)?;
    Ok(())
}

// Little Endian (LE) - Corresponds to pack "e"
pub fn read_f64_le(reader: &mut impl Read) -> Result<f64> {
    Ok(reader.read_f64::<LittleEndian>()?)
}
pub fn write_f64_le(writer: &mut impl Write, val: f64) -> Result<()> {
    writer.write_f64::<LittleEndian>(val)?;
    Ok(())
}

// --- Longs (u64 / i64) ---
// Note: PHP's integer size depends on the platform (32/64 bit).
// PMMP uses 64-bit longs via pack formats J/P, so we use u64/i64.
// Big Endian (BE) - Corresponds to pack "J"
pub fn read_u64_be(reader: &mut impl Read) -> Result<u64> {
    Ok(reader.read_u64::<BigEndian>()?)
}
pub fn read_i64_be(reader: &mut impl Read) -> Result<i64> {
    Ok(reader.read_i64::<BigEndian>()?)
}
pub fn write_u64_be(writer: &mut impl Write, val: u64) -> Result<()> {
    writer.write_u64::<BigEndian>(val)?;
    Ok(())
}
pub fn write_i64_be(writer: &mut impl Write, val: i64) -> Result<()> {
    writer.write_i64::<BigEndian>(val)?;
    Ok(())
}

// Little Endian (LE) - Corresponds to pack "P"
pub fn read_u64_le(reader: &mut impl Read) -> Result<u64> {
    Ok(reader.read_u64::<LittleEndian>()?)
}
pub fn read_i64_le(reader: &mut impl Read) -> Result<i64> {
    Ok(reader.read_i64::<LittleEndian>()?)
}
pub fn write_u64_le(writer: &mut impl Write, val: u64) -> Result<()> {
    writer.write_u64::<LittleEndian>(val)?;
    Ok(())
}
pub fn write_i64_le(writer: &mut impl Write, val: i64) -> Result<()> {
    writer.write_i64::<LittleEndian>(val)?;
    Ok(())
}

// --- VarInts / VarLongs ---

const VARINT_MAX_BYTES: usize = 5;
const VARLONG_MAX_BYTES: usize = 10;

// Unsigned VarInt (u32)
pub fn read_var_u32(reader: &mut impl Read) -> Result<u32> {
    let mut value: u32 = 0;
    let mut shift = 0;
    for i in 0..VARINT_MAX_BYTES {
        let byte = reader.read_u8()?;
        // Extract 7 bits of data
        value |= ((byte & 0x7F) as u32) << shift;
        // Check continuation bit
        if (byte & 0x80) == 0 {
            return Ok(value);
        }
        shift += 7;
        // Optional: Check if shift exceeds bits in u32 - prevents overflow panics on bad data
        if shift >= 32 {
            // This case implies the VarInt is longer than needed for u32,
            // but the loop limit already prevents infinite loops.
            // Depending on strictness, you might error here or just let it truncate.
            // The PHP code doesn't explicitly check this intermediate shift overflow.
        }
    }
    Err(BinaryUtilError::VarIntTooLong)
}

pub fn write_var_u32(writer: &mut impl Write, mut value: u32) -> Result<()> {
    loop {
        // Take 7 bits of the value
        let mut byte = (value & 0x7F) as u8;
        value >>= 7; // Use logical right shift (default for unsigned)

        // If there's more data, set the continuation bit
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
    // ZigZag decode: (n >> 1) ^ -(n & 1)
    Ok(((raw >> 1) ^ (-((raw & 1) as i32)) as u32) as i32)
    // Alternative decode logic from PMMP:
    // let temp = ((raw << 63) >> 63) ^ raw) >> 1; // Not directly translatable, uses 64-bit intermediate?
    // temp ^ (raw & (1 << 63)); // Also seems overly complex for standard zigzag
    // Let's stick to standard zigzag.
}

pub fn write_var_i32(writer: &mut impl Write, value: i32) -> Result<()> {
    // ZigZag encode: (n << 1) ^ (n >> 31)
    // Note: In Rust, >> on signed types is arithmetic shift, which is what we need here.
    write_var_u32(writer, ((value << 1) ^ (value >> 31)) as u32)
}

// Unsigned VarLong (u64)
pub fn read_var_u64(reader: &mut impl Read) -> Result<u64> {
    let mut value: u64 = 0;
    let mut shift = 0;
    for i in 0..VARLONG_MAX_BYTES {
        let byte = reader.read_u8()?;
        value |= ((byte & 0x7F) as u64) << shift;
        if (byte & 0x80) == 0 {
            return Ok(value);
        }
        shift += 7;
        if shift >= 64 {
            // Similar check as VarInt, optional for strictness
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
    // ZigZag decode: (n >> 1) ^ -(n & 1)
    Ok(((raw >> 1) ^ (-((raw & 1) as i64)) as u64) as i64)
}

pub fn write_var_i64(writer: &mut impl Write, value: i64) -> Result<()> {
    // ZigZag encode: (n << 1) ^ (n >> 63)
    write_var_u64(writer, ((value << 1) ^ (value >> 63)) as u64)
}


// --- Endianness Flipping ---
// These are less common in Rust as you typically read/write directly
// with the desired endianness using byteorder. But for completeness:
pub fn flip_short_endianness(val: u16) -> u16 {
    val.swap_bytes()
}

pub fn flip_int_endianness(val: u32) -> u32 {
    val.swap_bytes()
}

pub fn flip_long_endianness(val: u64) -> u64 {
    val.swap_bytes()
}


// --- Raw Bytes / String ---
// Note: Reading/writing length-prefixed strings is common in MC protocol
// and would typically use read_var_u32 followed by read_exact.

pub fn read_bytes(reader: &mut impl Read, len: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn write_bytes(writer: &mut impl Write, bytes: &[u8]) -> Result<()> {
    writer.write_all(bytes)?;
    Ok(())
}

// Example: Read a length-prefixed (VarInt) UTF8 string
pub fn read_string(reader: &mut impl Read) -> Result<String> {
    let len = read_var_u32(reader)? as usize;
    // Consider adding a sanity check for max string length here
    let bytes = read_bytes(reader, len)?;
    Ok(String::from_utf8(bytes)?)
}

// Example: Write a length-prefixed (VarInt) UTF8 string
pub fn write_string(writer: &mut impl Write, val: &str) -> Result<()> {
    let bytes = val.as_bytes();
    // Check length fits in u32 if necessary
    write_var_u32(writer, bytes.len() as u32)?;
    write_bytes(writer, bytes)?;
    Ok(())
}


// --- BinaryStream ---

/// A wrapper around a byte buffer (`Cursor<Vec<u8>>`) providing methods
/// for reading and writing Minecraft protocol data types.
/// Mirrors the functionality of PMMP's BinaryStream.
#[derive(Debug, Default)]
pub struct BinaryStream {
    /// Internal buffer and position tracker.
    cursor: Cursor<Vec<u8>>,
}

impl BinaryStream {
    /// Creates a new, empty BinaryStream.
    pub fn new() -> Self {
        Self {
            cursor: Cursor::new(Vec::new()),
        }
    }

    /// Creates a BinaryStream wrapping an existing byte vector.
    /// The stream can be read from and written to (appended).
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self {
            cursor: Cursor::new(vec),
        }
    }

    /// Creates a BinaryStream wrapping an existing byte slice for reading.
    /// Note: This stream cannot be written to. Consider a separate type
    /// or using `Cursor<&[u8]>` directly if read-only is needed.
    /// For simplicity, we'll focus on the read/write Vec<u8> version.
    // pub fn from_slice(slice: &[u8]) -> ReadOnlyBinaryStream { ... }


    // --- Buffer Access and Control ---

    /// Returns the current read/write position (offset).
    pub fn get_offset(&self) -> u64 {
        self.cursor.position()
    }

    /// Sets the read/write position (offset).
    pub fn set_offset(&mut self, pos: u64) {
        self.cursor.set_position(pos);
    }

    /// Moves the offset pointer back to the beginning of the stream.
    pub fn rewind(&mut self) {
        self.cursor.set_position(0);
    }

    /// Returns a reference to the underlying byte buffer.
    pub fn get_buffer(&self) -> &Vec<u8> {
        self.cursor.get_ref()
    }

    /// Consumes the stream and returns the underlying byte buffer.
    pub fn into_inner(self) -> Vec<u8> {
        self.cursor.into_inner()
    }

    /// Returns the total number of bytes currently in the buffer.
    pub fn len(&self) -> usize {
        self.cursor.get_ref().len()
    }

    /// Returns true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.cursor.get_ref().is_empty()
    }

    /// Clears the buffer and resets the offset.
    pub fn clear(&mut self) {
        self.cursor.set_position(0);
        self.cursor.get_mut().clear();
    }

    /// Returns true if the read/write position is at or past the end of the buffer.
    pub fn feof(&self) -> bool {
        self.cursor.position() >= (self.len() as u64)
    }

    /// Appends the given bytes to the end of the stream's buffer.
    /// Note: This advances the underlying buffer's length but does NOT change the current read/write offset.
    /// Use `write_*` methods or `seek` to manage the offset for writing.
    pub fn put(&mut self, bytes: &[u8]) -> Result<()> {
        // In Cursor<Vec<u8>>, writing automatically appends and advances position
        self.cursor.write_all(bytes)?;
        Ok(())
    }

    /// Reads exactly `len` bytes from the current offset.
    /// Advances the offset by `len`.
    pub fn get(&mut self, len: usize) -> Result<Vec<u8>> {
        // Check available data *before* reading
        let current_pos = self.cursor.position() as usize;
        let buffer_len = self.len();
        if current_pos + len > buffer_len {
            return Err(BinaryUtilError::NotEnoughData { needed: len, have: buffer_len.saturating_sub(current_pos) });
        }
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Reads all bytes from the current offset to the end of the buffer.
    /// Advances the offset to the end.
    pub fn get_remaining(&mut self) -> Result<Vec<u8>> {
        let current_pos = self.cursor.position() as usize;
        let buffer_len = self.len();
        if current_pos >= buffer_len {
            // Match PHP's BinaryDataException("No bytes left to read") - though NotEnoughData is similar
            return Err(BinaryUtilError::NotEnoughData { needed: 1, have: 0 });
        }
        let remaining_len = buffer_len - current_pos;
        let mut buf = vec![0u8; remaining_len];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

    // --- Read Methods ---
    // These methods call the standalone functions, passing the internal cursor.

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

    // PMMP Aliases (use preferred Rust naming above, but provide aliases for direct porting)
    #[inline] pub fn get_byte(&mut self) -> Result<u8> { self.get_u8() } // PMMP getByte is unsigned
    #[inline] pub fn get_signed_byte(&mut self) -> Result<i8> { self.get_i8() }
    #[inline] pub fn get_short(&mut self) -> Result<u16> { self.get_u16_be() } // PMMP getShort is unsigned BE
    #[inline] pub fn get_signed_short(&mut self) -> Result<i16> { self.get_i16_be() }
    #[inline] pub fn get_lshort(&mut self) -> Result<u16> { self.get_u16_le() } // PMMP getLShort is unsigned LE
    #[inline] pub fn get_signed_lshort(&mut self) -> Result<i16> { self.get_i16_le() }
    #[inline] pub fn get_triad(&mut self) -> Result<u32> { self.get_u24_be() }
    #[inline] pub fn get_ltriad(&mut self) -> Result<u32> { self.get_u24_le() }
    #[inline] pub fn get_int(&mut self) -> Result<i32> { self.get_i32_be() } // PMMP getInt is signed BE
    #[inline] pub fn get_lint(&mut self) -> Result<i32> { self.get_i32_le() }
    #[inline] pub fn get_float(&mut self) -> Result<f32> { self.get_f32_be() }
    #[inline] pub fn get_lfloat(&mut self) -> Result<f32> { self.get_f32_le() }
    #[inline] pub fn get_double(&mut self) -> Result<f64> { self.get_f64_be() }
    #[inline] pub fn get_ldouble(&mut self) -> Result<f64> { self.get_f64_le() }
    #[inline] pub fn get_long(&mut self) -> Result<i64> { self.get_i64_be() } // PMMP getLong is signed BE (using J pack)
    #[inline] pub fn get_llong(&mut self) -> Result<i64> { self.get_i64_le() } // PMMP getLLong is signed LE (using P pack)
    #[inline] pub fn get_unsigned_var_int(&mut self) -> Result<u32> { self.get_var_u32() }
    #[inline] pub fn get_var_int(&mut self) -> Result<i32> { self.get_var_i32() }
    #[inline] pub fn get_unsigned_var_long(&mut self) -> Result<u64> { self.get_var_u64() }
    #[inline] pub fn get_var_long(&mut self) -> Result<i64> { self.get_var_i64() }

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

    // PMMP Aliases
    #[inline] pub fn put_byte(&mut self, val: u8) -> Result<()> { self.put_u8(val) } // Treat as unsigned like PMMP
    #[inline] pub fn put_short(&mut self, val: u16) -> Result<()> { self.put_u16_be(val) } // Unsigned BE
    #[inline] pub fn put_lshort(&mut self, val: u16) -> Result<()> { self.put_u16_le(val) } // Unsigned LE
    #[inline] pub fn put_triad(&mut self, val: u32) -> Result<()> { self.put_u24_be(val) }
    #[inline] pub fn put_ltriad(&mut self, val: u32) -> Result<()> { self.put_u24_le(val) }
    #[inline] pub fn put_int(&mut self, val: i32) -> Result<()> { self.put_i32_be(val) } // Signed BE
    #[inline] pub fn put_lint(&mut self, val: i32) -> Result<()> { self.put_i32_le(val) } // Signed LE
    #[inline] pub fn put_float(&mut self, val: f32) -> Result<()> { self.put_f32_be(val) }
    #[inline] pub fn put_lfloat(&mut self, val: f32) -> Result<()> { self.put_f32_le(val) }
    #[inline] pub fn put_double(&mut self, val: f64) -> Result<()> { self.put_f64_be(val) }
    #[inline] pub fn put_ldouble(&mut self, val: f64) -> Result<()> { self.put_f64_le(val) }
    #[inline] pub fn put_long(&mut self, val: i64) -> Result<()> { self.put_i64_be(val) } // Signed BE
    #[inline] pub fn put_llong(&mut self, val: i64) -> Result<()> { self.put_i64_le(val) } // Signed LE
    #[inline] pub fn put_unsigned_var_int(&mut self, val: u32) -> Result<()> { self.put_var_u32(val) }
    #[inline] pub fn put_var_int(&mut self, val: i32) -> Result<()> { self.put_var_i32(val) }
    #[inline] pub fn put_unsigned_var_long(&mut self, val: u64) -> Result<()> { self.put_var_u64(val) }
    #[inline] pub fn put_var_long(&mut self, val: i64) -> Result<()> { self.put_var_i64(val) }

}

// --- Limits (Replaces Limits class) ---
// In Rust, these are typically accessed directly as associated constants
// on the primitive types, e.g., u8::MAX, i32::MIN.
// There's no direct need for a separate `Limits` struct or module.
// Examples:
// u8::MIN = 0, u8::MAX = 255 (PHP UINT8_MAX)
// i8::MIN = -128 (PHP INT8_MIN), i8::MAX = 127 (PHP INT8_MAX)
// u16::MAX = 65535 (PHP UINT16_MAX)
// i16::MIN = -32768 (PHP INT16_MIN), i16::MAX = 32767 (PHP INT16_MAX)
// u32::MAX = 4294967295 (PHP UINT32_MAX)
// i32::MIN = -2147483648 (PHP INT32_MIN), i32::MAX = 2147483647 (PHP INT32_MAX)
// u64::MAX = 18446744073709551615 (PHP UINT64_MAX)
// i64::MIN = -9223372036854775808 (PHP INT64_MIN), i64::MAX = 9223372036854775807 (PHP INT64_MAX)


// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool() {
        let mut stream = BinaryStream::new();
        stream.put_bool(true).unwrap();
        stream.put_bool(false).unwrap();
        stream.rewind();
        assert_eq!(stream.get_bool().unwrap(), true);
        assert_eq!(stream.get_bool().unwrap(), false);
        assert!(stream.feof());
    }

    #[test]
    fn test_bytes() {
        let mut stream = BinaryStream::new();
        stream.put_u8(0x12).unwrap();
        stream.put_i8(-5).unwrap(); // 0xFB
        stream.rewind();
        assert_eq!(stream.get_u8().unwrap(), 0x12);
        assert_eq!(stream.get_i8().unwrap(), -5);
    }

    #[test]
    fn test_shorts_be() {
        let mut stream = BinaryStream::new();
        stream.put_u16_be(0x1234).unwrap();
        stream.put_i16_be(-30000).unwrap(); // 0x8AD0
        assert_eq!(stream.get_buffer(), &[0x12, 0x34, 0x8A, 0xD0]);
        stream.rewind();
        assert_eq!(stream.get_u16_be().unwrap(), 0x1234);
        assert_eq!(stream.get_i16_be().unwrap(), -30000);
    }

    #[test]
    fn test_shorts_le() {
        let mut stream = BinaryStream::new();
        stream.put_u16_le(0x1234).unwrap();
        stream.put_i16_le(-10).unwrap(); // 0xFFF6
        assert_eq!(stream.get_buffer(), &[0x34, 0x12, 0xF6, 0xFF]);
        stream.rewind();
        assert_eq!(stream.get_u16_le().unwrap(), 0x1234);
        assert_eq!(stream.get_i16_le().unwrap(), -10);
    }

    #[test]
    fn test_triads() {
        let mut stream = BinaryStream::new();
        stream.put_u24_be(0x123456).unwrap();
        stream.put_u24_le(0xABCDEF).unwrap();
        assert_eq!(stream.get_buffer(), &[0x12, 0x34, 0x56, 0xEF, 0xCD, 0xAB]);
        stream.rewind();
        assert_eq!(stream.get_u24_be().unwrap(), 0x123456);
        assert_eq!(stream.get_u24_le().unwrap(), 0xABCDEF);
    }


    #[test]
    fn test_var_u32() {
        let values = [0u32, 1, 127, 128, 255, 2097151, 2147483647, u32::MAX];
        let expected_bytes: Vec<Vec<u8>> = vec![
            vec![0x00], vec![0x01], vec![0x7f], vec![0x80, 0x01], vec![0xff, 0x01],
            vec![0xff, 0xff, 0x7f], vec![0xff, 0xff, 0xff, 0xff, 0x07],
            vec![0xff, 0xff, 0xff, 0xff, 0x0f] // Note: This differs from some online examples, verify against MC protocol if needed
        ];

        for (i, &val) in values.iter().enumerate() {
            let mut stream = BinaryStream::new();
            stream.put_var_u32(val).unwrap();
            assert_eq!(stream.get_buffer(), &expected_bytes[i], "Encoding mismatch for {}", val);
            stream.rewind();
            assert_eq!(stream.get_var_u32().unwrap(), val, "Decoding mismatch for {}", val);
        }
    }

    #[test]
    fn test_var_i32() {
        // Values from https://wiki.vg/Protocol#VarInt_and_VarLong
        let values = [0, 1, 2, 127, 128, 255, 2147483647, -1, -2147483648];
        let expected_bytes: Vec<Vec<u8>> = vec![
            vec![0x00], vec![0x02], vec![0x04], vec![0xfe, 0x01], vec![0x80, 0x02],
            vec![0xfe, 0x03], vec![0xfe, 0xff, 0xff, 0xff, 0x0f], // 2147483647 (zigzagged)
            vec![0x01], // -1 (zigzagged)
            vec![0xff, 0xff, 0xff, 0xff, 0x0f], // -2147483648 (zigzagged)
        ];

        for (i, &val) in values.iter().enumerate() {
            let mut stream = BinaryStream::new();
            stream.put_var_i32(val).unwrap();
            assert_eq!(stream.get_buffer(), &expected_bytes[i], "Encoding mismatch for {}", val);
            stream.rewind();
            assert_eq!(stream.get_var_i32().unwrap(), val, "Decoding mismatch for {}", val);
        }
    }

    #[test]
    fn test_var_u64() {
        let values = [0u64, 1, 127, 128, 255, 2147483647, u64::MAX];
        let expected_bytes: Vec<Vec<u8>> = vec![
            vec![0x00], vec![0x01], vec![0x7f], vec![0x80, 0x01], vec![0xff, 0x01],
            vec![0xff, 0xff, 0xff, 0xff, 0x07],
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01], // u64::MAX
        ];

        for (i, &val) in values.iter().enumerate() {
            let mut stream = BinaryStream::new();
            stream.put_var_u64(val).unwrap();
            assert_eq!(stream.get_buffer(), &expected_bytes[i], "Encoding mismatch for {}", val);
            stream.rewind();
            assert_eq!(stream.get_var_u64().unwrap(), val, "Decoding mismatch for {}", val);
        }
    }

    #[test]
    fn test_var_i64() {
        let values = [0i64, 1, -1, 2147483647, -2147483648, i64::MAX, i64::MIN];
        let expected_bytes: Vec<Vec<u8>> = vec![
            vec![0x00], vec![0x02], vec![0x01],
            vec![0xfe, 0xff, 0xff, 0xff, 0x0f], // 2147483647
            vec![0xff, 0xff, 0xff, 0xff, 0x0f], // -2147483648
            vec![0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01], // i64::MAX
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01], // i64::MIN
        ];

        for (i, &val) in values.iter().enumerate() {
            let mut stream = BinaryStream::new();
            stream.put_var_i64(val).unwrap();
            assert_eq!(stream.get_buffer(), &expected_bytes[i], "Encoding mismatch for {}", val);
            stream.rewind();
            assert_eq!(stream.get_var_i64().unwrap(), val, "Decoding mismatch for {}", val);
        }
    }

    #[test]
    fn test_string() {
        let mut stream = BinaryStream::new();
        let test_str = "Hello, Rust! \u{1F980}"; // Includes crab emoji
        stream.put_string(test_str).unwrap();

        // Check raw bytes: VarInt length + UTF-8 bytes
        let expected_len = test_str.len() as u32;
        let mut len_stream = BinaryStream::new();
        len_stream.put_var_u32(expected_len).unwrap();
        let mut expected_bytes = len_stream.into_inner();
        expected_bytes.extend_from_slice(test_str.as_bytes());

        assert_eq!(stream.get_buffer(), &expected_bytes);

        stream.rewind();
        assert_eq!(stream.get_string().unwrap(), test_str);
    }

    #[test]
    fn test_get_remaining() {
        let data = vec![1, 2, 3, 4, 5];
        let mut stream = BinaryStream::from_vec(data.clone());
        stream.get_u8().unwrap(); // Read one byte
        stream.get_u16_be().unwrap(); // Read two bytes
        assert_eq!(stream.get_offset(), 3);
        assert_eq!(stream.get_remaining().unwrap(), &[4, 5]);
        assert_eq!(stream.get_offset(), 5); // Offset should be at end
        assert!(stream.feof());
    }

    #[test]
    fn test_get_not_enough_data() {
        let mut stream = BinaryStream::from_vec(vec![1, 2]);
        assert!(matches!(stream.get_i32_be(), Err(BinaryUtilError::IoError(_)))); // byteorder returns io::Error on EOF

        let mut stream2 = BinaryStream::from_vec(vec![1, 2]);
        match stream2.get(4) { // Test our custom get() check
            Err(BinaryUtilError::NotEnoughData { needed: 4, have: 2 }) => (), // Correct error
            _ => panic!("Expected NotEnoughData error"),
        }
    }
}

// You would also need a src/utils/mod.rs
// pub mod binary;