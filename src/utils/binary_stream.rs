// src/utils/binary_stream.rs
#![allow(dead_code)]

use crate::utils::binary;
use crate::utils::error::{BinaryDataException, Result};
use std::convert::TryInto;

#[derive(Debug, Clone, Default)]
pub struct BinaryStream {
    buffer: Vec<u8>,
    offset: usize,
}

impl BinaryStream {
    pub fn new() -> Self {
        Self { buffer: Vec::new(), offset: 0 }
    }

    pub fn with_buffer(buffer: Vec<u8>, offset: usize) -> Self {
        Self { buffer, offset }
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Self { buffer: slice.to_vec(), offset: 0 }
    }

    pub fn rewind(&mut self) {
        self.offset = 0;
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }

    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn get_mut_buffer(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }

    #[inline]
    fn ensure_available(&self, len: usize) -> Result<()> {
        if len == 0 {
            return Ok(());
        }
        if self.offset.checked_add(len).is_none() || self.offset + len > self.buffer.len() {
            let remaining = self.buffer.len().saturating_sub(self.offset);
            Err(BinaryDataException::new(format!(
                "Not enough bytes left in buffer: need {}, have {}",
                len, remaining
            )))
        } else {
            Ok(())
        }
    }

    pub fn get(&mut self, len: usize) -> Result<&[u8]> {
        self.ensure_available(len)?;
        let start = self.offset;
        self.offset += len;
        Ok(&self.buffer[start..self.offset])
    }

    pub fn get_remaining(&mut self) -> Result<&[u8]> {
        if self.offset >= self.buffer.len() {
            // Return empty slice instead of erroring if already at end
            if self.offset == self.buffer.len() {
                return Ok(&self.buffer[self.offset..]);
            }
            Err(BinaryDataException::from_str("No bytes left to read"))
        } else {
            let start = self.offset;
            self.offset = self.buffer.len();
            Ok(&self.buffer[start..])
        }
    }

    pub fn put(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    pub fn get_bool(&mut self) -> Result<bool> {
        let byte = self.get_byte()?;
        Ok(byte != 0x00)
    }

    pub fn put_bool(&mut self, v: bool) {
        self.put_byte(if v { 0x01 } else { 0x00 });
    }

    pub fn get_byte(&mut self) -> Result<u8> {
        let bytes = self.get(1)?;
        Ok(bytes[0])
    }

    pub fn get_signed_byte(&mut self) -> Result<i8> {
        Ok(self.get_byte()? as i8)
    }

    pub fn put_byte(&mut self, v: u8) {
        self.put(&[v]);
    }

    pub fn get_short(&mut self) -> Result<u16> {
        let bytes = self.get(binary::SIZEOF_SHORT)?;
        binary::read_short(bytes)
    }

    pub fn get_signed_short(&mut self) -> Result<i16> {
        let bytes = self.get(binary::SIZEOF_SHORT)?;
        binary::read_signed_short(bytes)
    }

    pub fn put_short(&mut self, v: u16) -> Result<()> {
        let bytes = binary::write_short(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn put_signed_short(&mut self, v: i16) -> Result<()> {
        let bytes = binary::write_signed_short(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn get_lshort(&mut self) -> Result<u16> {
        let bytes = self.get(binary::SIZEOF_SHORT)?;
        binary::read_lshort(bytes)
    }

    pub fn get_signed_lshort(&mut self) -> Result<i16> {
        let bytes = self.get(binary::SIZEOF_SHORT)?;
        binary::read_signed_lshort(bytes)
    }

    pub fn put_lshort(&mut self, v: u16) -> Result<()> {
        let bytes = binary::write_lshort(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn put_signed_lshort(&mut self, v: i16) -> Result<()> {
        let bytes = binary::write_signed_lshort(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn get_triad(&mut self) -> Result<u32> {
        let bytes = self.get(3)?;
        binary::read_triad(bytes)
    }

    pub fn put_triad(&mut self, v: u32) -> Result<()> {
        let bytes = binary::write_triad(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn get_ltriad(&mut self) -> Result<u32> {
        let bytes = self.get(3)?;
        binary::read_ltriad(bytes)
    }

    pub fn put_ltriad(&mut self, v: u32) -> Result<()> {
        let bytes = binary::write_ltriad(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn get_int(&mut self) -> Result<i32> {
        let bytes = self.get(binary::SIZEOF_INT)?;
        binary::read_int(bytes)
    }

    pub fn get_unsigned_int(&mut self) -> Result<u32> {
        let bytes = self.get(binary::SIZEOF_INT)?;
        binary::read_unsigned_int(bytes)
    }

    pub fn put_int(&mut self, v: i32) -> Result<()> {
        let bytes = binary::write_int(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn put_unsigned_int(&mut self, v: u32) -> Result<()> {
        let bytes = binary::write_unsigned_int(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn get_lint(&mut self) -> Result<i32> {
        let bytes = self.get(binary::SIZEOF_INT)?;
        binary::read_lint(bytes)
    }

    pub fn get_unsigned_lint(&mut self) -> Result<u32> {
        let bytes = self.get(binary::SIZEOF_INT)?;
        binary::read_unsigned_lint(bytes)
    }

    pub fn put_lint(&mut self, v: i32) -> Result<()> {
        let bytes = binary::write_lint(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn put_unsigned_lint(&mut self, v: u32) -> Result<()> {
        let bytes = binary::write_unsigned_lint(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn get_float(&mut self) -> Result<f32> {
        let bytes = self.get(binary::SIZEOF_FLOAT)?;
        binary::read_float(bytes)
    }

    pub fn put_float(&mut self, v: f32) -> Result<()> {
        let bytes = binary::write_float(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn get_lfloat(&mut self) -> Result<f32> {
        let bytes = self.get(binary::SIZEOF_FLOAT)?;
        binary::read_lfloat(bytes)
    }

    pub fn put_lfloat(&mut self, v: f32) -> Result<()> {
        let bytes = binary::write_lfloat(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn get_double(&mut self) -> Result<f64> {
        let bytes = self.get(binary::SIZEOF_DOUBLE)?;
        binary::read_double(bytes)
    }

    pub fn put_double(&mut self, v: f64) -> Result<()> {
        let bytes = binary::write_double(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn get_ldouble(&mut self) -> Result<f64> {
        let bytes = self.get(binary::SIZEOF_DOUBLE)?;
        binary::read_ldouble(bytes)
    }

    pub fn put_ldouble(&mut self, v: f64) -> Result<()> {
        let bytes = binary::write_ldouble(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn get_long(&mut self) -> Result<i64> {
        let bytes = self.get(binary::SIZEOF_LONG)?;
        binary::read_long(bytes)
    }

    pub fn get_unsigned_long(&mut self) -> Result<u64> {
        let bytes = self.get(binary::SIZEOF_LONG)?;
        binary::read_unsigned_long(bytes)
    }

    pub fn put_long(&mut self, v: i64) -> Result<()> {
        let bytes = binary::write_long(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn put_unsigned_long(&mut self, v: u64) -> Result<()> {
        let bytes = binary::write_unsigned_long(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn get_llong(&mut self) -> Result<i64> {
        let bytes = self.get(binary::SIZEOF_LONG)?;
        binary::read_llong(bytes)
    }

    pub fn get_unsigned_llong(&mut self) -> Result<u64> {
        let bytes = self.get(binary::SIZEOF_LONG)?;
        binary::read_unsigned_llong(bytes)
    }

    pub fn put_llong(&mut self, v: i64) -> Result<()> {
        let bytes = binary::write_llong(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn put_unsigned_llong(&mut self, v: u64) -> Result<()> {
        let bytes = binary::write_unsigned_llong(v)?;
        self.put(&bytes);
        Ok(())
    }

    pub fn get_unsigned_var_int(&mut self) -> Result<u32> {
        self.ensure_available(1)?;
        let mut temp_offset = self.offset;
        let result = binary::read_unsigned_var_int(&self.buffer, &mut temp_offset);
        if result.is_ok() {
            self.offset = temp_offset;
        }
        result
    }

    pub fn put_unsigned_var_int(&mut self, v: u32) {
        let bytes = binary::write_unsigned_var_int(v);
        self.put(&bytes);
    }

    pub fn get_var_int(&mut self) -> Result<i32> {
        self.ensure_available(1)?;
        let mut temp_offset = self.offset;
        let result = binary::read_var_int(&self.buffer, &mut temp_offset);
        if result.is_ok() {
            self.offset = temp_offset;
        }
        result
    }

    pub fn put_var_int(&mut self, v: i32) {
        let bytes = binary::write_var_int(v);
        self.put(&bytes);
    }

    pub fn get_unsigned_var_long(&mut self) -> Result<u64> {
        self.ensure_available(1)?;
        let mut temp_offset = self.offset;
        let result = binary::read_unsigned_var_long(&self.buffer, &mut temp_offset);
        if result.is_ok() {
            self.offset = temp_offset;
        }
        result
    }

    pub fn put_unsigned_var_long(&mut self, v: u64) {
        let bytes = binary::write_unsigned_var_long(v);
        self.put(&bytes);
    }

    pub fn get_var_long(&mut self) -> Result<i64> {
        self.ensure_available(1)?;
        let mut temp_offset = self.offset;
        let result = binary::read_var_long(&self.buffer, &mut temp_offset);
        if result.is_ok() {
            self.offset = temp_offset;
        }
        result
    }

    pub fn put_var_long(&mut self, v: i64) {
        let bytes = binary::write_var_long(v);
        self.put(&bytes);
    }

    pub fn feof(&self) -> bool {
        self.offset >= self.buffer.len()
    }

    pub fn read_string(&mut self) -> Result<String> {
        let len = self.get_unsigned_var_int()? as usize;
        let bytes = self.get(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|e| BinaryDataException::new(e.to_string()))
    }

    pub fn write_string(&mut self, v: &str) {
        let bytes = v.as_bytes();
        self.put_unsigned_var_int(bytes.len().try_into().unwrap_or(u32::MAX));
        self.put(bytes);
    }
}