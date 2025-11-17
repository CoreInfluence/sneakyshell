//! Server error types

use thiserror::Error;

/// Server-related errors
#[derive(Error, Debug)]
pub enum ServerError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(#[from] reticulum_core::NetworkError),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(#[from] shell_proto::ProtocolError),

    /// Command execution error
    #[error("Command execution error: {0}")]
    Execution(String),

    /// Session error
    #[error("Session error: {0}")]
    Session(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,
}

/// Result type for server operations
pub type Result<T> = std::result::Result<T, ServerError>;
