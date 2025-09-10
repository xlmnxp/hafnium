# Hafnium Handshake Protocol

This document describes the cryptographic handshake protocol implemented in Hafnium for secure peer-to-peer communication.

## Overview

The Hafnium handshake protocol establishes a secure encrypted channel between a client and server using:
- **P-256 ECDH** for key exchange
- **ChaCha20Poly1305** for data encryption
- **Base58** encoding for public key representation
- **Bincode** for message serialization

## Protocol Flow

### 1. Client Initiates Connection (HELO)

The client sends a `HELO` message containing:
- Client's static public key
- Client's ephemeral public key

```rust
HandshakeMessage::Helo {
    public_key: Vec<u8>,           // Client's static public key (SEC1 format)
    ephemeral_public_key: Vec<u8>, // Client's ephemeral public key (SEC1 format)
}
```

### 2. Server Responds (EHLO)

The server responds with an `EHLO` message containing:
- Server's static public key
- Server's ephemeral public key

```rust
HandshakeMessage::Ehlo {
    public_key: Vec<u8>,           // Server's static public key (SEC1 format)
    ephemeral_public_key: Vec<u8>, // Server's ephemeral public key (SEC1 format)
}
```

### 3. Shared Secret Derivation

Both parties compute two shared secrets:
1. **Static shared secret**: `ECDH(client_static_private, server_static_public)`
2. **Ephemeral shared secret**: `ECDH(client_ephemeral_private, server_ephemeral_public)`

The final shared secret is derived by XORing both secrets:
```rust
combined_secret = static_shared_secret ⊕ ephemeral_shared_secret
```

### 4. Encrypted Communication

After handshake completion, all subsequent communication uses:
- **ChaCha20Poly1305** encryption with the combined shared secret
- **XNonce** (192-bit) for each message
- **Length-prefixed** message format

## Message Format

### Handshake Messages

All handshake messages are serialized using Bincode and sent with a 4-byte length prefix:

```
[4 bytes: length][bincode serialized message]
```

### Encrypted Messages

After handshake, messages are encrypted and sent with length prefix:

```
[4 bytes: length][24 bytes: nonce][encrypted data]
```

## Security Features

### Forward Secrecy
- Uses ephemeral keys for each handshake
- Even if static keys are compromised, past communications remain secure

### Authentication
- Both parties authenticate each other through public key verification
- Shared secret derivation ensures only parties with correct private keys can communicate

### Encryption
- ChaCha20Poly1305 provides authenticated encryption
- Each message uses a unique nonce
- Protects against replay attacks

## Implementation Details

### Key Generation
```rust
// Generate static key pair (stored in hafnium.toml)
let private_key = EphemeralSecret::new();
let public_key = private_key.public_key();

// Generate ephemeral key pair (per connection)
let ephemeral_secret = EphemeralSecret::new();
let ephemeral_public = ephemeral_secret.public_key();
```

### Handshake Client
```rust
let handshake_client = HandshakeClient::new();
let result = handshake_client.perform_handshake(&mut stream).await?;
// result contains peer_public_key and shared_secret
```

### Handshake Server
```rust
let handshake_server = HandshakeServer::new();
let result = handshake_server.perform_handshake(&mut stream).await?;
// result contains peer_public_key and shared_secret
```

### Encrypted Communication
```rust
// Send encrypted message
send_encrypted_message(&mut stream, data, &shared_secret).await?;

// Receive encrypted message
let decrypted = receive_encrypted_message(&mut stream, &shared_secret).await?;
```

## Error Handling

The protocol includes comprehensive error handling:

- **Invalid public keys**: Rejected during handshake
- **Handshake timeout**: 30-second timeout for handshake completion
- **Decryption failures**: Connection terminated on invalid encrypted data
- **Message format errors**: Proper error responses sent

## Testing

Run the handshake test:
```bash
./test_handshake.sh
```

This test:
1. Generates key pairs for server and client
2. Starts the server daemon
3. Connects a client and performs handshake
4. Verifies encrypted communication
5. Cleans up resources

## Security Considerations

### Current Implementation
- ✅ P-256 ECDH for key exchange
- ✅ ChaCha20Poly1305 for encryption
- ✅ Ephemeral keys for forward secrecy
- ✅ Authenticated encryption
- ✅ Proper nonce handling

### Future Improvements
- 🔄 HKDF for shared secret derivation (instead of XOR)
- 🔄 Certificate-based authentication
- 🔄 Perfect Forward Secrecy with key rotation
- 🔄 Message authentication beyond encryption

## Usage Example

### Server Side
```rust
// In daemon.rs
let handshake_server = HandshakeServer::new();
let result = handshake_server.perform_handshake(&mut stream).await?;

// Store peer information
let peer = Peer {
    id: peer_id.clone(),
    addr,
    public_key: base58_encode(&result.peer_public_key),
    shared_secret: result.shared_secret,
    // ...
};
```

### Client Side
```rust
// In client.rs
let handshake_client = HandshakeClient::new();
let result = handshake_client.perform_handshake(&mut stream).await?;

// Send encrypted message
send_encrypted_message(&mut stream, b"Hello!", &result.shared_secret).await?;
```

This handshake implementation provides a solid foundation for secure peer-to-peer communication in the Hafnium reverse proxy system.
