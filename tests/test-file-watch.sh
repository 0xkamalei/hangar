#!/bin/bash

# Test script for demonstrating the current.yaml file watching feature

echo "🧪 Testing current.yaml file watching feature"
echo ""
echo "This script will:"
echo "1. Start the serve command in the background"
echo "2. Wait for the server to start"
echo "3. Modify current.yaml to trigger a reload"
echo "4. Verify the server reloaded the config"
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Step 1: Starting serve command...${NC}"
# This assumes hangar is built and in the path
# You can adjust this to use cargo run if needed
cargo run --manifest-path src-tauri/Cargo.toml -- serve --port 8081 &
SERVER_PID=$!

echo -e "${GREEN}Server started with PID: $SERVER_PID${NC}"
echo ""

echo -e "${BLUE}Step 2: Waiting for server to initialize...${NC}"
sleep 3
echo ""

echo -e "${BLUE}Step 3: Checking server health...${NC}"
curl -s http://127.0.0.1:8081/health
echo ""
echo ""

echo -e "${YELLOW}Step 4: Watch the server output for reload messages...${NC}"
echo "Now you can manually edit the current.yaml file in another terminal."
echo "The server should detect the change and reload automatically."
echo ""
echo "To test:"
echo "  1. Open ~/.hangar/current.yaml in an editor"
echo "  2. Make a small change (add a comment, modify a value)"
echo "  3. Save the file"
echo "  4. Watch this terminal for the reload message"
echo ""
echo "Press Ctrl+C to stop the server when done testing."
echo ""

# Wait for the server process
wait $SERVER_PID
