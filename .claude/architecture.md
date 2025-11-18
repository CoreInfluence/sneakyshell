# Architecture & Design Decisions

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│              shell-client / shell-server                     │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Reticulum Protocol                         │
│  ┌─────────┐  ┌──────────┐  ┌─────────┐  ┌───────────────┐  │
│  │ Identity │  │   Link   │  │ Packet  │  │   Transport   │  │
│  │ X25519   │  │ Channels │  │ Routing │  │  Path Table   │  │
│  │ Ed25519  │  │ Forward  │  │ Announce│  │  Link Table   │  │
│  │ Ratchet  │  │ Secrecy  │  │ Proof   │  │  Routing      │  │
│  └─────────┘  └──────────┘  └─────────┘  └───────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Transport Interfaces                       │
│    ┌─────┐    ┌─────┐    ┌───────┐    ┌───────────────┐     │
│    │ I2P │    │ TCP │    │  UDP  │    │  Local (IPC)  │     │
│    │ SAM │    │HDLC │    │       │    │  Unix Socket  │     │
│    └─────┘    └─────┘    └───────┘    └───────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

- **Identity:** X25519 (encryption) + Ed25519 (signatures) dual-keypair system
- **Link:** Encrypted bidirectional channels with forward secrecy via ratchet keys
- **Packet:** Wire format serialization, all packet types (DATA, ANNOUNCE, LINKREQUEST, PROOF)
- **Transport:** Path table, link table, routing, announce propagation
- **Interfaces:** Pluggable transports (I2P primary, TCP/UDP planned)

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

### 3. Network Stack: Full Reticulum Protocol

**Full Rust Implementation:**
This project implements the complete Reticulum network protocol in Rust, aiming for wire-format compatibility with the Python reference implementation.

**Reticulum Protocol Features:**
- X25519 + Ed25519 dual-keypair identities
- ECIES encryption with Token cipher (AES-256-CBC + HMAC)
- Link-based channels with forward secrecy via ratchet keys
- 3-packet handshake for link establishment
- Announce/path discovery for mesh routing
- Resource transfer system for large data
- 500-byte network MTU

**I2P as Primary Transport:**
- Anonymous routing layer
- Resistant to traffic analysis
- Decentralized infrastructure
- Suitable for security research

**Integration Strategy:**
- Implement full Reticulum protocol stack in reticulum-core
- I2P wrapped in Reticulum Interface trait
- Future transports (TCP, UDP) also use Interface trait
- Wire-compatible with Python Reticulum network

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

### 6. Authentication: Reticulum Dual-Keypair Identities

**Rationale:**
- Full Reticulum identity system with two keypairs per identity
- X25519 for encryption and key exchange (ECDH)
- Ed25519 for digital signatures and authentication
- 128-bit truncated hash addressing
- Ratchet keys for forward secrecy

**Identity Structure:**
```rust
pub struct Identity {
    pub x25519_public: [u8; 32],   // Encryption
    pub ed25519_public: [u8; 32],  // Signatures
    pub hash: [u8; 16],            // Truncated address
}
```

**Security Properties:**
- Mutual identity verification via signatures
- ECIES encryption for single destinations
- Link-based forward secrecy via ECDH + HKDF
- Ratchet keys rotated every 30 minutes
- Prevents MITM attacks via cryptographic proofs

## Protocol Design

### Reticulum Link-Based Communication

Shell commands are sent over established **Reticulum Links** for forward secrecy:

```
1. Link Establishment (3-packet handshake, 297 bytes)
   Client → Server: LINKREQUEST (X25519 + Ed25519 public keys)
   Server → Client: PROOF (signature over link_id + keys)
   Client → Server: RTT measurement
   Link becomes ACTIVE

2. Command Execution (over Link)
   Client → Server: DATA packet with COMMAND_REQUEST
   Server → Client: DATA packet with COMMAND_RESPONSE
   (Large outputs use Resource transfer)

3. Link Teardown
   Either → Either: LinkClose context packet
```

### Reticulum Packet Types

```rust
pub enum PacketType {
    Data = 0x00,        // Standard data transmission
    Announce = 0x01,    // Destination advertisement
    LinkRequest = 0x02, // Link establishment
    Proof = 0x03,       // Delivery/link confirmation
}
```

### Shell Message Types (over Links)

```rust
enum ShellMessage {
    CommandRequest { id: u64, command: String, args: Vec<String>, env: Option<HashMap> },
    CommandResponse { id: u64, stdout: Vec<u8>, stderr: Vec<u8>, exit_code: i32 },
}
```

Shell messages are sent as DATA packets with appropriate context values over established Links.

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

### Phase 4: Reticulum Protocol (Current)
- Full protocol implementation in Rust
- X25519 + Ed25519 identities
- Link establishment and management
- Path discovery and routing
- Resource transfer system

### Phase 5: Additional Transports
- TCP Interface with HDLC framing
- UDP Interface
- Local Interface (IPC)
- Potential: LoRa via RNode

### Phase 6: Advanced Features
- Interactive PTY support over Links
- File transfer via Reticulum Resources
- Multi-hop mesh routing
- Multiple concurrent sessions
- Interoperability with Python Reticulum network

### Phase 7: Production Hardening
- Multi-platform support (Windows, macOS)
- Configurable command restrictions
- Advanced audit logging
- Resource quotas and rate limiting
