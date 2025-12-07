// Cronicle Server - Main entry point (Rust implementation)
// Original Copyright (c) 2015 - 2023 Joseph Huckaby
// Rust port contributors
// Released under the MIT License

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod engine;
mod server;
mod storage;
mod scheduler;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = "Cronicle";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cronicle=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("{} Server v{} starting up", APP_NAME, VERSION);

    // Check Node.js version requirement notice
    // Note: This is a Rust implementation, so Node.js is not required
    info!("Running Rust implementation - Node.js is not required");

    // Load configuration
    let config = config::Config::load("conf/config.json").await?;
    info!("Configuration loaded successfully");

    // Initialize and start the server
    let server = server::Server::new(config).await?;
    
    info!("{} Server initialization complete", APP_NAME);
    
    // Start the server
    server.run().await?;

    Ok(())
}
