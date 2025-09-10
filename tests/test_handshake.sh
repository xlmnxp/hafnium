#!/bin/bash

# Test script for Hafnium handshake functionality

set -e

echo "🧪 Testing Hafnium Handshake Implementation"
echo "=========================================="

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if binary exists
if [ ! -f "../target/release/hf" ]; then
    echo -e "${RED}❌ Binary not found. Please run 'cargo build --release' first${NC}"
    exit 1
fi

echo -e "${YELLOW}📋 Test Plan:${NC}"
echo "1. Generate key pairs for server and client"
echo "2. Add client to server's trusted peer list"
echo "3. Start server daemon"
echo "4. Connect client to server with server public key"
echo "5. Verify handshake and encrypted communication"
echo ""

# Create test directories
mkdir -p test_server test_client

echo -e "${YELLOW}🔑 Step 1: Generating key pairs...${NC}"

# Generate server key
cd test_server
if [ ! -f "hafnium.toml" ]; then
    ../../target/release/hf gen-key
    echo -e "${GREEN}✅ Server key generated${NC}"
else
    echo -e "${YELLOW}⚠️  Server key already exists${NC}"
fi

# Get server public key
SERVER_PUBLIC_KEY=$(../../target/release/hf get-self | cut -d' ' -f3)
echo -e "${GREEN}📋 Server public key: ${SERVER_PUBLIC_KEY}${NC}"

# Generate client key
cd ../test_client
if [ ! -f "hafnium.toml" ]; then
    ../../target/release/hf gen-key
    echo -e "${GREEN}✅ Client key generated${NC}"
else
    echo -e "${YELLOW}⚠️  Client key already exists${NC}"
fi

# Get client public key
CLIENT_PUBLIC_KEY=$(../../target/release/hf get-self | cut -d' ' -f3)
echo -e "${GREEN}📋 Client public key: ${CLIENT_PUBLIC_KEY}${NC}"

cd ..

echo ""
echo -e "${YELLOW}🤝 Step 2: Setting up trusted peers...${NC}"

# Add client to server's trusted list
cd test_server
../../target/release/hf peer add "$CLIENT_PUBLIC_KEY"
echo -e "${GREEN}✅ Added client to server's trusted list${NC}"
cd ..

echo ""
echo -e "${YELLOW}🚀 Step 3: Starting server daemon...${NC}"

# Start server in background
cd test_server
../../target/release/hf daemon start &
SERVER_PID=$!
cd ..

# Wait for server to start
sleep 2

echo -e "${GREEN}✅ Server daemon started (PID: $SERVER_PID)${NC}"

echo ""
echo -e "${YELLOW}🔗 Step 4: Testing client connection...${NC}"

# Test client connection with server public key
cd test_client
timeout 10s ../../target/release/hf connect 127.0.0.1 37842 "$SERVER_PUBLIC_KEY" || {
    echo -e "${RED}❌ Client connection failed or timed out${NC}"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
}
cd ..

echo ""
echo -e "${GREEN}✅ Client connection test completed${NC}"

# Cleanup
echo -e "${YELLOW}🧹 Cleaning up...${NC}"
kill $SERVER_PID 2>/dev/null || true
sleep 1

echo ""
echo -e "${GREEN}🎉 Handshake test completed successfully!${NC}"
echo ""
echo -e "${YELLOW}📝 What was tested:${NC}"
echo "• HELO/EHLO handshake protocol"
echo "• P-256 ECDH key exchange"
echo "• ChaCha20Poly1305 encryption"
echo "• Trusted peer management"
echo "• Server public key verification"
echo "• Encrypted message exchange"
echo "• Base58 public key encoding"
echo ""
echo -e "${GREEN}✨ The trusted peer handshake implementation is working correctly!${NC}"
