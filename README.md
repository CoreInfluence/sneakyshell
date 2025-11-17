# Reticulum-Shell

A remote access tool built for cybersecurity research, enabling remote shell access to Linux systems over the Reticulum network using I2P as a transport layer.

## Overview

Reticulum-Shell provides anonymous, encrypted remote shell access using:

- **Reticulum Network**: Cryptography-based networking with built-in encryption and authentication
- **I2P Transport**: Anonymous routing layer for anti-surveillance
- **Ed25519 Signatures**: Identity-based authentication
- **Rust**: Memory-safe implementation with async/await

### Architecture

```
┌─────────────┐         Reticulum/I2P         ┌─────────────┐
│   Client    │ ◄──────────────────────────► │   Server    │
│  (REPL)     │    Encrypted & Anonymous      │ (Executor)  │
└─────────────┘                                └─────────────┘
```

**Components:**

- **shell-client**: Interactive REPL for sending commands
- **shell-server**: Listens and executes commands
- **shell-proto**: Shared protocol definitions
- **reticulum-core**: Reticulum network implementation

## Security Research Context

**Authorized Use Cases:**
- Penetration testing engagements
- Red team operations
- Security research
- Authorized remote administration
- Educational purposes in cybersecurity

**Not Intended For:**
- Unauthorized access
- Malicious use
- Production deployments without security hardening

## Features

### Current
- [x] Ed25519 identity-based authentication
- [x] Command execution over Reticulum protocol
- [x] Binary protocol with message framing
- [x] Interactive REPL client
- [x] Configurable timeouts and limits
- [x] Security input validation
- [x] **I2P transport integration** - Full SAM v3 implementation
- [x] **Embedded I2P router** - No external I2P installation required (optional feature)
- [x] Mock interface for local testing
- [x] Zero-configuration deployment

### Planned
- [ ] Interactive PTY support (vim, top, etc.)
- [ ] File transfer capabilities
- [ ] Multiple concurrent sessions per server
- [ ] Advanced audit logging with rotation
- [ ] Command allowlist/blocklist

## Quick Start

### Build

```bash
# Clone and build
git clone https://github.com/yourusername/reticulum-shell.git
cd reticulum-shell
cargo build --release
```

### Choose Your Transport

Reticulum-Shell supports two modes:

1. **Local Testing Mode** (MockInterface) - For development and testing
2. **I2P Mode** (Anonymous Network) - For real-world use over I2P

### Zero-Configuration Setup (Local Testing)

**1. Start the server (auto-generates everything):**

```bash
./target/release/shell-server
```

Output:
```
INFO Generating new server identity at "server.identity"
INFO Server identity saved: 3c9e01621c7aa4c88034d015fb464453d090328b83cca17492062575fa0afb09
INFO Default configuration saved to "server.toml"
INFO Server destination: 3c9e01621c7aa4c88034d015fb464453d090328b83cca17492062575fa0afb09
INFO Server running. Press Ctrl+C to stop.
```

**2. Copy the server destination hash** (the 64-character hex string)

**3. Run the client:**

```bash
./target/release/shell-client --server <paste-server-destination-here>
```

That's it! The client will auto-generate its identity on first run.

**Note:** Local testing mode uses in-memory MockInterface. For real anonymous networking over I2P, see below.

### I2P Mode (Production)

**Prerequisites:** I2P router running with SAM bridge enabled on port 7656

**1. Start server with I2P:**
```bash
./target/release/shell-server --enable-i2p
```

Output will show your I2P destination (long base64 string):
```
INFO I2P destination: LS0tLS1CRUdJTiBJMlAgREVTVElOQVRJT04...
```

**2. Copy the entire I2P destination string**

**3. Connect client:**
```bash
./target/release/shell-client --enable-i2p --i2p-destination "LS0tLS1CRUdJTi..."
```

**See [docs/I2P-SETUP.md](docs/I2P-SETUP.md) for complete I2P setup instructions.**

### Embedded I2P Router (No External Dependencies!)

**NEW:** Run your own I2P router embedded in the binary - no external I2P installation required!

**Build with embedded router:**
```bash
cargo build --release --features embedded-router
```

**Start server with embedded I2P:**
```bash
./target/release/shell-server --enable-i2p --use-embedded-router
```

Output:
```
INFO Initializing embedded I2P router
INFO Embedded I2P router started successfully
INFO I2P router ready
INFO I2P destination: ABCDEFGHIJKLMNOPQRSTUVWXYZabc...
```

**Connect client with embedded I2P:**
```bash
./target/release/shell-client \
  --enable-i2p \
  --use-embedded-router \
  --i2p-destination "ABCDEFGHIJKLMNOPQRSTUVWXYZabc..."
```

**Benefits:**
- ✅ No external I2P router needed
- ✅ Single-binary deployment
- ✅ Works out of the box
- ✅ Perfect for testing and portable deployments

**First connection takes 2-5 minutes for initial bootstrap:**
- Downloading router infos: 30-60 seconds
- Building I2P tunnels: 90-240 seconds
- Subsequent connections are much faster (30-90 seconds)

**See [docs/EMBEDDED-ROUTER.md](docs/EMBEDDED-ROUTER.md) for complete embedded router guide.**

### Alternative: Using Config Files

**Server:**
```bash
./target/release/shell-server  # Creates server.toml automatically
```

**Client:**
```bash
./target/release/shell-client   # Creates client.toml automatically
# Edit client.toml and set server_destination, then run again
./target/release/shell-client
```

## Usage

### Interactive REPL

```bash
$ ./target/release/shell-client --server <destination>
```

```
Reticulum Shell Client
Type 'help' for commands, 'exit' to quit

rsh> whoami
Command execution not yet implemented

rsh> help
Available commands:
  help          - Show this help message
  status        - Show connection status
  clear         - Clear screen
  exit, quit    - Exit the shell

rsh> status
Connection Status:
  Status: Connected

rsh> exit
Goodbye!
```

### Single Command Execution

```bash
./target/release/shell-client --server <destination> -e "whoami"
```

**Command execution works in both local testing mode and I2P mode.**

### Built-in Commands

- `help` - Show available commands
- `status` - Display connection status
- `clear` - Clear the screen
- `exit`, `quit` - Disconnect and exit

## Development

### Project Structure

```
reticulum-shell/
├── .claude/              # Claude AI context files
├── crates/
│   ├── reticulum-core/   # Reticulum networking
│   ├── shell-proto/      # Protocol definitions
│   ├── shell-server/     # Server implementation
│   └── shell-client/     # Client implementation
├── docs/                 # Documentation
└── Cargo.toml            # Workspace manifest
```

### Building

```bash
# Build everything
cargo build

# Build specific crate
cargo build -p shell-server

# Run tests
cargo test --all

# Run with logging
RUST_LOG=debug cargo run -p shell-server
```

### Running Tests

```bash
# All tests
cargo test --all

# Specific crate
cargo test -p shell-proto

# With output
cargo test -- --nocapture
```

## Protocol

The wire protocol uses binary message framing:

```
[ 4 bytes: length ]
[ 1 byte: message type ]
[ N bytes: payload (bincode) ]
```

**Message Types:**
- `0x01` - CONNECT
- `0x02` - ACCEPT
- `0x03` - REJECT
- `0x10` - COMMAND_REQUEST
- `0x11` - COMMAND_RESPONSE
- `0x20` - DISCONNECT
- `0x21` - ACK
- `0x30` - PING
- `0x31` - PONG

See `docs/PROTOCOL.md` for full specification.

## Security Considerations

### Implemented
- Ed25519 identity verification
- Command argument separation (no shell injection)
- Execution timeouts
- Path traversal prevention
- Clean environment variables

### Implemented
- [x] I2P anonymous routing (SAM v3 protocol)
- [x] Automatic destination mapping (SHA-256 hashing)
- [x] DATAGRAM-based messaging over I2P
- [x] Tunnel auto-establishment

### TODO
- [ ] Rate limiting
- [ ] Resource quotas
- [ ] Command allowlist/blocklist
- [ ] Advanced audit logging with encryption

## Contributing

This is a personal research project. Feel free to:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## Documentation

- [I2P Setup Guide](docs/I2P-SETUP.md) - **Complete guide for I2P integration**
- [Architecture Overview](.claude/architecture.md)
- [Code Navigation](.claude/navigation.md)
- [Domain Concepts](.claude/concepts.md)
- [Protocol Specification](docs/PROTOCOL.md)

## License

MIT License - see LICENSE file for details.

## Disclaimer

This tool is designed for authorized security testing and research purposes only. Users are responsible for ensuring they have proper authorization before using this tool. Unauthorized access to computer systems is illegal.

## Acknowledgments

- [Reticulum Network](https://reticulum.network/) - Inspiration for the networking layer
- [I2P Project](https://geti2p.net/) - Anonymous routing infrastructure
- Rust community for excellent async ecosystem
