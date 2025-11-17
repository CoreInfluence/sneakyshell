//! Reticulum packet structures

use crate::{DestinationHash, NetworkError, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};

/// Packet type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PacketType {
    /// Data packet
    Data = 0x00,

    /// Announce packet (destination announcement)
    Announce = 0x01,

    /// Link request
    LinkRequest = 0x02,

    /// Link response
    LinkResponse = 0x03,

    /// Proof packet
    Proof = 0x04,
}

impl PacketType {
    /// Convert from byte
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0x00 => Ok(PacketType::Data),
            0x01 => Ok(PacketType::Announce),
            0x02 => Ok(PacketType::LinkRequest),
            0x03 => Ok(PacketType::LinkResponse),
            0x04 => Ok(PacketType::Proof),
            _ => Err(NetworkError::Packet(format!("Invalid packet type: {}", value))),
        }
    }
}

/// A Reticulum packet
#[derive(Debug, Clone)]
pub struct Packet {
    /// Packet type
    pub packet_type: PacketType,

    /// Destination hash
    pub destination: DestinationHash,

    /// Payload data
    pub data: Bytes,

    /// Optional signature
    pub signature: Option<Vec<u8>>,
}

impl Packet {
    /// Create a new packet
    pub fn new(packet_type: PacketType, destination: DestinationHash, data: Vec<u8>) -> Self {
        Self {
            packet_type,
            destination,
            data: Bytes::from(data),
            signature: None,
        }
    }

    /// Create a data packet
    pub fn data(destination: DestinationHash, payload: Vec<u8>) -> Self {
        Self::new(PacketType::Data, destination, payload)
    }

    /// Create an announce packet
    pub fn announce(destination: DestinationHash, payload: Vec<u8>) -> Self {
        Self::new(PacketType::Announce, destination, payload)
    }

    /// Add signature to packet
    pub fn with_signature(mut self, signature: Vec<u8>) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Encode packet to bytes
    ///
    /// Format:
    /// ```text
    /// [ 1 byte: packet type ]
    /// [ 32 bytes: destination hash ]
    /// [ 2 bytes: data length (u16, big-endian) ]
    /// [ N bytes: data ]
    /// [ 1 byte: signature flag (0x00 or 0x01) ]
    /// [ 64 bytes: signature (if flag is 0x01) ]
    /// ```
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();

        // Packet type
        buf.put_u8(self.packet_type as u8);

        // Destination
        buf.put_slice(&self.destination);

        // Data length and data
        buf.put_u16(self.data.len() as u16);
        buf.put_slice(&self.data);

        // Signature
        if let Some(sig) = &self.signature {
            buf.put_u8(0x01); // Signature present
            buf.put_slice(sig);
        } else {
            buf.put_u8(0x00); // No signature
        }

        buf.to_vec()
    }

    /// Decode packet from bytes
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 35 {
            // Minimum: type(1) + dest(32) + len(2)
            return Err(NetworkError::Packet("Packet too short".to_string()));
        }

        let mut buf = &data[..];

        // Read packet type
        let packet_type = PacketType::from_u8(buf.get_u8())?;

        // Read destination
        let mut destination = [0u8; 32];
        buf.copy_to_slice(&mut destination);

        // Read data length
        let data_len = buf.get_u16() as usize;

        if buf.len() < data_len + 1 {
            return Err(NetworkError::Packet("Invalid data length".to_string()));
        }

        // Read data
        let payload = buf.copy_to_bytes(data_len);

        // Read signature flag
        let sig_flag = buf.get_u8();

        let signature = if sig_flag == 0x01 {
            if buf.len() < 64 {
                return Err(NetworkError::Packet("Invalid signature length".to_string()));
            }
            let mut sig = vec![0u8; 64];
            buf.copy_to_slice(&mut sig);
            Some(sig)
        } else {
            None
        };

        Ok(Self {
            packet_type,
            destination,
            data: payload,
            signature,
        })
    }

    /// Get the signable portion of the packet (for verification)
    pub fn signable_data(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();
        buf.put_u8(self.packet_type as u8);
        buf.put_slice(&self.destination);
        buf.put_u16(self.data.len() as u16);
        buf.put_slice(&self.data);
        buf.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_encode_decode() {
        let destination = [42u8; 32];
        let data = b"Hello, Reticulum!".to_vec();

        let packet = Packet::data(destination, data.clone());
        let encoded = packet.encode();
        let decoded = Packet::decode(&encoded).unwrap();

        assert_eq!(decoded.packet_type, PacketType::Data);
        assert_eq!(decoded.destination, destination);
        assert_eq!(decoded.data.as_ref(), data.as_slice());
        assert!(decoded.signature.is_none());
    }

    #[test]
    fn test_packet_with_signature() {
        let destination = [42u8; 32];
        let data = b"Test data".to_vec();
        let signature = vec![0xAB; 64];

        let packet = Packet::data(destination, data.clone()).with_signature(signature.clone());
        let encoded = packet.encode();
        let decoded = Packet::decode(&encoded).unwrap();

        assert_eq!(decoded.signature, Some(signature));
    }

    #[test]
    fn test_packet_types() {
        let dest = [0u8; 32];

        let announce = Packet::announce(dest, vec![]);
        assert_eq!(announce.packet_type, PacketType::Announce);

        let data = Packet::data(dest, vec![]);
        assert_eq!(data.packet_type, PacketType::Data);
    }
}
