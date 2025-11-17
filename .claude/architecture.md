# Architecture & Design Decisions

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Reticulum Network                        │
│                      (over I2P)                              │
└─────────────────────────────────────────────────────────────┘
              ↑                            ↑
              │                            │
    ┌─────────┴─────────┐      ┌──────────┴──────────┐
    │   Shell Server    │      │    Shell Client     │
    │  (Listener/       │      │   (Connector/       │
    │   Executor)       │      │    REPL)            │
    └─────────┬─────────┘      └──────────┬──────────┘
              │                            │
              │                            │
    ┌─────────┴─────────┐      ┌──────────┴──────────┐
    │  Reticulum Core   │      │  Reticulum Core     │
    │   - Identity      │      │   - Identity        │
    │   - I2P Transport │      │   - I2P Transport   │
    │   - Packet Mgmt   │      │   - Packet Mgmt     │
    └───────────────────┘      └─────────────────────┘
```

## Crate Architecture

### Cargo Workspace Structure

**reticulum-shell/** (workspace root)
- `reticulum-core`: Core networking library (shared)
- `shell-proto`: Protocol definitions (shared)
- `shell-server`: Server binary
- `shell-client`: Client binary

### Dependency Flow

```
shell-server  →  shell-proto  →  reticulum-core
     ↓               ↓
shell-client ────────┘
```

## Key Design Decisions

### 1. Language: Rust

**Rationale:**
- Memory safety without garbage collection
- Excellent async/await support via Tokio
- Strong type system prevents protocol errors
- Zero-cost abstractions for performance
- Growing security tools ecosystem

**Alternatives Considered:**
- Python: Rejected due to performance and deployment complexity
- Go: Rejected due to preference for Rust's safety guarantees
- C/C++: Rejected due to memory safety concerns

### 2. Shell Model: Command Execution (Not Interactive PTY)

**MVP Approach:**
- Client sends complete commands
- Server executes and returns output (stdout/stderr)
- Simpler to implement and debug
- Sufficient for most remote administration tasks

**Future Enhancement:**
- Interactive PTY support for full shell sessions
- Requires pseudo-terminal allocation and handling
- Added complexity for terminal emulation

**Trade-offs:**
- ✅ Simpler protocol
- ✅ Easier error handling
- ✅ Better logging and audit trail
- ❌ No interactive programs (vim, top, etc.)
- ❌ No job control

### 3. Network Stack: Reticulum over I2P

**Reticulum Benefits:**
- Built-in encryption and authentication
- Identity-based routing (no IP addresses)
- Resilient mesh networking
- Perfect for anonymous communications

**I2P Benefits:**
- Anonymous routing layer
- Resistant to traffic analysis
- Decentralized infrastructure
- Suitable for security research

**Integration Strategy:**
- Use I2P as transport interface for Reticulum
- Reticulum handles packet routing and crypto
- I2P handles anonymity and anti-surveillance

**I2P Router Options:**

1. **Embedded Router (Emissary)** - Feature: `embedded-router`
   - Pure Rust I2P implementation
   - Zero external dependencies
   - Automatic HTTPS reseeding for bootstrap
   - Single-binary deployment
   - 64-256 MB memory footprint
   - First run: 2-5 minutes (reseed + tunnel building)
   - Subsequent runs: 30-90 seconds

2. **External Router (i2pd/Java I2P)**
   - Traditional SAM-based integration
   - Requires separate I2P router process
   - Suitable for shared I2P usage
   - Lower startup overhead (router already running)

### 4. Async Runtime: Tokio

**Rationale:**
- Industry standard for Rust async
- Excellent performance characteristics
- Rich ecosystem of async libraries
- Built-in timers, channels, and sync primitives

### 5. Serialization: Bincode + Serde

**Rationale:**
- Compact binary format (smaller than JSON/MessagePack)
- Fast serialization/deserialization
- Strong typing via Serde
- Easy version evolution

**Alternatives Considered:**
- Protocol Buffers: More complex, unnecessary overhead
- MessagePack: Larger message sizes
- JSON: Too verbose for binary protocol

### 6. Authentication: Reticulum Native Identities

**Rationale:**
- Built into Reticulum protocol
- Ed25519 cryptographic signatures
- No separate auth layer needed
- Automatic key management

**Security Properties:**
- Server verifies client identity
- Client verifies server identity
- Prevents MITM attacks
- Perfect forward secrecy (depending on Reticulum implementation)

## Protocol Design

### Message Flow

```
1. Connection Establishment
   Client → Server: CONNECT (with identity)
   Server → Client: ACCEPT or REJECT

2. Command Execution
   Client → Server: COMMAND_REQUEST {id, cmd, args}
   Server → Client: COMMAND_RESPONSE {id, stdout, stderr, exit_code}

3. Session Management
   Client → Server: DISCONNECT
   Server → Client: ACK
```

### Message Types (shell-proto)

```rust
enum Message {
    Connect { client_identity: Identity },
    Accept { server_identity: Identity, session_id: SessionId },
    Reject { reason: String },
    CommandRequest { id: u64, command: String, args: Vec<String>, env: Option<HashMap> },
    CommandResponse { id: u64, stdout: Vec<u8>, stderr: Vec<u8>, exit_code: i32 },
    Disconnect,
    Ack,
}
```

## Security Model

### Threat Model

**In Scope:**
- Network eavesdropping (mitigated by Reticulum encryption + I2P)
- Unauthorized access (mitigated by identity-based auth)
- Command injection (mitigated by argument separation)
- Resource exhaustion (mitigated by timeouts and limits)

**Out of Scope (for MVP):**
- Compromised endpoints (OS-level security)
- Advanced persistence mechanisms
- Anti-forensics capabilities

### Security Controls

1. **Authentication:** Reticulum identity verification
2. **Encryption:** End-to-end via Reticulum
3. **Anonymity:** I2P transport layer
4. **Input Validation:** Command argument sanitization
5. **Resource Limits:** Execution timeouts, memory caps
6. **Audit Logging:** All commands logged with timestamps
7. **Privilege Separation:** Server runs with minimal privileges

## Error Handling Strategy

**Pattern:** Thiserror for custom error types

```rust
#[derive(Error, Debug)]
enum ShellError {
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Command execution failed: {0}")]
    Execution(String),

    #[error("Authentication failed: {0}")]
    Auth(String),
}
```

**Propagation:** Use `Result<T, ShellError>` throughout, handle at boundaries

## Testing Strategy

1. **Unit Tests:** Each module tests internal logic
2. **Integration Tests:** Cross-crate interactions
3. **End-to-End Tests:** Full server-client scenarios
4. **Security Tests:** Fuzzing, invalid inputs, resource exhaustion

## Performance Considerations

- Async I/O for concurrent connections
- Zero-copy where possible (bytes instead of strings)
- Connection pooling for I2P circuits
- Lazy initialization of resources

## Future Extensibility

### Phase 2 Enhancements
- Interactive PTY support
- File transfer (upload/download)
- Port forwarding
- Multiple concurrent sessions

### Phase 3 Enhancements
- Multi-platform support (Windows, macOS)
- Plugin system for commands
- Advanced persistence mechanisms
- Configurable command restrictions
