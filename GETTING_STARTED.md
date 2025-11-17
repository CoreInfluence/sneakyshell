# Getting Started with Reticulum-Shell

## TL;DR - 30 Second Setup

```bash
# Build
cargo build --release

# Terminal 1 - Start server (copies destination hash)
./target/release/shell-server

# Terminal 2 - Connect client
./target/release/shell-client --server <paste-destination-hash>
```

Done! 🎉

---

## Detailed Walkthrough

### Prerequisites

- **Rust 1.70+**: Install from https://rustup.rs
- **Linux**: Currently targets Linux (Ubuntu, Fedora, etc.)
- **Git**: For cloning the repository

### Step 1: Build

```bash
git clone https://github.com/yourusername/reticulum-shell.git
cd reticulum-shell
cargo build --release
```

**Build time:** ~2-3 minutes on first build

**Output binaries:**
- `target/release/shell-server` (2.2 MB)
- `target/release/shell-client` (2.9 MB)

### Step 2: Start the Server

```bash
./target/release/shell-server
```

**What happens:**
1. ✅ Generates cryptographic identity → `server.identity`
2. ✅ Creates default config → `server.toml`
3. ✅ Displays server destination hash (save this!)
4. ✅ Starts listening

**Example output:**
```
INFO Configuration file not found, creating default configuration
INFO Generating new server identity at "server.identity"
INFO Server identity saved: 3c9e01621c7aa4c88034d015fb464453d090328b83cca17492062575fa0afb09
INFO Default configuration saved to "server.toml"
INFO Server destination: 3c9e01621c7aa4c88034d015fb464453d090328b83cca17492062575fa0afb09
INFO Listening on Reticulum network...
INFO Server running. Press Ctrl+C to stop.
```

**Important:** Copy the destination hash (the long hex string after "Server destination:")

### Step 3: Connect with Client

**Option A: Command-line argument (easiest)**

```bash
./target/release/shell-client --server 3c9e01621c7aa4c88034d015fb464453d090328b83cca17492062575fa0afb09
```

**Option B: Configuration file**

```bash
# First run creates client.toml
./target/release/shell-client

# Edit client.toml
nano client.toml
# Change: server_destination = "3c9e01621c7aa4c88034d015fb464453d090328b83cca17492062575fa0afb09"

# Run again
./target/release/shell-client
```

### Step 4: Use the REPL

Once connected:

```
Reticulum Shell Client
Type 'help' for commands, 'exit' to quit

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

## File Structure After Setup

```
reticulum-shell/
├── server.identity          # Server private key (64 bytes) 🔒
├── server.toml             # Server config (auto-generated)
├── client.identity          # Client private key (64 bytes) 🔒
├── client.toml             # Client config (auto-generated)
└── target/release/
    ├── shell-server        # Server binary
    └── shell-client        # Client binary
```

## Configuration

### Server Configuration (server.toml)

```toml
identity_path = "server.identity"
max_sessions = 10              # Max concurrent clients
command_timeout = 300          # 5 minutes max per command
audit_logging = true           # Log all commands
audit_log_path = "audit.log"
allowed_clients = []           # Empty = allow all
```

**Restrict to specific clients:**
```toml
allowed_clients = [
    "0150f06cd506d13d8350d59b6cee2c6231c6d9a197de858302b99b7af7b441b6",
]
```

### Client Configuration (client.toml)

```toml
identity_path = "client.identity"
server_destination = "3c9e..."  # 64-char hex from server
connection_timeout = 30
command_timeout = 300
```

## Security Best Practices

### 🔒 Protect Identity Files

```bash
# Set restrictive permissions
chmod 600 server.identity client.identity

# Never commit to git
echo "*.identity" >> .gitignore
```

### 🛡️ Production Deployment

1. **Run as dedicated user:**
   ```bash
   sudo useradd -r -s /bin/false reticulum-shell
   sudo -u reticulum-shell ./target/release/shell-server
   ```

2. **Configure allowed clients:**
   ```toml
   # server.toml
   allowed_clients = [
       "trusted_client_identity_1",
       "trusted_client_identity_2",
   ]
   ```

3. **Enable audit logging:**
   ```toml
   audit_logging = true
   audit_log_path = "/var/log/reticulum-shell/audit.log"
   ```

## Advanced Usage

### Generate Identities Separately

```bash
# Generate without starting server
./target/release/shell-server --generate-identity my-server.identity

./target/release/shell-client --generate-identity my-client.identity
```

### Custom Config Locations

```bash
# Server
./target/release/shell-server --config /etc/reticulum-shell/server.toml

# Client
./target/release/shell-client --config ~/.config/reticulum-shell/client.toml
```

### Single Command Execution

```bash
# Execute one command and exit
./target/release/shell-client --server <destination> -e "whoami"
```

### Verbose Logging

```bash
# Debug mode
./target/release/shell-server --verbose

# Or with environment variable
RUST_LOG=debug ./target/release/shell-server
```

## Troubleshooting

### "Server destination not configured"

**Symptom:** Client shows error on startup

**Solution:** Either:
- Use `--server <destination>` flag, OR
- Edit `client.toml` and set `server_destination`

### "Permission denied" on identity files

**Solution:**
```bash
chmod 600 *.identity
```

### Identity files already exist

The binaries will **reuse existing identities** if found. To start fresh:

```bash
rm server.identity client.identity server.toml client.toml
```

### Build fails

```bash
# Clean and rebuild
cargo clean
cargo build --release
```

## Current Status

✅ **Working:**
- Cryptographic identity generation (Ed25519)
- Binary protocol with message framing
- Command execution framework
- Interactive REPL
- Auto-configuration
- Security validations

⏳ **In Progress:**
- I2P transport layer integration
- End-to-end network communication

**Note:** The protocol, authentication, and command execution are fully implemented and tested. The I2P transport layer is the remaining piece for full remote connectivity.

## Next Steps

1. ✅ Build and test locally
2. ✅ Familiarize yourself with the REPL
3. 📖 Read [Protocol Specification](docs/PROTOCOL.md)
4. 🔧 [Contribute](CONTRIBUTING.md) to I2P integration

## Getting Help

- **Documentation:** See `docs/` directory
- **Architecture:** Read `.claude/architecture.md`
- **Issues:** Report at [GitHub Issues](https://github.com/yourusername/reticulum-shell/issues)

---

**Ready to dive deeper?** Check out the full [README.md](README.md) and [Protocol Specification](docs/PROTOCOL.md).
