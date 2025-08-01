#!/bin/bash
# Test audio streaming by generating a beep sound

echo "Testing audio streaming..."
echo "This will play a 1 second beep in 3 seconds..."
sleep 3

# Generate a 440Hz beep for 1 second
speaker-test -t sine -f 440 -l 1 2>/dev/null || \
paplay /usr/share/sounds/freedesktop/stereo/complete.oga 2>/dev/null || \
echo -e "\a"

echo "Audio test complete!"