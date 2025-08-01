#!/usr/bin/env python3
import asyncio
import websockets
import json
import time

async def test_audio():
    uri = "ws://localhost:4000/ws"
    
    try:
        async with websockets.connect(uri) as websocket:
            print("Connected to WebSocket")
            
            # Send audio start command
            start_msg = json.dumps({
                "type": "audio-control",
                "action": "start"
            })
            await websocket.send(start_msg)
            print("Sent audio-control start message")
            
            # Listen for messages for 5 seconds
            start_time = time.time()
            audio_chunks = 0
            
            while time.time() - start_time < 5:
                try:
                    message = await asyncio.wait_for(websocket.recv(), timeout=0.1)
                    data = json.loads(message)
                    
                    if data["type"] == "audio-stream":
                        audio_chunks += 1
                        print(f"Received audio chunk #{audio_chunks}, size: {len(data.get('data', ''))}")
                    else:
                        print(f"Received: {data['type']}")
                        
                except asyncio.TimeoutError:
                    continue
                except Exception as e:
                    print(f"Error: {e}")
            
            # Send stop command
            stop_msg = json.dumps({
                "type": "audio-control",
                "action": "stop"
            })
            await websocket.send(stop_msg)
            print(f"\nTotal audio chunks received: {audio_chunks}")
            
    except Exception as e:
        print(f"Connection error: {e}")

if __name__ == "__main__":
    asyncio.run(test_audio())