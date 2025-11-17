//! Shell Protocol Definitions
//!
//! This crate defines the wire protocol for reticulum-shell, including all message
//! types, serialization, and protocol versioning.

pub mod error;
pub mod messages;
pub mod protocol;

pub use error::{ProtocolError, Result};
pub use messages::{
    CommandRequest, CommandResponse, CommandStatus, ConnectMessage, Message, SessionId,
};
pub use protocol::{ProtocolCodec, ProtocolVersion, CURRENT_PROTOCOL_VERSION};
