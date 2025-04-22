#![allow(dead_code)]

use crate::log::level::LogLevel;
use crate::log::logger::Logger;
use std::error::Error;
use std::fmt; // Import fmt

// #[derive(Debug)] // Remove this line
pub struct PrefixedLogger {
    delegate: Box<dyn Logger>,
    prefix: String,
}

// Manual implementation of Debug
impl fmt::Debug for PrefixedLogger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrefixedLogger")
            .field("prefix", &self.prefix)
            // We can't easily debug the concrete type behind the Box<dyn Logger>
            // So we just indicate its presence and type.
            .field("delegate", &format_args!("Box<dyn Logger>"))
            .finish()
    }
}


impl PrefixedLogger {
    pub fn new(delegate: Box<dyn Logger>, prefix: String) -> Self {
        Self { delegate, prefix }
    }

    pub fn get_prefix(&self) -> &str {
        &self.prefix
    }

    pub fn set_prefix(&mut self, prefix: String) {
        self.prefix = prefix;
    }
}

impl Logger for PrefixedLogger {
    fn log(&self, level: LogLevel, message: &str) {
        let prefixed_message = format!("[{}] {}", self.prefix, message);
        self.delegate.log(level, &prefixed_message);
    }

    fn log_exception(&self, e: &(dyn Error + Send + Sync + 'static)) {
        // Prefixing doesn't usually apply to the structured exception log
        self.delegate.log_exception(e);
    }
}