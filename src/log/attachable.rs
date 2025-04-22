// src/log/attachable.rs

#![allow(dead_code)]

use crate::log::level::LogLevel;
use crate::log::logger::Logger;

pub type LoggerAttachment = Box<dyn Fn(LogLevel, &str) + Send + Sync>;

pub trait AttachableLogger: Logger {
    fn add_attachment(&mut self, attachment: LoggerAttachment);

    // Removing exact closures is hard. Usually done via IDs or indices.
    // PHP likely relies on object identity which doesn't map directly.
    // fn remove_attachment(&mut self, attachment: LoggerAttachment);

    fn remove_attachments(&mut self);

    fn get_attachments(&self) -> &[LoggerAttachment];
}