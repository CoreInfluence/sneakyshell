# Code Navigation Guide

## Project Structure

```
reticulum-shell/
├── .claude/                           # Claude context files (you are here!)
│   ├── context.md                     # Project status and quick start
│   ├── architecture.md                # Design decisions and patterns
│   ├── navigation.md                  # This file - code map
│   └── concepts.md                    # Domain knowledge and terminology
│
├── crates/                            # Cargo workspace members
│   │
│   ├── reticulum-core/               # Core networking library
│   │   ├── Cargo.toml                # Dependencies: tokio, serde, etc.
│   │   ├── src/
│   │   │   ├── lib.rs                # Public API exports
│   │   │   ├── identity.rs           # Reticulum identity management
│   │   │   ├── packet.rs             # Packet structure and parsing
│   │   │   ├── interface.rs          # Network interface abstraction (MockInterface, I2pInterface)
│   │   │   ├── sam.rs                # SAM v3 protocol client implementation
│   │   │   ├── embedded_router.rs    # Embedded I2P router (Emissary) wrapper
│   │   │   └── error.rs              # Error types for networking
│   │   └── tests/
│   │       └── integration_tests.rs  # Network stack tests
│   │
│   ├── shell-proto/                  # Shared protocol definitions
│   │   ├── Cargo.toml                # Dependencies: serde, bincode
│   │   └── src/
│   │       ├── lib.rs                # Protocol version and exports
│   │       ├── messages.rs           # Message type definitions
│   │       ├── protocol.rs           # Framing and serialization
│   │       └── error.rs              # Protocol error types
│   │
│   ├── shell-server/                 # Server binary
│   │   ├── Cargo.toml                # Dependencies: tokio, clap, tracing
│   │   ├── src/
│   │   │   ├── main.rs               # Entry point, CLI parsing
│   │   │   ├── lib.rs                # Server library exports
│   │   │   ├── listener.rs           # Listen for connections
│   │   │   ├── shell.rs              # Execute commands
│   │   │   ├── session.rs            # Session management
│   │   │   ├── config.rs             # Configuration handling
│   │   │   └── error.rs              # Server error types
│   │   ├── examples/
│   │   │   └── basic_server.rs       # Simple usage example
│   │   └── tests/
│   │       └── server_tests.rs       # Server integration tests
│   │
│   └── shell-client/                 # Client binary
│       ├── Cargo.toml                # Dependencies: tokio, clap, rustyline
│       ├── src/
│       │   ├── main.rs               # Entry point, CLI parsing
│       │   ├── lib.rs                # Client library exports
│       │   ├── connection.rs         # Connect to server
│       │   ├── repl.rs               # Command REPL loop
│       │   ├── config.rs             # Configuration handling
│       │   └── error.rs              # Client error types
│       ├── examples/
│       │   └── basic_client.rs       # Simple usage example
│       └── tests/
│           └── client_tests.rs       # Client integration tests
│
├── docs/                              # User documentation
│   ├── PROTOCOL.md                   # Wire protocol specification
│   ├── SETUP.md                      # Installation and configuration
│   └── USAGE.md                      # Usage examples and tutorials
│
├── scripts/                          # Build and utility scripts
│   ├── build.sh                      # Build all crates
│   └── test.sh                       # Run all tests
│
├── Cargo.toml                        # Workspace manifest
├── Cargo.lock                        # Dependency lock file
├── README.md                         # Project overview
├── LICENSE                           # License information
└── .gitignore                        # Git ignore rules
```

## Key Entry Points

### Starting the Server
- **File:** `crates/shell-server/src/main.rs`
- **Function:** `main()` - Parses CLI args, loads config, starts listener
- **Flow:** main() → listener::start() → session::handle_client()

### Starting the Client
- **File:** `crates/shell-client/src/main.rs`
- **Function:** `main()` - Parses CLI args, connects to server, starts REPL
- **Flow:** main() → connection::connect() → repl::run()

### Protocol Message Handling
- **File:** `crates/shell-proto/src/messages.rs`
- **Types:** All message definitions (Connect, CommandRequest, etc.)
- **Serialization:** `crates/shell-proto/src/protocol.rs`

### Command Execution
- **File:** `crates/shell-server/src/shell.rs`
- **Function:** `execute_command()` - Spawns process, captures output
- **Security:** Input validation and sandboxing logic

### Network Transport
- **File:** `crates/reticulum-core/src/lib.rs`
- **API:** Public interface for sending/receiving Reticulum packets
- **I2P Integration:**
  - `crates/reticulum-core/src/interface.rs` - Network interface abstraction
  - `crates/reticulum-core/src/sam.rs` - SAM v3 protocol client
  - `crates/reticulum-core/src/embedded_router.rs` - Embedded router (feature: embedded-router)

## Module Responsibilities

### reticulum-core
**Purpose:** Low-level Reticulum networking over I2P
- Identity creation and management
- Packet encoding/decoding
- Network interface abstraction (MockInterface for testing, I2pInterface for production)
- SAM v3 protocol client for I2P router communication
- Embedded I2P router integration (optional feature)
- Connection lifecycle

**Key Types:**
- `Identity` - Reticulum identity (public/private keypair)
- `Packet` - Network packet structure
- `NetworkInterface` - Trait for transport abstraction
- `MockInterface` - In-memory testing interface
- `I2pInterface` - I2P transport layer via SAM protocol
- `SamConnection` - Low-level SAM v3 client
- `EmbeddedRouter` - Emissary-based embedded I2P router (feature: embedded-router)
- `Destination` - Reticulum destination address

### shell-proto
**Purpose:** Wire protocol definitions shared between client and server
- Message type definitions
- Serialization/deserialization
- Protocol versioning
- Error types

**Key Types:**
- `Message` - Enum of all message types
- `CommandRequest` - Client command message
- `CommandResponse` - Server response message
- `ProtocolVersion` - Version negotiation

### shell-server
**Purpose:** Server-side logic for listening and executing commands
- Listen for incoming connections
- Authenticate clients
- Execute shell commands
- Manage client sessions

**Key Types:**
- `Server` - Main server struct
- `Session` - Per-client session
- `CommandExecutor` - Runs commands safely
- `ServerConfig` - Configuration

### shell-client
**Purpose:** Client-side logic for connecting and sending commands
- Connect to server
- Interactive command REPL
- Display command output
- Handle disconnections

**Key Types:**
- `Client` - Main client struct
- `Connection` - Server connection
- `Repl` - Interactive shell
- `ClientConfig` - Configuration

## Important Code Patterns

### Error Handling
```rust
// Use thiserror for all error types
#[derive(Error, Debug)]
enum MyError {
    #[error("description: {0}")]
    Variant(#[from] SourceError),
}

// Propagate with ? operator
fn my_function() -> Result<T, MyError> {
    let result = fallible_operation()?;
    Ok(result)
}
```

### Async Operations
```rust
// All I/O operations are async
async fn handle_client(stream: TcpStream) -> Result<(), Error> {
    let mut buf = vec![0u8; 4096];
    stream.read(&mut buf).await?;
    // ...
}
```

### Configuration
```rust
// TOML-based config with serde
#[derive(Deserialize)]
struct Config {
    listen_address: String,
    identity_path: PathBuf,
}

let config: Config = toml::from_str(&file_contents)?;
```

### Logging
```rust
// Structured logging with tracing
use tracing::{info, warn, error, debug};

info!(command = %cmd, "Executing command");
error!(error = ?e, "Command failed");
```

## Finding Things

### To Find...
- **Message definitions:** `crates/shell-proto/src/messages.rs`
- **Command execution:** `crates/shell-server/src/shell.rs`
- **I2P integration:**
  - `crates/reticulum-core/src/interface.rs` - Interface abstraction
  - `crates/reticulum-core/src/sam.rs` - SAM protocol
  - `crates/reticulum-core/src/embedded_router.rs` - Embedded router
- **Error types:** `*/src/error.rs` in each crate
- **Configuration:** `*/src/config.rs` in server/client
- **Tests:** `*/tests/*.rs` in each crate
- **Examples:** `*/examples/*.rs` in each crate

### To Understand...
- **Protocol flow:** Read `docs/PROTOCOL.md` + `architecture.md`
- **Current status:** Read `.claude/context.md`
- **Design decisions:** Read `.claude/architecture.md`
- **Reticulum concepts:** Read `.claude/concepts.md`

## Common Tasks

### Adding a New Message Type
1. Define in `shell-proto/src/messages.rs`
2. Add serialization in `shell-proto/src/protocol.rs`
3. Handle in server: `shell-server/src/listener.rs` or `session.rs`
4. Handle in client: `shell-client/src/connection.rs`
5. Update protocol version if breaking change

### Adding a New Command Feature
1. Extend `CommandRequest` in `shell-proto/src/messages.rs`
2. Update executor in `shell-server/src/shell.rs`
3. Update client REPL in `shell-client/src/repl.rs`
4. Add tests in relevant `tests/` directories

### Modifying I2P Behavior
1. For interface changes: Edit `reticulum-core/src/interface.rs`
2. For SAM protocol: Edit `reticulum-core/src/sam.rs`
3. For embedded router: Edit `reticulum-core/src/embedded_router.rs`
4. Test with integration tests in `reticulum-core/tests/`
