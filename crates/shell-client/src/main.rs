//! Shell Client - Remote shell client for Reticulum network
//!
//! Connects to a shell server and provides an interactive REPL for executing commands.

use clap::Parser;
use reticulum_core::{I2pInterface, NetworkInterface};
use shell_client::{client::Client, config::ClientConfig, repl::Repl, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server destination (hex string)
    #[arg(short, long)]
    server: Option<String>,

    /// Path to configuration file
    #[arg(short, long, default_value = "client.toml")]
    config: PathBuf,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Generate a new identity and exit
    #[arg(long)]
    generate_identity: Option<PathBuf>,

    /// Execute a single command and exit
    #[arg(short = 'e', long)]
    execute: Option<String>,

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

    /// Server I2P destination (base64 string)
    #[arg(long)]
    i2p_destination: Option<String>,
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
    let mut config = if args.config.exists() {
        info!("Loading configuration from {:?}", args.config);
        ClientConfig::load_from_file(&args.config)?
    } else {
        info!("Configuration file not found, creating default configuration");

        // Create default config
        let mut config = ClientConfig::default();

        // Generate identity if it doesn't exist
        if !config.identity_path.exists() {
            info!("Generating new client identity at {:?}", config.identity_path);
            config.identity.save_to_file(&config.identity_path)?;
            info!("Client identity saved: {}", config.identity.destination_hex());
        } else {
            // Load existing identity
            config.identity = reticulum_core::Identity::load_from_file(&config.identity_path)?;
        }

        // Save config for future use
        config.save_to_file(&args.config)?;
        info!("Default configuration saved to {:?}", args.config);
        info!("IMPORTANT: Edit {:?} and set the server_destination", args.config);

        config
    };

    // Override server if provided via CLI
    if let Some(server) = args.server {
        config.server_destination = server;
    }

    // Override I2P settings with CLI args if provided
    let enable_i2p = args.enable_i2p || config.enable_i2p;
    let sam_address = args.sam_address.unwrap_or(config.sam_address.clone());
    let server_i2p_dest = args.i2p_destination.or(config.server_i2p_destination.clone());

    #[cfg(feature = "embedded-router")]
    let use_embedded = args.use_embedded_router
        || matches!(config.router_mode, reticulum_core::RouterMode::Embedded);

    info!("Client identity: {}", config.identity.destination_hex());

    // Create client with optional I2P interface
    let client = if enable_i2p {
        // Create I2P interface (embedded or external)
        let i2p_interface = {
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
                    Ok(iface) => {
                        info!("I2P interface created successfully");
                        info!("Client I2P destination: {}", iface.local_destination());
                        info!("Client I2P destination hash: {}", hex::encode(iface.local_destination_hash()));
                        iface
                    }
                    Err(e) => {
                        error!("Failed to create I2P interface: {}", e);
                        return Err(e.into());
                    }
                }
            } else {
                info!("Connecting to external I2P router via SAM bridge at {}", sam_address);

                match I2pInterface::new(&sam_address).await {
                    Ok(iface) => {
                        info!("I2P interface created successfully");
                        info!("Client I2P destination: {}", iface.local_destination());
                        info!("Client I2P destination hash: {}", hex::encode(iface.local_destination_hash()));
                        iface
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
                    Ok(iface) => {
                        info!("I2P interface created successfully");
                        info!("Client I2P destination: {}", iface.local_destination());
                        info!("Client I2P destination hash: {}", hex::encode(iface.local_destination_hash()));
                        iface
                    }
                    Err(e) => {
                        error!("Failed to create I2P interface: {}", e);
                        error!("Make sure I2P router is running with SAM bridge enabled on {}", sam_address);
                        return Err(e.into());
                    }
                }
            }
        };

        // Parse and register server I2P destination
        let server_dest_hash = if let Some(ref i2p_dest) = server_i2p_dest {
            info!("Registering server I2P destination: {}...", &i2p_dest[..20.min(i2p_dest.len())]);
            i2p_interface.register_destination(i2p_dest.clone()).await
        } else {
            error!("I2P enabled but no server I2P destination provided");
            error!("Use --i2p-destination flag or set server_i2p_destination in config");
            return Err(shell_client::ClientError::Config(
                "Missing server I2P destination".to_string()
            ).into());
        };

        info!("Server I2P destination hash: {}", hex::encode(server_dest_hash));

        let interface: Arc<dyn NetworkInterface> = Arc::new(i2p_interface);
        Client::with_interface(config, interface, server_dest_hash).await?
    } else {
        info!("Connecting to server: {}", config.server_destination);
        Client::new(config).await?
    };

    // Connect to server
    client.connect().await?;
    info!("Connected to server");

    // Execute single command or start REPL
    if let Some(command) = args.execute {
        // Execute single command
        let parts: Vec<String> = shell_words::split(&command)
            .map_err(|e| shell_client::ClientError::Config(format!("Invalid command: {}", e)))?;

        if parts.is_empty() {
            error!("Empty command");
            return Ok(());
        }

        let cmd = parts[0].clone();
        let cmd_args = parts[1..].to_vec();

        match client.execute_command(cmd, cmd_args).await {
            Ok(response) => {
                print!("{}", String::from_utf8_lossy(&response.stdout));
                eprint!("{}", String::from_utf8_lossy(&response.stderr));
                std::process::exit(response.exit_code);
            }
            Err(e) => {
                error!("Command execution failed: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Start interactive REPL
        let mut repl = Repl::new(client);
        if let Err(e) = repl.run().await {
            error!("REPL error: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
