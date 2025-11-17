//! Reticulum Core - Networking layer for Reticulum over I2P
//!
//! This crate provides the core networking functionality for the Reticulum protocol,
//! including identity management, packet handling, and I2P transport.

pub mod error;
pub mod identity;
pub mod interface;
pub mod packet;
pub mod sam;

#[cfg(feature = "embedded-router")]
pub mod embedded_router;

pub use error::{NetworkError, Result};
pub use identity::Identity;
pub use interface::{I2pInterface, MockInterface, NetworkInterface};
pub use packet::{Packet, PacketType};
pub use sam::SamConnection;

#[cfg(feature = "embedded-router")]
pub use embedded_router::{EmbeddedRouter, EmbeddedRouterConfig, RouterStats};

/// Reticulum destination address (32 bytes)
pub type DestinationHash = [u8; 32];

/// Router mode for I2P connectivity
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RouterMode {
    /// Use external I2P router via SAM
    External,
    /// Use embedded I2P router
    #[cfg(feature = "embedded-router")]
    Embedded,
}

impl Default for RouterMode {
    fn default() -> Self {
        RouterMode::External
    }
}

/// Re-exports
pub use ed25519_dalek;
pub use sha2;
