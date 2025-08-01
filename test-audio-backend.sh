#!/bin/bash
# Test if backend audio capture is working

echo "Testing backend audio capture..."

# First, let's test if we can capture audio with ffmpeg directly
echo "1. Testing direct ffmpeg capture..."
timeout 3 ffmpeg -f pulse -i default -t 2 -acodec libopus -b:a 128k -ar 48000 -ac 2 -f webm test-capture.webm -y 2>&1 | grep -E "size=|Stream|Output"

echo -e "\n2. File size of captured audio:"
ls -lh test-capture.webm 2>/dev/null || echo "No file created"

echo -e "\n3. Testing if we can play the captured file:"
ffplay -nodisp -autoexit test-capture.webm 2>&1 | head -5 || echo "ffplay not available"

echo -e "\n4. Checking default PulseAudio sink:"
pactl get-default-sink

echo -e "\n5. Testing monitor capture:"
SINK=$(pactl get-default-sink)
echo "Using monitor: ${SINK}.monitor"
timeout 3 ffmpeg -f pulse -i "${SINK}.monitor" -t 2 -acodec libopus -b:a 128k -ar 48000 -ac 2 -f webm test-monitor.webm -y 2>&1 | grep -E "size=|Stream|Output"

echo -e "\n6. Monitor file size:"
ls -lh test-monitor.webm 2>/dev/null || echo "No file created"

echo -e "\nDone!"