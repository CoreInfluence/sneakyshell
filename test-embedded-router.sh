#!/bin/bash
# Test script for embedded I2P router functionality
# This script helps verify the embedded router feature works correctly

set -e  # Exit on error

COLOR_RED='\033[0;31m'
COLOR_GREEN='\033[0;32m'
COLOR_YELLOW='\033[1;33m'
COLOR_BLUE='\033[0;34m'
COLOR_RESET='\033[0m'

echo -e "${COLOR_BLUE}========================================${COLOR_RESET}"
echo -e "${COLOR_BLUE}Reticulum-Shell Embedded Router Test${COLOR_RESET}"
echo -e "${COLOR_BLUE}========================================${COLOR_RESET}"
echo ""

# Check if binaries exist
echo -e "${COLOR_YELLOW}[1/5] Checking binaries...${COLOR_RESET}"
if [ ! -f "target/release/shell-server" ] || [ ! -f "target/release/shell-client" ]; then
    echo -e "${COLOR_RED}Error: Binaries not found!${COLOR_RESET}"
    echo "Please run: cargo build --release --features embedded-router"
    exit 1
fi
echo -e "${COLOR_GREEN}✓ Binaries found${COLOR_RESET}"
echo ""

# Create test directories
echo -e "${COLOR_YELLOW}[2/5] Setting up test environment...${COLOR_RESET}"
TEST_DIR="/tmp/reticulum-test-$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"
echo "Test directory: $TEST_DIR"
echo -e "${COLOR_GREEN}✓ Environment ready${COLOR_RESET}"
echo ""

# Start server
echo -e "${COLOR_YELLOW}[3/5] Starting server with embedded router...${COLOR_RESET}"
echo "This will take 30-60 seconds for initial tunnel establishment."
echo ""

BINARY_PATH="$(cd - > /dev/null && pwd)/target/release"

# Start server in background and capture output
"$BINARY_PATH/shell-server" \
    --enable-i2p \
    --use-embedded-router \
    -v \
    > server.log 2>&1 &

SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait for server to be ready
echo "Waiting for server to initialize..."
MAX_WAIT=120  # 2 minutes
WAITED=0
SERVER_READY=false

while [ $WAITED -lt $MAX_WAIT ]; do
    if grep -q "Listening on Reticulum network" server.log 2>/dev/null; then
        SERVER_READY=true
        break
    fi

    if ! kill -0 $SERVER_PID 2>/dev/null; then
        echo -e "${COLOR_RED}Error: Server process died!${COLOR_RESET}"
        echo "Server log:"
        cat server.log
        exit 1
    fi

    sleep 2
    WAITED=$((WAITED + 2))
    echo -n "."
done
echo ""

if [ "$SERVER_READY" = false ]; then
    echo -e "${COLOR_RED}Error: Server did not start within $MAX_WAIT seconds${COLOR_RESET}"
    echo "Server log:"
    tail -50 server.log
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

echo -e "${COLOR_GREEN}✓ Server started successfully${COLOR_RESET}"
echo ""

# Extract I2P destination
I2P_DEST=$(grep "I2P destination:" server.log | head -1 | sed 's/.*I2P destination: //' | awk '{print $1}')

if [ -z "$I2P_DEST" ]; then
    echo -e "${COLOR_RED}Error: Could not find I2P destination in server log${COLOR_RESET}"
    echo "Server log:"
    cat server.log
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

echo "Server I2P destination: ${I2P_DEST:0:40}..."
echo ""

# Start client
echo -e "${COLOR_YELLOW}[4/5] Starting client with embedded router...${COLOR_RESET}"
echo "This will also take 30-60 seconds for tunnel establishment."
echo ""

# Create a test command file
echo "pwd" > test_command.txt
echo "echo 'Hello from embedded I2P!'" >> test_command.txt
echo "exit" >> test_command.txt

# Start client with test command
timeout 180 "$BINARY_PATH/shell-client" \
    --enable-i2p \
    --use-embedded-router \
    --i2p-destination "$I2P_DEST" \
    --execute "echo 'Test successful'" \
    -v \
    > client.log 2>&1 || true

CLIENT_EXIT=$?

echo ""
echo -e "${COLOR_YELLOW}[5/5] Checking results...${COLOR_RESET}"
echo ""

# Check if client connected successfully
if grep -q "Connected to server" client.log; then
    echo -e "${COLOR_GREEN}✓ Client connected successfully${COLOR_RESET}"

    if grep -q "Test successful" client.log; then
        echo -e "${COLOR_GREEN}✓ Command executed successfully${COLOR_RESET}"
        echo ""
        echo -e "${COLOR_GREEN}========================================${COLOR_RESET}"
        echo -e "${COLOR_GREEN}ALL TESTS PASSED!${COLOR_RESET}"
        echo -e "${COLOR_GREEN}========================================${COLOR_RESET}"
        TEST_RESULT=0
    else
        echo -e "${COLOR_YELLOW}⚠ Client connected but command may have failed${COLOR_RESET}"
        echo "Client output:"
        grep -A 5 "Connected to server" client.log || cat client.log
        TEST_RESULT=1
    fi
else
    echo -e "${COLOR_RED}✗ Client failed to connect${COLOR_RESET}"
    echo ""
    echo "Client log (last 50 lines):"
    tail -50 client.log
    echo ""
    echo "Server log (last 50 lines):"
    tail -50 server.log
    TEST_RESULT=1
fi

echo ""
echo -e "${COLOR_BLUE}Cleaning up...${COLOR_RESET}"

# Cleanup
kill $SERVER_PID 2>/dev/null || true
sleep 2

# Kill any remaining processes
pkill -f "shell-server.*embedded-router" || true
pkill -f "shell-client.*embedded-router" || true

echo ""
echo "Test logs saved in: $TEST_DIR"
echo "  - server.log"
echo "  - client.log"
echo ""

if [ $TEST_RESULT -eq 0 ]; then
    echo -e "${COLOR_GREEN}You can now use the embedded router feature!${COLOR_RESET}"
    echo ""
    echo "Try it manually:"
    echo "  1. Terminal 1: ./target/release/shell-server --enable-i2p --use-embedded-router"
    echo "  2. Copy the I2P destination from server output"
    echo "  3. Terminal 2: ./target/release/shell-client --enable-i2p --use-embedded-router --i2p-destination '<DEST>'"
    echo ""
    echo "See docs/EMBEDDED-ROUTER.md for complete usage guide."
else
    echo -e "${COLOR_YELLOW}Some tests failed. See troubleshooting section in docs/EMBEDDED-ROUTER.md${COLOR_RESET}"
fi

echo ""
echo "Cleanup: rm -rf $TEST_DIR"
echo ""

exit $TEST_RESULT
