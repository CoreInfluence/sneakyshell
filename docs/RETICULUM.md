# Reticulum Protocol Reference

This document provides a comprehensive reference for the Reticulum network protocol as implemented in reticulum-shell. The implementation aims for wire-format compatibility with the [Python reference implementation](https://github.com/markqvist/Reticulum).

## Overview

Reticulum is a cryptography-based networking stack designed for resilient mesh networking across any physical medium. It provides:

- **End-to-end encryption** with forward secrecy
- **Identity-based addressing** using public key cryptography
- **Transport agnostic** - works over I2P, TCP, UDP, LoRa, packet radio, etc.
- **Mesh routing** with announce/path discovery
- **Reliable delivery** via Link-based channels and Resources

## Table of Contents

1. [Identity System](#identity-system)
2. [Destination System](#destination-system)
3. [Packet Format](#packet-format)
4. [Link Establishment](#link-establishment)
5. [Path Discovery](#path-discovery)
6. [Resource Transfer](#resource-transfer)
7. [Transport Interfaces](#transport-interfaces)
8. [Cryptographic Primitives](#cryptographic-primitives)

---

## Identity System

### Dual-Keypair Architecture

Each Reticulum identity consists of two keypairs:

| Keypair | Algorithm | Size | Purpose |
|---------|-----------|------|---------|
| Encryption | X25519 | 256 bits | Asymmetric encryption, key exchange |
| Signature | Ed25519 | 256 bits | Digital signatures, authentication |

**Total public identity size:** 512 bits (64 bytes)

### Address Derivation

```
Full Hash    = SHA-256(X25519_public || Ed25519_public)  [32 bytes]
Address Hash = First 128 bits of Full Hash               [16 bytes]
```

The 16-byte truncated hash is used as the identity address throughout the network.

### Ratchet Keys (Forward Secrecy)

Identities can generate rotating X25519 keys for forward secrecy:

- **Generation interval:** 30 minutes (default)
- **Retention period:** Up to 30 days
- **Maximum stored:** 512 keys per destination
- **Key ID:** 80-bit truncated hash of key

Ratchet keys are announced with destination advertisements for recipients to use.

### Rust Data Structure

```rust
pub struct Identity {
    // Public components
    pub x25519_public: [u8; 32],
    pub ed25519_public: [u8; 32],
    pub hash: [u8; 16],        // Truncated address
    pub full_hash: [u8; 32],   // Full SHA-256

    // Private components (for own identity)
    x25519_private: Option<[u8; 32]>,
    ed25519_private: Option<[u8; 64]>,

    // Ratchet keys
    ratchet_keys: Vec<RatchetKey>,
}
```

---

## Destination System

### Destination Types

| Type | Value | Encryption | Use Case |
|------|-------|------------|----------|
| SINGLE | 0x00 | ECIES (asymmetric) | Point-to-point encrypted |
| GROUP | 0x01 | Symmetric (pre-shared) | Multi-party encrypted |
| PLAIN | 0x02 | None | Unencrypted broadcast |
| LINK | 0x03 | Link-derived keys | Established channels |

### Direction

- **IN (0x11):** Receive and decrypt packets
- **OUT (0x12):** Send encrypted packets

### Naming Convention

Destinations are named hierarchically:
```
app_name.aspect1.aspect2...aspectN
```

The name is hashed via SHA-256 to produce the destination address.

### Rust Data Structure

```rust
pub struct Destination {
    pub hash: [u8; 16],
    pub dest_type: DestinationType,
    pub direction: Direction,
    pub identity: Option<Identity>,
    pub name: String,
    pub aspects: Vec<String>,

    // Callbacks
    pub link_established_callback: Option<LinkCallback>,
    pub packet_callback: Option<PacketCallback>,

    // GROUP-specific
    symmetric_key: Option<[u8; 32]>,

    // Ratchet configuration
    ratchets_enabled: bool,
    ratchet_interval: Duration,
}
```

---

## Packet Format

### Overall Structure

```
[HEADER 2 bytes] [ADDRESSES 16/32 bytes] [CONTEXT 1 byte] [DATA 0-465 bytes]
```

Optional IFAC (Interface Authentication Code) field after header for authenticated interfaces.

### Header Byte 1 (Flags)

```
Bit 7: IFAC Flag      (0=open, 1=authenticated)
Bit 6: Header Type    (0=Type1, 1=Type2)
Bit 5: Context Flag   (0=no context, 1=has context)
Bit 4: Propagation    (0=broadcast, 1=transport)
Bits 3-2: Dest Type   (00=single, 01=group, 10=plain, 11=link)
Bits 1-0: Packet Type (00=data, 01=announce, 10=linkrequest, 11=proof)
```

### Header Byte 2

- Hop count (8 bits, max 128)

### Address Fields

- **Type 1 Header:** Single 16-byte destination hash
- **Type 2 Header:** Two 16-byte fields (transport_id + destination)

### Packet Types

| Type | Value | Description |
|------|-------|-------------|
| DATA | 0x00 | Standard data transmission |
| ANNOUNCE | 0x01 | Destination advertisement |
| LINKREQUEST | 0x02 | Link establishment request |
| PROOF | 0x03 | Delivery/link confirmation |

### Context Values

```rust
pub enum PacketContext {
    None = 0x00,
    Resource = 0x01,
    ResourceAdv = 0x02,
    ResourceReq = 0x03,
    ResourceHmu = 0x04,
    ResourcePrf = 0x05,
    ResourceIcl = 0x06,
    ResourceRcl = 0x07,
    CacheRequest = 0x08,
    Request = 0x09,
    Response = 0x0A,
    PathResponse = 0x0B,
    Command = 0x0C,
    CommandStatus = 0x0D,
    Channel = 0x0E,
    Keepalive = 0xFA,
    LinkIdentify = 0xFB,
    LinkClose = 0xFC,
    LinkProof = 0xFD,
    LinkRtt = 0xFE,
    LinkProofRtt = 0xFF,
}
```

### MTU Constants

```rust
const NETWORK_MTU: usize = 500;        // Maximum packet size
const MDU: usize = 402;                 // Maximum data unit
const ENCRYPTED_MDU: usize = 383;       // Encrypted payload limit
const PLAIN_MDU: usize = 464;           // Unencrypted payload limit
const TRUNCATED_HASH_LEN: usize = 16;   // Address length
```

---

## Link Establishment

Links provide bidirectional encrypted channels with forward secrecy.

### State Machine

```
PENDING → HANDSHAKE → ACTIVE → STALE → CLOSED
```

| State | Description |
|-------|-------------|
| PENDING | Link request sent, awaiting response |
| HANDSHAKE | DH exchange complete, awaiting proof |
| ACTIVE | Fully operational |
| STALE | No recent traffic |
| CLOSED | Terminated |

### 3-Packet Handshake (297 bytes total)

**Step 1: Link Request (83 bytes)**
- Initiator generates ephemeral X25519 + Ed25519 keypairs
- Sends: X25519 public + Ed25519 public + optional data

**Step 2: Link Proof (115 bytes)**
- Responder generates keypairs, performs X25519 ECDH
- Sends: Signature over link_id + public keys
- Derives shared secret

**Step 3: RTT Measurement (99 bytes)**
- Initiator validates proof
- Sends RTT measurement packet
- Link becomes ACTIVE

### Key Derivation

```rust
// X25519 ECDH
let shared_secret = x25519_ecdh(private_key, peer_public_key);  // 32 bytes

// HKDF expansion
let derived_key = hkdf_sha256(
    salt: link_id,
    ikm: shared_secret,
    info: None,
    length: 64
);

// Split for Token cipher
let signing_key = &derived_key[0..32];    // HMAC key
let encryption_key = &derived_key[32..64]; // AES key
```

### Keepalive Management

```rust
fn calculate_keepalive(rtt: Duration) -> Duration {
    let interval = rtt.as_secs_f64() * (360.0 / 1.75);
    Duration::from_secs_f64(interval.clamp(5.0, 360.0))
}

const STALE_TIME: Duration = 2 * keepalive;
const STALE_GRACE: Duration = Duration::from_secs(5);
```

### Closing Reasons

```rust
pub enum LinkClosingReason {
    Timeout = 0x01,
    InitiatorClosed = 0x02,
    DestinationClosed = 0x03,
}
```

---

## Path Discovery

### Announce Packets

Announce packets advertise destinations to the network:

1. Destination hash (16 bytes)
2. Public key material (64 bytes)
3. Optional ratchet key
4. Random blob (replay prevention)
5. Application-specific data
6. Ed25519 signature

### Propagation Rules

1. **Duplicate rejection:** Exact announces are ignored
2. **Hop tracking:** Record source and hop count
3. **Maximum hops:** 128 (PATHFINDER_M)
4. **Path expiration:** 7 days (PATHFINDER_E)

### Timing Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Path request timeout | 15 seconds | Time to wait for path response |
| Minimum request interval | 20 seconds | Minimum time between requests |
| Announce bandwidth cap | 2% | Maximum interface bandwidth for announces |

### Path Table

```rust
pub struct PathEntry {
    pub destination_hash: [u8; 16],
    pub next_hop: [u8; 16],
    pub hops: u8,
    pub expires: Instant,
    pub received_from: InterfaceId,
    pub announce_packet_hash: [u8; 32],
}

pub struct PathTable {
    entries: HashMap<[u8; 16], PathEntry>,
}
```

---

## Resource Transfer

Resources enable reliable transfer of large data over Links.

### Chunking

- **SDU:** Service Data Unit = Packet MDU
- **Parts:** `ceil(data_size / sdu)`
- **Compression:** BZ2 for data <= 64MB (if smaller than original)

### Transfer Protocol

**Sender:**
1. Advertise resource with hashmap segment
2. Receive part requests
3. Send requested parts
4. Await completion proof

**Receiver:**
1. Accept advertisement
2. Request parts via sliding window
3. Assemble received parts
4. Send proof (hash of data)

### Window Management

```rust
const WINDOW_MIN: usize = 2;
const WINDOW_MAX_FAST: usize = 75;
const RATE_FAST: u32 = 50_000;     // 50 Kbps
const RATE_VERY_SLOW: u32 = 2_000; // 2 Kbps
```

### Retry Logic

```rust
const MAX_RETRIES: u32 = 16;
const MAX_ADV_RETRIES: u32 = 4;
const PART_TIMEOUT_FACTOR: f64 = 4.0;
```

---

## Transport Interfaces

### Interface Trait

```rust
pub trait Interface: Send + Sync {
    fn send(&self, data: &[u8]) -> Result<()>;
    fn receive(&self) -> Option<Vec<u8>>;
    fn hw_mtu(&self) -> usize;
    fn bitrate(&self) -> u32;
    fn name(&self) -> &str;
    fn mode(&self) -> InterfaceMode;
    fn is_online(&self) -> bool;
}

pub enum InterfaceMode {
    Full,
    PointToPoint,
    AccessPoint,
    Roaming,
    Boundary,
    Gateway,
}
```

### Available Interfaces

| Interface | Description | Default MTU | Framing |
|-----------|-------------|-------------|---------|
| I2P | Anonymous routing via SAM | 1064 bytes | None |
| TCP | Point-to-point streams | 500 bytes | HDLC/KISS |
| UDP | Broadcast/unicast | 1064 bytes | None |
| Local | IPC between instances | 500 bytes | HDLC |

### I2P Interface

```rust
pub struct I2pInterface {
    sam_address: SocketAddr,  // 127.0.0.1:7656
    session_id: String,
    destination: String,
    dest_hash: [u8; 32],
}

const I2P_DEFAULT_MTU: usize = 1064;
const I2P_BITRATE: u32 = 256_000;  // 256 kbps
```

---

## Cryptographic Primitives

### Required Algorithms

| Function | Algorithm | Key Size | Notes |
|----------|-----------|----------|-------|
| Key Exchange | X25519 | 256 bits | Curve25519 ECDH |
| Signatures | Ed25519 | 256 bits | EdDSA |
| Key Derivation | HKDF-SHA256 | Variable | RFC 5869 |
| Symmetric Encryption | AES-256-CBC | 256 bits | PKCS7 padding |
| MAC | HMAC-SHA256 | 256 bits | Message authentication |
| Hashing | SHA-256, SHA-512 | 256/512 bits | Various uses |

### ECIES Encryption (SINGLE destinations)

```rust
fn encrypt_ecies(
    plaintext: &[u8],
    recipient_public: &[u8; 32],
    identity_hash: &[u8; 32]
) -> Vec<u8> {
    // 1. Generate ephemeral X25519 keypair
    let (ephemeral_private, ephemeral_public) = x25519_generate();

    // 2. ECDH with recipient
    let shared = x25519_ecdh(&ephemeral_private, recipient_public);

    // 3. HKDF key derivation
    let derived = hkdf_sha256(identity_hash, shared, None, 64);

    // 4. Token cipher (AES-256-CBC + HMAC)
    let ciphertext = token_encrypt(plaintext, &derived);

    // 5. Return ephemeral public + ciphertext
    [ephemeral_public.to_vec(), ciphertext].concat()
}
```

### Token Cipher Format

```
[IV 16 bytes] + [ciphertext] + [HMAC-SHA256 32 bytes]
```

**Total overhead:** 48 bytes

**Key split from 64-byte derived key:**
- Bytes 0-31: HMAC signing key
- Bytes 32-63: AES encryption key

---

## Implementation Status

### Completed
- [x] Basic I2P transport (SAM v3)
- [x] Ed25519 signatures
- [x] Embedded I2P router

### In Progress
- [ ] X25519 key exchange
- [ ] Dual-keypair identities
- [ ] ECIES encryption
- [ ] Link establishment
- [ ] Packet serialization

### Planned
- [ ] Path discovery
- [ ] Resource transfer
- [ ] TCP/UDP interfaces
- [ ] Channel abstraction

---

## References

- [Reticulum Network](https://reticulum.network/) - Official documentation
- [Python Reference Implementation](https://github.com/markqvist/Reticulum) - Authoritative source
- [reticulum-rs](https://github.com/BeechatNetworkSystemsLtd/reticulum-rs) - Existing Rust implementation
- [I2P SAM v3 Specification](https://geti2p.net/en/docs/api/samv3) - I2P transport protocol

---

## Wire Format Compatibility

The implementation must produce byte-exact packets compatible with the Python reference. Key considerations:

1. **Byte ordering:** Big-endian for network fields
2. **Hash truncation:** Exactly 16 bytes for addresses
3. **Signature format:** Ed25519 standard (64 bytes)
4. **Padding:** PKCS7 for AES-CBC

Test all packet types against the Python implementation during development.
