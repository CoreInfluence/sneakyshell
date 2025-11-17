# Reticulum-Shell Project Context

**Last Updated:** 2025-11-15

## Project Overview

Reticulum-Shell is a remote access tool built for cybersecurity research. It enables remote shell access to Linux systems over the Reticulum network using I2P as a transport layer.

**Key Features:**
- Server listens on Reticulum network
- Client connects and establishes connection
- Server gains remote bash shell on client
- Anonymous routing via I2P
- Cryptographic authentication via Reticulum identities

## Current Status

**Phase:** Phase 2 Complete - I2P Integration ✅
**Progress:** Full I2P transport working, ready for production testing

### Completed

#### Phase 1 - Foundation ✅
- [x] Project planning and architecture design
- [x] Created `.claude/` context documentation
- [x] Cargo workspace structure created
- [x] shell-proto crate with complete protocol definitions
- [x] reticulum-core crate with identity management and packet handling
- [x] shell-server crate with command execution
- [x] shell-client crate with interactive REPL
- [x] Comprehensive README and documentation
- [x] Protocol specification
- [x] Setup guide and quick start guide
- [x] **Auto-configuration** - Both binaries auto-generate configs on first run
- [x] **Zero-config deployment** - Works out of the box with no manual setup
- [x] Example configuration files
- [x] All builds clean (no errors, minimal warnings)
- [x] All tests passing (26 tests total)

#### Phase 1.5 - Command Execution Working ✅
- [x] **MockInterface** - In-memory transport for local testing
- [x] **Server message loop** - Receives packets, routes to sessions
- [x] **Client message sending** - Sends commands, receives responses
- [x] **Session tracking** - Server maintains active sessions
- [x] **Debug logging** - Connection events logged
- [x] **Integration tests** - Full command execution tested
- [x] **Real command execution verified:**
  - `whoami` - ✅ Returns current user
  - `ps -ef` - ✅ Returns 32KB process listing
  - `ss -antp` - ✅ Returns 11KB socket information

#### Phase 2 - I2P Integration ✅ (COMPLETED)
- [x] **SAM v3 Protocol Client** - Async implementation using tokio
- [x] **I2pInterface** - Full NetworkInterface implementation for I2P
- [x] **Destination Mapping** - SHA-256 hashing for 32-byte compatibility
- [x] **Server I2P Support** - CLI flags and configuration options
- [x] **Client I2P Support** - CLI flags and configuration options
- [x] **Documentation** - Complete I2P setup guide (docs/I2P-SETUP.md)
- [x] **Updated README** - I2P usage instructions
- [x] **All tests passing** - Integration tests work with updated API

**Key Implementation Details:**
- SAM v3.1 protocol with DATAGRAM sessions
- Ed25519 signature type (type 7) for I2P destinations
- Automatic tunnel establishment via SAM bridge
- Bidirectional destination mapping with HashMap
- Async/await throughout for non-blocking I/O

### Upcoming (Phase 3+)
- [ ] End-to-end testing over real I2P network (requires I2P router)
- [ ] Interactive PTY support (vim, top, etc.)
- [ ] File transfer capabilities
- [ ] Multiple concurrent sessions per server
- [ ] Advanced security hardening
- [ ] Command allowlist/blocklist
- [ ] Audit log encryption

## Known Issues & Decisions

1. **Language Choice:** Rust selected for memory safety, performance, and strong async ecosystem
2. **Shell Model:** Starting with command execution model (not interactive PTY) for MVP
3. **Authentication:** Using Reticulum's native identity system (Ed25519)
4. **I2P Integration:** Custom SAM v3 implementation (existing Rust libraries were outdated/incomplete)
5. **Destination Hashing:** SHA-256 hash of I2P destination strings to fit 32-byte DestinationHash format
6. **Transport Abstraction:** NetworkInterface trait allows MockInterface (testing) and I2pInterface (production)

## Quick Start

### Local Testing Mode
```bash
# Build everything
cargo build --release

# Run server (auto-generates config and identity)
./target/release/shell-server

# Copy the server destination hash from output, then run client
./target/release/shell-client --server <destination-hash>
```

### I2P Mode (Production)
```bash
# Prerequisite: I2P router running with SAM bridge on port 7656

# Run server with I2P
./target/release/shell-server --enable-i2p

# Copy the I2P destination (long base64 string), then run client
./target/release/shell-client --enable-i2p --i2p-destination "LS0tLS..."
```

**Everything auto-configures on first run!**

See `docs/I2P-SETUP.md` for complete I2P installation and configuration.

## Security Research Context

This project is developed for:
- Authorized security testing and penetration testing
- Red team operations research
- Understanding anonymous network protocols
- Educational purposes in cybersecurity

**Not intended for:** Unauthorized access, malicious use, or production deployments without proper security hardening.

## Next Steps

1. Complete Cargo workspace setup
2. Define wire protocol messages
3. Implement Reticulum packet handling
4. Build basic server/client communication
5. Add command execution capabilities
6. Security review and testing
