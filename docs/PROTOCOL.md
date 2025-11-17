# Protocol Specification

## Overview

The Reticulum-Shell protocol defines the wire format and message exchange patterns for remote shell access over the Reticulum network.

**Protocol Version:** 1
**Transport:** Reticulum over I2P
**Serialization:** Bincode (binary)
**Encoding:** Big-endian

## Message Framing

All messages use a length-prefixed frame format:

```
┌────────────┬───────────────┬─────────────────┐
│  Length    │  Type         │   Payload       │
│  (4 bytes) │  (1 byte)     │   (N bytes)     │
│  u32 BE    │  u8           │   bincode       │
└────────────┴───────────────┴─────────────────┘
```

**Fields:**
- **Length**: Total message size (type + payload), big-endian u32
- **Type**: Message type identifier (see table below)
- **Payload**: Bincode-serialized message structure

**Constraints:**
- Maximum message size: 1 MB (1,048,576 bytes)
- Minimum message size: 5 bytes (length + type + empty payload)

## Message Types

| Type | Code | Direction | Description |
|------|------|-----------|-------------|
| CONNECT | `0x01` | Client → Server | Connection request |
| ACCEPT | `0x02` | Server → Client | Connection accepted |
| REJECT | `0x03` | Server → Client | Connection rejected |
| COMMAND_REQUEST | `0x10` | Client → Server | Execute command |
| COMMAND_RESPONSE | `0x11` | Server → Client | Command result |
| DISCONNECT | `0x20` | Either | Graceful disconnect |
| ACK | `0x21` | Either | Acknowledgment |
| PING | `0x30` | Either | Keep-alive ping |
| PONG | `0x31` | Either | Keep-alive response |

## Connection Phase

### 1. CONNECT

Client initiates connection with identity and capabilities.

**Type:** `0x01`

**Payload:**
```rust
struct ConnectMessage {
    protocol_version: u32,        // Must be 1
    client_identity: Vec<u8>,     // Ed25519 public key (32 bytes)
    capabilities: Vec<String>,    // Client capabilities
    auth_token: Option<String>,   // Optional auth token
}
```

**Example:**
```
protocol_version: 1
client_identity: [0x12, 0x34, ..., 0xAB] (32 bytes)
capabilities: ["command-exec"]
auth_token: None
```

### 2. ACCEPT

Server accepts connection and provides session ID.

**Type:** `0x02`

**Payload:**
```rust
struct AcceptMessage {
    protocol_version: u32,        // Protocol version used
    server_identity: Vec<u8>,     // Ed25519 public key (32 bytes)
    session_id: [u8; 16],        // Unique session identifier
    capabilities: Vec<String>,    // Server capabilities
}
```

### 3. REJECT

Server rejects connection with reason.

**Type:** `0x03`

**Payload:**
```rust
struct RejectMessage {
    reason: String,              // Human-readable reason
    error_code: u32,            // Numeric error code
}
```

**Error Codes:**
- `1` - Invalid message format
- `2` - Protocol version mismatch
- `3` - Authentication failed
- `4` - Maximum sessions reached
- `5` - Server shutting down

## Command Execution Phase

### 4. COMMAND_REQUEST

Client requests command execution.

**Type:** `0x10`

**Payload:**
```rust
struct CommandRequest {
    id: u64,                              // Unique request ID
    command: String,                      // Command to execute
    args: Vec<String>,                    // Command arguments
    env: Option<HashMap<String, String>>, // Environment variables
    timeout: Option<u64>,                 // Timeout in seconds
    working_dir: Option<String>,          // Working directory
}
```

**Example:**
```rust
CommandRequest {
    id: 42,
    command: "ls",
    args: ["-la", "/tmp"],
    env: Some({"PATH": "/usr/bin"}),
    timeout: Some(30),
    working_dir: Some("/home/user"),
}
```

**Constraints:**
- `command` must not be empty
- `args` must not contain null bytes
- `working_dir` must not contain `..` (path traversal protection)
- `timeout` defaults to server configuration if None

### 5. COMMAND_RESPONSE

Server responds with command results.

**Type:** `0x11`

**Payload:**
```rust
struct CommandResponse {
    id: u64,                    // Matches request ID
    status: CommandStatus,      // Execution status
    stdout: Vec<u8>,           // Standard output (raw bytes)
    stderr: Vec<u8>,           // Standard error (raw bytes)
    exit_code: i32,            // Process exit code
    execution_time_ms: u64,    // Execution time in milliseconds
}

enum CommandStatus {
    Success,    // Exit code 0
    Error,      // Non-zero exit code
    Timeout,    // Execution timed out
    Killed,     // Process was killed
}
```

**Notes:**
- `stdout` and `stderr` are raw bytes (may not be UTF-8)
- `exit_code` is -1 for timeout/killed
- Client matches response to request using `id` field

## Session Management

### 6. DISCONNECT

Either side can initiate graceful disconnect.

**Type:** `0x20`

**Payload:**
```rust
struct DisconnectMessage {
    reason: Option<String>,     // Optional reason
}
```

**Flow:**
```
Client → Server: DISCONNECT
Server → Client: ACK
```

### 7. ACK

Acknowledge received message.

**Type:** `0x21`

**Payload:**
```rust
struct AckMessage {
    message_id: u64,           // ID of acknowledged message
}
```

## Keep-Alive

### 8. PING / PONG

**PING Type:** `0x30`
**PONG Type:** `0x31`

**Payload:** None (empty)

**Purpose:** Detect connection loss

**Flow:**
```
Client → Server: PING
Server → Client: PONG
```

**Timing:**
- Send PING every 60 seconds of inactivity
- Expect PONG within 10 seconds
- Disconnect after 3 failed PINGs

## Protocol Flow

### Successful Session

```
Client                          Server
  │                               │
  │──── CONNECT ─────────────────>│
  │                               │ (verify identity)
  │<──────────────────── ACCEPT ──│
  │                               │
  │──── COMMAND_REQUEST ─────────>│
  │  (id=1, cmd="whoami")         │ (execute)
  │<─────── COMMAND_RESPONSE ─────│
  │  (id=1, stdout="root\n")      │
  │                               │
  │──── COMMAND_REQUEST ─────────>│
  │  (id=2, cmd="ls")             │ (execute)
  │<─────── COMMAND_RESPONSE ─────│
  │  (id=2, stdout="...", exit=0) │
  │                               │
  │──── DISCONNECT ──────────────>│
  │<──────────────────────── ACK ─│
  │                               │
```

### Rejected Connection

```
Client                          Server
  │                               │
  │──── CONNECT ─────────────────>│
  │  (version=999)                │ (check version)
  │<──────────────────── REJECT ──│
  │  (reason="version mismatch")  │
  │                               │
  [Connection closed]
```

### Command Timeout

```
Client                          Server
  │                               │
  │──── COMMAND_REQUEST ─────────>│
  │  (id=10, cmd="sleep 999")     │ (execute with timeout)
  │                               │
  │<─────── COMMAND_RESPONSE ─────│
  │  (id=10, status=Timeout)      │
  │                               │
```

## Security Considerations

### Authentication

1. **Identity Verification:**
   - All messages signed with Ed25519 private key
   - Recipient verifies signature using public key
   - Prevents impersonation and MITM attacks

2. **Session Binding:**
   - Session ID tied to client identity
   - Cannot be hijacked or replayed

### Encryption

- **Reticulum Layer:** End-to-end encryption of all packets
- **I2P Layer:** Anonymous routing with garlic encryption
- **Combined:** Defense-in-depth approach

### Input Validation

**Server MUST validate:**
- Command is not empty
- Arguments do not contain malicious patterns
- Working directory does not contain `..`
- Environment variables are reasonable

**Client SHOULD:**
- Sanitize user input before sending
- Validate server responses

### Resource Limits

**Server enforces:**
- Maximum message size (1 MB)
- Command execution timeout
- Maximum concurrent sessions
- Rate limiting (future)

## Error Handling

### Malformed Messages

If a message cannot be parsed:
1. Log the error
2. Send REJECT or disconnect
3. Do not crash

### Network Errors

- Connection loss: Detect via PING/PONG
- Timeout: Return CommandResponse with Timeout status
- I/O errors: Log and disconnect gracefully

### Protocol Violations

Examples:
- Wrong message type in sequence
- Invalid protocol version
- Message too large

**Action:** Send REJECT with appropriate error code and disconnect

## Version Negotiation

**Current Version:** 1

**Future Versions:**
- Client sends highest supported version
- Server responds with version it will use
- Must be ≤ client's version
- Backward compatibility via feature detection

**Capabilities:**
- `"command-exec"` - Basic command execution
- `"file-transfer"` - File upload/download (future)
- `"pty"` - Interactive PTY (future)
- `"port-forward"` - Port forwarding (future)

## Extensions

Future protocol extensions will be added via capabilities negotiation without breaking existing clients.

**Planned:**
- File transfer messages (`0x40-0x4F`)
- PTY control messages (`0x50-0x5F`)
- Port forwarding messages (`0x60-0x6F`)

## Reference Implementation

See `crates/shell-proto/` for the Rust reference implementation.
