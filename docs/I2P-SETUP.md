# I2P Integration Setup Guide

This guide explains how to use Reticulum-Shell over the I2P (Invisible Internet Project) anonymous network.

## What is I2P?

I2P is an anonymous overlay network that provides:
- **Garlic routing**: Multi-layered encryption
- **Hidden services**: Anonymous endpoints
- **Peer-to-peer architecture**: No central servers
- **Resistance to traffic analysis**: Protection against surveillance

## Prerequisites

### Install I2P Router

#### Option 1: I2P Router (Java-based)

**Linux:**
```bash
# Download and install I2P
wget https://geti2p.net/en/download/latest -O i2pinstall.jar
java -jar i2pinstall.jar -console

# Start I2P router
~/i2p/i2prouter start
```

**macOS:**
```bash
# Using Homebrew
brew install i2p

# Start I2P router
brew services start i2p
```

#### Option 2: i2pd (C++ implementation)

**Linux (Debian/Ubuntu):**
```bash
sudo apt-get install i2pd
sudo systemctl enable i2pd
sudo systemctl start i2pd
```

**Arch Linux:**
```bash
sudo pacman -S i2pd
sudo systemctl enable i2pd
sudo systemctl start i2pd
```

### Enable SAM Bridge

The SAM (Simple Anonymous Messaging) bridge must be enabled for Reticulum-Shell to communicate with I2P.

**For I2P Router (Java):**
1. Open web console: http://127.0.0.1:7657
2. Navigate to: Configure → Clients
3. Enable "SAM application bridge"
4. Default port: 7656

**For i2pd:**
SAM is enabled by default on port 7656. Check `/etc/i2pd/i2pd.conf`:
```ini
[sam]
enabled = true
address = 127.0.0.1
port = 7656
```

## Using Reticulum-Shell with I2P

### 1. Start Server with I2P

```bash
./target/release/shell-server --enable-i2p
```

**Expected output:**
```
INFO  Connecting to I2P network via SAM bridge at 127.0.0.1:7656
INFO  SAM handshake successful
INFO  Generated I2P destination: LS0tLS1CRUdJTiBJMlAgREVT...
INFO  SAM DATAGRAM session created: retic-a1b2c3d4...
INFO  I2P interface created successfully
INFO  I2P destination: LS0tLS1CRUdJTiBJMlAgREVT...
INFO  I2P destination hash: a1b2c3d4e5f6...
INFO  Listening on Reticulum network...
```

**Copy the I2P destination** (the long base64 string starting with "LS0tLS..."). The client will need this.

### 2. Start Client with I2P

```bash
./target/release/shell-client --enable-i2p --i2p-destination "LS0tLS1CRUdJTiBJMlAgREVT..."
```

Replace the destination string with the actual server I2P destination from step 1.

**Expected output:**
```
INFO  Connecting to I2P network via SAM bridge at 127.0.0.1:7656
INFO  SAM handshake successful
INFO  Generated I2P destination: LS0tLS1CRUdJTiBJMlAgREVT...
INFO  Client I2P destination: LS0tLS1CRUdJTiBJMlAgREVT...
INFO  Registering server I2P destination: LS0tLS1CRUdJTiBJMlAgREVT...
INFO  Server I2P destination hash: a1b2c3d4e5f6...
INFO  Connected to server
shell>
```

### 3. Execute Commands

Once connected, use the interactive shell:

```bash
shell> whoami
zero

shell> ps -ef
UID        PID  PPID  C STIME TTY          TIME CMD
root         1     0  0 10:30 ?        00:00:00 /sbin/init
...

shell> exit
```

## Configuration Files

### Server Configuration (server.toml)

```toml
identity_path = "server.identity"
max_sessions = 10
command_timeout = 300
audit_logging = true
audit_log_path = "audit.log"
allowed_clients = []

# I2P Configuration
enable_i2p = true
sam_address = "127.0.0.1:7656"
```

### Client Configuration (client.toml)

```toml
identity_path = "client.identity"
server_destination = "0000000000000000000000000000000000000000000000000000000000000000"
connection_timeout = 30
command_timeout = 300

# I2P Configuration
enable_i2p = true
sam_address = "127.0.0.1:7656"
server_i2p_destination = "LS0tLS1CRUdJTiBJMlAgREVT..."  # Server's I2P destination
```

## Command Line Options

### Server

```bash
shell-server [OPTIONS]

Options:
      --enable-i2p              Enable I2P transport
      --sam-address <ADDRESS>   SAM bridge address (default: 127.0.0.1:7656)
  -c, --config <FILE>           Path to configuration file
  -v, --verbose                 Verbose logging
      --generate-identity <PATH> Generate a new identity and exit
  -h, --help                    Print help
```

### Client

```bash
shell-client [OPTIONS]

Options:
      --enable-i2p                     Enable I2P transport
      --sam-address <ADDRESS>          SAM bridge address (default: 127.0.0.1:7656)
      --i2p-destination <DESTINATION>  Server I2P destination (base64 string)
  -s, --server <DESTINATION>           Server destination (hex string, for non-I2P)
  -c, --config <FILE>                  Path to configuration file
  -v, --verbose                        Verbose logging
  -e, --execute <COMMAND>              Execute a single command and exit
  -h, --help                           Print help
```

## Troubleshooting

### Error: "Failed to connect to SAM"

**Problem:** I2P router not running or SAM bridge disabled

**Solution:**
1. Check I2P router status:
   ```bash
   # For I2P Router
   curl http://127.0.0.1:7657

   # For i2pd
   sudo systemctl status i2pd
   ```

2. Verify SAM bridge is listening:
   ```bash
   netstat -tulpn | grep 7656
   # or
   ss -tulpn | grep 7656
   ```

3. Check SAM bridge is enabled in I2P configuration

### Error: "Unknown destination - not registered"

**Problem:** Client doesn't have the server's I2P destination

**Solution:** Make sure you're passing the correct server I2P destination via `--i2p-destination` flag

### Slow First Connection

**Expected behavior:** First connection over I2P may take 30-60 seconds while tunnels are established. Subsequent connections will be faster.

### Connection Timeout

**Problem:** I2P tunnels not fully established

**Solution:**
1. Wait 2-3 minutes after starting I2P router
2. Check I2P router console for tunnel status
3. Increase connection timeout in client config

## Security Considerations

### I2P Destination Security

- **Destination = Public Identity**: The I2P destination string contains your public key
- **Share carefully**: Only share your server destination with authorized clients
- **Rotation**: Consider regenerating identities periodically for enhanced security

### Network Anonymity

- **I2P provides anonymity**: Both client and server IP addresses are hidden
- **Timing attacks**: Be aware that traffic correlation may still be possible
- **Tunnel length**: Default I2P tunnel length provides good anonymity

### Operational Security

1. **Don't mix I2P and non-I2P modes**: Use one transport method consistently
2. **Monitor SAM bridge**: Restrict SAM bridge to localhost only
3. **Firewall rules**: Ensure I2P router ports are properly configured
4. **Audit logging**: Enable audit logging on the server for accountability

## How It Works

### I2P Destination Mapping

Reticulum-Shell uses 32-byte destination hashes internally, but I2P destinations are ~500+ bytes. The system:

1. **Server generates** an I2P destination via SAM
2. **Client registers** the server's I2P destination
3. **Hash mapping**: SHA-256 hash of I2P destination → 32-byte identifier
4. **Automatic routing**: Client and server maintain destination mapping tables
5. **Transparent communication**: Protocol messages flow over I2P DATAGRAM sessions

### SAM Protocol Flow

```
Client                    SAM Bridge                    Server
  |                            |                            |
  |-- HELLO VERSION ---------->|                            |
  |<- HELLO REPLY OK -----------|                            |
  |                            |                            |
  |-- SESSION CREATE --------->|                            |
  |<- SESSION STATUS OK --------|                            |
  |                            |                            |
  |-- DATAGRAM SEND ---------->|--- I2P Tunnel ------------>|
  |                            |                            |
  |                            |<-- I2P Tunnel -------------|
  |<- DATAGRAM RECEIVED -------|                            |
```

## Performance

- **Latency**: Expect 500ms - 3000ms latency (I2P overhead)
- **Bandwidth**: DATAGRAM mode suitable for command execution
- **Reliability**: I2P provides best-effort delivery
- **Tunnel establishment**: 30-60 seconds for first connection

## Advanced Configuration

### Custom SAM Port

If your SAM bridge runs on a different port:

```bash
shell-server --enable-i2p --sam-address 127.0.0.1:7656
shell-client --enable-i2p --sam-address 127.0.0.1:7656 --i2p-destination "..."
```

### Remote SAM Bridge

**Warning:** Only use over trusted networks (VPN/localhost)

```bash
shell-server --enable-i2p --sam-address 192.168.1.100:7656
```

### Multiple I2P Routers

Run separate I2P instances for client and server if needed:

```bash
# Server with I2P router on port 7656
shell-server --enable-i2p

# Client with I2P router on port 7657
shell-client --enable-i2p --sam-address 127.0.0.1:7657 --i2p-destination "..."
```

## Testing I2P Integration

### 1. Manual I2P Test (without running actual I2P)

You can test the build without I2P by running without the --enable-i2p flag:

```bash
# This will use MockInterface for testing
cargo test
```

All tests should pass.

### 2. Test with Real I2P

Requires I2P router running:

```bash
# Run ignored tests that require I2P
cargo test --all -- --ignored
```

### 3. End-to-End Test Over I2P

Terminal 1 (Server):
```bash
./target/release/shell-server --enable-i2p -v
# Copy the I2P destination
```

Terminal 2 (Client):
```bash
./target/release/shell-client --enable-i2p --i2p-destination "PASTE_DESTINATION" -v -e "whoami"
```

Should execute command and exit.

## Further Reading

- [I2P Documentation](https://geti2p.net/en/docs)
- [SAM Protocol Specification](https://geti2p.net/en/docs/api/samv3)
- [I2P Network Database](https://geti2p.net/en/docs/how/network-database)
- [i2pd Documentation](https://i2pd.readthedocs.io/)
