//! Network listener for incoming connections

use crate::{config::ServerConfig, session::Session, shell::CommandExecutor, Result};
use shell_proto::{
    messages::{AcceptMessage, ConnectMessage, RejectMessage},
    Message, CURRENT_PROTOCOL_VERSION,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Connection listener
pub struct Listener {
    /// Server configuration
    config: Arc<ServerConfig>,

    /// Command executor
    executor: Arc<CommandExecutor>,

    /// Active sessions
    sessions: Arc<RwLock<Vec<Arc<Session>>>>,
}

impl Listener {
    /// Create a new listener
    pub fn new(config: ServerConfig) -> Self {
        let executor = Arc::new(CommandExecutor::new(config.command_timeout));

        Self {
            config: Arc::new(config),
            executor,
            sessions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Handle incoming connection
    pub async fn handle_connection(&self, message: Message) -> Result<Message> {
        match message {
            Message::Connect(connect_msg) => {
                self.handle_connect(connect_msg).await
            }
            _ => {
                warn!("Unexpected message type during connection");
                Ok(Message::Reject(RejectMessage {
                    reason: "Expected CONNECT message".to_string(),
                    error_code: 1,
                }))
            }
        }
    }

    /// Handle CONNECT message
    async fn handle_connect(&self, connect: ConnectMessage) -> Result<Message> {
        debug!(
            client = %hex::encode(&connect.client_identity),
            protocol_version = connect.protocol_version,
            "Handling connection request"
        );

        // Check protocol version
        if connect.protocol_version != CURRENT_PROTOCOL_VERSION {
            warn!(
                expected = CURRENT_PROTOCOL_VERSION,
                actual = connect.protocol_version,
                "Protocol version mismatch"
            );
            return Ok(Message::Reject(RejectMessage {
                reason: format!(
                    "Protocol version mismatch: expected {}, got {}",
                    CURRENT_PROTOCOL_VERSION, connect.protocol_version
                ),
                error_code: 2,
            }));
        }

        // Check if client is allowed
        if !self.config.is_client_allowed(&connect.client_identity) {
            warn!(
                client = %hex::encode(&connect.client_identity),
                "Client not in allowed list"
            );
            return Ok(Message::Reject(RejectMessage {
                reason: "Client not authorized".to_string(),
                error_code: 3,
            }));
        }

        // Check session limit
        {
            let sessions = self.sessions.read().await;
            if sessions.len() >= self.config.max_sessions {
                warn!("Maximum session limit reached");
                return Ok(Message::Reject(RejectMessage {
                    reason: "Maximum sessions reached".to_string(),
                    error_code: 4,
                }));
            }
        }

        // Create new session
        let session = Arc::new(Session::new(
            connect.client_identity.clone(),
            self.executor.clone(),
        ));

        // Add to active sessions
        {
            let mut sessions = self.sessions.write().await;
            sessions.push(session.clone());
        }

        info!(
            session_id = %session.id_string(),
            client = %hex::encode(&connect.client_identity),
            "Connection accepted"
        );

        // Send ACCEPT message
        Ok(Message::Accept(AcceptMessage {
            protocol_version: CURRENT_PROTOCOL_VERSION,
            server_identity: self.config.identity.public_key(),
            session_id: session.id,
            capabilities: vec!["command-exec".to_string()],
        }))
    }

    /// Get number of active sessions
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// Clean up inactive sessions
    pub async fn cleanup_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_session| {
            // This is a blocking operation in async context
            // In a real implementation, we'd use a different approach
            // For now, we'll keep all sessions
            true
        });
    }

    /// Get the command executor
    pub fn executor(&self) -> Arc<CommandExecutor> {
        Arc::clone(&self.executor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_listener_creation() {
        let config = ServerConfig::default();
        let listener = Listener::new(config);

        assert_eq!(listener.session_count().await, 0);
    }

    #[tokio::test]
    async fn test_handle_connect_success() {
        let config = ServerConfig::default();
        let listener = Listener::new(config);

        let connect = ConnectMessage {
            protocol_version: CURRENT_PROTOCOL_VERSION,
            client_identity: vec![1, 2, 3, 4],
            capabilities: vec![],
            auth_token: None,
        };

        let response = listener.handle_connection(Message::Connect(connect)).await.unwrap();

        assert!(matches!(response, Message::Accept(_)));
        assert_eq!(listener.session_count().await, 1);
    }

    #[tokio::test]
    async fn test_handle_connect_version_mismatch() {
        let config = ServerConfig::default();
        let listener = Listener::new(config);

        let connect = ConnectMessage {
            protocol_version: 999, // Wrong version
            client_identity: vec![1, 2, 3, 4],
            capabilities: vec![],
            auth_token: None,
        };

        let response = listener.handle_connection(Message::Connect(connect)).await.unwrap();

        match response {
            Message::Reject(reject) => {
                assert_eq!(reject.error_code, 2);
            }
            _ => panic!("Expected Reject message"),
        }
    }
}
