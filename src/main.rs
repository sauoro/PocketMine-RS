mod utils;
mod raklib;

use tracing::info; // Use tracing example

#[tokio::main] // Use tokio main for async
async fn main() {
    // Initialize tracing subscriber (basic example)
    tracing_subscriber::fmt::init();

    info!("PocketMine-rs starting...");

    // --- Example usage of BinaryStream (keep if desired) ---
    // ... (previous example code) ...
    // --- End Example ---

    // TODO: Initialize and run the RakNet server here
    // Example placeholder:
    // let server_addr = "0.0.0.0:19132"; // Or 127.0.0.1:19132
    // match raklib::server::run_server(server_addr).await {
    //     Ok(_) => info!("Server finished cleanly."),
    //     Err(e) => tracing::error!("Server exited with error: {}", e),
    // }

    info!("PocketMine-rs finished (for now).");
}