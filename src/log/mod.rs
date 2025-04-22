#![allow(dead_code)]

mod attachable;
mod buffered;
mod global;
mod level;
mod logger;
mod prefixed;
mod simple;

pub use attachable::{AttachableLogger, LoggerAttachment};
pub use buffered::BufferedLogger;
pub use global::GlobalLogger;
pub use level::LogLevel;
pub use logger::Logger;
pub use prefixed::PrefixedLogger;
pub use simple::SimpleLogger;

// Example trait implementations (Optional, depending on needs)
// If SimpleLogger should be attachable or buffered, implement those traits here or in simple.rs

// impl AttachableLogger for SimpleLogger { ... }
// impl BufferedLogger for SimpleLogger { ... }