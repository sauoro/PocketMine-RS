// src/utils/binary.rs
#![allow(dead_code)]

use crate::utils::error::{BinaryDataException, Result};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor}; // Removed unused Write

pub const SIZEOF_SHORT: usize = 2;
pub const SIZEOF_INT: usize = 4;
pub const SIZEOF_LONG: usize = 8;
pub const SIZEOF_FLOAT: usize = 4;
pub const SIZEOF_DOUBLE: usize = 8;

#[inline]
fn check_length(bytes: &[u8], needed: usize) -> Result<()> {
    if bytes.len() < needed {
        Err(BinaryDataException::new(format!(
            "Not enough bytes: need {}, have {}",
            needed,
            bytes.len()
        )))
    } else {
        Ok(())
    }
}

#[inline]
pub fn sign_byte(value: i64) -> i8 {
    value as i8
}

#[inline]
pub fn unsign_byte(value: i64) -> u8 {
    value as u8
}

#[inline]
pub fn sign_short(value: i64) -> i16 {
    value as i16
}

#[inline]
pub fn unsign_short(value: i64) -> u16 {
    value as u16
}

#[inline]
pub fn sign_int(value: i64) -> i32 {
    value as i32
}

#[inline]
pub fn unsign_int(value: i64) -> u32 {
    value as u32
}

#[inline]
pub fn flip_short_endianness(value: u16) -> u16 {
    value.swap_bytes()
}

#[inline]
pub fn flip_int_endianness(value: u32) -> u32 {
    value.swap_bytes()
}

#[inline]
pub fn flip_long_endianness(value: u64) -> u64 {
    value.swap_bytes()
}

pub fn read_bool(b: &[u8]) -> Result<bool> {
    check_length(b, 1)?;
    Ok(b[0] != 0x00)
}

pub fn write_bool(b: bool) -> Vec<u8> {
    vec![if b { 0x01 } else { 0x00 }]
}

pub fn read_byte(c: &[u8]) -> Result<u8> {
    check_length(c, 1)?;
    Ok(c[0])
}

pub fn read_signed_byte(c: &[u8]) -> Result<i8> {
    check_length(c, 1)?;
    Ok(c[0] as i8)
}

pub fn write_byte(c: u8) -> Vec<u8> {
    vec![c]
}

pub fn read_short(str: &[u8]) -> Result<u16> {
    check_length(str, SIZEOF_SHORT)?;
    let mut cursor = Cursor::new(str);
    cursor.read_u16::<BigEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn read_signed_short(str: &[u8]) -> Result<i16> {
    check_length(str, SIZEOF_SHORT)?;
    let mut cursor = Cursor::new(str);
    cursor.read_i16::<BigEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn write_short(value: u16) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_SHORT);
    buf.write_u16::<BigEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn write_signed_short(value: i16) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_SHORT);
    buf.write_i16::<BigEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn read_lshort(str: &[u8]) -> Result<u16> {
    check_length(str, SIZEOF_SHORT)?;
    let mut cursor = Cursor::new(str);
    cursor.read_u16::<LittleEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn read_signed_lshort(str: &[u8]) -> Result<i16> {
    check_length(str, SIZEOF_SHORT)?;
    let mut cursor = Cursor::new(str);
    cursor.read_i16::<LittleEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn write_lshort(value: u16) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_SHORT);
    buf.write_u16::<LittleEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn write_signed_lshort(value: i16) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_SHORT);
    buf.write_i16::<LittleEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn read_triad(str: &[u8]) -> Result<u32> {
    check_length(str, 3)?;
    Ok(((str[0] as u32) << 16) | ((str[1] as u32) << 8) | (str[2] as u32))
}

pub fn write_triad(value: u32) -> Result<Vec<u8>> {
    if value > 0xFFFFFF {
        return Err(BinaryDataException::from_str("Value too large for Triad"));
    }
    let mut buf = Vec::with_capacity(3);
    buf.push((value >> 16) as u8);
    buf.push((value >> 8) as u8);
    buf.push(value as u8);
    Ok(buf)
}

pub fn read_ltriad(str: &[u8]) -> Result<u32> {
    check_length(str, 3)?;
    Ok((str[0] as u32) | ((str[1] as u32) << 8) | ((str[2] as u32) << 16))
}

pub fn write_ltriad(value: u32) -> Result<Vec<u8>> {
    if value > 0xFFFFFF {
        return Err(BinaryDataException::from_str("Value too large for LTriad"));
    }
    let mut buf = Vec::with_capacity(3);
    buf.push(value as u8);
    buf.push((value >> 8) as u8);
    buf.push((value >> 16) as u8);
    Ok(buf)
}

pub fn read_int(str: &[u8]) -> Result<i32> {
    check_length(str, SIZEOF_INT)?;
    let mut cursor = Cursor::new(str);
    cursor.read_i32::<BigEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn read_unsigned_int(str: &[u8]) -> Result<u32> {
    check_length(str, SIZEOF_INT)?;
    let mut cursor = Cursor::new(str);
    cursor.read_u32::<BigEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn write_int(value: i32) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_INT);
    buf.write_i32::<BigEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn write_unsigned_int(value: u32) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_INT);
    buf.write_u32::<BigEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn read_lint(str: &[u8]) -> Result<i32> {
    check_length(str, SIZEOF_INT)?;
    let mut cursor = Cursor::new(str);
    cursor.read_i32::<LittleEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn read_unsigned_lint(str: &[u8]) -> Result<u32> {
    check_length(str, SIZEOF_INT)?;
    let mut cursor = Cursor::new(str);
    cursor.read_u32::<LittleEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn write_lint(value: i32) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_INT);
    buf.write_i32::<LittleEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn write_unsigned_lint(value: u32) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_INT);
    buf.write_u32::<LittleEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn read_float(str: &[u8]) -> Result<f32> {
    check_length(str, SIZEOF_FLOAT)?;
    let mut cursor = Cursor::new(str);
    cursor.read_f32::<BigEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn write_float(value: f32) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_FLOAT);
    buf.write_f32::<BigEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn read_lfloat(str: &[u8]) -> Result<f32> {
    check_length(str, SIZEOF_FLOAT)?;
    let mut cursor = Cursor::new(str);
    cursor.read_f32::<LittleEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn write_lfloat(value: f32) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_FLOAT);
    buf.write_f32::<LittleEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn read_double(str: &[u8]) -> Result<f64> {
    check_length(str, SIZEOF_DOUBLE)?;
    let mut cursor = Cursor::new(str);
    cursor.read_f64::<BigEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn write_double(value: f64) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_DOUBLE);
    buf.write_f64::<BigEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn read_ldouble(str: &[u8]) -> Result<f64> {
    check_length(str, SIZEOF_DOUBLE)?;
    let mut cursor = Cursor::new(str);
    cursor.read_f64::<LittleEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn write_ldouble(value: f64) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_DOUBLE);
    buf.write_f64::<LittleEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn read_long(str: &[u8]) -> Result<i64> {
    check_length(str, SIZEOF_LONG)?;
    let mut cursor = Cursor::new(str);
    cursor.read_i64::<BigEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn read_unsigned_long(str: &[u8]) -> Result<u64> {
    check_length(str, SIZEOF_LONG)?;
    let mut cursor = Cursor::new(str);
    cursor.read_u64::<BigEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn write_long(value: i64) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_LONG);
    buf.write_i64::<BigEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn write_unsigned_long(value: u64) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_LONG);
    buf.write_u64::<BigEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn read_llong(str: &[u8]) -> Result<i64> {
    check_length(str, SIZEOF_LONG)?;
    let mut cursor = Cursor::new(str);
    cursor.read_i64::<LittleEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn read_unsigned_llong(str: &[u8]) -> Result<u64> {
    check_length(str, SIZEOF_LONG)?;
    let mut cursor = Cursor::new(str);
    cursor.read_u64::<LittleEndian>().map_err(|e| BinaryDataException::new(e.to_string()))
}

pub fn write_llong(value: i64) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_LONG);
    buf.write_i64::<LittleEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn write_unsigned_llong(value: u64) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(SIZEOF_LONG);
    buf.write_u64::<LittleEndian>(value).map_err(|e| BinaryDataException::new(e.to_string()))?;
    Ok(buf)
}

pub fn read_unsigned_var_int(buffer: &[u8], offset: &mut usize) -> Result<u32> {
    let mut value: u32 = 0;
    let initial_offset = *offset;
    for i in 0..5 {
        if *offset >= buffer.len() {
            *offset = initial_offset;
            return Err(BinaryDataException::from_str("No bytes left in buffer"));
        }
        let byte = buffer[*offset];
        *offset += 1;
        value |= ((byte & 0x7F) as u32) << (i * 7);
        if (byte & 0x80) == 0 {
            return Ok(value);
        }
    }
    *offset = initial_offset;
    Err(BinaryDataException::from_str("VarInt did not terminate after 5 bytes!"))
}

pub fn write_unsigned_var_int(mut value: u32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(5);
    loop {
        if (value & !0x7F) == 0 {
            buf.push(value as u8);
            return buf;
        }
        buf.push(((value & 0x7F) | 0x80) as u8);
        value >>= 7;
        if buf.len() >= 5 {
            // In PM, this would panic. Consider returning Err instead.
            panic!("Value {} too large to be encoded as a VarInt", value);
        }
    }
}

pub fn read_var_int(buffer: &[u8], offset: &mut usize) -> Result<i32> {
    let raw = read_unsigned_var_int(buffer, offset)?;
    let temp = (raw >> 1) ^ (-((raw & 1) as i32)) as u32;
    Ok(temp as i32)
}

pub fn write_var_int(value: i32) -> Vec<u8> {
    write_unsigned_var_int(((value << 1) ^ (value >> 31)) as u32)
}

pub fn read_unsigned_var_long(buffer: &[u8], offset: &mut usize) -> Result<u64> {
    let mut value: u64 = 0;
    let initial_offset = *offset;
    for i in 0..10 {
        if *offset >= buffer.len() {
            *offset = initial_offset;
            return Err(BinaryDataException::from_str("No bytes left in buffer"));
        }
        let byte = buffer[*offset];
        *offset += 1;
        value |= ((byte & 0x7F) as u64) << (i * 7);
        if (byte & 0x80) == 0 {
            return Ok(value);
        }
    }
    *offset = initial_offset;
    Err(BinaryDataException::from_str("VarLong did not terminate after 10 bytes!"))
}

pub fn write_unsigned_var_long(mut value: u64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(10);
    loop {
        if (value & !0x7F) == 0 {
            buf.push(value as u8);
            return buf;
        }
        buf.push(((value & 0x7F) | 0x80) as u8);
        value >>= 7;
        if buf.len() >= 10 {
            // In PM, this would panic. Consider returning Err instead.
            panic!("Value {} too large to be encoded as a VarLong", value);
        }
    }
}

pub fn read_var_long(buffer: &[u8], offset: &mut usize) -> Result<i64> {
    let raw = read_unsigned_var_long(buffer, offset)?;
    let temp = (raw >> 1) ^ (-((raw & 1) as i64)) as u64;
    Ok(temp as i64)
}

pub fn write_var_long(value: i64) -> Vec<u8> {
    write_unsigned_var_long(((value << 1) ^ (value >> 63)) as u64)
}