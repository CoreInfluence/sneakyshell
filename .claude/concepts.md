# Domain Concepts & Terminology

## Reticulum Network

### What is Reticulum?

Reticulum is a cryptography-based networking stack for building resilient, distributed networks. It operates independently of IP-based networks and provides built-in encryption, authentication, and routing.

**Key Characteristics:**
- **Identity-based routing:** Nodes addressed by cryptographic identities, not IP addresses
- **End-to-end encryption:** All packets encrypted by default
- **Transport agnostic:** Can run over any medium (I2P, LoRa, packet radio, TCP, UDP, etc.)
- **Mesh networking:** Automatically discovers routes and builds mesh topologies
- **Minimal overhead:** Designed for low-bandwidth environments

### Core Concepts

#### Identity
A Reticulum identity is an Ed25519 keypair that serves as both:
1. The node's address on the network
2. Authentication credentials

```
Identity = Private Key + Public Key
Address = Hash(Public Key)
```

**Usage in this project:**
- Server has an identity (server address)
- Client has an identity (client address)
- Both verify each other during connection

#### Destination
A destination is a named endpoint within Reticulum where packets can be sent.

```
Destination = Identity + App Name + Aspects
Example: <server_identity>/shell/default
```

**Usage in this project:**
- Server announces a destination for incoming shell connections
- Clients discover and connect to this destination

#### Packet
The basic unit of communication in Reticulum.

**Properties:**
- Encrypted with recipient's public key
- Signed with sender's private key
- Automatically routed through the mesh
- Variable size (up to ~400 bytes efficient, larger requires fragmentation)

**Usage in this project:**
- Our protocol messages are encapsulated in Reticulum packets
- Large command outputs may require packet fragmentation

#### Transport Interface
An interface provides connectivity over a specific medium.

**Common Types:**
- `I2PInterface` - Communicates over I2P network
- `TCPInterface` - Direct TCP connections
- `UDPInterface` - UDP broadcast/multicast
- `RNodeInterface` - LoRa radio devices

**Usage in this project:**
- We implement `I2PInterface` for anonymous routing
- Could add TCP/UDP for local testing

### Reticulum Protocol Flow

```
1. Identity Generation
   Server: Creates identity, derives address
   Client: Creates identity, derives address

2. Destination Announcement
   Server: Announces destination on network
   Network: Propagates announcement

3. Path Discovery
   Client: Requests path to server destination
   Network: Returns route information

4. Connection Establishment
   Client: Sends packet to server destination
   Server: Verifies client identity signature
   Server: Responds with encrypted packet

5. Data Exchange
   Both: Send encrypted, signed packets
   Network: Routes packets automatically
```

## I2P (Invisible Internet Project)

### What is I2P?

I2P is an anonymous overlay network that provides:
- **Hidden services:** Services accessible only within I2P
- **Anonymous routing:** Sender and receiver locations hidden
- **Traffic mixing:** Makes traffic analysis difficult
- **Garlic routing:** Multiple messages bundled together

**vs Tor:**
- Tor: Designed for anonymous access to regular internet
- I2P: Designed for anonymous services within the network
- I2P: Better for P2P applications
- I2P: Unidirectional tunnels (separate inbound/outbound)

### I2P Concepts

#### Destination
An I2P destination is a cryptographic identifier (like a Reticulum identity).

**Format:** 516+ byte structure containing:
- Public key for encryption
- Signing key for authentication
- Certificate (optional extensions)

**Usage in this project:**
- Reticulum creates I2P destinations for transport
- Each Reticulum node has one or more I2P destinations

#### Tunnel
Unidirectional encrypted path through the I2P network.

**Types:**
- **Inbound Tunnel:** For receiving data
- **Outbound Tunnel:** For sending data

**Properties:**
- Each hop adds encryption layer (garlic routing)
- Typically 3 hops (configurable)
- Rebuilt periodically for security

**Usage in this project:**
- Reticulum packets flow through I2P tunnels
- Server has inbound tunnel for receiving connections
- Client has outbound tunnel for sending commands

#### Garlic Routing
Multiple messages encrypted in layers and sent as one "garlic clove."

**Benefits:**
- Hides message count
- Improves efficiency
- Increases anonymity

#### NetDB (Network Database)
Distributed database storing:
- Router information
- Destination information (like DNS)

**Usage in this project:**
- Server publishes destination to NetDB
- Client queries NetDB to find server

### I2P Integration with Reticulum

```
┌─────────────────────────────────────┐
│     Reticulum Packet                │
│  (encrypted, signed, addressed)     │
└──────────────┬──────────────────────┘
               │
               ↓
┌─────────────────────────────────────┐
│     I2P Message                     │
│  (garlic-routed through tunnels)    │
└──────────────┬──────────────────────┘
               │
               ↓
┌─────────────────────────────────────┐
│     I2P Network                     │
│  (onion routing across routers)     │
└─────────────────────────────────────┘
```

**Why this architecture?**
1. Reticulum provides application-layer encryption & identity
2. I2P provides network-layer anonymity & anti-surveillance
3. Together: End-to-end encrypted + anonymous

## Wire Protocol (shell-proto)

### Protocol Overview

Our custom protocol defines messages for remote shell access.

**Design Principles:**
- Binary encoding (compact)
- Versioned (future compatibility)
- Request-response pattern
- Strongly typed messages

### Message Types

#### Connection Phase

**CONNECT**
```rust
struct ConnectMessage {
    protocol_version: u32,
    client_identity: Vec<u8>,  // Reticulum identity public key
    capabilities: Vec<String>,  // Optional feature flags
}
```

**ACCEPT**
```rust
struct AcceptMessage {
    protocol_version: u32,
    server_identity: Vec<u8>,
    session_id: [u8; 16],      // Unique session identifier
}
```

**REJECT**
```rust
struct RejectMessage {
    reason: String,
    error_code: u32,
}
```

#### Command Execution Phase

**COMMAND_REQUEST**
```rust
struct CommandRequest {
    id: u64,                           // Request ID for matching responses
    command: String,                   // Command to execute (e.g., "ls")
    args: Vec<String>,                 // Arguments (e.g., ["-la", "/tmp"])
    env: Option<HashMap<String, String>>, // Environment variables
    timeout: Option<u64>,              // Execution timeout (seconds)
    working_dir: Option<String>,       // Working directory
}
```

**COMMAND_RESPONSE**
```rust
struct CommandResponse {
    id: u64,                    // Matches request ID
    status: CommandStatus,      // Success, Timeout, Error
    stdout: Vec<u8>,            // Standard output (raw bytes)
    stderr: Vec<u8>,            // Standard error (raw bytes)
    exit_code: i32,             // Process exit code
    execution_time: u64,        // Milliseconds
}

enum CommandStatus {
    Success,
    Timeout,
    Error,
    Killed,
}
```

#### Session Management

**DISCONNECT**
```rust
struct DisconnectMessage {
    reason: Option<String>,
}
```

**ACK**
```rust
struct AckMessage {
    message_id: u64,  // ID of message being acknowledged
}
```

### Serialization Format

**Using Bincode:**
```
[ 4 bytes: message length ]
[ 1 byte: message type ]
[ N bytes: message payload (bincode-encoded) ]
```

**Message Types:**
- `0x01`: CONNECT
- `0x02`: ACCEPT
- `0x03`: REJECT
- `0x10`: COMMAND_REQUEST
- `0x11`: COMMAND_RESPONSE
- `0x20`: DISCONNECT
- `0x21`: ACK

### Protocol Flow Example

```
Client                                Server
  |                                      |
  |-- CONNECT ────────────────────────> |
  |                                      | (verify identity)
  | <────────────────────────── ACCEPT --|
  |                                      |
  |-- COMMAND_REQUEST (id=1) ─────────> |
  |    {cmd: "whoami"}                   | (execute)
  | <────────────── COMMAND_RESPONSE  --|
  |    {id: 1, stdout: "root\n"}         |
  |                                      |
  |-- COMMAND_REQUEST (id=2) ─────────> |
  |    {cmd: "ls", args: ["-la"]}        | (execute)
  | <────────────── COMMAND_RESPONSE  --|
  |    {id: 2, stdout: "...", exit: 0}   |
  |                                      |
  |-- DISCONNECT ──────────────────────> |
  | <────────────────────────────── ACK --|
  |                                      |
```

## Security Model

### Cryptographic Primitives

**Reticulum Identity (Ed25519):**
- Key size: 256 bits
- Signature size: 512 bits
- Fast verification
- Collision resistant

**I2P Encryption:**
- ElGamal/AES+SessionTags for tunnel encryption
- HMAC-SHA256 for integrity
- EdDSA for signing

### Authentication Flow

```
1. Client sends CONNECT with client_identity
2. Server verifies:
   - Signature is valid for client_identity
   - Packet came from claimed identity
3. Server sends ACCEPT with server_identity
4. Client verifies:
   - Signature is valid for server_identity
   - Packet came from claimed identity
5. Both sides have mutually authenticated
```

### Threat Mitigations

| Threat | Mitigation |
|--------|-----------|
| Network eavesdropping | Reticulum encryption + I2P tunnels |
| Man-in-the-middle | Identity verification via signatures |
| Replay attacks | Reticulum packet timestamps & nonces |
| Traffic analysis | I2P garlic routing & mixing |
| Command injection | Argument separation, no shell evaluation |
| Resource exhaustion | Timeouts, memory limits, rate limiting |
| Unauthorized access | Cryptographic identity authentication |

## Command Execution Model

### Current Model: Simple Execution

```rust
// Pseudocode
fn execute_command(request: CommandRequest) -> CommandResponse {
    let mut cmd = Command::new(&request.command);
    cmd.args(&request.args);
    cmd.env_clear();
    if let Some(env) = request.env {
        cmd.envs(env);
    }

    let output = cmd.output()?;

    CommandResponse {
        id: request.id,
        status: if output.status.success() { Success } else { Error },
        stdout: output.stdout,
        stderr: output.stderr,
        exit_code: output.status.code().unwrap_or(-1),
    }
}
```

**Characteristics:**
- Non-interactive
- Fire-and-forget
- Complete output after execution
- No job control

### Future Model: Interactive PTY

```rust
// Pseudocode for future enhancement
fn execute_pty(request: CommandRequest) -> PtySession {
    let pty = PtyMaster::open()?;
    let fork = pty.fork()?;

    match fork {
        Parent(child_pid) => {
            // Server reads from PTY, sends to client
            // Server receives from client, writes to PTY
        }
        Child => {
            exec(&request.command, &request.args);
        }
    }
}
```

**Characteristics:**
- Interactive (vim, top, etc.)
- Terminal emulation
- Job control signals
- More complex

## Performance Considerations

### Latency Sources

1. **I2P Tunnel Building:** 30-60 seconds initially
2. **Reticulum Path Discovery:** 1-5 seconds
3. **Command Execution:** Varies by command
4. **Packet Transit:** ~500ms average in I2P

**Mitigation Strategies:**
- Keep tunnels alive
- Cache Reticulum paths
- Connection pooling
- Asynchronous execution

### Bandwidth Constraints

**I2P Typical Throughput:**
- 100-500 KB/s for well-connected nodes
- Lower for new nodes or poor routes

**Implications:**
- Large command outputs may be slow
- File transfers need chunking
- Consider compression for large outputs

## Glossary

- **Aspect:** Reticulum destination property (like a sub-address)
- **Garlic Routing:** I2P's onion routing with message bundling
- **Identity:** Ed25519 keypair used as network address
- **Lease:** Time-limited route in I2P NetDB
- **Packet:** Basic Reticulum message unit
- **PTY:** Pseudo-terminal for interactive shells
- **Reticulum:** Cryptography-based networking stack
- **Session:** A connected client-server relationship
- **Tunnel:** Encrypted I2P path for routing messages
