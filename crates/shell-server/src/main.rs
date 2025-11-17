//! Shell Server - Remote shell access server over Reticulum
//!
//! This server listens for incoming connections over the Reticulum network
//! and executes commands from authenticated clients.

use clap::Parser;
use reticulum_core::{I2pInterface, NetworkInterface};
use shell_server::{config::ServerConfig, server::Server, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "server.toml")]
    config: PathBuf,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Generate a new identity and exit
    #[arg(long)]
    generate_identity: Option<PathBuf>,

    /// Enable I2P transport
    #[arg(long)]
    enable_i2p: bool,

    /// Use embedded I2P router instead of external router
    #[cfg(feature = "embedded-router")]
    #[arg(long)]
    use_embedded_router: bool,

    /// SAM bridge address for external router (default: 127.0.0.1:7656)
    #[arg(long)]
    sam_address: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    // Handle identity generation
    if let Some(identity_path) = args.generate_identity {
        info!("Generating new identity at {:?}", identity_path);
        let identity = reticulum_core::Identity::generate();
        identity.save_to_file(&identity_path)?;
        info!("Identity saved: {}", identity.destination_hex());
        return Ok(());
    }

    // Load or create configuration
    let config = if args.config.exists() {
        info!("Loading configuration from {:?}", args.config);
        ServerConfig::load_from_file(&args.config)?
    } else {
        info!("Configuration file not found, creating default configuration");

        // Create default config
        let mut config = ServerConfig::default();

        // Generate identity if it doesn't exist
        if !config.identity_path.exists() {
            info!("Generating new server identity at {:?}", config.identity_path);
            config.identity.save_to_file(&config.identity_path)?;
            info!("Server identity saved: {}", config.identity.destination_hex());
        } else {
            // Load existing identity
            config.identity = reticulum_core::Identity::load_from_file(&config.identity_path)?;
        }

        // Save config for future use
        config.save_to_file(&args.config)?;
        info!("Default configuration saved to {:?}", args.config);

        config
    };

    info!("Server destination: {}", config.identity.destination_hex());

    // Override config with CLI args if provided
    let enable_i2p = args.enable_i2p || config.enable_i2p;
    let sam_address = args.sam_address.unwrap_or(config.sam_address.clone());

    #[cfg(feature = "embedded-router")]
    let use_embedded = args.use_embedded_router
        || matches!(config.router_mode, reticulum_core::RouterMode::Embedded);

    // Create server with optional I2P interface
    let server = if enable_i2p {
        #[cfg(feature = "embedded-router")]
        if use_embedded {
            info!("Starting embedded I2P router...");

            let router = reticulum_core::EmbeddedRouter::new(config.embedded_router.clone())
                .await
                .map_err(|e| {
                    error!("Failed to start embedded router: {}", e);
                    e
                })?;

            info!("Embedded router started successfully");

            // Wait for router to be ready
            router.wait_ready().await?;

            info!("Connecting to embedded router via SAM...");
            match I2pInterface::new_embedded(&router).await {
                Ok(i2p_interface) => {
                    info!("I2P interface created successfully");
                    info!("I2P destination: {}", i2p_interface.local_destination());
                    info!("I2P destination hash: {}", hex::encode(i2p_interface.local_destination_hash()));

                    let interface: Arc<dyn NetworkInterface> = Arc::new(i2p_interface);
                    Server::with_interface(config, interface).await?
                }
                Err(e) => {
                    error!("Failed to create I2P interface: {}", e);
                    return Err(e.into());
                }
            }
        } else {
            info!("Connecting to external I2P router via SAM bridge at {}", sam_address);

            match I2pInterface::new(&sam_address).await {
                Ok(i2p_interface) => {
                    info!("I2P interface created successfully");
                    info!("I2P destination: {}", i2p_interface.local_destination());
                    info!("I2P destination hash: {}", hex::encode(i2p_interface.local_destination_hash()));

                    let interface: Arc<dyn NetworkInterface> = Arc::new(i2p_interface);
                    Server::with_interface(config, interface).await?
                }
                Err(e) => {
                    error!("Failed to create I2P interface: {}", e);
                    error!("Make sure I2P router is running with SAM bridge enabled on {}", sam_address);
                    return Err(e.into());
                }
            }
        }

        #[cfg(not(feature = "embedded-router"))]
        {
            info!("Connecting to external I2P router via SAM bridge at {}", sam_address);

            match I2pInterface::new(&sam_address).await {
                Ok(i2p_interface) => {
                    info!("I2P interface created successfully");
                    info!("I2P destination: {}", i2p_interface.local_destination());
                    info!("I2P destination hash: {}", hex::encode(i2p_interface.local_destination_hash()));

                    let interface: Arc<dyn NetworkInterface> = Arc::new(i2p_interface);
                    Server::with_interface(config, interface).await?
                }
                Err(e) => {
                    error!("Failed to create I2P interface: {}", e);
                    error!("Make sure I2P router is running with SAM bridge enabled on {}", sam_address);
                    return Err(e.into());
                }
            }
        }
    } else {
        warn!("I2P transport not enabled - server will run without network interface");
        info!("To enable I2P: use --enable-i2p flag or set enable_i2p=true in config");
        Server::new(config).await?
    };

    info!("Listening on Reticulum network...");

    // Run server
    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
        return Err(e);
    }

    Ok(())
}
