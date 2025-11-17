//! Client session management

use crate::{shell::CommandExecutor, Result, ServerError};
use shell_proto::{messages::AckMessage, Message, SessionId};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// A client session
pub struct Session {
    /// Session ID
    pub id: SessionId,

    /// Client identity (public key)
    pub client_identity: Vec<u8>,

    /// Command executor
    executor: Arc<CommandExecutor>,

    /// Session state
    state: Arc<RwLock<SessionState>>,
}

/// Session state
#[derive(Debug, Clone, PartialEq, Eq)]
enum SessionState {
    /// Session is active
    Active,

    /// Session is disconnecting
    #[allow(dead_code)]
    Disconnecting,

    /// Session is closed
    Closed,
}

impl Session {
    /// Create a new session
    pub fn new(client_identity: Vec<u8>, executor: Arc<CommandExecutor>) -> Self {
        let session_id = Uuid::new_v4().as_bytes().clone();

        info!(
            session_id = %Uuid::from_bytes(session_id),
            client = %hex::encode(&client_identity),
            "New session created"
        );

        Self {
            id: session_id,
            client_identity,
            executor,
            state: Arc::new(RwLock::new(SessionState::Active)),
        }
    }

    /// Handle a message from the client
    pub async fn handle_message(&self, message: Message) -> Result<Option<Message>> {
        // Check session state
        {
            let state = self.state.read().await;
            if *state != SessionState::Active {
                return Err(ServerError::Session("Session is not active".to_string()));
            }
        }

        match message {
            Message::CommandRequest(req) => {
                debug!(
                    session_id = %Uuid::from_bytes(self.id),
                    command_id = req.id,
                    "Handling command request"
                );

                // Validate request
                self.executor.validate_request(&req)?;

                // Execute command
                let response = self.executor.execute(req).await?;

                Ok(Some(Message::CommandResponse(response)))
            }

            Message::Disconnect(msg) => {
                info!(
                    session_id = %Uuid::from_bytes(self.id),
                    reason = ?msg.reason,
                    "Client disconnecting"
                );

                self.close().await?;

                Ok(Some(Message::Ack(AckMessage { message_id: 0 })))
            }

            Message::Ping => {
                debug!(
                    session_id = %Uuid::from_bytes(self.id),
                    "Ping received"
                );
                Ok(Some(Message::Pong))
            }

            _ => {
                debug!(
                    session_id = %Uuid::from_bytes(self.id),
                    "Unexpected message type"
                );
                Ok(None)
            }
        }
    }

    /// Close the session
    pub async fn close(&self) -> Result<()> {
        let mut state = self.state.write().await;
        *state = SessionState::Closed;

        info!(
            session_id = %Uuid::from_bytes(self.id),
            "Session closed"
        );

        Ok(())
    }

    /// Check if session is active
    pub async fn is_active(&self) -> bool {
        let state = self.state.read().await;
        *state == SessionState::Active
    }

    /// Get session ID as UUID string
    pub fn id_string(&self) -> String {
        Uuid::from_bytes(self.id).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shell_proto::CommandRequest;

    #[tokio::test]
    async fn test_session_creation() {
        let client_identity = vec![1, 2, 3, 4];
        let executor = Arc::new(CommandExecutor::new(30));
        let session = Session::new(client_identity.clone(), executor);

        assert_eq!(session.client_identity, client_identity);
        assert!(session.is_active().await);
    }

    #[tokio::test]
    async fn test_session_close() {
        let executor = Arc::new(CommandExecutor::new(30));
        let session = Session::new(vec![1, 2, 3], executor);

        assert!(session.is_active().await);

        session.close().await.unwrap();

        assert!(!session.is_active().await);
    }

    #[tokio::test]
    async fn test_handle_ping() {
        let executor = Arc::new(CommandExecutor::new(30));
        let session = Session::new(vec![1, 2, 3], executor);

        let response = session.handle_message(Message::Ping).await.unwrap();

        assert!(matches!(response, Some(Message::Pong)));
    }
}
