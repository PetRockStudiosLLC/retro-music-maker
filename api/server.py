"""FastAPI server for Retro Music Maker.

Exposes the Python audio engine via REST + WebSocket endpoints
so the React frontend can manage nodes, connections, and audio playback.
"""
import asyncio
import json
import threading
from contextlib import asynccontextmanager
from typing import List, Dict, Any, Optional

from fastapi import FastAPI, WebSocket, WebSocketDisconnect, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse
import numpy as np

import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Load nodes immediately on import
from core import NodeRegistry

_nodes_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "nodes")
if os.path.isdir(_nodes_dir):
    import importlib.util
    for root, dirs, files in os.walk(_nodes_dir):
        dirs[:] = [d for d in dirs if d != "__pycache__"]
        for f in files:
            if f.endswith(".py") and not f.startswith("__"):
                filepath = os.path.join(root, f)
                try:
                    spec = importlib.util.spec_from_file_location(f"nodes.{f[:-3]}", filepath)
                    if spec and spec.loader:
                        module = importlib.util.module_from_spec(spec)
                        spec.loader.exec_module(module)
                except Exception:
                    pass

from core.engine import Engine


# ─── Engine Proxy ───────────────────────────────────────────────────────────

class EngineProxy:
    """Wraps the Python Engine for API access."""

    def __init__(self):
        self.engine = Engine()
        self.lock = threading.Lock()

    @property
    def node_types(self) -> dict:
        """Return all registered node types with their metadata."""
        try:
            nodes = NodeRegistry.get_all_nodes()
        except Exception:
            nodes = {}
        result = {}
        for name, cls in nodes.items():
            category = getattr(cls, 'category', 'General')
            result[name] = {
                "name": name,
                "category": category,
                "inputs": [],
                "outputs": [],
                "params": {},
            }
            try:
                try:
                    temp_node = cls(node_id="temp")
                except TypeError:
                    temp_node = cls()
                inputs = [{"name": n, "type": type(p).__name__} for n, p in temp_node._inputs.items()]
                outputs = [{"name": n, "type": type(p).__name__} for n, p in temp_node._outputs.items()]
                node_params = dict(temp_node._params) if hasattr(temp_node, '_params') else {}
                # Convert non-JSON types
                clean_params = {}
                for k, v in node_params.items():
                    if isinstance(v, np.ndarray):
                        clean_params[k] = v.tolist() if v.ndim > 0 else float(v)
                    else:
                        clean_params[k] = v
                result[name]["inputs"] = inputs
                result[name]["outputs"] = outputs
                result[name]["params"] = clean_params
            except Exception:
                pass
        return result

    @property
    def node_count(self) -> int:
        with self.lock:
            return len(self.engine.list_nodes())

    def get_graph_state(self) -> dict:
        """Return the current engine state as a dict."""
        with self.lock:
            return self.engine.to_dict()

    def add_node(self, node_type: str, node_id: Optional[str] = None) -> str:
        """Add a node of the given type. Returns node_id."""
        cls = NodeRegistry.get_node_class(node_type)
        with self.lock:
            node = cls(node_id=node_id)
            nid = self.engine.add_node(node)
            return nid

    def remove_node(self, node_id: str) -> None:
        with self.lock:
            self.engine.remove_node(node_id)

    def connect_nodes(
        self,
        src_id: str,
        src_port: str,
        dst_id: str,
        dst_port: str,
    ) -> None:
        with self.lock:
            self.engine.connect(src_id, src_port, dst_id, dst_port)

    def disconnect_nodes(self, src_id: str, dst_id: str) -> None:
        with self.lock:
            # Find the edge and remove it
            edges = list(self.engine.graph._edges)
            for s, d in edges:
                if s == src_id and d == dst_id:
                    self.engine.graph._edges.remove((s, d))
                    break

    def update_param(self, node_id: str, param_name: str, value: Any) -> None:
        with self.lock:
            node = self.engine.graph.nodes.get(node_id)
            if node is None:
                raise KeyError(f"Node {node_id} not found")
            node.params[param_name] = value

    def start_audio(self) -> bool:
        try:
            with self.lock:
                self.engine.start()
            return True
        except Exception as e:
            raise HTTPException(status_code=500, detail=str(e))

    def stop_audio(self) -> bool:
        with self.lock:
            self.engine.stop()
        return True

    @property
    def is_running(self) -> bool:
        return self.engine.is_running

    def process_block(self, block_size: int = 64) -> dict:
        """Process a single audio block."""
        with self.lock:
            return self.engine.graph.process_block(block_size)

    def export_wav(self, duration: float = 2.0, sample_rate: int = 44100) -> bytes:
        """Render audio to WAV bytes."""
        n = int(sample_rate * duration)
        samples = np.zeros(n, dtype=np.float32)
        bs = 64
        recorded = 0
        with self.lock:
            self.engine.start()
        try:
            while recorded < n and self.engine.is_running:
                try:
                    results = self.process_block(bs)
                    for nid, nout in results.items():
                        node = self.engine.graph.nodes.get(nid)
                        if node:
                            for pn, pt in node.outputs.items():
                                sig = pt.value
                                if hasattr(sig, '__len__') and len(sig) > 0:
                                    s = recorded
                                    e = min(s + len(sig), n)
                                    if e > s:
                                        samples[s:e] += sig[: e - s]
                    recorded = min(recorded + bs, n)
                except Exception:
                    pass
                import time

                time.sleep(0.001)
        finally:
            with self.lock:
                self.engine.stop()

        import wave
        import io

        buf = io.BytesIO()
        with wave.open(buf, "wb") as wf:
            wf.setnchannels(1)
            wf.setsampwidth(2)
            wf.setframerate(sample_rate)
            samples = np.clip(samples, -1.0, 1.0)
            wf.writeframes(np.int16(samples * 32767).tobytes())
        buf.seek(0)
        return buf.read()

    def load_from_dict(self, data: dict) -> None:
        """Load engine state from a dict."""
        with self.lock:
            self.engine = Engine.from_dict(data)


# ─── App Lifecycle ──────────────────────────────────────────────────────────

engine_proxy = EngineProxy()
connected_clients: set = set()


@asynccontextmanager
async def lifespan(app: FastAPI):
    yield


# ─── FastAPI App ────────────────────────────────────────────────────────────

app = FastAPI(title="Retro Music Maker", version="2.0", lifespan=lifespan)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# ─── REST Endpoints ─────────────────────────────────────────────────────────


@app.get("/api/node-types")
def get_node_types():
    """Get all available node types with metadata."""
    return engine_proxy.node_types


@app.get("/api/node-count")
def get_node_count():
    """Get number of nodes in the graph."""
    return {"count": engine_proxy.node_count}


@app.get("/api/graph")
def get_graph():
    """Get full graph state."""
    try:
        return engine_proxy.get_graph_state()
    except Exception as e:
        import traceback
        traceback.print_exc()
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/graph/new")
def new_graph():
    """Create a new empty graph."""
    engine_proxy.stop_audio()
    engine_proxy.engine = Engine()
    return {"status": "ok", "nodes": 0}


@app.post("/api/graph/test-simple")
def create_test_graph():
    """Create a simple test graph: SineOscillator -> LowpassFilter -> AudioOutput."""
    import threading
    with engine_proxy.lock:
        engine_proxy.engine.stop()
        engine_proxy.engine.graph.clear()

    sine = NodeRegistry.get_node_class("SineOscillator")(node_id="sineosc_000")
    sine._params["freq"] = 440.0
    sine._params["amplitude"] = 0.3

    lpf = NodeRegistry.get_node_class("LowpassFilter")(node_id="lowpass_000")
    lpf._params["cutoff"] = 2000.0
    lpf._params["resonance"] = 0.0

    output = NodeRegistry.get_node_class("AudioOutput")(node_id="audioout_000")

    with engine_proxy.lock:
        engine_proxy.engine.add_node(sine)
        engine_proxy.engine.add_node(lpf)
        engine_proxy.engine.add_node(output)
        engine_proxy.engine.connect("sineosc_000", "audio", "lowpass_000", "audio")
        engine_proxy.engine.connect("lowpass_000", "audio", "audioout_000", "audio")

    return {"status": "ok", "nodes": 3, "message": "Test graph created: SineOscillator -> LowpassFilter -> AudioOutput"}


@app.post("/api/graph/load")
def load_graph(data: dict):
    """Load a graph from JSON data."""
    engine_proxy.stop_audio()
    engine_proxy.load_from_dict(data)
    return {"status": "ok", "nodes": engine_proxy.node_count}


@app.post("/api/graph/save")
def save_graph(data: dict):
    """Save a graph (replaces current)."""
    engine_proxy.stop_audio()
    engine_proxy.load_from_dict(data)
    return {"status": "ok", "nodes": engine_proxy.node_count}


@app.post("/api/node/{node_type}/add")
def add_node(node_type: str, body: dict = {}):
    """Add a new node of the given type."""
    nid = engine_proxy.add_node(node_type, body.get("node_id"))
    node_types = engine_proxy.node_types
    node_info = node_types.get(node_type, {})
    return {
        "node_id": nid,
        "node_type": node_type,
        "category": node_info.get("category", "General"),
    }


@app.delete("/api/node/{node_id}")
def delete_node(node_id: str):
    """Remove a node."""
    engine_proxy.remove_node(node_id)
    return {"status": "ok"}


@app.post("/api/node/{node_id}/param")
def set_param(node_id: str, body: dict):
    """Set a parameter on a node."""
    param_name = body.get("name")
    value = body.get("value")
    if not param_name or value is None:
        raise HTTPException(status_code=400, detail="name and value required")
    engine_proxy.update_param(node_id, param_name, value)
    return {"status": "ok"}


@app.post("/api/connect")
def connect_nodes(
    src_id: str, src_port: str, dst_id: str, dst_port: str
) -> dict:
    """Connect two nodes."""
    try:
        engine_proxy.connect_nodes(src_id, src_port, dst_id, dst_port)
        return {"status": "ok"}
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Connect failed: {str(e)}")


@app.delete("/api/connect")
def disconnect_nodes(src_id: str, dst_id: str) -> dict:
    """Disconnect two nodes."""
    try:
        engine_proxy.disconnect_nodes(src_id, dst_id)
        return {"status": "ok"}
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Disconnect failed: {str(e)}")


@app.post("/audio/start")
def audio_start() -> dict:
    """Start audio playback."""
    success = engine_proxy.start_audio()
    return {"status": "ok", "running": True}


@app.post("/audio/stop")
def audio_stop() -> dict:
    """Stop audio playback."""
    engine_proxy.stop_audio()
    return {"status": "ok", "running": False}


@app.get("/audio/status")
def audio_status() -> dict:
    """Get audio status."""
    return {"running": engine_proxy.is_running}


@app.post("/audio/export")
def audio_export(body: dict = {}):
    """Export audio to WAV. Returns base64-encoded WAV."""
    import base64

    duration = body.get("duration", 2.0)
    try:
        wav_data = engine_proxy.export_wav(duration=duration)
        return {"data": base64.b64encode(wav_data).decode("ascii")}
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Export failed: {str(e)}")


# ─── WebSocket Endpoint ────────────────────────────────────────────────────


async def audio_stream_task(websocket: WebSocket, block_size: int):
    """Background task that streams audio blocks to a WebSocket client."""
    print(f"audio_stream_task started, block_size={block_size}")
    try:
        while True:
            try:
                with engine_proxy.lock:
                    results = engine_proxy.engine.graph.process_block(block_size)
                    # Find AudioOutput node(s) and use their processed output
                    output = np.zeros(block_size, dtype=np.float32)
                    for node_id, node_signal in results.items():
                        node = engine_proxy.engine.graph.nodes.get(node_id)
                        if not node:
                            continue
                        # Sum AudioOutput node signals as the final mix
                        if node.__class__.__name__ == 'AudioOutput':
                            if hasattr(node_signal, '__len__') and len(node_signal) > 0:
                                sig = node_signal[:block_size] if len(node_signal) >= block_size else node_signal
                                output[:len(sig)] += sig
                    # Fallback: if no AudioOutput, sum all node outputs
                    if np.max(np.abs(output)) == 0:
                        for node_id, node_signal in results.items():
                            if hasattr(node_signal, '__len__') and len(node_signal) > 0:
                                sig = node_signal[:block_size] if len(node_signal) >= block_size else node_signal
                                output[:len(sig)] += sig
                        output = np.clip(output * 0.3, -1.0, 1.0)
                    else:
                        output = np.clip(output * 0.5, -1.0, 1.0)

                await websocket.send_bytes(output.tobytes())
                print(f"Sent audio chunk: {block_size} samples")
            except asyncio.CancelledError:
                print("audio_stream_task cancelled")
                raise
            except Exception as e:
                print(f"Audio stream error: {e}")
                break
            await asyncio.sleep(0.001)
    except asyncio.CancelledError:
        raise
    except WebSocketDisconnect:
        print("audio_stream_task: client disconnected")
    except Exception as e:
        print(f"Audio stream task error: {e}")


@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    """WebSocket connection for real-time engine state updates and audio streaming."""
    await websocket.accept()

    # Track connected clients for audio streaming
    connected_clients.add(websocket)
    streaming_task: Optional[asyncio.Task] = None

    try:
        # Send initial state
        state = engine_proxy.get_graph_state()
        await websocket.send_json({"type": "init", "data": state})

        while True:
            try:
                text_data = await websocket.receive_text()
                data = json.loads(text_data)

                if data.get("type") == "start_audio":
                    try:
                        with engine_proxy.lock:
                            engine_proxy.engine._running = True
                            engine_proxy.engine.graph.start()
                        await websocket.send_json({"type": "audio_started"})
                    except Exception as e:
                        print(f"Start audio error: {e}")
                        await websocket.send_json({"type": "error", "detail": str(e)})
                elif data.get("type") == "stop_audio":
                    with engine_proxy.lock:
                        engine_proxy.engine._running = False
                        engine_proxy.engine.graph.stop()
                    if streaming_task:
                        streaming_task.cancel()
                        streaming_task = None
                    await websocket.send_json({"type": "audio_stopped"})
                elif data.get("type") == "start_streaming":
                    sample_rate = data.get("sample_rate", 44100)
                    block_size = data.get("block_size", 512)
                    print(f"Starting streaming: sample_rate={sample_rate}, block_size={block_size}")
                    await websocket.send_json({
                        "type": "streaming_started",
                        "sample_rate": sample_rate,
                        "block_size": block_size,
                    })
                    if streaming_task:
                        streaming_task.cancel()
                    streaming_task = asyncio.create_task(
                        audio_stream_task(websocket, block_size)
                    )
                    print(f"Streaming task created: {streaming_task}")
                elif data.get("type") == "add_node":
                    nid = engine_proxy.add_node(data["node_type"])
                    node_types = engine_proxy.node_types
                    node_info = node_types.get(data["node_type"], {})
                    await websocket.send_json({
                        "type": "node_added",
                        "node_id": nid,
                        "node_type": data["node_type"],
                        "category": node_info.get("category", "General"),
                    })
                elif data.get("type") == "remove_node":
                    engine_proxy.remove_node(data["node_id"])
                elif data.get("type") == "connect":
                    engine_proxy.connect_nodes(
                        data["src_id"], data["src_port"], data["dst_id"], data["dst_port"]
                    )
                    await websocket.send_json({"type": "connected"})
                elif data.get("type") == "disconnect":
                    engine_proxy.disconnect_nodes(data["src_id"], data["dst_id"])
                    await websocket.send_json({"type": "disconnected"})
                elif data.get("type") == "set_param":
                    engine_proxy.update_param(
                        data["node_id"], data["param_name"], data["value"]
                    )
                elif data.get("type") == "export":
                    duration = data.get("duration", 2.0)
                    import base64
                    wav_data = engine_proxy.export_wav(duration=duration)
                    await websocket.send_json({
                        "type": "exported",
                        "data": base64.b64encode(wav_data).decode("ascii"),
                    })
            except Exception as e:
                print(f"WebSocket message error: {e}")
                await websocket.send_json({"type": "error", "detail": str(e)})

    except WebSocketDisconnect:
        pass
    except Exception as e:
        print(f"WebSocket error: {e}")
    finally:
        if streaming_task:
            streaming_task.cancel()
        connected_clients.discard(websocket)
