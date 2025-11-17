# Setup Guide

Complete installation and configuration guide for Reticulum-Shell.

## Prerequisites

### System Requirements

- **Operating System:** Linux (Ubuntu 20.04+, Fedora 35+, or similar)
- **Rust:** 1.70 or later
- **I2P Router:** Latest stable version (when I2P integration is complete)
- **Storage:** ~100 MB for build artifacts
- **Network:** Internet connection for dependency downloads

### Install Rust

```bash
# Install rustup (Rust installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add to PATH (add to ~/.bashrc or ~/.zshrc)
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Install I2P (Future)

**Note:** I2P integration is planned but not yet implemented.

```bash
# Ubuntu/Debian
sudo apt-add-repository ppa:i2p-maintainers/i2p
sudo apt update
sudo apt install i2p

# Fedora
sudo dnf install i2p

# Start I2P router
i2prouter start
```

## Building from Source

### Clone Repository

```bash
git clone https://github.com/yourusername/reticulum-shell.git
cd reticulum-shell
```

### Build All Crates

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (optimized, recommended)
cargo build --release
```

**Build Output:**
- Server: `target/release/shell-server`
- Client: `target/release/shell-client`

### Run Tests

```bash
# Run all tests
cargo test --all

# Run with output
cargo test --all -- --show-output

# Run specific crate tests
cargo test -p shell-proto
```

## Initial Configuration

### 1. Generate Server Identity

```bash
./target/release/shell-server --generate-identity server.identity
```

**Output:**
```
Generating new identity at "server.identity"
Identity saved: a3f5c8d9e2b1a7c6f4e3d2c1b0a9f8e7d6c5b4a3f2e1d0c9b8a7f6e5d4c3b2a1
```

Save this destination hash - clients will need it to connect.

### 2. Generate Client Identity

```bash
./target/release/shell-client --generate-identity client.identity
```

### 3. Create Server Configuration

Create `server.toml`:

```toml
# Path to server identity file
identity_path = "server.identity"

# Maximum number of concurrent client sessions
max_sessions = 10

# Default command execution timeout (seconds)
command_timeout = 300

# Enable audit logging
audit_logging = true
audit_log_path = "server-audit.log"

# Allowed client identities (hex-encoded public keys)
# Empty list = allow all clients
allowed_clients = []

# To restrict to specific clients:
# allowed_clients = [
#     "abc123...",
#     "def456...",
# ]
```

### 4. Create Client Configuration

Create `client.toml`:

```toml
# Path to client identity file
identity_path = "client.identity"

# Server destination hash (from step 1)
server_destination = "a3f5c8d9e2b1a7c6f4e3d2c1b0a9f8e7d6c5b4a3f2e1d0c9b8a7f6e5d4c3b2a1"

# Connection timeout (seconds)
connection_timeout = 30

# Command execution timeout (seconds)
command_timeout = 300
```

## Running the Server

### Basic Usage

```bash
./target/release/shell-server --config server.toml
```

### With Debug Logging

```bash
RUST_LOG=debug ./target/release/shell-server --config server.toml
```

### As a Service (systemd)

Create `/etc/systemd/system/reticulum-shell-server.service`:

```ini
[Unit]
Description=Reticulum Shell Server
After=network.target

[Service]
Type=simple
User=reticulum-shell
WorkingDirectory=/opt/reticulum-shell
ExecStart=/opt/reticulum-shell/shell-server --config /opt/reticulum-shell/server.toml
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

**Enable and start:**

```bash
sudo systemctl daemon-reload
sudo systemctl enable reticulum-shell-server
sudo systemctl start reticulum-shell-server
sudo systemctl status reticulum-shell-server
```

## Running the Client

### Interactive Mode

```bash
./target/release/shell-client --config client.toml
```

### Single Command Mode

```bash
./target/release/shell-client --config client.toml -e "whoami"
```

### Override Server Destination

```bash
./target/release/shell-client --server <destination-hash>
```

## Security Hardening

### Server Security

1. **Restrict Client Access:**

```toml
# server.toml
allowed_clients = [
    "trusted_client_pubkey_1",
    "trusted_client_pubkey_2",
]
```

2. **Run as Dedicated User:**

```bash
# Create dedicated user
sudo useradd -r -s /bin/false reticulum-shell

# Set ownership
sudo chown reticulum-shell:reticulum-shell /opt/reticulum-shell
sudo chmod 700 /opt/reticulum-shell

# Protect identity file
chmod 600 server.identity
```

3. **Enable Audit Logging:**

```toml
# server.toml
audit_logging = true
audit_log_path = "/var/log/reticulum-shell/audit.log"
```

4. **Configure Timeouts:**

```toml
# server.toml
command_timeout = 300  # 5 minutes max per command
max_sessions = 5       # Limit concurrent connections
```

### Client Security

1. **Protect Identity File:**

```bash
chmod 600 client.identity
```

2. **Use Configuration Files:**

Instead of CLI arguments (visible in process list), use config files.

3. **Verify Server Identity:**

Always verify the server's destination hash before connecting.

## Firewall Configuration

### I2P Ports (When Implemented)

```bash
# Allow I2P traffic
sudo ufw allow 4444/tcp   # I2P HTTP proxy
sudo ufw allow 6668/tcp   # I2P SAM bridge
sudo ufw allow 7656/tcp   # I2P console
```

### Local Testing

For local testing without I2P:

```bash
# Allow direct TCP (if using TCP interface)
sudo ufw allow 4242/tcp
```

## Troubleshooting

### Build Errors

**Error:** `cannot find crate 'xyz'`

**Solution:**
```bash
cargo clean
cargo build --release
```

**Error:** `linker 'cc' not found`

**Solution:**
```bash
# Ubuntu/Debian
sudo apt install build-essential

# Fedora
sudo dnf install gcc
```

### Runtime Errors

**Error:** `Identity file not found`

**Solution:**
```bash
# Generate identity
./target/release/shell-server --generate-identity server.identity
```

**Error:** `Permission denied`

**Solution:**
```bash
# Check file permissions
ls -l server.identity
chmod 600 server.identity

# Check executable permissions
chmod +x target/release/shell-server
```

**Error:** `Connection refused`

**Solution:**
1. Ensure server is running
2. Check server destination hash
3. Verify I2P router is running (when implemented)
4. Check firewall rules

### Logging

Enable debug logging for diagnostics:

```bash
RUST_LOG=debug ./target/release/shell-server --config server.toml 2>&1 | tee server.log
```

**Log Levels:**
- `error` - Critical errors only
- `warn` - Warnings and errors
- `info` - General information (default)
- `debug` - Detailed debugging
- `trace` - Very verbose (all events)

### Common Issues

1. **"Protocol version mismatch"**
   - Client and server versions differ
   - Rebuild both from same source

2. **"Maximum sessions reached"**
   - Increase `max_sessions` in `server.toml`
   - Close inactive sessions

3. **"Command timed out"**
   - Increase `command_timeout` in config
   - Check server resources

## Performance Tuning

### Server Optimization

```toml
# server.toml
max_sessions = 50              # Increase if needed
command_timeout = 600          # Longer for slow commands
```

### Build Optimizations

For maximum performance:

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

Rebuild:
```bash
cargo build --release
```

## Updating

### Update from Git

```bash
cd reticulum-shell
git pull
cargo build --release
```

### Migration Notes

**v0.1.0 → v0.2.0:** (Future)
- Configuration format may change
- Check release notes for migration guide

## Backup

### Important Files

Backup these files regularly:

```bash
# Identities (CRITICAL - cannot be recovered if lost)
server.identity
client.identity

# Configurations
server.toml
client.toml

# Audit logs (if enabled)
server-audit.log
```

### Backup Script

```bash
#!/bin/bash
BACKUP_DIR="/backup/reticulum-shell/$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"
cp server.identity client.identity *.toml "$BACKUP_DIR/"
```

## Next Steps

- Read [Usage Guide](USAGE.md) for command reference
- Review [Protocol Specification](PROTOCOL.md) for protocol details
- Check [Architecture Overview](../.claude/architecture.md) for design
