# Reticulum-Shell

A remote access tool built for cybersecurity research, enabling remote shell access to Linux systems using the Reticulum network protocol with I2P as an anonymous transport layer.

## Overview

Reticulum-Shell provides anonymous, encrypted remote shell access using a **full Rust implementation of the Reticulum network protocol**:

- **Reticulum Protocol**: Complete implementation of the Reticulum networking stack
  - X25519 + Ed25519 dual-keypair identities
  - Link-based encrypted channels with forward secrecy
  - Announce/path discovery for mesh routing
  - Resource transfer system for reliable data delivery
- **Multiple Transports**: I2P (anonymous), TCP, UDP, Local IPC
- **Rust**: Memory-safe, high-performance implementation with async/await

### Architecture

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

**Components:**

- **shell-client**: Interactive REPL for sending commands
- **shell-server**: Listens and executes commands over Reticulum Links
- **shell-proto**: Shell-specific message definitions
- **reticulum-core**: Full Reticulum protocol implementation (Identity, Link, Packet, Transport, Interfaces)

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

### Current (Phase 3 Complete)
- [x] Ed25519 identity-based authentication
- [x] Command execution with binary protocol
- [x] Interactive REPL client
- [x] Configurable timeouts and limits
- [x] Security input validation
- [x] **I2P transport** - Full SAM v3 implementation
- [x] **Embedded I2P router** - No external I2P installation required
- [x] Mock interface for local testing
- [x] Zero-configuration deployment

### In Progress (Phase 4: Reticulum Protocol)
- [ ] **Full Reticulum protocol implementation in Rust**
  - [ ] X25519 + Ed25519 dual-keypair identities
  - [ ] ECIES encryption with Token cipher (AES-256-CBC + HMAC)
  - [ ] Ratchet keys for forward secrecy
  - [ ] Complete packet wire format (DATA, ANNOUNCE, LINKREQUEST, PROOF)
  - [ ] Link establishment with 3-packet handshake
  - [ ] Path discovery and announce propagation
  - [ ] Resource transfer system
  - [ ] Transport core with routing tables

### Planned (Future Phases)
- [ ] Additional transports (TCP, UDP, Local IPC)
- [ ] Interactive PTY support (vim, top, etc.)
- [ ] File transfer via Reticulum Resources
- [ ] Multiple concurrent sessions per server
- [ ] Multi-hop mesh routing
- [ ] Interoperability with Python Reticulum reference

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
INFO I2P destination: LS0tLS1CRUdJTiBJMlAgREVTVElOQVRJT04...  ← COPY THIS ENTIRE STRING!
INFO I2P destination hash: 1a2b3c4d5e6f7890abcdef1234567890abcdef1234567890abcdef1234567890
INFO Server destination: 9795adcdb0156d76afb40e545382e5423ec16e094911fb88689cd91b171fcced
```

**2. Copy the entire I2P destination string (the long base64 string)**

**⚠️ IMPORTANT: Use the "I2P destination" (long base64), NOT the "Server destination" (hex hash)!**

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
INFO I2P destination: jWQnSctWzWmLkboVVlRaQmgn5AaMo3uxGQx3H4sxUK7jAiFRKhjj...  ← COPY THIS!
INFO I2P destination hash: 454fc6243fc04b9874359ad6267a7df24afcb3732697af129fa5e34565deedba
INFO Server destination: 9795adcdb0156d76afb40e545382e5423ec16e094911fb88689cd91b171fcced
```

**⚠️ IMPORTANT: Copy the I2P destination (long base64 string), NOT the server destination hash!**

**Connect client with embedded I2P:**
```bash
./target/release/shell-client \
  --enable-i2p \
  --use-embedded-router \
  --i2p-destination "jWQnSctWzWmLkboVVlRaQmgn5AaMo3uxGQx3H4sxUK7jAiFRKhjj..."
```

**Common Mistake:**
- ❌ WRONG: Using "Server destination" (short hex hash like `9795adcd...`)
- ✅ CORRECT: Using "I2P destination" (long base64 string like `jWQnSctW...`)

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

Reticulum-Shell uses the **Reticulum network protocol** for all communication. The wire format follows the Reticulum specification:

### Packet Structure
```
[HEADER 2 bytes] [ADDRESSES 16/32 bytes] [CONTEXT 1 byte] [DATA 0-465 bytes]
```

### Reticulum Packet Types
- `0x00` - DATA (encrypted payload)
- `0x01` - ANNOUNCE (destination advertisement)
- `0x02` - LINKREQUEST (connection establishment)
- `0x03` - PROOF (delivery/link confirmation)

### Link-Based Communication
Shell commands are sent over established **Reticulum Links**:
1. Client requests Link to server Destination
2. 3-packet handshake establishes encrypted channel
3. Commands sent as Link packets with forward secrecy
4. Large outputs transferred via Resource system

### Shell Message Types (over Links)
- `0x10` - COMMAND_REQUEST
- `0x11` - COMMAND_RESPONSE
- `0x20` - DISCONNECT

See [docs/RETICULUM.md](docs/RETICULUM.md) for complete Reticulum protocol specification.
See [docs/PROTOCOL.md](docs/PROTOCOL.md) for shell-specific message details.

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

### Protocol & Architecture
- [Reticulum Protocol Reference](docs/RETICULUM.md) - **Complete Reticulum protocol specification**
- [Shell Protocol Specification](docs/PROTOCOL.md) - Shell-specific message format
- [Architecture Overview](.claude/architecture.md) - System design and components
- [Domain Concepts](.claude/concepts.md) - Reticulum terminology and concepts

### Setup & Usage
- [I2P Setup Guide](docs/I2P-SETUP.md) - External I2P router configuration
- [Embedded Router Guide](docs/EMBEDDED-ROUTER.md) - Built-in I2P router usage
- [Code Navigation](.claude/navigation.md) - Source code organization

## License

MIT License - see LICENSE file for details.

## Disclaimer

This tool is designed for authorized security testing and research purposes only. Users are responsible for ensuring they have proper authorization before using this tool. Unauthorized access to computer systems is illegal.

## Acknowledgments

- [Reticulum Network](https://reticulum.network/) - Protocol specification and reference implementation
- [markqvist/Reticulum](https://github.com/markqvist/Reticulum) - Python reference implementation
- [I2P Project](https://geti2p.net/) - Anonymous routing infrastructure
- [Emissary](https://github.com/altonen/emissary) - Pure Rust I2P router
- Rust community for excellent async and cryptography ecosystem
