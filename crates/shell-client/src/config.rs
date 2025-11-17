//! Client configuration

use crate::{ClientError, Result};
use reticulum_core::Identity;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Client identity (loaded, not serialized as private key)
    #[serde(skip, default = "default_identity")]
    pub identity: Identity,

    /// Path to identity file
    pub identity_path: PathBuf,

    /// Server destination (hex string)
    pub server_destination: String,

    /// Connection timeout (seconds)
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,

    /// Command timeout (seconds)
    #[serde(default = "default_command_timeout")]
    pub command_timeout: u64,

    /// Enable I2P transport
    #[serde(default)]
    pub enable_i2p: bool,

    /// I2P router mode (external or embedded)
    #[serde(default)]
    pub router_mode: reticulum_core::RouterMode,

    /// SAM bridge address for I2P (used in External mode)
    #[serde(default = "default_sam_address")]
    pub sam_address: String,

    /// Embedded router configuration (used in Embedded mode)
    #[cfg(feature = "embedded-router")]
    #[serde(default)]
    pub embedded_router: reticulum_core::EmbeddedRouterConfig,

    /// Server I2P destination (base64 string, if using I2P)
    #[serde(default)]
    pub server_i2p_destination: Option<String>,
}

fn default_sam_address() -> String {
    "127.0.0.1:7656".to_string()
}

fn default_identity() -> Identity {
    Identity::generate()
}

fn default_connection_timeout() -> u64 {
    30
}

fn default_command_timeout() -> u64 {
    300 // 5 minutes
}

impl ClientConfig {
    /// Load configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let mut config: ClientConfig = toml::from_str(&contents)
            .map_err(|e| ClientError::Config(format!("Failed to parse config: {}", e)))?;

        // Load identity
        config.identity = Identity::load_from_file(&config.identity_path)?;

        Ok(config)
    }

    /// Create a default configuration
    pub fn default() -> Self {
        Self {
            identity: Identity::generate(),
            identity_path: PathBuf::from("client.identity"),
            server_destination: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            connection_timeout: default_connection_timeout(),
            command_timeout: default_command_timeout(),
            enable_i2p: false,
            router_mode: reticulum_core::RouterMode::default(),
            sam_address: default_sam_address(),
            #[cfg(feature = "embedded-router")]
            embedded_router: reticulum_core::EmbeddedRouterConfig::default(),
            server_i2p_destination: None,
        }
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| ClientError::Config(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(path, toml_string)?;
        Ok(())
    }

    /// Parse server destination from hex string
    pub fn parse_server_destination(&self) -> Result<[u8; 32]> {
        let bytes = hex::decode(&self.server_destination)
            .map_err(|e| ClientError::Config(format!("Invalid server destination hex: {}", e)))?;

        if bytes.len() != 32 {
            return Err(ClientError::Config(
                "Server destination must be 32 bytes".to_string(),
            ));
        }

        let mut dest = [0u8; 32];
        dest.copy_from_slice(&bytes);
        Ok(dest)
    }
}
