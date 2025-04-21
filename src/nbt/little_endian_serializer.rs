// src/nbt/little_endian_serializer.rs
#![allow(dead_code)]

use crate::utils::{BinaryStream, limits};
use crate::nbt::error::{NbtError, Result};
use crate::nbt::serializer::{NbtRead, NbtWrite, NbtReader, NbtWriter};
use crate::nbt::tag::{self, Tag, TagType};
use crate::nbt::reader_tracker::ReaderTracker;
use crate::nbt::tree_root::TreeRoot;
use std::convert::TryInto;


pub struct LittleEndianNbtSerializer {
    stream: BinaryStream,
}

impl LittleEndianNbtSerializer {
    pub fn new(stream: BinaryStream) -> Self {
        Self { stream }
    }

    pub fn from_bytes(buffer: &[u8]) -> Self {
        Self::new(BinaryStream::from_slice(buffer))
    }

    // --- Root Read/Write Logic ---
    fn read_root(&mut self, max_depth: usize) -> Result<TreeRoot> {
        let type_id = self.read_byte()?;
        if type_id == TagType::End as u8 {
            return Err(NbtError::new_data_error("Found TAG_End at the start of buffer"));
        }
        let tag_type = TagType::from_id(type_id)
            .ok_or_else(|| NbtError::new_data_error(&format!("Invalid root tag type ID: {}", type_id)))?;

        let root_name = self.read_string()?;
        let mut tracker = ReaderTracker::new(max_depth);
        let root_tag = tag::create_tag(tag_type, self, &mut tracker)?;
        TreeRoot::new(root_name, root_tag)
    }

    fn write_root(&mut self, root: &TreeRoot) -> Result<()> {
        self.write_byte(root.get_tag().get_type() as u8)?;
        self.write_string(root.get_name())?;
        root.get_tag().write(self)
    }

    // --- Public API ---
    pub fn read(&mut self, max_depth: usize) -> Result<TreeRoot> {
        self.stream.rewind();
        self.read_root(max_depth)
    }

    pub fn read_from_buffer(buffer: &[u8], max_depth: usize) -> Result<TreeRoot> {
        let mut serializer = Self::from_bytes(buffer);
        serializer.read_root(max_depth)
    }

    pub fn write(&mut self, data: &TreeRoot) -> Result<()> {
        self.stream = BinaryStream::new();
        self.write_root(data)?;
        Ok(())
    }

    pub fn write_to_bytes(data: &TreeRoot) -> Result<Vec<u8>> {
        let mut serializer = Self::new(BinaryStream::new());
        serializer.write(data)?;
        Ok(serializer.stream.get_buffer().to_vec())
    }

    pub fn read_headless(&mut self, root_type_id: u8, max_depth: usize) -> Result<Box<dyn Tag>> {
        let root_type = TagType::from_id(root_type_id)
            .ok_or_else(|| NbtError::new_data_error(&format!("Invalid headless root tag type ID: {}", root_type_id)))?;
        if root_type == TagType::End {
            return Err(NbtError::new_data_error("Cannot read headless TAG_End"));
        }
        let mut tracker = ReaderTracker::new(max_depth);
        tag::create_tag(root_type, self, &mut tracker)
    }

    pub fn read_headless_from_buffer(buffer: &[u8], root_type_id: u8, max_depth: usize) -> Result<Box<dyn Tag>> {
        let mut serializer = Self::from_bytes(buffer);
        serializer.read_headless(root_type_id, max_depth)
    }

    pub fn write_headless(&mut self, data: &dyn Tag) -> Result<()> {
        self.stream = BinaryStream::new();
        data.write(self)
    }

    pub fn write_headless_to_bytes(data: &dyn Tag) -> Result<Vec<u8>> {
        let mut serializer = Self::new(BinaryStream::new());
        serializer.write_headless(data)?;
        Ok(serializer.stream.get_buffer().to_vec())
    }

    pub fn read_multiple(&mut self, max_depth: usize) -> Result<Vec<TreeRoot>> {
        let mut results = Vec::new();
        while !self.stream.feof() {
            let current_offset = self.stream.get_offset();
            match self.read_root(max_depth) {
                Ok(root) => results.push(root),
                Err(NbtError::IoError(e)) => { // Match on the error variant directly
                    if self.stream.get_offset() == current_offset && e.to_string().contains("Not enough bytes") {
                        break; // Clean EOF suspected
                    }
                    else {
                        return Err(NbtError::IoError(e)); // Propagate other IO errors or partial reads
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(results)
    }

    pub fn read_multiple_from_buffer(buffer: &[u8], max_depth: usize) -> Result<Vec<TreeRoot>> {
        let mut serializer = Self::from_bytes(buffer);
        serializer.read_multiple(max_depth)
    }

    pub fn write_multiple(&mut self, data: &[TreeRoot]) -> Result<()> {
        self.stream = BinaryStream::new();
        for root in data {
            self.write_root(root)?;
        }
        Ok(())
    }

    pub fn write_multiple_to_bytes(data: &[TreeRoot]) -> Result<Vec<u8>> {
        let mut serializer = Self::new(BinaryStream::new());
        serializer.write_multiple(data)?;
        Ok(serializer.stream.get_buffer().to_vec())
    }


    pub fn get_buffer(&self) -> &[u8] {
        self.stream.get_buffer()
    }

    // --- String Length Checks ---
    fn check_read_string_length(len: i16) -> Result<usize> {
        if len < 0 {
            Err(NbtError::new_data_error(&format!("NBT string length cannot be negative ({})", len)))
        } else {
            Ok(len as usize)
        }
    }

    fn check_write_string_length(len: usize) -> Result<i16> {
        if len > limits::I16_MAX as usize {
            Err(NbtError::new_invalid_tag_value(&format!("NBT string length too large ({} > {})", len, limits::I16_MAX)))
        } else {
            Ok(len.try_into()?)
        }
    }
}

impl NbtRead for LittleEndianNbtSerializer {
    fn read_byte(&mut self) -> Result<u8> { Ok(self.stream.get_byte()?) }
    fn read_signed_byte(&mut self) -> Result<i8> { Ok(self.stream.get_signed_byte()?) }
    fn read_short(&mut self) -> Result<i16> { Ok(self.stream.get_signed_lshort()?) }
    fn read_signed_short(&mut self) -> Result<i16> { Ok(self.stream.get_signed_lshort()?) }
    fn read_int(&mut self) -> Result<i32> { Ok(self.stream.get_lint()?) }
    fn read_long(&mut self) -> Result<i64> { Ok(self.stream.get_llong()?) }
    fn read_float(&mut self) -> Result<f32> { Ok(self.stream.get_lfloat()?) }
    fn read_double(&mut self) -> Result<f64> { Ok(self.stream.get_ldouble()?) }

    fn read_byte_array(&mut self) -> Result<Vec<u8>> {
        let length = self.read_int()?;
        if length < 0 {
            return Err(NbtError::new_data_error(&format!("ByteArray length cannot be less than zero ({})", length)));
        }
        let usize_length: usize = length.try_into().map_err(|_| NbtError::new_data_error("ByteArray length too large"))?;
        Ok(self.stream.get(usize_length)?.to_vec())
    }

    fn read_string(&mut self) -> Result<String> {
        let length = self.read_short()?;
        let usize_length = Self::check_read_string_length(length)?;
        let bytes = self.stream.get(usize_length)?;
        String::from_utf8(bytes.to_vec()).map_err(NbtError::from)
    }

    fn read_int_array(&mut self) -> Result<Vec<i32>> {
        let length = self.read_int()?;
        if length < 0 {
            return Err(NbtError::new_data_error(&format!("IntArray length cannot be less than zero ({})", length)));
        }
        let usize_length: usize = length.try_into().map_err(|_| NbtError::new_data_error("IntArray length too large"))?;
        let mut result = Vec::with_capacity(usize_length);
        for _ in 0..usize_length {
            result.push(self.read_int()?);
        }
        Ok(result)
    }
}

impl NbtWrite for LittleEndianNbtSerializer {
    fn write_byte(&mut self, v: u8) -> Result<()> { Ok(self.stream.put_byte(v)) }
    fn write_signed_byte(&mut self, v: i8) -> Result<()> { Ok(self.stream.put_byte(v as u8)) }
    fn write_short(&mut self, v: i16) -> Result<()> { Ok(self.stream.put_signed_lshort(v)?) }
    fn write_int(&mut self, v: i32) -> Result<()> { Ok(self.stream.put_lint(v)?) }
    fn write_long(&mut self, v: i64) -> Result<()> { Ok(self.stream.put_llong(v)?) }
    fn write_float(&mut self, v: f32) -> Result<()> { Ok(self.stream.put_lfloat(v)?) }
    fn write_double(&mut self, v: f64) -> Result<()> { Ok(self.stream.put_ldouble(v)?) }

    fn write_byte_array(&mut self, v: &[u8]) -> Result<()> {
        let len: i32 = v.len().try_into().map_err(|_| NbtError::new_invalid_tag_value("ByteArray length too large for i32"))?;
        self.write_int(len)?;
        Ok(self.stream.put(v))
    }

    fn write_string(&mut self, v: &str) -> Result<()> {
        let len = Self::check_write_string_length(v.len())?;
        self.write_short(len)?;
        Ok(self.stream.put(v.as_bytes()))
    }

    fn write_int_array(&mut self, v: &[i32]) -> Result<()> {
        let len: i32 = v.len().try_into().map_err(|_| NbtError::new_invalid_tag_value("IntArray length too large for i32"))?;
        self.write_int(len)?;
        for &val in v {
            self.write_int(val)?;
        }
        Ok(())
    }
}

impl NbtReader for LittleEndianNbtSerializer {
    fn stream(&self) -> &BinaryStream { &self.stream }
    fn stream_mut(&mut self) -> &mut BinaryStream { &mut self.stream }
}

impl NbtWriter for LittleEndianNbtSerializer {
    fn stream(&self) -> &BinaryStream { &self.stream }
    fn stream_mut(&mut self) -> &mut BinaryStream { &mut self.stream }
}