//! Protocol framing and serialization

use crate::{Message, ProtocolError, Result};
use bytes::{Buf, BufMut, BytesMut};

/// Current protocol version
pub const CURRENT_PROTOCOL_VERSION: u32 = 1;

/// Maximum message size (1 MB)
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// Protocol version type
pub type ProtocolVersion = u32;

/// Protocol codec for encoding/decoding messages
pub struct ProtocolCodec;

impl ProtocolCodec {
    /// Encode a message into bytes
    ///
    /// Frame format:
    /// ```text
    /// [ 4 bytes: message length (u32, big-endian) ]
    /// [ 1 byte: message type ]
    /// [ N bytes: message payload (bincode-encoded) ]
    /// ```
    pub fn encode(message: &Message) -> Result<Vec<u8>> {
        // Serialize the message
        let payload = bincode::serialize(message)?;

        // Check size limit
        if payload.len() > MAX_MESSAGE_SIZE {
            return Err(ProtocolError::MessageTooLarge {
                size: payload.len(),
                max: MAX_MESSAGE_SIZE,
            });
        }

        // Create frame
        let mut frame = BytesMut::with_capacity(5 + payload.len());

        // Write length (4 bytes)
        frame.put_u32((payload.len() + 1) as u32);

        // Write message type (1 byte)
        frame.put_u8(message.message_type());

        // Write payload
        frame.put_slice(&payload);

        Ok(frame.to_vec())
    }

    /// Decode a message from bytes
    ///
    /// Returns the decoded message and the number of bytes consumed
    pub fn decode(buf: &mut BytesMut) -> Result<Option<Message>> {
        // Need at least 4 bytes for length
        if buf.len() < 4 {
            return Ok(None);
        }

        // Read length without consuming
        let length = {
            let mut length_bytes = &buf[..4];
            length_bytes.get_u32() as usize
        };

        // Check size limit
        if length > MAX_MESSAGE_SIZE {
            return Err(ProtocolError::MessageTooLarge {
                size: length,
                max: MAX_MESSAGE_SIZE,
            });
        }

        // Need full message
        if buf.len() < 4 + length {
            return Ok(None);
        }

        // Consume length bytes
        buf.advance(4);

        // Read message type
        let _message_type = buf.get_u8();

        // Read payload
        let payload_len = length - 1; // Subtract message type byte
        let payload = buf.split_to(payload_len);

        // Deserialize message
        let message: Message = bincode::deserialize(&payload)?;

        Ok(Some(message))
    }

    /// Try to decode multiple messages from a buffer
    pub fn decode_multiple(buf: &mut BytesMut) -> Result<Vec<Message>> {
        let mut messages = Vec::new();

        loop {
            match Self::decode(buf)? {
                Some(msg) => messages.push(msg),
                None => break,
            }
        }

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::CommandRequest;

    #[test]
    fn test_encode_decode() {
        let req = CommandRequest {
            id: 42,
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            env: None,
            timeout: None,
            working_dir: None,
        };

        let msg = Message::CommandRequest(req.clone());

        // Encode
        let encoded = ProtocolCodec::encode(&msg).unwrap();

        // Decode
        let mut buf = BytesMut::from(&encoded[..]);
        let decoded = ProtocolCodec::decode(&mut buf).unwrap().unwrap();

        match decoded {
            Message::CommandRequest(decoded_req) => {
                assert_eq!(decoded_req.id, req.id);
                assert_eq!(decoded_req.command, req.command);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_partial_message() {
        let msg = Message::Ping;
        let encoded = ProtocolCodec::encode(&msg).unwrap();

        // Only send first 2 bytes
        let mut buf = BytesMut::from(&encoded[..2]);
        let result = ProtocolCodec::decode(&mut buf).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_messages() {
        let msg1 = Message::Ping;
        let msg2 = Message::Pong;

        let encoded1 = ProtocolCodec::encode(&msg1).unwrap();
        let encoded2 = ProtocolCodec::encode(&msg2).unwrap();

        let mut buf = BytesMut::new();
        buf.extend_from_slice(&encoded1);
        buf.extend_from_slice(&encoded2);

        let messages = ProtocolCodec::decode_multiple(&mut buf).unwrap();

        assert_eq!(messages.len(), 2);
        assert!(matches!(messages[0], Message::Ping));
        assert!(matches!(messages[1], Message::Pong));
    }

    #[test]
    fn test_message_too_large() {
        // Create a message that's too large
        let large_cmd = CommandRequest {
            id: 1,
            command: "x".repeat(MAX_MESSAGE_SIZE),
            args: vec![],
            env: None,
            timeout: None,
            working_dir: None,
        };

        let msg = Message::CommandRequest(large_cmd);
        let result = ProtocolCodec::encode(&msg);

        assert!(matches!(result, Err(ProtocolError::MessageTooLarge { .. })));
    }
}
