#![allow(clippy::too_many_arguments, clippy::enum_variant_names)]
// src/main.rs
mod utils;
mod color;
mod math;
mod nbt;
mod log;
mod raknet;

use log::Logger;

fn main() {
    let logger = Box::new(log::SimpleLogger::new());
    logger.info("PocketMine-RS starting...");
}