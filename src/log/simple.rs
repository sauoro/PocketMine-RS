// src/log/simple.rs

#![allow(dead_code)]

use crate::log::level::LogLevel;
use crate::log::logger::Logger;
use std::error::Error;
use std::fmt::Write;

#[derive(Debug, Clone, Default)]
pub struct SimpleLogger;

impl SimpleLogger {
    pub fn new() -> Self {
        SimpleLogger {}
    }
}

impl Logger for SimpleLogger {
    fn log(&self, level: LogLevel, message: &str) {
        // In a real server, this should go to a proper logging framework
        // or file/console handler. Stdout might get messy.
        println!("[{}] {}", level.to_str(), message);
    }

    fn log_exception(&self, e: &(dyn Error + Send + Sync + 'static)) {
        // Custom formatting closer to PHP's default exception output
        let mut output = String::new();
        writeln!(output, "[{}] {}: {}", LogLevel::Critical, std::any::type_name_of_val(e), e)
            .expect("Failed to write exception header");

        let mut current_source = e.source();
        let mut cause_level = 1;
        while let Some(source) = current_source {
            writeln!(output, "Caused by ({}) {}: {}", cause_level, std::any::type_name_of_val(source), source)
                .expect("Failed to write exception cause");
            current_source = source.source();
            cause_level += 1;
        }

        // Backtrace would go here if enabled/desired
        // let backtrace = std::backtrace::Backtrace::capture();
        // if backtrace.status() == std::backtrace::BacktraceStatus::Captured {
        //     writeln!(output, "Stack trace:\n{}", backtrace).expect("Failed to write backtrace");
        // }

        println!("{}", output.trim_end()); // Print the formatted string
    }
}