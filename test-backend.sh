#!/bin/bash

echo "Testing WebMux Backend API..."

# Check if backend is running
echo -e "\n1. Testing API connectivity..."
curl -s http://localhost:4000/api/stats | jq . || echo "Failed to connect to backend on port 4000"

# List sessions
echo -e "\n2. Listing TMUX sessions..."
curl -s http://localhost:4000/api/sessions | jq .

# Create a test session
echo -e "\n3. Creating test session..."
curl -s -X POST http://localhost:4000/api/sessions \
  -H "Content-Type: application/json" \
  -d '{"name": "test-session"}' | jq .

# List sessions again
echo -e "\n4. Listing sessions after creation..."
curl -s http://localhost:4000/api/sessions | jq .

# Test WebSocket connection
echo -e "\n5. Testing WebSocket connection..."
echo "Note: For full WebSocket testing, use the web interface or test-ws-audio.py"

# List windows in test session
echo -e "\n6. Listing windows in test session..."
curl -s http://localhost:4000/api/sessions/test-session/windows | jq .

# Clean up - kill test session
echo -e "\n7. Cleaning up test session..."
curl -s -X POST http://localhost:4000/api/sessions/test-session/kill | jq .

echo -e "\nBackend test complete!"