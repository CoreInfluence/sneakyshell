//! Server configuration

use crate::{Result, ServerError};
use reticulum_core::Identity;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server identity (loaded, not serialized as private key)
    #[serde(skip, default = "default_identity")]
    pub identity: Identity,

    /// Path to identity file
    pub identity_path: PathBuf,

    /// Maximum concurrent sessions
    #[serde(default = "default_max_sessions")]
    pub max_sessions: usize,

    /// Command execution timeout (seconds)
    #[serde(default = "default_command_timeout")]
    pub command_timeout: u64,

    /// Enable audit logging
    #[serde(default = "default_audit_logging")]
    pub audit_logging: bool,

    /// Audit log path
    #[serde(default = "default_audit_log_path")]
    pub audit_log_path: PathBuf,

    /// Allowed client identities (empty = allow all)
    #[serde(default)]
    pub allowed_clients: Vec<String>,

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
}

fn default_sam_address() -> String {
    "127.0.0.1:7656".to_string()
}

fn default_identity() -> Identity {
    Identity::generate()
}

fn default_max_sessions() -> usize {
    10
}

fn default_command_timeout() -> u64 {
    300 // 5 minutes
}

fn default_audit_logging() -> bool {
    true
}

fn default_audit_log_path() -> PathBuf {
    PathBuf::from("audit.log")
}

impl ServerConfig {
    /// Load configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let mut config: ServerConfig = toml::from_str(&contents)
            .map_err(|e| ServerError::Config(format!("Failed to parse config: {}", e)))?;

        // Load identity
        config.identity = Identity::load_from_file(&config.identity_path)?;

        Ok(config)
    }

    /// Create a default configuration
    pub fn default() -> Self {
        Self {
            identity: Identity::generate(),
            identity_path: PathBuf::from("server.identity"),
            max_sessions: default_max_sessions(),
            command_timeout: default_command_timeout(),
            audit_logging: default_audit_logging(),
            audit_log_path: default_audit_log_path(),
            allowed_clients: vec![],
            enable_i2p: false,
            router_mode: reticulum_core::RouterMode::default(),
            sam_address: default_sam_address(),
            #[cfg(feature = "embedded-router")]
            embedded_router: reticulum_core::EmbeddedRouterConfig::default(),
        }
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| ServerError::Config(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(path, toml_string)?;
        Ok(())
    }

    /// Check if a client identity is allowed
    pub fn is_client_allowed(&self, client_identity: &[u8]) -> bool {
        if self.allowed_clients.is_empty() {
            return true; // Allow all if list is empty
        }

        let client_hex = hex::encode(client_identity);
        self.allowed_clients.contains(&client_hex)
    }
}
