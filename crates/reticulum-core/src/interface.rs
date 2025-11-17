//! Network interface abstraction
//!
//! This module provides an abstraction layer for different transport mechanisms
//! (I2P, TCP, UDP, etc.)

use crate::{Packet, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Network interface trait
///
/// Implementations provide transport over different media (I2P, TCP, UDP, etc.)
#[async_trait]
pub trait NetworkInterface: Send + Sync {
    /// Send a packet through this interface
    async fn send(&self, packet: &Packet) -> Result<()>;

    /// Receive a packet from this interface
    async fn receive(&self) -> Result<Packet>;

    /// Get the interface name
    fn name(&self) -> &str;

    /// Check if interface is ready
    async fn is_ready(&self) -> bool;

    /// Close the interface
    async fn close(&self) -> Result<()>;
}

// Mock interface for local testing
/// Mock network interface using in-memory channels
/// This allows testing the full message flow without I2P
pub struct MockInterface {
    name: String,
    rx: Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<Packet>>>,
    tx: tokio::sync::mpsc::UnboundedSender<Packet>,
}

impl MockInterface {
    /// Create a pair of mock interfaces (client and server)
    pub fn create_pair() -> (Self, Self) {
        let (client_tx, server_rx) = tokio::sync::mpsc::unbounded_channel();
        let (server_tx, client_rx) = tokio::sync::mpsc::unbounded_channel();

        let client = Self {
            name: "mock-client".to_string(),
            rx: Arc::new(Mutex::new(client_rx)),
            tx: client_tx,
        };

        let server = Self {
            name: "mock-server".to_string(),
            rx: Arc::new(Mutex::new(server_rx)),
            tx: server_tx,
        };

        (client, server)
    }
}

#[async_trait]
impl NetworkInterface for MockInterface {
    async fn send(&self, packet: &Packet) -> Result<()> {
        self.tx
            .send(packet.clone())
            .map_err(|_| crate::NetworkError::Connection("Send failed".to_string()))?;
        Ok(())
    }

    async fn receive(&self) -> Result<Packet> {
        let mut rx = self.rx.lock().await;
        rx.recv()
            .await
            .ok_or_else(|| crate::NetworkError::Connection("Channel closed".to_string()))
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn is_ready(&self) -> bool {
        true
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

/// I2P network interface using SAM protocol
pub struct I2pInterface {
    name: String,
    sam_conn: Arc<Mutex<crate::sam::SamConnection>>,
    session_id: String,
    local_destination: String,
    /// Map 32-byte hashes to full I2P destinations
    destination_map: Arc<Mutex<std::collections::HashMap<[u8; 32], String>>>,
}

impl I2pInterface {
    /// Create a new I2P interface
    pub async fn new(sam_addr: &str) -> Result<Self> {
        use sha2::{Digest, Sha256};

        tracing::info!("Connecting to I2P SAM bridge at {}", sam_addr);

        let mut sam = crate::sam::SamConnection::connect(sam_addr).await?;

        // Generate I2P destination (returns PRIV key with both public and private)
        let destination = sam.dest_generate().await?;
        tracing::info!("Generated I2P destination: {}...", &destination[..20]);

        // Create session ID
        let session_id = format!("retic-{}", uuid::Uuid::new_v4());

        // Create DATAGRAM session with the generated destination
        sam.session_create_datagram(&session_id, Some(&destination)).await?;

        // Compute our own destination hash
        let mut hasher = Sha256::new();
        hasher.update(destination.as_bytes());
        let hash: [u8; 32] = hasher.finalize().into();

        let mut dest_map = std::collections::HashMap::new();
        dest_map.insert(hash, destination.clone());

        Ok(Self {
            name: "i2p".to_string(),
            sam_conn: Arc::new(Mutex::new(sam)),
            session_id,
            local_destination: destination,
            destination_map: Arc::new(Mutex::new(dest_map)),
        })
    }

    /// Create a new I2P interface connected to an embedded router
    #[cfg(feature = "embedded-router")]
    pub async fn new_embedded(router: &crate::EmbeddedRouter) -> Result<Self> {
        let sam_addr = router
            .sam_address()
            .ok_or_else(|| crate::NetworkError::I2p("SAM not enabled in embedded router".to_string()))?;

        tracing::info!("Connecting to embedded router SAM at {}", sam_addr);

        // Wait a moment for SAM server to be fully ready
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Self::new(&sam_addr).await
    }

    /// Register an I2P destination (map hash to full destination)
    pub async fn register_destination(&self, i2p_dest: String) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(i2p_dest.as_bytes());
        let hash: [u8; 32] = hasher.finalize().into();

        let mut map = self.destination_map.lock().await;
        map.insert(hash, i2p_dest);

        hash
    }

    /// Get the local I2P destination
    pub fn local_destination(&self) -> &str {
        &self.local_destination
    }

    /// Get the local destination hash
    pub fn local_destination_hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(self.local_destination.as_bytes());
        hasher.finalize().into()
    }
}

#[async_trait]
impl NetworkInterface for I2pInterface {
    async fn send(&self, packet: &Packet) -> Result<()> {
        use tracing::debug;

        // Look up the full I2P destination from the hash
        let dest_map = self.destination_map.lock().await;
        let i2p_dest = dest_map.get(&packet.destination)
            .ok_or_else(|| crate::NetworkError::I2p(
                "Unknown destination - not registered".to_string()
            ))?;

        debug!("Sending packet to I2P destination: {}...", &i2p_dest[..20]);

        // Encode the packet
        let encoded = packet.encode();

        // Send via SAM
        let mut sam = self.sam_conn.lock().await;
        sam.datagram_send(&self.session_id, i2p_dest, &encoded).await?;

        Ok(())
    }

    async fn receive(&self) -> Result<Packet> {
        use sha2::{Digest, Sha256};
        use tracing::debug;

        // Receive datagram via SAM
        let (source_dest, data) = {
            let mut sam = self.sam_conn.lock().await;
            sam.datagram_receive().await?
        };

        debug!("Received packet from I2P destination: {}...", &source_dest[..20]);

        // Hash the source destination to create the 32-byte identifier
        let mut hasher = Sha256::new();
        hasher.update(source_dest.as_bytes());
        let source_hash: [u8; 32] = hasher.finalize().into();

        // Register this destination for future sends
        {
            let mut dest_map = self.destination_map.lock().await;
            dest_map.insert(source_hash, source_dest);
        }

        // Decode the packet
        Packet::decode(&data)
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn is_ready(&self) -> bool {
        true // If we constructed successfully, we're ready
    }

    async fn close(&self) -> Result<()> {
        tracing::info!("Closing I2P interface");
        // SAM connection will be dropped automatically
        Ok(())
    }
}
