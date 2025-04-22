// src/log/logger.rs

#![allow(dead_code)]

use crate::log::level::LogLevel;
use std::error::Error;

pub trait Logger: Send + Sync {
    fn emergency(&self, message: &str) {
        self.log(LogLevel::Emergency, message);
    }

    fn alert(&self, message: &str) {
        self.log(LogLevel::Alert, message);
    }

    fn critical(&self, message: &str) {
        self.log(LogLevel::Critical, message);
    }

    fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }

    fn warning(&self, message: &str) {
        self.log(LogLevel::Warning, message);
    }

    fn notice(&self, message: &str) {
        self.log(LogLevel::Notice, message);
    }

    fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    fn log(&self, level: LogLevel, message: &str);

    fn log_exception(&self, e: &(dyn Error + Send + Sync + 'static)) {
        let mut msg = format!("Error: {}", e);
        let mut current_source = e.source();
        while let Some(source) = current_source {
            msg.push_str(&format!("\nCaused by: {}", source));
            current_source = source.source();
        }
        self.critical(&msg);
        // Optionally log the backtrace if available/desired, might need 'backtrace' crate
        // For simplicity, we follow PHP's getTraceAsString roughly by just logging chained sources.
    }
}

// Allow Box<dyn Logger> to be used as a Logger
impl Logger for Box<dyn Logger> {
    fn emergency(&self, message: &str) {
        (**self).emergency(message)
    }
    fn alert(&self, message: &str) {
        (**self).alert(message)
    }
    fn critical(&self, message: &str) {
        (**self).critical(message)
    }
    fn error(&self, message: &str) {
        (**self).error(message)
    }
    fn warning(&self, message: &str) {
        (**self).warning(message)
    }
    fn notice(&self, message: &str) {
        (**self).notice(message)
    }
    fn info(&self, message: &str) {
        (**self).info(message)
    }
    fn debug(&self, message: &str) {
        (**self).debug(message)
    }
    fn log(&self, level: LogLevel, message: &str) {
        (**self).log(level, message)
    }
    fn log_exception(&self, e: &(dyn Error + Send + Sync + 'static)) {
        (**self).log_exception(e)
    }
}