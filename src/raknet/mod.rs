// src/raknet/mod.rs

pub mod utils;
pub mod server;
pub mod protocol;
pub mod generic;

pub const DEFAULT_PROTOCOL_VERSION: u8 = 6;
pub const SYSTEM_ADDRESS_COUNT: usize = 20;
