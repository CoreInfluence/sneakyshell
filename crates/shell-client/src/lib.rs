//! Shell Client Library
//!
//! Core functionality for the remote shell client

pub mod client;
pub mod config;
pub mod error;
pub mod repl;

pub use error::{ClientError, Result};
