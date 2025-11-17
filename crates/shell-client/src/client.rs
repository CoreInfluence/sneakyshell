//! Client connection management

use crate::{config::ClientConfig, ClientError, Result};
use reticulum_core::{NetworkInterface, Packet};
use shell_proto::{
    CommandRequest, CommandResponse, ConnectMessage, Message, ProtocolCodec, SessionId,
    CURRENT_PROTOCOL_VERSION,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Connection state
#[derive(Debug, Clone, PartialEq, Eq)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
}

/// Shell client
pub struct Client {
    /// Client configuration
    config: Arc<ClientConfig>,

    /// Connection state
    state: Arc<RwLock<ConnectionState>>,

    /// Session ID (if connected)
    session_id: Arc<RwLock<Option<SessionId>>>,

    /// Request ID counter
    next_request_id: Arc<AtomicU64>,

    /// Network interface
    interface: Option<Arc<dyn NetworkInterface>>,

    /// Server destination
    server_destination: [u8; 32],
}

impl Client {
    /// Create a new client
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let server_dest = config.parse_server_destination()?;

        Ok(Self {
            config: Arc::new(config),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            session_id: Arc::new(RwLock::new(None)),
            next_request_id: Arc::new(AtomicU64::new(1)),
            interface: None,
            server_destination: server_dest,
        })
    }

    /// Create a client with a specific network interface (for testing or I2P)
    /// If server_destination is provided, it overrides the config
    pub async fn with_interface(
        config: ClientConfig,
        interface: Arc<dyn NetworkInterface>,
        server_destination: [u8; 32],
    ) -> Result<Self> {
        Ok(Self {
            config: Arc::new(config),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            session_id: Arc::new(RwLock::new(None)),
            next_request_id: Arc::new(AtomicU64::new(1)),
            interface: Some(interface),
            server_destination,
        })
    }

    /// Connect to server
    pub async fn connect(&self) -> Result<()> {
        // Check current state
        {
            let state = self.state.read().await;
            if *state == ConnectionState::Connected {
                return Ok(());
            }
        }

        // Update state to connecting
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Connecting;
        }

        // Check for placeholder server destination
        if self.config.server_destination == "0000000000000000000000000000000000000000000000000000000000000000"
            || self.config.server_destination.is_empty() {
            return Err(ClientError::Config(
                "Server destination not configured. Please set server_destination in client.toml".to_string()
            ));
        }

        info!("Connecting to server: {}", self.config.server_destination);

        // Check if we have an interface
        let interface = self.interface.as_ref().ok_or_else(|| {
            ClientError::NotConnected
        })?;

        // Send CONNECT message
        let connect_msg = ConnectMessage {
            protocol_version: CURRENT_PROTOCOL_VERSION,
            client_identity: self.config.identity.public_key(),
            capabilities: vec!["command-exec".to_string()],
            auth_token: None,
        };

        debug!("Sending CONNECT message");

        // Encode and send
        let message = Message::Connect(connect_msg);
        let encoded = ProtocolCodec::encode(&message)?;
        let packet = Packet::data(self.server_destination, encoded);
        interface.send(&packet).await?;

        // Receive response
        let response_packet = interface.receive().await?;
        let mut buf = bytes::BytesMut::from(response_packet.data.as_ref());
        let response_msg = ProtocolCodec::decode(&mut buf)?
            .ok_or_else(|| ClientError::Connection("No response from server".to_string()))?;

        // Handle response
        match response_msg {
            Message::Accept(accept) => {
                info!("Connection accepted by server");

                // Update state
                {
                    let mut state = self.state.write().await;
                    *state = ConnectionState::Connected;
                }

                {
                    let mut session = self.session_id.write().await;
                    *session = Some(accept.session_id);
                }

                info!("Connected successfully");
                Ok(())
            }
            Message::Reject(reject) => {
                {
                    let mut state = self.state.write().await;
                    *state = ConnectionState::Disconnected;
                }
                Err(ClientError::Rejected(reject.reason))
            }
            _ => {
                {
                    let mut state = self.state.write().await;
                    *state = ConnectionState::Disconnected;
                }
                Err(ClientError::Connection(
                    "Unexpected response from server".to_string(),
                ))
            }
        }
    }

    /// Execute a command on the server
    pub async fn execute_command(
        &self,
        command: String,
        args: Vec<String>,
    ) -> Result<CommandResponse> {
        // Check connection state
        {
            let state = self.state.read().await;
            if *state != ConnectionState::Connected {
                return Err(ClientError::NotConnected);
            }
        }

        // Check if we have an interface
        let interface = self.interface.as_ref().ok_or_else(|| {
            ClientError::NotConnected
        })?;

        let request_id = self.next_request_id.fetch_add(1, Ordering::SeqCst);

        debug!(
            id = request_id,
            command = %command,
            args = ?args,
            "Executing command"
        );

        let request = CommandRequest {
            id: request_id,
            command,
            args,
            env: None,
            timeout: Some(self.config.command_timeout),
            working_dir: None,
        };

        // Encode and send request
        let message = Message::CommandRequest(request);
        let encoded = ProtocolCodec::encode(&message)?;
        let packet = Packet::data(self.server_destination, encoded);
        interface.send(&packet).await?;

        debug!("Command request sent, waiting for response");

        // Receive response
        let response_packet = interface.receive().await?;
        let mut buf = bytes::BytesMut::from(response_packet.data.as_ref());
        let response_msg = ProtocolCodec::decode(&mut buf)?
            .ok_or_else(|| ClientError::Connection("No response from server".to_string()))?;

        // Handle response
        match response_msg {
            Message::CommandResponse(response) => {
                debug!(
                    id = response.id,
                    exit_code = response.exit_code,
                    "Received command response"
                );
                Ok(response)
            }
            _ => Err(ClientError::Connection(
                "Unexpected response type".to_string(),
            )),
        }
    }

    /// Disconnect from server
    pub async fn disconnect(&self) -> Result<()> {
        {
            let state = self.state.read().await;
            if *state == ConnectionState::Disconnected {
                return Ok(());
            }
        }

        info!("Disconnecting from server");

        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Disconnecting;
        }

        // TODO: Send DISCONNECT message

        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Disconnected;
        }

        {
            let mut session = self.session_id.write().await;
            *session = None;
        }

        info!("Disconnected");

        Ok(())
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        let state = self.state.read().await;
        *state == ConnectionState::Connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let config = ClientConfig::default();
        let client = Client::new(config).await.unwrap();

        assert!(!client.is_connected().await);
    }
}
