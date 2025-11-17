//! Shell Server Library
//!
//! Core functionality for the remote shell server

pub mod config;
pub mod error;
pub mod listener;
pub mod server;
pub mod session;
pub mod shell;

pub use error::{Result, ServerError};
