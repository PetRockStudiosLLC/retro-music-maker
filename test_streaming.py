import asyncio
import websockets
import json
import numpy as np

async def test():
    uri = 'ws://localhost:8000/ws'
    async with websockets.connect(uri) as ws:
        init = json.loads(await ws.recv())
        print('Init:', init)
        
        import requests
        resp = requests.post('http://localhost:8000/api/graph/test-simple')
        print('Test graph:', resp.json())
        
        await ws.send(json.dumps({'type': 'start_streaming', 'sample_rate': 44100, 'block_size': 512}))
        print('Sent start_streaming')
        
        for i in range(5):
            try:
                msg = await asyncio.wait_for(ws.recv(), timeout=3)
                if isinstance(msg, bytes):
                    audio = np.frombuffer(msg, dtype=np.float32)
                    print(f'Chunk {i}: {len(audio)} samples, max={np.max(np.abs(audio)):.4f}')
                else:
                    print(f'Msg {i}:', json.loads(msg))
            except asyncio.TimeoutError:
                print(f'Timeout waiting for msg {i}')
                break

if __name__ == '__main__':
    asyncio.run(test())
