# I2P Implementation Technical Notes

This document describes the technical details of the I2P integration in Reticulum-Shell.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Reticulum-Shell                          │
├─────────────────────────────────────────────────────────────┤
│  shell-server              │           shell-client          │
│  ┌─────────────┐          │          ┌─────────────┐       │
│  │   Server    │          │          │   Client    │       │
│  └──────┬──────┘          │          └──────┬──────┘       │
│         │                  │                 │               │
├─────────┼──────────────────┼─────────────────┼──────────────┤
│         │  reticulum-core  │                 │               │
│         │                  │                 │               │
│  ┌──────▼──────┐          │          ┌──────▼──────┐       │
│  │ I2pInterface│          │          │ I2pInterface│       │
│  └──────┬──────┘          │          └──────┬──────┘       │
│         │                  │                 │               │
│  ┌──────▼──────┐          │          ┌──────▼──────┐       │
│  │SamConnection│          │          │SamConnection│       │
│  └──────┬──────┘          │          └──────┬──────┘       │
└─────────┼──────────────────┴─────────────────┼──────────────┘
          │                                     │
          │           SAM v3.1 Protocol         │
          │                                     │
┌─────────▼─────────────────────────────────────▼──────────────┐
│                    I2P Router (Java/i2pd)                     │
│                     SAM Bridge: port 7656                     │
└───────────────────────────────────────────────────────────────┘
          │                                     │
          │          I2P Garlic Routing         │
          │                                     │
          └─────────────────┬───────────────────┘
                            │
                      I2P Network
```

## Component Details

### 1. SamConnection (`reticulum-core/src/sam.rs`)

**Purpose:** Low-level SAM v3.1 protocol client

**Key Methods:**
- `connect(addr)` - Establishes TCP connection and performs HELLO handshake
- `dest_generate()` - Generates new I2P destination with Ed25519 signatures
- `session_create_datagram(session_id, destination)` - Creates DATAGRAM session
- `datagram_send(session_id, destination, data)` - Sends raw bytes over I2P
- `datagram_receive()` - Receives incoming I2P datagrams

**Implementation Details:**
```rust
pub struct SamConnection {
    reader: BufReader<TcpStream>,  // Async tokio stream
}
```

- Uses `tokio::net::TcpStream` for async I/O
- `BufReader` for line-based protocol parsing
- All methods are async (return `impl Future`)
- Ed25519 signature type (type 7) for I2P destinations

**Protocol Flow:**
```
Client                      SAM Bridge
  |                              |
  |-- HELLO VERSION MIN=3.1 ---->|
  |<-- HELLO REPLY RESULT=OK ----|
  |                              |
  |-- DEST GENERATE TYPE=7 ----->|
  |<-- DEST REPLY PUB=... --------|
  |                              |
  |-- SESSION CREATE ----------->|
  |<-- SESSION STATUS OK ---------|
  |                              |
  |-- DATAGRAM SEND ------------>|
  |                              |
  |<-- DATAGRAM RECEIVED --------|
```

### 2. I2pInterface (`reticulum-core/src/interface.rs`)

**Purpose:** High-level NetworkInterface implementation for I2P

**Key Features:**
- Implements `NetworkInterface` trait
- Manages I2P destination mapping
- Automatic SHA-256 hashing for 32-byte compatibility
- Thread-safe with `Arc<Mutex<>>`

**Structure:**
```rust
pub struct I2pInterface {
    name: String,
    sam_conn: Arc<Mutex<SamConnection>>,
    session_id: String,
    local_destination: String,
    destination_map: Arc<Mutex<HashMap<[u8; 32], String>>>,
}
```

**Destination Mapping Logic:**

The Reticulum protocol uses 32-byte destination hashes, but I2P destinations are 500+ byte base64 strings. We solve this with:

1. **Hash Creation:**
   ```rust
   let hash = SHA256(i2p_destination_string)  // 32 bytes
   ```

2. **Bidirectional Mapping:**
   - `destination_map: HashMap<[u8; 32], String>`
   - Maps 32-byte hash → full I2P destination
   - Updated automatically on send/receive

3. **Send Flow:**
   ```rust
   async fn send(&self, packet: &Packet) -> Result<()> {
       // packet.destination is [u8; 32]
       let i2p_dest = self.destination_map.get(&packet.destination)?;
       let encoded = packet.encode();
       self.sam_conn.datagram_send(&self.session_id, i2p_dest, &encoded).await
   }
   ```

4. **Receive Flow:**
   ```rust
   async fn receive(&self) -> Result<Packet> {
       let (source_i2p_dest, data) = self.sam_conn.datagram_receive().await?;

       // Hash the source to get 32-byte identifier
       let source_hash = SHA256(source_i2p_dest);

       // Register for future sends
       self.destination_map.insert(source_hash, source_i2p_dest);

       // Decode packet (contains source_hash as destination)
       Packet::decode(&data)
   }
   ```

### 3. Server Integration (`shell-server/src/main.rs`)

**CLI Options:**
```bash
--enable-i2p              # Enable I2P transport
--sam-address <ADDR>      # Override SAM bridge address
```

**Initialization Flow:**
```rust
let server = if enable_i2p {
    let i2p_interface = I2pInterface::new(&sam_address).await?;
    info!("I2P destination: {}", i2p_interface.local_destination());

    let interface: Arc<dyn NetworkInterface> = Arc::new(i2p_interface);
    Server::with_interface(config, interface).await?
} else {
    Server::new(config).await?  // No network interface
};

server.run().await?;
```

**Message Loop:**
- Server receives packets via `interface.receive().await`
- Packets are decoded into protocol messages
- Responses are sent via `interface.send(&response_packet).await`
- All I2P details abstracted by NetworkInterface trait

### 4. Client Integration (`shell-client/src/main.rs`)

**CLI Options:**
```bash
--enable-i2p                     # Enable I2P transport
--sam-address <ADDR>             # Override SAM bridge address
--i2p-destination <DESTINATION>  # Server's I2P destination (base64)
```

**Initialization Flow:**
```rust
let client = if enable_i2p {
    let i2p_interface = I2pInterface::new(&sam_address).await?;

    // Register server's I2P destination
    let server_dest_hash = i2p_interface
        .register_destination(server_i2p_dest.clone())
        .await;

    let interface: Arc<dyn NetworkInterface> = Arc::new(i2p_interface);
    Client::with_interface(config, interface, server_dest_hash).await?
} else {
    Client::new(config).await?  // Uses MockInterface for testing
};

client.connect().await?;
```

**Key Difference from Server:**
- Client must know server's I2P destination upfront
- `register_destination()` creates the hash mapping before first send
- Server learns client's destination dynamically on first receive

## Protocol Message Flow Over I2P

### Connection Establishment

```
Client                                Server
  |                                     |
  | 1. Generate I2P destination         | 1. Generate I2P destination
  | 2. Create SAM DATAGRAM session      | 2. Create SAM DATAGRAM session
  | 3. Register server I2P dest         | 3. Wait for connections
  |                                     |
  |-- CONNECT message ----------------->| 4. Receive DATAGRAM
  |   (via I2P tunnels)                 | 5. Extract source I2P dest
  |                                     | 6. Hash source dest (32 bytes)
  |                                     | 7. Register in destination_map
  |                                     | 8. Verify protocol version
  |                                     | 9. Check allowed_clients
  |                                     | 10. Create session
  |                                     |
  |<-- ACCEPT message -------------------| 11. Send response via I2P
  |    (includes session_id)            |
  |                                     |
```

### Command Execution

```
Client                                Server
  |                                     |
  |-- COMMAND_REQUEST ----------------->|
  |   {                                 | 1. Lookup session by ID
  |     id: 1,                          | 2. Extract command & args
  |     command: "whoami",              | 3. Validate request
  |     args: [],                       | 4. Execute via shell executor
  |     timeout: 300                    | 5. Capture stdout/stderr
  |   }                                 | 6. Build response
  |                                     |
  |<-- COMMAND_RESPONSE -----------------|
  |   {                                 |
  |     id: 1,                          |
  |     exit_code: 0,                   |
  |     stdout: b"zero\n",              |
  |     stderr: b"",                    |
  |   }                                 |
  |                                     |
```

## Data Encoding Layers

### Layer 1: Protocol Message (shell-proto)
```
[ 4 bytes: message length ]
[ 1 byte: message type ]
[ N bytes: bincode-encoded payload ]
```

### Layer 2: Reticulum Packet (reticulum-core)
```
[ 1 byte: packet type ]
[ 32 bytes: destination hash ]
[ 2 bytes: data length ]
[ N bytes: protocol message from Layer 1 ]
[ 1 byte: signature flag ]
[ 64 bytes: Ed25519 signature (optional) ]
```

### Layer 3: SAM DATAGRAM (I2P)
```
Header: "DATAGRAM SEND ID=session DESTINATION=base64 SIZE=len\n"
Data:   [ Layer 2 packet bytes ]
```

### Layer 4: I2P Garlic Routing
- Multiple layers of encryption
- Onion routing through I2P tunnels
- Hidden source/destination IP addresses

## Security Properties

### Cryptographic Identity (Layer 1)
- **Ed25519 keypairs** for client/server identities
- Public key = 32-byte destination hash
- Messages signed with private key
- Signature verification on receive

### I2P Destination Security (Layer 2)
- **I2P destination** = public identity (500+ bytes)
- Contains I2P public key for end-to-end encryption
- Signature type 7 (Ed25519) for future crypto-agility

### Network Anonymity (Layer 3)
- **Garlic routing**: Multi-hop encrypted tunnels
- **Unidirectional tunnels**: Separate inbound/outbound
- **Hidden IP addresses**: Both endpoints anonymous
- **Resistance to traffic analysis**: Constant-rate cover traffic

## Performance Considerations

### Latency
- **Local MockInterface**: ~1ms
- **I2P Network**: 500ms - 3000ms
  - Tunnel establishment: 30-60 seconds (first connection)
  - Subsequent messages: 500-2000ms
  - Variable based on network conditions

### Throughput
- **DATAGRAM mode**: Best-effort delivery
- **Max message size**: ~32KB (I2P limitation)
- **Fragmentation**: Handled by I2P router
- **Retransmission**: Not implemented (DATAGRAM is unreliable)

### Memory Usage
- **SamConnection**: ~1KB (TCP buffer)
- **I2pInterface**: ~1KB + (num_destinations × 550 bytes)
- **Destination map grows**: One entry per unique peer

## Error Handling

### SAM Connection Errors
```rust
NetworkError::I2p(String)
```

**Common Errors:**
1. "Failed to connect to SAM" → I2P router not running
2. "Handshake failed" → SAM version mismatch
3. "Session creation failed" → Invalid destination or ID conflict
4. "Unknown destination" → Destination not in mapping table

### Recovery Strategies
- **Connection loss**: Requires full restart (SAM session invalid)
- **Unknown destination**: Pre-register or learn on receive
- **Timeout**: Increase timeout values (I2P is slow)

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
#[ignore]  // Requires I2P router
async fn test_sam_connection() { ... }
```

### Integration Tests
```rust
// Uses MockInterface (no I2P required)
#[tokio::test]
async fn test_full_command_execution_flow() {
    let (client_iface, server_iface) = MockInterface::create_pair();
    // Test protocol without I2P overhead
}
```

### Manual Testing
```bash
# Terminal 1: I2P router
i2prouter start

# Terminal 2: Server
./target/release/shell-server --enable-i2p -v

# Terminal 3: Client
./target/release/shell-client --enable-i2p --i2p-destination "..." -v -e "whoami"
```

## Future Enhancements

### 1. Streaming Sessions (SAM STREAM)
Current DATAGRAM mode is best-effort. For reliability:
- Use SAM STREAM style (TCP-like)
- Persistent bi-directional connection
- Automatic retransmission

### 2. Multiple Concurrent Sessions
Current limitation: One client per server instance
- Add connection pooling
- Session multiplexing
- Load balancing across tunnels

### 3. Destination Persistence
Currently destinations regenerated on restart:
- Save I2P destination to file
- Reuse same destination across restarts
- Allows for "stable" server addresses

### 4. Advanced Tunnel Configuration
- Custom tunnel length (default: 3 hops)
- Backup tunnels for redundancy
- Bandwidth limits per session

## Debugging Tips

### Enable Verbose Logging
```bash
RUST_LOG=debug ./target/release/shell-server --enable-i2p
```

### Monitor SAM Protocol
```bash
# Use tcpdump to see SAM commands
sudo tcpdump -i lo -A port 7656
```

### Check I2P Router Status
```bash
# Web console
firefox http://127.0.0.1:7657

# i2pd status
curl http://127.0.0.1:7070
```

### Common Debug Patterns
```rust
use tracing::{debug, info, warn, error};

debug!(
    destination = %hex::encode(&packet.destination),
    data_len = packet.data.len(),
    "Received packet"
);
```

## References

- [SAM v3 Specification](https://geti2p.net/en/docs/api/samv3)
- [I2P Technical Documentation](https://geti2p.net/en/docs/how/intro)
- [Ed25519 Signature Scheme](https://ed25519.cr.yp.to/)
- [Tokio Async Runtime](https://tokio.rs/)
