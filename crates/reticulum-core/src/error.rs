//! Network error types

use thiserror::Error;

/// Network-related errors
#[derive(Error, Debug)]
pub enum NetworkError {
    /// Identity error
    #[error("Identity error: {0}")]
    Identity(String),

    /// Packet error
    #[error("Packet error: {0}")]
    Packet(String),

    /// I2P transport error
    #[error("I2P error: {0}")]
    I2p(String),

    /// Cryptographic error
    #[error("Cryptographic error: {0}")]
    Crypto(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Invalid destination
    #[error("Invalid destination: {0}")]
    InvalidDestination(String),
}

/// Result type for network operations
pub type Result<T> = std::result::Result<T, NetworkError>;
