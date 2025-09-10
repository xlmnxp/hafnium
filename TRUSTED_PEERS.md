# Hafnium Trusted Peer System

This document describes the trusted peer management system in Hafnium, which provides secure access control for peer-to-peer connections.

## Overview

The trusted peer system ensures that only authorized peers can connect to a Hafnium server. This adds an additional layer of security beyond the cryptographic handshake by implementing a whitelist-based access control mechanism.

## Key Features

- **Whitelist-based Access Control**: Only peers in the trusted list can connect
- **Public Key Authentication**: Peers are identified by their public keys
- **Server Public Key Verification**: Clients must provide the correct server public key
- **CLI Management**: Easy management of trusted peers through command-line interface
- **Persistent Storage**: Trusted peer list is stored in the configuration file

## Workflow

### 1. Server Setup

1. **Generate Server Key Pair**:
   ```bash
   hf gen-key
   ```

2. **Get Server Public Key**:
   ```bash
   hf get-self
   # Output: Public Key: <base58-encoded-public-key>
   ```

3. **Share Server Public Key**: Provide the public key to clients who need to connect

### 2. Client Setup

1. **Generate Client Key Pair**:
   ```bash
   hf gen-key
   ```

2. **Get Client Public Key**:
   ```bash
   hf get-self
   # Output: Public Key: <base58-encoded-public-key>
   ```

3. **Share Client Public Key**: Provide the public key to the server administrator

### 3. Server Administrator Setup

1. **Add Client to Trusted List**:
   ```bash
   hf peer add <client-public-key>
   ```

2. **Verify Trusted Peers**:
   ```bash
   hf peer list
   ```

3. **Start Server**:
   ```bash
   hf daemon start
   ```

### 4. Client Connection

1. **Connect with Server Public Key**:
   ```bash
   hf connect <server-ip> <server-port> <server-public-key>
   ```

## CLI Commands

### Peer Management Commands

#### Add Trusted Peer
```bash
hf peer add <public-key> [name]
```
- Adds a public key to the trusted peer list
- Optional name parameter for identification
- Example: `hf peer add 1A2B3C4D5E6F... "Alice's Laptop"`

#### Remove Trusted Peer
```bash
hf peer remove <public-key>
```
- Removes a public key from the trusted peer list
- Example: `hf peer remove 1A2B3C4D5E6F...`

#### List Trusted Peers
```bash
hf peer list
```
- Shows all trusted peers
- Displays public keys in the trusted list

#### Show Peer Status
```bash
hf peer show <public-key>
```
- Checks if a specific public key is trusted
- Returns whether the peer is in the trusted list

### Connection Command

#### Connect to Server
```bash
hf connect <server-ip> <server-port> <server-public-key>
```
- Connects to a Hafnium server
- Requires the server's public key for verification
- Example: `hf connect 192.168.1.100 37842 1A2B3C4D5E6F...`

## Security Model

### Access Control Flow

1. **Client Initiates Connection**: Client connects to server with server public key
2. **Handshake Process**: HELO/EHLO exchange with cryptographic authentication
3. **Public Key Verification**: Server verifies client's public key against trusted list
4. **Connection Authorization**: Only trusted peers are allowed to proceed
5. **Encrypted Communication**: All subsequent communication is encrypted

### Security Benefits

- **Prevents Unauthorized Access**: Only whitelisted peers can connect
- **Public Key Verification**: Both client and server verify each other's identity
- **No Password-based Authentication**: Uses cryptographic keys instead of passwords
- **Audit Trail**: All trusted peers are explicitly listed and manageable

## Configuration File

The trusted peer list is stored in `hafnium.toml`:

```toml
private_key = "base58-encoded-private-key"
trusted_peers = [
    "base58-encoded-public-key-1",
    "base58-encoded-public-key-2",
    "base58-encoded-public-key-3"
]
```

## Error Handling

### Common Error Scenarios

1. **Untrusted Peer Connection**:
   ```
   ERROR: Peer <public-key> not in trusted list, rejecting connection
   ```

2. **Server Public Key Mismatch**:
   ```
   ERROR: Server public key mismatch! Expected: <expected>, Received: <received>
   ```

3. **Invalid Public Key Format**:
   ```
   ERROR: Invalid public key format
   ```

### Troubleshooting

1. **Client Cannot Connect**:
   - Verify client's public key is in server's trusted list
   - Check server public key is correct
   - Ensure server daemon is running

2. **Server Rejects Connection**:
   - Add client's public key to trusted list
   - Restart server daemon after adding peers
   - Verify public key format (Base58 encoded)

## Example Usage

### Complete Setup Example

```bash
# Server setup
cd server_directory
hf gen-key
SERVER_PUBLIC_KEY=$(hf get-self | cut -d' ' -f3)
echo "Server public key: $SERVER_PUBLIC_KEY"

# Client setup
cd client_directory
hf gen-key
CLIENT_PUBLIC_KEY=$(hf get-self | cut -d' ' -f3)
echo "Client public key: $CLIENT_PUBLIC_KEY"

# Server administrator adds client
cd server_directory
hf peer add "$CLIENT_PUBLIC_KEY"
hf peer list

# Start server
hf daemon start

# Client connects
cd client_directory
hf connect 192.168.1.100 37842 "$SERVER_PUBLIC_KEY"
```

### Managing Multiple Peers

```bash
# Add multiple peers
hf peer add "1A2B3C4D5E6F..." "Alice's Laptop"
hf peer add "2B3C4D5E6F7G..." "Bob's Desktop"
hf peer add "3C4D5E6F7G8H..." "Charlie's Server"

# List all trusted peers
hf peer list
# Output:
# Trusted peers:
#   1. 1A2B3C4D5E6F...
#   2. 2B3C4D5E6F7G...
#   3. 3C4D5E6F7G8H...

# Remove a peer
hf peer remove "2B3C4D5E6F7G..."

# Check if peer is trusted
hf peer show "1A2B3C4D5E6F..."
# Output: Peer 1A2B3C4D5E6F... is trusted
```

## Security Considerations

### Best Practices

1. **Key Management**:
   - Store private keys securely
   - Use strong random number generation
   - Regularly rotate keys if needed

2. **Trusted Peer Management**:
   - Only add peers you trust
   - Remove peers that are no longer needed
   - Regularly audit the trusted peer list

3. **Network Security**:
   - Use the trusted peer system in conjunction with network firewalls
   - Monitor connection logs
   - Implement proper network segmentation

### Limitations

1. **Static Trust Model**: Trusted peers must be manually added/removed
2. **No Certificate Authority**: No centralized certificate management
3. **Public Key Distribution**: Public keys must be shared out-of-band
4. **No Revocation Mechanism**: No automatic key revocation system

## Future Enhancements

- **Certificate-based Authentication**: Integration with X.509 certificates
- **Dynamic Trust Management**: Runtime addition/removal of trusted peers
- **Key Rotation**: Automatic key rotation and renewal
- **Audit Logging**: Detailed logging of peer connections and authentication
- **Role-based Access**: Different permission levels for different peers

This trusted peer system provides a robust foundation for secure peer-to-peer communication in Hafnium while maintaining simplicity and ease of use.
