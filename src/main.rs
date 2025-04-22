// src/main.rs
mod utils;
mod color;
mod math;
mod nbt;
mod log;

use crate::log::Logger;

fn main() {
    // Example usage of the global logger
    log::GlobalLogger::info("Server starting...");
    log::GlobalLogger::warning("This is a warning message.");

    if let Err(e) = std::fs::read_to_string("non_existent_file.txt") {
        // Box the error to satisfy the trait object requirements if needed
        // let boxed_error: Box<dyn std::error::Error + Send + Sync + 'static> = Box::new(e);
        // log::GlobalLogger::log_exception(&*boxed_error);
        // Or directly if the error type already impls Error + Send + Sync + 'static
        log::GlobalLogger::log_exception(&e);
    }


    // Example using a prefixed logger
    let simple_logger = Box::new(log::SimpleLogger::new());
    let mut plugin_logger = log::PrefixedLogger::new(simple_logger, "MyPlugin".to_string());

    plugin_logger.info("Plugin loaded!");
    plugin_logger.debug("Some debug info from plugin.");

    // Change prefix
    plugin_logger.set_prefix("MyPlugin-Updated".to_string());
    plugin_logger.warning("Plugin prefix changed.");

    // Replace global logger
    // log::GlobalLogger::set(Box::new(plugin_logger)); // Now global logs are prefixed
    // log::GlobalLogger::info("This message will now be prefixed.");

    log::GlobalLogger::critical("Server shutting down simulation.");
}