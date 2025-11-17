//! Protocol error types

use thiserror::Error;

/// Protocol-related errors
#[derive(Error, Debug)]
pub enum ProtocolError {
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Incompatible protocol version
    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },

    /// Invalid message type
    #[error("Invalid message type: {0}")]
    InvalidMessageType(u8),

    /// Message too large
    #[error("Message too large: {size} bytes (max: {max})")]
    MessageTooLarge { size: usize, max: usize },

    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<bincode::Error> for ProtocolError {
    fn from(err: bincode::Error) -> Self {
        ProtocolError::Serialization(err.to_string())
    }
}

/// Result type for protocol operations
pub type Result<T> = std::result::Result<T, ProtocolError>;
