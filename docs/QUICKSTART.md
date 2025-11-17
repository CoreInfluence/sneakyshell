# Quick Start Guide

Get up and running with Reticulum-Shell in 5 minutes!

## Build

```bash
cargo build --release
```

## Server Setup

### 1. Run Server (Auto-generates everything)

```bash
./target/release/shell-server
```

**On first run, this will:**
- Generate `server.identity` (your cryptographic identity)
- Create `server.toml` (configuration file)
- Display your server's destination hash

**Example output:**
```
INFO Generating new server identity at "server.identity"
INFO Server identity saved: a3f5c8d9e2b1a7c6f4e3d2c1b0a9f8e7d6c5b4a3f2e1d0c9b8a7f6e5d4c3b2a1
INFO Default configuration saved to "server.toml"
INFO Server destination: a3f5c8d9e2b1a7c6f4e3d2c1b0a9f8e7d6c5b4a3f2e1d0c9b8a7f6e5d4c3b2a1
INFO Listening on Reticulum network...
INFO Server running. Press Ctrl+C to stop.
```

**Copy the destination hash!** You'll need it for the client.

### 2. Configure (Optional)

Edit `server.toml` to customize:
- `max_sessions` - Max concurrent clients (default: 10)
- `command_timeout` - Max command execution time (default: 300s)
- `allowed_clients` - Whitelist specific client identities (default: allow all)

## Client Setup

### 1. Run Client (Auto-generates identity)

```bash
./target/release/shell-client
```

**On first run, this will:**
- Generate `client.identity`
- Create `client.toml` with placeholder server destination
- Prompt you to configure the server destination

**Example output:**
```
INFO Generating new client identity at "client.identity"
INFO Client identity saved: b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5
INFO Default configuration saved to "client.toml"
IMPORTANT: Edit "client.toml" and set the server_destination
```

### 2. Configure Server Destination

Edit `client.toml` and paste the server's destination hash:

```toml
server_destination = "a3f5c8d9e2b1a7c6f4e3d2c1b0a9f8e7d6c5b4a3f2e1d0c9b8a7f6e5d4c3b2a1"
```

**Or** pass it via command line:

```bash
./target/release/shell-client --server a3f5c8d9e2b1a7c6f4e3d2c1b0a9f8e7d6c5b4a3f2e1d0c9b8a7f6e5d4c3b2a1
```

### 3. Connect!

```bash
./target/release/shell-client
```

You'll see the interactive REPL:

```
Reticulum Shell Client
Type 'help' for commands, 'exit' to quit

rsh> whoami
root

rsh> uname -a
Linux server 6.17.7-300.fc43.x86_64 #1 SMP PREEMPT_DYNAMIC x86_64 GNU/Linux

rsh> exit
Goodbye!
```

## Usage Examples

### Interactive Mode

```bash
./target/release/shell-client
```

Then type commands:
```
rsh> ls -la /tmp
rsh> cat /etc/hostname
rsh> ps aux
rsh> exit
```

### Single Command Mode

Execute one command and exit:

```bash
./target/release/shell-client -e "whoami"
./target/release/shell-client -e "df -h"
./target/release/shell-client -e "cat /etc/os-release"
```

### Built-in Commands

- `help` - Show available commands
- `status` - Show connection status
- `clear` - Clear screen
- `exit`, `quit` - Disconnect and exit

## File Structure

After setup, you'll have:

```
reticulum-shell/
├── server.identity          # Server private key (KEEP SECURE!)
├── server.toml             # Server configuration
├── client.identity          # Client private key (KEEP SECURE!)
├── client.toml             # Client configuration
├── server-audit.log        # Audit log (if enabled)
└── target/release/
    ├── shell-server        # Server binary
    └── shell-client        # Client binary
```

## Security Notes

⚠️ **IMPORTANT:**
- **Never share** `.identity` files - they contain private keys!
- Set `chmod 600` on identity files:
  ```bash
  chmod 600 *.identity
  ```
- For production: configure `allowed_clients` in `server.toml`
- Audit logs track all executed commands

## Troubleshooting

### "Server destination not configured"

**Problem:** Client config has placeholder destination

**Solution:** Edit `client.toml` or use `--server <destination>`

### "Permission denied" on identity files

**Solution:**
```bash
chmod 600 server.identity client.identity
```

### "Connection refused" or "Not connected"

**Current Status:** I2P transport not yet implemented

The core functionality works, but actual network communication over I2P needs to be completed. For now, the protocol, authentication, and command execution are all tested and working.

## Next Steps

1. **Test locally** - Server and client binaries are working
2. **Customize** - Edit `.toml` files for your needs
3. **I2P Integration** - Next phase: implement I2P transport layer

See the main [README.md](../README.md) for detailed documentation.
