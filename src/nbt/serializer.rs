// src/nbt/serializer.rs
#![allow(dead_code)]

use crate::utils::BinaryStream;
use crate::nbt::error::Result;

// No NbtTag import needed here

// Equivalent to NbtStreamReader in PHP
pub trait NbtRead {
    fn read_byte(&mut self) -> Result<u8>;
    fn read_signed_byte(&mut self) -> Result<i8>;
    fn read_short(&mut self) -> Result<i16>;
    fn read_signed_short(&mut self) -> Result<i16>;
    fn read_int(&mut self) -> Result<i32>;
    fn read_long(&mut self) -> Result<i64>;
    fn read_float(&mut self) -> Result<f32>;
    fn read_double(&mut self) -> Result<f64>;
    fn read_byte_array(&mut self) -> Result<Vec<u8>>;
    fn read_string(&mut self) -> Result<String>;
    fn read_int_array(&mut self) -> Result<Vec<i32>>;
}

// Equivalent to NbtStreamWriter in PHP
pub trait NbtWrite {
    fn write_byte(&mut self, v: u8) -> Result<()>;
    fn write_signed_byte(&mut self, v: i8) -> Result<()>;
    fn write_short(&mut self, v: i16) -> Result<()>;
    fn write_int(&mut self, v: i32) -> Result<()>;
    fn write_long(&mut self, v: i64) -> Result<()>;
    fn write_float(&mut self, v: f32) -> Result<()>;
    fn write_double(&mut self, v: f64) -> Result<()>;
    fn write_byte_array(&mut self, v: &[u8]) -> Result<()>;
    fn write_string(&mut self, v: &str) -> Result<()>;
    fn write_int_array(&mut self, v: &[i32]) -> Result<()>;
}

// Traits combining reader/writer with the underlying stream access
pub trait NbtReader: NbtRead {
    fn stream(&self) -> &BinaryStream;
    fn stream_mut(&mut self) -> &mut BinaryStream;
}

pub trait NbtWriter: NbtWrite {
    fn stream(&self) -> &BinaryStream;
    fn stream_mut(&mut self) -> &mut BinaryStream;
}