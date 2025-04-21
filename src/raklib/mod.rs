// Declare the submodules
pub mod error;
pub mod generic;
pub mod protocol;
pub mod server;
pub mod utils;
// pub mod client; // Uncomment when needed

// Re-export key items if desired, e.g.:
// pub use server::Server;
// pub use error::RakLibError;

/// Default RakNet protocol version supported by this library.
pub const DEFAULT_PROTOCOL_VERSION: u8 = 11; // MCBE often uses 10 or 11

/// Default number of system addresses advertised (like in PHP RakLib).
/// Vanilla RakNet uses 10, MCBE uses 20.
pub const DEFAULT_SYSTEM_ADDRESS_COUNT: usize = 20; // Matches MCPE