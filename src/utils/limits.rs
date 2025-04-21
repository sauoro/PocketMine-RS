// src/utils/limits.rs

#![allow(dead_code)]

pub const UINT8_MAX: u8 = 0xff;
pub const INT8_MIN: i8 = -0x7f - 1;
pub const INT8_MAX: i8 = 0x7f;

pub const UINT16_MAX: u16 = 0xffff;
pub const INT16_MIN: i16 = -0x7fff - 1;
pub const INT16_MAX: i16 = 0x7fff;

pub const UINT32_MAX: u32 = 0xffffffff;
pub const INT32_MIN: i32 = -0x7fffffff - 1;
pub const INT32_MAX: i32 = 0x7fffffff;

pub const UINT64_MAX: u64 = 0xffffffffffffffff;
pub const INT64_MIN: i64 = -0x7fffffffffffffff - 1;
pub const INT64_MAX: i64 = 0x7fffffffffffffff;