// src/log/global.rs

#![allow(dead_code)]

use crate::log::level::LogLevel;
use crate::log::logger::Logger;
use crate::log::simple::SimpleLogger;
use once_cell::sync::Lazy;
use std::error::Error;
use std::sync::Mutex;

// Use Lazy<Mutex<Box<dyn Logger>>> to allow initialization and later replacement safely.
static GLOBAL_LOGGER: Lazy<Mutex<Box<dyn Logger>>> =
    Lazy::new(|| Mutex::new(Box::new(SimpleLogger::new())));

pub struct GlobalLogger;

impl GlobalLogger {
    // Provides direct logging methods acting on the global instance
    pub fn emergency(message: &str) {
        GLOBAL_LOGGER.lock().unwrap().emergency(message);
    }

    pub fn alert(message: &str) {
        GLOBAL_LOGGER.lock().unwrap().alert(message);
    }

    pub fn critical(message: &str) {
        GLOBAL_LOGGER.lock().unwrap().critical(message);
    }

    pub fn error(message: &str) {
        GLOBAL_LOGGER.lock().unwrap().error(message);
    }

    pub fn warning(message: &str) {
        GLOBAL_LOGGER.lock().unwrap().warning(message);
    }

    pub fn notice(message: &str) {
        GLOBAL_LOGGER.lock().unwrap().notice(message);
    }

    pub fn info(message: &str) {
        GLOBAL_LOGGER.lock().unwrap().info(message);
    }

    pub fn debug(message: &str) {
        GLOBAL_LOGGER.lock().unwrap().debug(message);
    }

    pub fn log(level: LogLevel, message: &str) {
        GLOBAL_LOGGER.lock().unwrap().log(level, message);
    }

    pub fn log_exception(e: &(dyn Error + Send + Sync + 'static)) {
        GLOBAL_LOGGER.lock().unwrap().log_exception(e);
    }

    /// Replaces the current global logger instance.
    pub fn set(logger: Box<dyn Logger>) {
        let mut guard = GLOBAL_LOGGER.lock().unwrap();
        *guard = logger;
    }

    /// Executes a closure with a mutable reference to the global logger.
    /// Be cautious with long-running operations holding the lock.
    pub fn with_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut (dyn Logger)) -> R,
    {
        let mut guard = GLOBAL_LOGGER.lock().unwrap();
        f(&mut **guard)
    }

    /// Executes a closure with an immutable reference to the global logger.
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&(dyn Logger)) -> R,
    {
        let guard = GLOBAL_LOGGER.lock().unwrap();
        f(&**guard)
    }

    // Note: Returning a direct reference `&'static (dyn Logger)` is complex
    // due to the MutexGuard. Providing `with`/`with_mut` or direct methods
    // like above is a safer pattern in Rust.
}