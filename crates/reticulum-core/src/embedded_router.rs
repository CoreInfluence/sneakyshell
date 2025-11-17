//! Embedded I2P router using Emissary
//!
//! This module provides an embedded I2P router implementation using the Emissary
//! pure Rust I2P stack. This eliminates the need for an external I2P router process.

#[cfg(feature = "embedded-router")]
use emissary_core::{
    events::EventSubscriber,
    router::RouterBuilder,
    Config as EmissaryConfig,
};

#[cfg(feature = "embedded-router")]
use emissary_util::runtime::tokio::Runtime as TokioRuntime;

use crate::{NetworkError, Result};
use std::path::PathBuf;
use tracing::{debug, info};

/// Configuration for the embedded I2P router
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmbeddedRouterConfig {
    /// Data directory for router state and NetDB
    pub data_dir: PathBuf,

    /// Bandwidth limit in KB/s (None = unlimited)
    pub bandwidth_limit_kbps: Option<u32>,

    /// Number of tunnels to maintain
    pub tunnel_quantity: u32,

    /// Whether to participate as a floodfill router
    pub enable_floodfill: bool,

    /// Port for incoming connections (0 = random)
    pub listen_port: u16,

    /// SAM TCP port (0 = random, None = disabled)
    pub sam_tcp_port: Option<u16>,

    /// SAM UDP port (0 = random, None = disabled)
    pub sam_udp_port: Option<u16>,
}

impl Default for EmbeddedRouterConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from(".reticulum-shell/i2p"),
            bandwidth_limit_kbps: Some(2048), // 2 MB/s
            tunnel_quantity: 2,
            enable_floodfill: false,          // Don't be a directory server
            listen_port: 0,                   // Random port
            sam_tcp_port: Some(0),            // Random SAM TCP port
            sam_udp_port: Some(0),            // Random SAM UDP port
        }
    }
}

/// Embedded I2P router wrapper
#[cfg(feature = "embedded-router")]
pub struct EmbeddedRouter {
    _event_subscriber: EventSubscriber,
    router_info: Vec<u8>,
    #[allow(dead_code)] // Will be used for router configuration later
    config: EmbeddedRouterConfig,
    /// Actual SAM TCP port (if SAM is enabled)
    sam_tcp_port: Option<u16>,
    /// Actual SAM UDP port (if SAM is enabled)
    sam_udp_port: Option<u16>,
}

#[cfg(feature = "embedded-router")]
impl EmbeddedRouter {
    /// Create and start a new embedded I2P router
    pub async fn new(config: EmbeddedRouterConfig) -> Result<Self> {
        info!("Initializing embedded I2P router");
        info!("Data directory: {:?}", config.data_dir);

        // Create data directory if it doesn't exist
        std::fs::create_dir_all(&config.data_dir).map_err(|e| {
            NetworkError::I2p(format!("Failed to create data directory: {}", e))
        })?;

        // Configure Emissary router with SAM support
        let mut emissary_config = EmissaryConfig::default();

        // Configure SAM if requested
        if config.sam_tcp_port.is_some() || config.sam_udp_port.is_some() {
            emissary_config.samv3_config = Some(emissary_core::SamConfig {
                tcp_port: config.sam_tcp_port.unwrap_or(0),
                udp_port: config.sam_udp_port.unwrap_or(0),
                host: "127.0.0.1".to_string(),
            });
        }

        debug!("Starting Emissary router with Tokio runtime");

        // Create and start router using RouterBuilder
        let (router, event_subscriber, router_info) =
            RouterBuilder::<TokioRuntime>::new(emissary_config)
                .build()
                .await
                .map_err(|e| NetworkError::I2p(format!("Failed to start router: {}", e)))?;

        // Get actual SAM ports from router
        let addr_info = router.protocol_address_info();
        let sam_tcp_port = addr_info.sam_tcp.map(|addr| addr.port());
        let sam_udp_port = addr_info.sam_udp.map(|addr| addr.port());

        info!("Embedded I2P router started successfully");
        if let Some(port) = sam_tcp_port {
            info!("SAM TCP port: {}", port);
        }
        if let Some(port) = sam_udp_port {
            info!("SAM UDP port: {}", port);
        }
        debug!("Router info size: {} bytes", router_info.len());

        // Spawn router as background task
        tokio::spawn(router);

        Ok(Self {
            _event_subscriber: event_subscriber,
            router_info,
            config,
            sam_tcp_port,
            sam_udp_port,
        })
    }

    /// Wait for the router to be ready (tunnels established)
    pub async fn wait_ready(&self) -> Result<()> {
        info!("Waiting for I2P tunnels to establish (may take 30-60 seconds)...");

        // TODO: Implement proper ready check using Emissary's API
        // For now, just wait a fixed duration
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        info!("I2P router ready");
        Ok(())
    }

    /// Get the router's I2P destination
    /// Returns the base64-encoded router info which serves as the I2P destination
    pub fn local_destination(&self) -> Result<String> {
        use base64::Engine;
        let base64_engine = base64::engine::general_purpose::STANDARD;
        Ok(base64_engine.encode(&self.router_info))
    }

    /// Get the SAM TCP address for internal connections
    /// Returns None if SAM is not enabled
    pub fn sam_address(&self) -> Option<String> {
        self.sam_tcp_port
            .map(|port| format!("127.0.0.1:{}", port))
    }

    /// Get the SAM TCP port
    pub fn sam_tcp_port(&self) -> Option<u16> {
        self.sam_tcp_port
    }

    /// Get the SAM UDP port
    pub fn sam_udp_port(&self) -> Option<u16> {
        self.sam_udp_port
    }

    /// Shutdown the router gracefully
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down embedded I2P router");

        // TODO: Implement graceful shutdown
        // - Close all tunnels
        // - Flush NetDB
        // - Save router state

        info!("Embedded I2P router shutdown complete");
        Ok(())
    }

    /// Get router statistics
    pub fn stats(&self) -> RouterStats {
        RouterStats {
            tunnels_active: 0,
            peers_known: 0,
            bandwidth_in: 0,
            bandwidth_out: 0,
        }
    }
}

/// Router statistics
#[derive(Debug, Clone)]
pub struct RouterStats {
    pub tunnels_active: usize,
    pub peers_known: usize,
    pub bandwidth_in: u64,
    pub bandwidth_out: u64,
}

// Stub implementation when feature is disabled
#[cfg(not(feature = "embedded-router"))]
pub struct EmbeddedRouter;

#[cfg(not(feature = "embedded-router"))]
impl EmbeddedRouter {
    pub async fn new(_config: EmbeddedRouterConfig) -> Result<Self> {
        Err(NetworkError::I2p(
            "Embedded router not available - compile with 'embedded-router' feature".to_string(),
        ))
    }

    pub async fn wait_ready(&self) -> Result<()> {
        Err(NetworkError::I2p(
            "Embedded router not available".to_string(),
        ))
    }

    pub fn local_destination(&self) -> Result<String> {
        Err(NetworkError::I2p(
            "Embedded router not available".to_string(),
        ))
    }

    pub async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    pub fn stats(&self) -> RouterStats {
        RouterStats {
            tunnels_active: 0,
            peers_known: 0,
            bandwidth_in: 0,
            bandwidth_out: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "embedded-router")]
    #[ignore] // Requires network access and time
    async fn test_embedded_router_creation() {
        let config = EmbeddedRouterConfig {
            data_dir: PathBuf::from("/tmp/reticulum-test-router"),
            ..Default::default()
        };

        let router = EmbeddedRouter::new(config).await;
        assert!(router.is_ok(), "Router creation should succeed");

        let router = router.unwrap();

        // Verify SAM ports are available
        assert!(router.sam_tcp_port().is_some(), "SAM TCP port should be available");
        assert!(router.sam_udp_port().is_some(), "SAM UDP port should be available");
        assert!(router.sam_address().is_some(), "SAM address should be available");

        let shutdown_result = router.shutdown().await;
        assert!(shutdown_result.is_ok(), "Shutdown should succeed");
    }

    #[tokio::test]
    #[cfg(feature = "embedded-router")]
    #[ignore] // Requires network access and time
    async fn test_embedded_router_with_interface() {
        let config = EmbeddedRouterConfig {
            data_dir: PathBuf::from("/tmp/reticulum-test-router-interface"),
            ..Default::default()
        };

        let router = EmbeddedRouter::new(config).await.expect("Router creation failed");

        info!("Waiting for router to be ready...");
        router.wait_ready().await.expect("Router ready failed");

        info!("Creating I2P interface...");
        let interface = crate::I2pInterface::new_embedded(&router).await;
        assert!(interface.is_ok(), "Interface creation should succeed");

        let interface = interface.unwrap();
        info!("I2P interface created with destination: {}", interface.local_destination());

        router.shutdown().await.expect("Shutdown failed");
    }
}
