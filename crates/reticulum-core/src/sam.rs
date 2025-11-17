//! SAM v3 (Simple Anonymous Messaging) client for I2P
//!
//! This module implements a lightweight SAM v3 client for connecting to the I2P router.
//! The SAM protocol allows applications to communicate over the I2P network using a
//! simple TCP socket-based interface.
//!
//! Default SAM port: 7656

use crate::{NetworkError, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tracing::{debug, info};

/// Default I2P SAM bridge port
pub const DEFAULT_SAM_PORT: u16 = 7656;

/// SAM protocol version
const SAM_VERSION: &str = "3.1";

/// A connection to the I2P SAM bridge
pub struct SamConnection {
    reader: BufReader<TcpStream>,
}

impl SamConnection {
    /// Connect to the SAM bridge
    pub async fn connect(addr: &str) -> Result<Self> {
        info!("Connecting to SAM bridge at {}", addr);

        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| NetworkError::I2p(format!("Failed to connect to SAM: {}", e)))?;

        let reader = BufReader::new(stream);
        let mut conn = Self { reader };

        // Perform SAM handshake
        conn.handshake().await?;

        Ok(conn)
    }

    /// Perform SAM protocol handshake
    async fn handshake(&mut self) -> Result<()> {
        debug!("Performing SAM handshake");

        // Send HELLO
        let hello = format!("HELLO VERSION MIN={} MAX={}\n", SAM_VERSION, SAM_VERSION);
        self.send_command(&hello).await?;

        // Read response
        let response = self.read_line().await?;
        debug!("SAM handshake response: {}", response);

        if !response.starts_with("HELLO REPLY") {
            return Err(NetworkError::I2p(format!(
                "Unexpected handshake response: {}",
                response
            )));
        }

        if !response.contains("RESULT=OK") {
            return Err(NetworkError::I2p(format!(
                "Handshake failed: {}",
                response
            )));
        }

        info!("SAM handshake successful");
        Ok(())
    }

    /// Generate a new I2P destination
    pub async fn dest_generate(&mut self) -> Result<String> {
        debug!("Generating I2P destination");

        // Use Ed25519 signature type (type 7)
        let command = "DEST GENERATE SIGNATURE_TYPE=7\n";
        self.send_command(command).await?;

        let response = self.read_line().await?;
        debug!("DEST GENERATE response: {}", response);

        if !response.starts_with("DEST REPLY") {
            return Err(NetworkError::I2p(format!(
                "Unexpected DEST GENERATE response: {}",
                response
            )));
        }

        // Extract PRIV from response (contains both public and private keys)
        // Format: DEST REPLY PUB=base64string PRIV=base64string
        // We need PRIV for creating persistent sessions with Emissary
        for part in response.split_whitespace() {
            if let Some(priv_key) = part.strip_prefix("PRIV=") {
                return Ok(priv_key.to_string());
            }
        }

        Err(NetworkError::I2p(
            "Failed to parse destination from response".to_string(),
        ))
    }

    /// Create a DATAGRAM session
    pub async fn session_create_datagram(
        &mut self,
        session_id: &str,
        destination: Option<&str>,
    ) -> Result<()> {
        debug!("Creating DATAGRAM session: {}", session_id);

        let dest_param = match destination {
            Some(d) => format!("DESTINATION={}", d),
            None => "DESTINATION=TRANSIENT".to_string(),
        };

        // Emissary SAM requires PORT and HOST for forwarded datagrams
        // Use port 0 to let the system choose a random port
        let command = format!(
            "SESSION CREATE STYLE=DATAGRAM ID={} {} SIGNATURE_TYPE=7 PORT=0 HOST=127.0.0.1 FROM_PORT=0\n",
            session_id, dest_param
        );

        self.send_command(&command).await?;

        let response = self.read_line().await?;
        debug!("SESSION CREATE response: {}", response);

        if !response.starts_with("SESSION STATUS") {
            return Err(NetworkError::I2p(format!(
                "Unexpected SESSION CREATE response: {}",
                response
            )));
        }

        if !response.contains("RESULT=OK") {
            return Err(NetworkError::I2p(format!(
                "Session creation failed: {}",
                response
            )));
        }

        info!("SAM DATAGRAM session created: {}", session_id);
        Ok(())
    }

    /// Send a datagram
    pub async fn datagram_send(&mut self, session_id: &str, destination: &str, data: &[u8]) -> Result<()> {
        debug!(
            "Sending datagram via session {}, {} bytes",
            session_id,
            data.len()
        );

        // DATAGRAM SEND format:
        // DATAGRAM SEND ID=sessionID DESTINATION=base64dest SIZE=size\n<data>
        let header = format!(
            "DATAGRAM SEND ID={} DESTINATION={} SIZE={}\n",
            session_id,
            destination,
            data.len()
        );

        self.reader
            .get_mut()
            .write_all(header.as_bytes())
            .await
            .map_err(|e| NetworkError::I2p(format!("Failed to send datagram header: {}", e)))?;

        self.reader
            .get_mut()
            .write_all(data)
            .await
            .map_err(|e| NetworkError::I2p(format!("Failed to send datagram data: {}", e)))?;

        self.reader
            .get_mut()
            .flush()
            .await
            .map_err(|e| NetworkError::I2p(format!("Failed to flush datagram: {}", e)))?;

        debug!("Datagram sent");
        Ok(())
    }

    /// Receive a datagram (async)
    /// Returns (source_destination, data)
    pub async fn datagram_receive(&mut self) -> Result<(String, Vec<u8>)> {
        debug!("Waiting for datagram...");

        let response = self.read_line().await?;

        if !response.starts_with("DATAGRAM RECEIVED") {
            return Err(NetworkError::I2p(format!(
                "Unexpected datagram response: {}",
                response
            )));
        }

        // Parse DESTINATION and SIZE from response
        let mut destination = None;
        let mut size = None;

        for part in response.split_whitespace() {
            if let Some(dest) = part.strip_prefix("DESTINATION=") {
                destination = Some(dest.to_string());
            }
            if let Some(s) = part.strip_prefix("SIZE=") {
                size = Some(
                    s.parse::<usize>()
                        .map_err(|_| NetworkError::I2p("Invalid SIZE in datagram".to_string()))?,
                );
            }
        }

        let destination = destination.ok_or_else(|| {
            NetworkError::I2p("Missing DESTINATION in datagram response".to_string())
        })?;

        let size =
            size.ok_or_else(|| NetworkError::I2p("Missing SIZE in datagram response".to_string()))?;

        // Read the data
        let mut data = vec![0u8; size];
        tokio::io::AsyncReadExt::read_exact(&mut self.reader, &mut data)
            .await
            .map_err(|e| NetworkError::I2p(format!("Failed to read datagram data: {}", e)))?;

        debug!("Received datagram from {}, {} bytes", destination, size);

        Ok((destination, data))
    }

    /// Send a command to SAM
    async fn send_command(&mut self, command: &str) -> Result<()> {
        self.reader
            .get_mut()
            .write_all(command.as_bytes())
            .await
            .map_err(|e| NetworkError::I2p(format!("Failed to send SAM command: {}", e)))?;

        self.reader
            .get_mut()
            .flush()
            .await
            .map_err(|e| NetworkError::I2p(format!("Failed to flush SAM command: {}", e)))?;

        Ok(())
    }

    /// Read a line from SAM
    async fn read_line(&mut self) -> Result<String> {
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .await
            .map_err(|e| NetworkError::I2p(format!("Failed to read SAM response: {}", e)))?;

        Ok(line.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires I2P router running
    async fn test_sam_connection() {
        let conn = SamConnection::connect("127.0.0.1:7656").await;
        match conn {
            Ok(_) => println!("SAM connection successful"),
            Err(e) => println!("SAM connection failed (expected if I2P not running): {}", e),
        }
    }

    #[tokio::test]
    #[ignore] // Requires I2P router running
    async fn test_dest_generate() {
        let mut conn = SamConnection::connect("127.0.0.1:7656").await.unwrap();
        let dest = conn.dest_generate().await.unwrap();
        println!("Generated destination: {}", dest);
        assert!(!dest.is_empty());
    }
}
