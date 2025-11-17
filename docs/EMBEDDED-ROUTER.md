# Embedded I2P Router Guide

**Version:** 1.0
**Feature:** `embedded-router`
**Status:** Stable

## Table of Contents

- [Overview](#overview)
- [When to Use Embedded Router](#when-to-use-embedded-router)
- [Building with Embedded Router](#building-with-embedded-router)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Server Setup](#server-setup)
- [Client Setup](#client-setup)
- [Testing the Connection](#testing-the-connection)
- [Troubleshooting](#troubleshooting)
- [Performance Tuning](#performance-tuning)
- [Architecture](#architecture)

---

## Overview

The **embedded I2P router** feature allows `reticulum-shell` to run its own I2P router process internally, eliminating the need to install and run a separate I2P router (like i2pd or Java I2P). This significantly simplifies deployment and reduces system dependencies.

### Key Benefits

✅ **Single-binary deployment** - No external I2P router required
✅ **Simplified setup** - No SAM bridge configuration needed
✅ **Portable** - Works out of the box on any supported platform
✅ **Automatic lifecycle** - Router starts/stops with your application
✅ **Lower resource usage** - Lightweight pure-Rust I2P implementation

### Implementation

The embedded router uses **Emissary**, a pure Rust I2P implementation, which provides:
- Full I2P network compatibility
- Tokio-based async runtime (matches reticulum-shell)
- Built-in SAM v3 server for internal communication
- Automatic tunnel management and NetDB operations

---

## When to Use Embedded Router

### ✅ Use Embedded Router When:

- **Deploying to environments without I2P** - Cloud servers, containers, air-gapped systems
- **Simplicity is priority** - You want "install and run" experience
- **Moderate traffic volume** - Typical shell session usage (< 100MB/day)
- **Testing/Development** - Quick local testing without infrastructure setup
- **Single-user scenarios** - One client per router instance

### ❌ Use External Router When:

- **High traffic volume** - Heavy file transfers, multiple concurrent connections
- **Shared I2P services** - Multiple applications using same I2P router
- **Existing I2P setup** - You already run i2pd/Java I2P for other services
- **Advanced routing features** - Floodfill participation, custom tunnel configs
- **Resource constraints** - Very limited memory (< 64MB available)

---

## Building with Embedded Router

### Prerequisites

```bash
# Rust toolchain (1.70+)
rustc --version

# Git (for Emissary dependency)
git --version
```

### Build Instructions

```bash
# Clone repository
cd /path/to/reticulum-shell

# Build with embedded router feature
cargo build --release --features embedded-router

# Binaries will be in target/release/
ls -lh target/release/shell-{server,client}
```

**Build time:** First build takes ~3-5 minutes (downloads and compiles Emissary).
**Binary size:** Approximately 15-20MB per binary (optimized release build).

### Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Linux x86_64 | ✅ Tested | Primary development platform |
| Linux ARM64 | ✅ Should work | Emissary supports ARM |
| macOS x86_64 | ✅ Should work | Standard Rust target |
| macOS ARM64 | ✅ Should work | Apple Silicon compatible |
| Windows x86_64 | ⚠️ Untested | May require firewall configuration |

---

## Quick Start

### 1. Start Server with Embedded Router

```bash
# Build with embedded router
cargo build --release --features embedded-router

# Start server
./target/release/shell-server \
  --enable-i2p \
  --use-embedded-router

# Server will:
# 1. Start embedded I2P router
# 2. Wait 30-60 seconds for tunnels to establish
# 3. Display I2P destination for clients
```

**Expected output:**
```
INFO  Initializing embedded I2P router
INFO  Data directory: ".reticulum-shell/i2p"
INFO  Embedded I2P router started successfully
INFO  SAM TCP port: 37421
INFO  Waiting for I2P tunnels to establish (may take 30-60 seconds)...
INFO  I2P router ready
INFO  Connecting to embedded router via SAM...
INFO  I2P interface created successfully
INFO  I2P destination: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789...
INFO  I2P destination hash: 1a2b3c4d5e6f7890abcdef1234567890abcdef1234567890abcdef1234567890
INFO  Listening on Reticulum network...
```

**⚠️ IMPORTANT:** Save the I2P destination (base64 string) - clients need this to connect!

### 2. Start Client with Embedded Router

```bash
# Start client (use server's I2P destination)
./target/release/shell-client \
  --enable-i2p \
  --use-embedded-router \
  --i2p-destination "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789..."

# Client will:
# 1. Start its own embedded I2P router
# 2. Wait for tunnels
# 3. Connect to server via I2P
```

**Expected output:**
```
INFO  Initializing embedded I2P router
INFO  Embedded I2P router started successfully
INFO  SAM TCP port: 42156
INFO  I2P router ready
INFO  Connecting to embedded router via SAM...
INFO  Client I2P destination: XYZ789abc...
INFO  Registering server I2P destination...
INFO  Connected to server
reticulum-shell>
```

### 3. Test the Connection

```bash
# In client REPL
reticulum-shell> echo "Hello from I2P!"
Hello from I2P!

reticulum-shell> pwd
/home/user

reticulum-shell> exit
```

---

## Configuration

### Server Configuration File (`server.toml`)

```toml
# Basic settings
identity_path = "server.identity"
max_sessions = 10
command_timeout = 300
audit_logging = true
audit_log_path = "audit.log"
allowed_clients = []

# I2P configuration
enable_i2p = true
router_mode = "Embedded"  # Use "External" for traditional SAM

# External router SAM address (only used if router_mode = "External")
sam_address = "127.0.0.1:7656"

# Embedded router configuration
[embedded_router]
data_dir = ".reticulum-shell/i2p"  # Router data directory
bandwidth_limit_kbps = 2048         # 2 MB/s bandwidth limit
tunnel_quantity = 2                 # Number of tunnels to maintain
enable_floodfill = false            # Don't act as directory server
listen_port = 0                     # Random port (0) or specific port
sam_tcp_port = 0                    # Random SAM port (0) or specific
sam_udp_port = 0                    # Random SAM UDP port
```

### Client Configuration File (`client.toml`)

```toml
# Basic settings
identity_path = "client.identity"
server_destination = "0000000000000000000000000000000000000000000000000000000000000000"
connection_timeout = 30
command_timeout = 300

# I2P configuration
enable_i2p = true
router_mode = "Embedded"  # Use "External" for traditional SAM
sam_address = "127.0.0.1:7656"

# Server's I2P destination (base64 string from server startup)
server_i2p_destination = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789..."

# Embedded router configuration
[embedded_router]
data_dir = ".reticulum-shell-client/i2p"
bandwidth_limit_kbps = 2048
tunnel_quantity = 2
enable_floodfill = false
listen_port = 0
sam_tcp_port = 0
sam_udp_port = 0
```

### Configuration Options Explained

| Option | Default | Description |
|--------|---------|-------------|
| `router_mode` | `"External"` | `"External"` or `"Embedded"` |
| `data_dir` | `.reticulum-shell/i2p` | Directory for NetDB and router state |
| `bandwidth_limit_kbps` | `2048` | Bandwidth limit in KB/s (2 MB/s default) |
| `tunnel_quantity` | `2` | Number of inbound/outbound tunnels |
| `enable_floodfill` | `false` | Act as I2P directory server (not recommended) |
| `listen_port` | `0` | I2P router port (0 = random) |
| `sam_tcp_port` | `0` | Internal SAM TCP port (0 = random) |
| `sam_udp_port` | `0` | Internal SAM UDP port (0 = random) |

---

## Server Setup

### Method 1: Using Configuration File (Recommended)

```bash
# 1. Create configuration
cat > server.toml <<EOF
identity_path = "server.identity"
enable_i2p = true
router_mode = "Embedded"

[embedded_router]
data_dir = "/var/lib/reticulum-shell/i2p"
bandwidth_limit_kbps = 4096  # 4 MB/s for production
tunnel_quantity = 3
EOF

# 2. Start server
./shell-server --config server.toml

# 3. Note the I2P destination from logs
# Example: I2P destination: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789...
```

### Method 2: Using CLI Flags

```bash
# Start with all settings via command line
./shell-server \
  --enable-i2p \
  --use-embedded-router \
  --config server.toml
```

### Systemd Service (Linux)

```ini
# /etc/systemd/system/reticulum-shell-server.service
[Unit]
Description=Reticulum Shell Server (Embedded I2P)
After=network.target

[Service]
Type=simple
User=reticulum
Group=reticulum
WorkingDirectory=/opt/reticulum-shell
ExecStart=/opt/reticulum-shell/shell-server --config /etc/reticulum-shell/server.toml
Restart=always
RestartSec=10

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/reticulum-shell

[Install]
WantedBy=multi-user.target
```

```bash
# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable reticulum-shell-server
sudo systemctl start reticulum-shell-server

# View logs
sudo journalctl -u reticulum-shell-server -f

# Check I2P destination
sudo journalctl -u reticulum-shell-server | grep "I2P destination:"
```

### Docker Container

```dockerfile
# Dockerfile
FROM rust:1.75-slim as builder

WORKDIR /build
COPY . .
RUN cargo build --release --features embedded-router

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/shell-server /usr/local/bin/
COPY server.toml /etc/reticulum-shell/

RUN useradd -m -u 1000 reticulum && \
    mkdir -p /var/lib/reticulum-shell/i2p && \
    chown -R reticulum:reticulum /var/lib/reticulum-shell

USER reticulum
WORKDIR /home/reticulum

EXPOSE 8080
CMD ["shell-server", "--config", "/etc/reticulum-shell/server.toml"]
```

```bash
# Build and run
docker build -t reticulum-shell-server .
docker run -d \
  --name reticulum-server \
  -v /var/lib/reticulum-shell:/var/lib/reticulum-shell \
  reticulum-shell-server

# Get I2P destination
docker logs reticulum-server | grep "I2P destination:"
```

---

## Client Setup

### Interactive Mode

```bash
# 1. Create configuration with server's I2P destination
cat > client.toml <<EOF
identity_path = "client.identity"
enable_i2p = true
router_mode = "Embedded"
server_i2p_destination = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789..."

[embedded_router]
data_dir = ".reticulum-shell-client/i2p"
EOF

# 2. Connect to server
./shell-client --config client.toml

# 3. Wait for connection (30-60 seconds first time)
# 4. Use interactive REPL
reticulum-shell> ls
reticulum-shell> hostname
reticulum-shell> exit
```

### Single Command Mode

```bash
# Execute single command and exit
./shell-client \
  --config client.toml \
  --execute "uptime"

# Example: Get server's disk usage
./shell-client \
  --config client.toml \
  --execute "df -h"
```

### Scripting

```bash
#!/bin/bash
# check-servers.sh - Check multiple servers via I2P

SERVERS=(
    "SERVER1_I2P_DESTINATION"
    "SERVER2_I2P_DESTINATION"
    "SERVER3_I2P_DESTINATION"
)

for dest in "${SERVERS[@]}"; do
    echo "Checking server: ${dest:0:20}..."

    ./shell-client \
        --enable-i2p \
        --use-embedded-router \
        --i2p-destination "$dest" \
        --execute "hostname && uptime" \
        || echo "Failed to connect"

    echo "---"
done
```

---

## Testing the Connection

### Step-by-Step Test Procedure

#### 1. Start Server

```bash
# Terminal 1: Start server
./shell-server --enable-i2p --use-embedded-router -v

# Wait for this message:
# INFO  I2P destination: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789...
# INFO  Listening on Reticulum network...

# Copy the I2P destination!
```

#### 2. Start Client

```bash
# Terminal 2: Start client (paste server's I2P destination)
./shell-client \
  --enable-i2p \
  --use-embedded-router \
  --i2p-destination "PASTE_SERVER_DESTINATION_HERE" \
  -v

# Wait for:
# INFO  Connected to server
# reticulum-shell>
```

#### 3. Run Test Commands

```bash
# Test basic command
reticulum-shell> echo "Connection successful!"

# Test environment
reticulum-shell> env | grep USER

# Test current directory
reticulum-shell> pwd

# Test file operations
reticulum-shell> ls -la

# Test command with arguments
reticulum-shell> uname -a

# Exit
reticulum-shell> exit
```

### Verify I2P Communication

```bash
# On server: Check router is running
ps aux | grep shell-server

# Check SAM port is listening
netstat -tuln | grep -E "37[0-9]{3}|127.0.0.1"

# On client: Same checks
ps aux | grep shell-client
netstat -tuln | grep -E "4[0-9]{4}|127.0.0.1"
```

### Performance Test

```bash
# Measure round-trip time
reticulum-shell> time echo "test"

# Typical latency over I2P: 2-10 seconds (depending on tunnel state)
```

---

## Troubleshooting

### Router Fails to Start

**Symptom:**
```
ERROR Failed to start embedded router: Failed to start router: ...
```

**Causes & Solutions:**

1. **Permission Denied on Data Directory**
   ```bash
   # Fix permissions
   mkdir -p .reticulum-shell/i2p
   chmod 700 .reticulum-shell/i2p
   ```

2. **Port Already in Use**
   ```bash
   # Check if another I2P router is running
   ps aux | grep -E "i2p|emissary"

   # Use specific ports in config if needed
   [embedded_router]
   listen_port = 12345  # Instead of 0 (random)
   ```

3. **Out of Memory**
   ```bash
   # Check available memory
   free -h

   # Reduce bandwidth limit in config
   [embedded_router]
   bandwidth_limit_kbps = 512  # Lower limit
   tunnel_quantity = 1         # Fewer tunnels
   ```

### Tunnels Not Establishing

**Symptom:**
```
INFO  Waiting for I2P tunnels to establish (may take 30-60 seconds)...
(hangs forever)
```

**Causes & Solutions:**

1. **Firewall Blocking I2P**
   ```bash
   # Check firewall rules
   sudo iptables -L -n

   # Allow I2P ports (if using specific port)
   sudo iptables -A INPUT -p tcp --dport 12345 -j ACCEPT
   sudo iptables -A INPUT -p udp --dport 12345 -j ACCEPT
   ```

2. **Network Connectivity Issues**
   ```bash
   # Test general internet connectivity
   ping -c 4 8.8.8.8

   # Check DNS resolution
   nslookup google.com

   # Test I2P reseeding (requires internet)
   curl -I https://reseed.i2p-projekt.de/
   ```

3. **Bootstrapping on First Run**
   - First run takes 1-3 minutes to bootstrap
   - Router needs to discover peers
   - Be patient on initial startup

   ```bash
   # Enable debug logging to see progress
   RUST_LOG=debug ./shell-server --enable-i2p --use-embedded-router
   ```

### Connection Timeout

**Symptom:**
```
ERROR Failed to create I2P interface: Connection timeout
```

**Solutions:**

1. **Increase Timeout**
   ```toml
   # In client.toml
   connection_timeout = 120  # 2 minutes instead of 30 seconds
   ```

2. **Verify Server is Running**
   ```bash
   # On server machine
   ps aux | grep shell-server

   # Check logs
   journalctl -u reticulum-shell-server -n 50
   ```

3. **Verify I2P Destination**
   ```bash
   # Double-check you copied the FULL destination
   # Should be ~516 characters (base64)
   echo "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789..." | wc -c
   ```

### SAM Connection Failed

**Symptom:**
```
ERROR Failed to create I2P interface: SAM not enabled in embedded router
```

**Solutions:**

1. **Check Router Configuration**
   ```toml
   [embedded_router]
   sam_tcp_port = 0  # Must be Some value, not None
   ```

2. **Wait for Router to Fully Start**
   ```bash
   # Ensure you see this message before connecting:
   # INFO  SAM TCP port: 37421
   ```

### High Memory Usage

**Symptom:**
- Process using > 256MB RAM
- System becoming slow

**Solutions:**

1. **Reduce Router Resources**
   ```toml
   [embedded_router]
   bandwidth_limit_kbps = 512   # Lower bandwidth
   tunnel_quantity = 1          # Minimum tunnels
   enable_floodfill = false     # Don't be directory server
   ```

2. **Monitor Memory Usage**
   ```bash
   # Check process memory
   ps aux | grep shell-server | awk '{print $6/1024 " MB"}'

   # Monitor over time
   watch -n 5 'ps aux | grep shell-server | awk "{print \$6/1024 \" MB\"}"'
   ```

3. **Consider External Router**
   - If memory is critical, use lightweight i2pd instead
   - External router can be shared across multiple applications

### Data Directory Corruption

**Symptom:**
```
ERROR Failed to start router: NetDB error
```

**Solution:**

```bash
# Backup and reset router data
mv .reticulum-shell/i2p .reticulum-shell/i2p.backup
mkdir -p .reticulum-shell/i2p

# Restart server (will bootstrap from scratch)
./shell-server --enable-i2p --use-embedded-router
```

### Slow Performance

**Symptoms:**
- Commands take > 10 seconds
- Frequent timeouts
- Poor throughput

**Diagnostic Steps:**

1. **Check Tunnel Status**
   ```bash
   # Enable verbose logging
   ./shell-server --enable-i2p --use-embedded-router -v

   # Look for tunnel-related messages
   grep -i "tunnel" server.log
   ```

2. **Increase Tunnel Quantity**
   ```toml
   [embedded_router]
   tunnel_quantity = 3  # More tunnels = better redundancy
   ```

3. **Check Network Latency**
   ```bash
   # Test base network latency
   ping -c 10 8.8.8.8

   # I2P adds 2-10s latency - this is normal
   ```

4. **Verify Sufficient Bandwidth**
   ```toml
   [embedded_router]
   bandwidth_limit_kbps = 4096  # Allow more bandwidth
   ```

### Debug Logging

**Enable full debug output:**

```bash
# Server
RUST_LOG=debug ./shell-server --enable-i2p --use-embedded-router 2>&1 | tee server-debug.log

# Client
RUST_LOG=debug ./shell-client --enable-i2p --use-embedded-router --i2p-destination "..." 2>&1 | tee client-debug.log
```

**Log locations:**
- Server: `server-debug.log`
- Audit log: `audit.log` (if enabled)
- Router data: `.reticulum-shell/i2p/`

---

## Performance Tuning

### Optimizing for Different Scenarios

#### Low-Latency Configuration

```toml
[embedded_router]
tunnel_quantity = 4              # More tunnels = better failover
bandwidth_limit_kbps = 8192      # 8 MB/s
```

**Use case:** Interactive shell sessions, real-time commands

#### Low-Memory Configuration

```toml
[embedded_router]
tunnel_quantity = 1              # Minimum tunnels
bandwidth_limit_kbps = 256       # 256 KB/s
```

**Use case:** Embedded devices, limited RAM environments

#### High-Throughput Configuration

```toml
[embedded_router]
tunnel_quantity = 5              # Many tunnels
bandwidth_limit_kbps = 16384     # 16 MB/s
```

**Use case:** File transfers, bulk operations

### Recommended Settings by Environment

| Environment | Tunnels | Bandwidth | Memory Usage |
|-------------|---------|-----------|--------------|
| Development | 2 | 2048 KB/s | ~64-128 MB |
| Production Server | 3-4 | 4096 KB/s | ~128-192 MB |
| Edge Device | 1-2 | 512 KB/s | ~32-64 MB |
| High Traffic | 4-5 | 8192 KB/s | ~192-256 MB |

### Startup Time Optimization

**First Run:** 1-3 minutes (bootstrapping)
**Subsequent Runs:** 30-60 seconds (cached NetDB)

```bash
# Keep data directory persistent across runs
[embedded_router]
data_dir = "/persistent/storage/i2p"  # Not in temp directory
```

---

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────┐
│                  reticulum-shell                        │
│                                                         │
│  ┌─────────────┐         ┌──────────────────┐         │
│  │   Server    │         │     Client       │         │
│  │    CLI      │         │      CLI         │         │
│  └──────┬──────┘         └────────┬─────────┘         │
│         │                         │                    │
│         ▼                         ▼                    │
│  ┌────────────────────────────────────────────┐       │
│  │         I2pInterface (SAM Client)          │       │
│  └──────────────┬─────────────────────────────┘       │
│                 │ (Internal TCP)                       │
│                 ▼                                      │
│  ┌─────────────────────────────────────────────┐      │
│  │        Embedded Router (Emissary)           │      │
│  │  ┌────────────┐  ┌──────────┐  ┌─────────┐ │      │
│  │  │ SAM Server │  │ Tunnels  │  │  NetDB  │ │      │
│  │  └────────────┘  └──────────┘  └─────────┘ │      │
│  └──────────────┬──────────────────────────────┘      │
│                 │                                      │
└─────────────────┼──────────────────────────────────────┘
                  │
                  ▼
         ┌────────────────┐
         │  I2P Network   │
         └────────────────┘
```

### Communication Flow

1. **Startup:**
   - EmbeddedRouter creates Emissary instance
   - Emissary starts SAM server on random port
   - Router begins bootstrapping (NetDB, tunnels)

2. **Connection Establishment:**
   - I2pInterface connects to local SAM port
   - Creates DATAGRAM session
   - Generates I2P destination

3. **Message Flow:**
   - Client → I2pInterface → SAM → Emissary → I2P Network
   - I2P Network → Emissary → SAM → I2pInterface → Server

### Comparison: Embedded vs External

| Aspect | Embedded Router | External Router |
|--------|----------------|-----------------|
| **Installation** | None (built-in) | i2pd or Java I2P |
| **Configuration** | Minimal | SAM bridge setup |
| **Dependencies** | None | External process |
| **Memory** | 64-256 MB | 128-512 MB (i2pd) |
| **Startup Time** | 30-60 seconds | Already running |
| **Shared Usage** | No | Yes (multiple apps) |
| **Control** | Full | Limited |
| **Portability** | Excellent | Requires install |

---

## Additional Resources

### Documentation

- [I2P Setup Guide](I2P-SETUP.md) - External I2P router setup
- [Protocol Documentation](PROTOCOL.md) - Reticulum protocol details
- [Quick Start Guide](QUICKSTART.md) - Basic usage tutorial

### External Links

- [Emissary GitHub](https://github.com/altonen/emissary) - Embedded router implementation
- [I2P Project](https://geti2p.net/) - I2P network documentation
- [SAM v3 Specification](https://geti2p.net/en/docs/api/samv3) - SAM protocol details

### Support

For issues with:
- **Embedded router feature**: Check this guide's troubleshooting section
- **Reticulum-shell bugs**: Open issue on project repository
- **I2P network issues**: See https://geti2p.net/en/about/intro

---

## Changelog

### Version 1.0 (2025-11-16)
- Initial embedded router implementation
- Emissary integration via git dependency
- CLI flags and configuration support
- Full server/client support

---

**End of Guide**
