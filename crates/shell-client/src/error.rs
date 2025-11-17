//! Client error types

use thiserror::Error;

/// Client-related errors
#[derive(Error, Debug)]
pub enum ClientError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(#[from] reticulum_core::NetworkError),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(#[from] shell_proto::ProtocolError),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Not connected
    #[error("Not connected to server")]
    NotConnected,

    /// Server rejected connection
    #[error("Server rejected connection: {0}")]
    Rejected(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// REPL error
    #[error("REPL error: {0}")]
    Repl(String),
}

/// Result type for client operations
pub type Result<T> = std::result::Result<T, ClientError>;
