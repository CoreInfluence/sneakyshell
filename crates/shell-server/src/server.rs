//! Main server implementation

use crate::{config::ServerConfig, listener::Listener, session::Session, Result, ServerError};
use reticulum_core::{NetworkInterface, Packet};
use shell_proto::{ProtocolCodec, SessionId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// The main server
pub struct Server {
    /// Server configuration
    config: Arc<ServerConfig>,

    /// Connection listener
    listener: Arc<Listener>,

    /// Network interface
    interface: Option<Arc<dyn NetworkInterface>>,

    /// Active sessions
    sessions: Arc<RwLock<HashMap<SessionId, Arc<Session>>>>,
}

impl Server {
    /// Create a new server
    pub async fn new(config: ServerConfig) -> Result<Self> {
        let listener = Arc::new(Listener::new(config.clone()));

        Ok(Self {
            config: Arc::new(config),
            listener,
            interface: None,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a server with a specific network interface (for testing)
    pub async fn with_interface(
        config: ServerConfig,
        interface: Arc<dyn NetworkInterface>,
    ) -> Result<Self> {
        let listener = Arc::new(Listener::new(config.clone()));

        Ok(Self {
            config: Arc::new(config),
            listener,
            interface: Some(interface),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Run the server
    pub async fn run(self) -> Result<()> {
        info!("Server starting...");
        info!("Destination: {}", self.config.identity.destination_hex());

        // Check if we have a network interface
        if let Some(ref interface) = self.interface {
            info!("Running with network interface: {}", interface.name());

            // Clone the Arc for the message loop
            let interface_clone = Arc::clone(interface);

            // Run message loop and wait for shutdown signal concurrently
            tokio::select! {
                result = self.message_loop(interface_clone) => {
                    if let Err(e) = result {
                        error!("Message loop error: {}", e);
                        return Err(e);
                    }
                }
                result = signal::ctrl_c() => {
                    match result {
                        Ok(()) => info!("Shutdown signal received"),
                        Err(err) => {
                            error!("Error waiting for shutdown signal: {}", err);
                            return Err(ServerError::Io(err));
                        }
                    }
                }
            }
        } else {
            warn!("No network interface configured - server will wait for Ctrl+C");
            info!("Server running. Press Ctrl+C to stop.");

            // Wait for shutdown signal
            match signal::ctrl_c().await {
                Ok(()) => {
                    info!("Shutdown signal received");
                }
                Err(err) => {
                    error!("Error waiting for shutdown signal: {}", err);
                    return Err(ServerError::Io(err));
                }
            }
        }

        info!("Server shutting down...");
        self.shutdown().await?;

        Ok(())
    }

    /// Message processing loop
    async fn message_loop(&self, interface: Arc<dyn NetworkInterface>) -> Result<()> {
        info!("Message loop started");

        loop {
            // Receive packet from network
            let packet = match interface.receive().await {
                Ok(p) => p,
                Err(e) => {
                    warn!("Error receiving packet: {}", e);
                    continue;
                }
            };

            debug!(
                destination = %hex::encode(&packet.destination),
                data_len = packet.data.len(),
                has_signature = packet.signature.is_some(),
                "Received packet"
            );

            // Try to decode as protocol message
            let mut buf = bytes::BytesMut::from(packet.data.as_ref());
            let messages = match ProtocolCodec::decode_multiple(&mut buf) {
                Ok(msgs) => msgs,
                Err(e) => {
                    warn!("Failed to decode packet as protocol message: {}", e);
                    continue;
                }
            };

            // Process each message
            for message in messages {
                use shell_proto::Message;

                let response = match message {
                    Message::Connect(ref connect) => {
                        debug!("Handling CONNECT message");

                        // Handle connection and get response
                        let response = self.listener.handle_connection(Message::Connect(connect.clone())).await?;

                        // If connection accepted, create and store session
                        if let Message::Accept(ref accept) = response {
                            debug!("Connection accepted, creating session");

                            let session = Arc::new(Session::new(
                                connect.client_identity.clone(),
                                self.listener.executor(),
                            ));

                            let mut sessions = self.sessions.write().await;
                            sessions.insert(accept.session_id, session);

                            info!(
                                session_id = %hex::encode(&accept.session_id),
                                client = %hex::encode(&connect.client_identity),
                                "Client connected - new session created"
                            );
                        }

                        response
                    }

                    Message::CommandRequest(_) | Message::Disconnect(_) | Message::Ping => {
                        debug!("Handling session message");

                        // For session messages, we need to find the session
                        // For now, use the first session (simplification for MVP)
                        let sessions = self.sessions.read().await;
                        if let Some((session_id, session)) = sessions.iter().next() {
                            debug!(session_id = %hex::encode(session_id), "Routing to session");

                            match session.handle_message(message).await? {
                                Some(msg) => msg,
                                None => {
                                    warn!("Session returned no response");
                                    continue;
                                }
                            }
                        } else {
                            warn!("No active sessions, dropping message");
                            continue;
                        }
                    }

                    _ => {
                        warn!("Unexpected message type in server loop");
                        continue;
                    }
                };

                debug!("Sending response");

                // Encode response
                let response_bytes = ProtocolCodec::encode(&response)?;

                // Send response packet
                let response_packet = Packet::data(packet.destination, response_bytes);
                interface.send(&response_packet).await?;

                debug!("Response sent");
            }
        }
    }

    /// Shutdown the server
    async fn shutdown(&self) -> Result<()> {
        info!("Closing active sessions...");
        // TODO: Close all active sessions
        info!("Server shutdown complete");
        Ok(())
    }
}
