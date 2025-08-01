#!/bin/bash
# Test audio streaming end-to-end

echo "Testing audio streaming..."

# 1. Test WebSocket connection
echo -e "\n1. Testing WebSocket connection to backend..."
curl -s -N \
  --include \
  --no-buffer \
  --header "Connection: Upgrade" \
  --header "Upgrade: websocket" \
  --header "Sec-WebSocket-Version: 13" \
  --header "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
  http://localhost:4000/ws | head -20

# 2. Test if ffmpeg can capture audio
echo -e "\n2. Testing ffmpeg audio capture..."
timeout 2 ffmpeg -f pulse -i default -t 1 -acodec libopus -b:a 128k -ar 48000 -ac 2 -f webm test-stream.webm -y 2>&1 | grep -E "Stream|size="

# 3. Check file size
echo -e "\n3. Check captured file:"
ls -la test-stream.webm

# 4. Send audio control message via WebSocket
echo -e "\n4. Testing audio control via WebSocket..."
cat > test-ws.js << 'EOF'
const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:4000/ws');

ws.on('open', () => {
  console.log('Connected to WebSocket');
  
  // Send audio start command
  ws.send(JSON.stringify({
    type: 'audio-control',
    action: 'start'
  }));
  
  console.log('Sent audio-control start message');
});

ws.on('message', (data) => {
  try {
    const msg = JSON.parse(data);
    console.log('Received:', msg.type);
    if (msg.type === 'audio-stream') {
      console.log('Got audio chunk, length:', msg.data.length);
    }
  } catch (e) {
    console.log('Non-JSON message:', data.toString().substring(0, 50));
  }
});

ws.on('error', (err) => {
  console.error('WebSocket error:', err);
});

// Keep running for 5 seconds
setTimeout(() => {
  ws.send(JSON.stringify({
    type: 'audio-control', 
    action: 'stop'
  }));
  ws.close();
  process.exit(0);
}, 5000);
EOF

# Run the test if ws module is available
if command -v node >/dev/null 2>&1; then
  echo "Running WebSocket test..."
  node test-ws.js || echo "Note: Install 'ws' module with: npm install ws"
else
  echo "Node.js not found, skipping WebSocket test"
fi

echo -e "\nDone!"