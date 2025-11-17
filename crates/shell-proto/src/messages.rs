//! Protocol message definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique session identifier
pub type SessionId = [u8; 16];

/// Protocol version
pub type ProtocolVersion = u32;

/// Top-level message enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Client initiates connection
    Connect(ConnectMessage),

    /// Server accepts connection
    Accept(AcceptMessage),

    /// Server rejects connection
    Reject(RejectMessage),

    /// Client requests command execution
    CommandRequest(CommandRequest),

    /// Server responds with command results
    CommandResponse(CommandResponse),

    /// Either side initiates disconnect
    Disconnect(DisconnectMessage),

    /// Acknowledgment message
    Ack(AckMessage),

    /// Keep-alive ping
    Ping,

    /// Keep-alive pong
    Pong,
}

/// Connection request from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectMessage {
    /// Protocol version the client speaks
    pub protocol_version: ProtocolVersion,

    /// Client's Reticulum identity (public key)
    pub client_identity: Vec<u8>,

    /// Optional capabilities the client supports
    pub capabilities: Vec<String>,

    /// Optional authentication token
    pub auth_token: Option<String>,
}

/// Server accepts connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptMessage {
    /// Protocol version the server will use
    pub protocol_version: ProtocolVersion,

    /// Server's Reticulum identity (public key)
    pub server_identity: Vec<u8>,

    /// Unique session identifier
    pub session_id: SessionId,

    /// Server capabilities
    pub capabilities: Vec<String>,
}

/// Server rejects connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectMessage {
    /// Rejection reason
    pub reason: String,

    /// Error code
    pub error_code: u32,
}

/// Command execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    /// Unique request ID (for matching responses)
    pub id: u64,

    /// Command to execute (e.g., "ls", "whoami")
    pub command: String,

    /// Command arguments
    pub args: Vec<String>,

    /// Optional environment variables
    pub env: Option<HashMap<String, String>>,

    /// Optional execution timeout (seconds)
    pub timeout: Option<u64>,

    /// Optional working directory
    pub working_dir: Option<String>,
}

/// Command execution response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    /// Request ID this response is for
    pub id: u64,

    /// Execution status
    pub status: CommandStatus,

    /// Standard output (raw bytes)
    pub stdout: Vec<u8>,

    /// Standard error (raw bytes)
    pub stderr: Vec<u8>,

    /// Process exit code
    pub exit_code: i32,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Command execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommandStatus {
    /// Command completed successfully
    Success,

    /// Command execution timed out
    Timeout,

    /// Command execution failed
    Error,

    /// Command was killed
    Killed,
}

/// Disconnect message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisconnectMessage {
    /// Optional reason for disconnection
    pub reason: Option<String>,
}

/// Acknowledgment message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckMessage {
    /// ID of message being acknowledged
    pub message_id: u64,
}

impl Message {
    /// Get message type identifier
    pub fn message_type(&self) -> u8 {
        match self {
            Message::Connect(_) => 0x01,
            Message::Accept(_) => 0x02,
            Message::Reject(_) => 0x03,
            Message::CommandRequest(_) => 0x10,
            Message::CommandResponse(_) => 0x11,
            Message::Disconnect(_) => 0x20,
            Message::Ack(_) => 0x21,
            Message::Ping => 0x30,
            Message::Pong => 0x31,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_types() {
        assert_eq!(Message::Ping.message_type(), 0x30);
        assert_eq!(Message::Pong.message_type(), 0x31);
    }

    #[test]
    fn test_command_request_serialization() {
        let req = CommandRequest {
            id: 123,
            command: "ls".to_string(),
            args: vec!["-la".to_string()],
            env: None,
            timeout: Some(30),
            working_dir: Some("/tmp".to_string()),
        };

        let msg = Message::CommandRequest(req.clone());
        let serialized = bincode::serialize(&msg).unwrap();
        let deserialized: Message = bincode::deserialize(&serialized).unwrap();

        match deserialized {
            Message::CommandRequest(decoded) => {
                assert_eq!(decoded.id, req.id);
                assert_eq!(decoded.command, req.command);
                assert_eq!(decoded.args, req.args);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
