// src/log/buffered.rs
#![allow(dead_code)]

use crate::log::logger::Logger;

pub trait BufferedLogger: Logger {
    fn buffer(&self, buffered_fn: impl FnOnce());
}