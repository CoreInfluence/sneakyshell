# Reticulum-Shell Project Context

**Last Updated:** 2025-11-17

## Project Overview

Reticulum-Shell is a remote access tool built for cybersecurity research. It provides remote shell access to Linux systems using a **full Rust implementation of the Reticulum network protocol** with I2P as an anonymous transport layer.

**Key Features:**
- Full Reticulum protocol implementation in Rust
- X25519 + Ed25519 dual-keypair identities
- Link-based encrypted channels with forward secrecy
- Announce/path discovery for mesh routing
- Multiple transports: I2P, TCP, UDP, Local IPC
- Embedded I2P router (no external dependencies)
- Zero-configuration deployment

## Current Status

**Phase:** Phase 4 - Reticulum Protocol Implementation 🚧
**Progress:** Documentation complete, implementation starting

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

#### Phase 3 - Embedded Router ✅ (COMPLETED)
- [x] **Emissary Integration** - Pure Rust I2P implementation
- [x] **Automatic Reseeding** - Downloads 100 router infos from HTTPS servers
- [x] **SAM Server** - Embedded SAM v3 server for internal communication
- [x] **NTCP2 Transport** - TCP-based I2P router-to-router protocol
- [x] **RouterMode Configuration** - External vs Embedded router selection
- [x] **Server CLI Integration** - `--use-embedded-router` flag
- [x] **Client CLI Integration** - `--use-embedded-router` flag
- [x] **Documentation** - Complete embedded router guide (docs/EMBEDDED-ROUTER.md)
- [x] **Updated README** - Embedded router usage instructions
- [x] **Claude Context** - Updated architecture.md, concepts.md, navigation.md
- [x] **Bootstrapping Fix** - Resolved tunnel building issues
- [x] **All tests passing** - Build succeeds with embedded-router feature

**Key Implementation Details:**
- emissary-core + emissary-util git dependencies
- Automatic HTTPS reseeding from 12 trusted servers
- 100 router infos downloaded and verified on startup
- SHA-256 hashing + digital signature verification
- NTCP2 with random encryption keys per session
- First run: 2-5 minutes (reseed 30-60s + tunnel build 90-240s)
- Subsequent runs: 30-90 seconds (cached peers)
- Memory: 64-256 MB depending on tunnel quantity
- Single-binary deployment with zero external dependencies

### Phase 4 - Reticulum Protocol Implementation (In Progress)

#### 4.1 Core Protocol Foundation
- [ ] X25519 key exchange (add x25519-dalek)
- [ ] HKDF key derivation (add hkdf crate)
- [ ] AES-256-CBC + HMAC Token cipher
- [ ] ECIES encryption scheme
- [ ] Dual-keypair Identity system
- [ ] Complete packet wire format

#### 4.2 Transport Layer & Routing
- [ ] Transport core with path table
- [ ] Interface trait abstraction
- [ ] Announce/path discovery
- [ ] Message routing logic

#### 4.3 Link Establishment
- [ ] Link state machine
- [ ] 3-packet handshake
- [ ] Key derivation (HKDF)
- [ ] Keepalive management

#### 4.4 I2P Transport Integration
- [ ] Wrap existing SAM code in Interface trait
- [ ] HDLC/KISS framing
- [ ] Destination mapping

#### 4.5 Resource Transfer
- [ ] Chunking system
- [ ] Sliding window transfer
- [ ] BZ2 compression

#### 4.6 Shell Integration
- [ ] Update shell-proto for Links
- [ ] Commands over Link channels
- [ ] Large outputs via Resources

### Future Phases
- [ ] Additional transports (TCP, UDP, Local IPC)
- [ ] Interactive PTY support (vim, top, etc.)
- [ ] File transfer via Reticulum Resources
- [ ] Multi-hop mesh routing
- [ ] Interoperability testing with Python Reticulum

## Known Issues & Decisions

1. **Language Choice:** Rust selected for memory safety, performance, and strong async ecosystem
2. **Shell Model:** Starting with command execution model (not interactive PTY) for MVP
3. **Reticulum Protocol:** Full Rust implementation for wire compatibility with Python reference
4. **Identity System:** Dual-keypair (X25519 + Ed25519) as per Reticulum specification
5. **I2P Integration:** Custom SAM v3 implementation wrapped in Reticulum Interface trait
6. **Transport Abstraction:** Reticulum Interface trait allows I2P, TCP, UDP, Local transports
7. **Embedded Router:** Emissary (pure Rust I2P) selected for zero-dependency deployment
8. **Git Dependencies:** Using Emissary from GitHub to get zip 6.0 fix (not yet on crates.io)
9. **Reseeding:** Automatic HTTPS reseed on first run for network bootstrap (100 router infos)
10. **Bootstrap Timing:** First run 2-5 minutes (normal for I2P), subsequent runs 30-90 seconds
11. **Wire Compatibility:** All packets must be byte-exact with Python Reticulum implementation
12. **Link-Based Communication:** Shell commands sent over Reticulum Links for forward secrecy

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

### I2P Mode with Embedded Router (Recommended)
```bash
# Build with embedded router feature
cargo build --release --features embedded-router

# Run server (downloads router infos, builds tunnels, starts SAM)
./target/release/shell-server --enable-i2p --use-embedded-router

# Copy the I2P destination (long base64 string), then run client
./target/release/shell-client --enable-i2p --use-embedded-router \
  --i2p-destination "LS0tLS..."
```

**First connection takes 2-5 minutes:**
- Downloading router infos: 30-60 seconds
- Building I2P tunnels: 90-240 seconds
- Subsequent connections: 30-90 seconds

### I2P Mode with External Router
```bash
# Prerequisite: I2P router running with SAM bridge on port 7656

# Run server with I2P
./target/release/shell-server --enable-i2p

# Copy the I2P destination (long base64 string), then run client
./target/release/shell-client --enable-i2p --i2p-destination "LS0tLS..."
```

**Everything auto-configures on first run!**

See:
- `docs/EMBEDDED-ROUTER.md` for embedded router usage and configuration
- `docs/I2P-SETUP.md` for external I2P router installation

## Security Research Context

This project is developed for:
- Authorized security testing and penetration testing
- Red team operations research
- Understanding anonymous network protocols
- Educational purposes in cybersecurity

**Not intended for:** Unauthorized access, malicious use, or production deployments without proper security hardening.

## Next Steps

### Immediate (Phase 4.1)
1. Add cryptographic dependencies (x25519-dalek, hkdf, aes, cbc, hmac)
2. Implement dual-keypair Identity system
3. Implement ECIES encryption with Token cipher
4. Define Reticulum packet structures

### Short-term (Phase 4.2-4.3)
5. Implement Transport core with routing tables
6. Create Interface trait abstraction
7. Implement Link state machine and handshake
8. Add announce/path discovery

### Medium-term (Phase 4.4-4.6)
9. Wrap I2P SAM in Interface trait
10. Implement Resource transfer system
11. Update shell-proto for Link-based communication
12. Test interoperability with Python Reticulum
