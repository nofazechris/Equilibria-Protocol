"""
Equilibria — WebSocket bridge
Broadcasts keeper logs to the frontend in real time
Run alongside keeper.py
"""
import asyncio
import json
import websockets
from datetime import datetime

clients = set()

async def handler(websocket):
    clients.add(websocket)
    try:
        await websocket.wait_closed()
    finally:
        clients.discard(websocket)

async def broadcast(message: dict):
    if clients:
        data = json.dumps(message)
        await asyncio.gather(*[c.send(data) for c in clients], return_exceptions=True)

async def main():
    print(f"[{datetime.now().strftime('%H:%M:%S')}] WebSocket server started on ws://localhost:8765")
    async with websockets.serve(handler, "localhost", 8765):
        await asyncio.Future()

if __name__ == "__main__":
    asyncio.run(main())