# Retro Music Maker — Current Architecture Documentation

## Overview
Node-based retro music creation app. Python audio engine + React web frontend communicating via FastAPI REST + WebSocket.

---

## Current Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Browser (localhost:5173)                                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ React + React Flow (Node Graph Editor)                 │  │
│  │  ┌──────────┐  ┌────────────┐  ┌──────────────────┐  │  │
│  │  │ Palette  │  │  Canvas    │  │ Properties Panel │  │  │
│  │  │ (drag)   │  │ (ReactFlow)│  │ (param editor)   │  │  │
│  │  └──────────┘  └────────────┘  └──────────────────┘  │  │
│  │  ┌──────────────────────────────────────────────────┐  │  │
│  │  │ Console (log output)                             │  │  │
│  │  └──────────────────────────────────────────────────┘  │  │
│  │  ┌──────────────────────────────────────────────────┐  │  │
│  │  │ AudioPlayer (AudioWorkletNode)                    │  │  │
│  │  └──────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
│         │ REST /api/*              │ WebSocket /ws          │
└─────────┼──────────────────────────┼────────────────────────┘
          │                          │
          ▼                          ▼
┌─────────────────────────────────────────────────────────────┐
│  FastAPI Server (localhost:8000)                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ EngineProxy (thread-safe wrapper)                      │   │
│  │  ┌────────────────────────────────────────────────┐  │   │
│  │  │ Python Audio Engine (core/)                      │  │   │
│  │  │  Graph (DAG, topological sort)                   │  │   │
│  │  │  Node (base class, ports, params)                │  │   │
│  │  │  Port (Audio, Control, Trigger)                  │  │   │
│  │  │  AudioBackend (sounddevice)                      │  │   │
│  │  │  Scheduler (sample-accurate timing)              │  │   │
│  │  └────────────────────────────────────────────────┘  │   │
│  │  ┌────────────────────────────────────────────────┐  │   │
│  │  │ 22 Node Types (nodes/)                          │  │   │
│  │  │  Oscillators: Sine, Square, Saw, Triangle,      │  │   │
│  │  │    Noise, ChipSound                             │  │   │
│  │  │  Filters: Lowpass, Highpass, Bandpass, Bitcrush │  │   │
│  │  │  Envelopes: ADSR, EnvelopeSequencer             │  │   │
│  │  │  Effects: Delay, Distortion, Chorus, Flanger    │  │   │
│  │  │  Mixers: Mixer, Splitter, Crossfader            │  │   │
│  │  │  Input: LFO                                     │  │   │
│  │  │  Output: AudioOutput                            │  │   │
│  │  │  Sequencer: StepSequencer                       │  │   │
│  │  └────────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Engine (Python)

### File Structure
```
core/
  __init__.py       — NodeRegistry (register/unregister/get/load_plugins)
  port.py           — Port, AudioPort, ControlPort, TriggerPort
  node.py           — Node base class (ports, params, process, to_dict/from_dict)
  graph.py          — Graph (DAG, topological sort, process_block, serialization)
  engine.py         — Engine (graph + backend + scheduler, audio callback)
  audio_backend.py  — AudioBackend (sounddevice wrapper, callback loop)
  scheduler.py      — Scheduler (sample-accurate timing events)
```

### Key Concepts

**Port System** (`core/port.py`):
- `AudioPort` — numpy float32 arrays, block processing
- `ControlPort` — single float values
- `TriggerPort` — boolean events
- Connection tracking via `_connected` list of `(port, signal)` tuples
- Value resolution: sums connected signals, falls back to default

**Node System** (`core/node.py`):
- Each node has unique string ID, typed input/output ports, params dict
- `_setup_ports()` and `_setup_params()` overridden by subclasses
- `process(block_size)` returns numpy array
- Serialization via `to_dict()` / `from_dict()`

**Graph** (`core/graph.py`):
- DAG with topological sort for processing order
- `process_block(block_size)` processes all nodes in order
- Edge tracking: `List[Tuple[str, str]]` — (source_id, target_id)
- Cycle detection in topological sort
- Full serialization/deserialization

**Engine** (`core/engine.py`):
- Wraps Graph + AudioBackend + Scheduler
- `_audio_callback(frames)` called by backend for each block
- Collects output from all nodes, sums audio ports, applies gain
- Thread-safe with `_lock`

### Node Registry (`core/__init__.py`)
- Decorator-based registration: `@NodeRegistry.register`
- Auto-discovery from `plugins/` directory
- 22 registered node types across 8 categories

---

## Backend API (FastAPI)

### File Structure
```
api/
  server.py  — FastAPI app, EngineProxy, REST endpoints, WebSocket handler
```

### REST Endpoints
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/node-types` | All registered node types + metadata |
| GET | `/api/graph` | Current graph state (nodes + edges) |
| GET | `/api/node-count` | Number of nodes |
| POST | `/api/node/{type}/add` | Add node of given type |
| DELETE | `/api/node/{id}` | Remove node |
| POST | `/api/node/{id}/param` | Set node parameter |
| POST | `/api/connect` | Connect two nodes |
| DELETE | `/api/connect` | Disconnect nodes |
| POST | `/api/graph/new` | Clear graph |
| POST | `/api/graph/test-simple` | Create test graph |
| POST | `/api/graph/save` | Save graph |
| POST | `/api/graph/load` | Load graph from JSON |
| POST | `/audio/start` | Start audio engine |
| POST | `/audio/stop` | Stop audio engine |
| GET | `/audio/export` | Export WAV |

### WebSocket (`/ws`)
- **Init**: Sends `{"type": "init", "data": {...}}` on connect
- **Messages** (client → server):
  - `start_audio`, `stop_audio`
  - `start_streaming` — `{sample_rate, block_size}`
  - `add_node`, `remove_node`
  - `connect`, `disconnect`
  - `set_param`, `export`
- **Audio streaming**: Binary chunks (float32 ArrayBuffer) sent to client
- **Issue**: `receive_text()` required (not raw `receive()`)

### EngineProxy
- Thread-safe wrapper around Engine
- `threading.Lock` protects all engine operations
- `node_types` property creates temp nodes to introspect ports/params

---

## Frontend (React + TypeScript)

### File Structure
```
frontend/
  package.json          — React 18, @xyflow/react 12, Vite 6, TypeScript 5
  vite.config.ts        — Dev server + proxy to localhost:8000
  tsconfig.json         — Strict TS config
  src/
    App.tsx             — Main component (toolbar, canvas, panels)
    main.tsx            — Entry point
    index.css           — Global styles
    audio-stream-processor.js  — AudioWorklet processor
    types/
      nodes.ts          — TypeScript interfaces
    hooks/
      useWebSocket.ts   — WebSocket + REST sync hook
      useAudioPlayer.ts — Web Audio API playback hook
    components/
      CustomNode.tsx    — React Flow custom node renderer
      Palette.tsx       — Draggable node type sidebar
      PropertiesPanel.tsx — Parameter editor
      Console.tsx       — Log output panel
```

### Key Components

**App.tsx**:
- React Flow canvas with custom nodes
- Drag from palette → drop on canvas → REST call to add node
- Toolbar: Import/Export/New/Save, Start/Stop, Export WAV
- Keyboard shortcuts: Delete (remove node), Space (play/pause)
- Coordinate conversion: screen → ReactFlow position

**useWebSocket.ts**:
- WebSocket connection to `ws://localhost:8000/ws`
- REST sync on mount: fetches `/api/node-types` + `/api/graph`
- Reconnection: 3s timeout on disconnect
- Methods: `addNode`, `removeNode`, `connectPorts`, `setParam`, etc.
- Audio chunk callback via `setOnAudioChunk`

**useAudioPlayer.ts**:
- `AudioWorkletNode` for streaming playback
- Receives float32 chunks via `postMessage` to worklet
- Start/stop streaming controls

**CustomNode.tsx**:
- React Flow custom node component
- Renders port handles (input/output)
- Category-colored header
- Compact node body with name + type

### Vite Proxy
```typescript
proxy: {
  "/api": { target: "http://localhost:8000", changeOrigin: true },
  "/audio": { target: "http://localhost:8000", changeOrigin: true },
  "/ws": { target: "http://localhost:8000", changeOrigin: true, ws: true },
}
```

---

## Known Issues & Fixes

| Issue | Root Cause | Fix |
|-------|-----------|-----|
| WebSocket 500 on `/api/graph` | Backend process died (shell timeout) | Start as background process |
| `ConnectionClosedError` | `receive()` vs `receive_text()` | Use `receive_text()` in WebSocket handler |
| `zustand provider` error | `useReactFlow()` hook misuse | Use ref-based coordinate conversion |
| Node types empty on import | Lazy loading issue | Load nodes at module level |
| `TypeError` in node_types | numpy arrays not JSON-serializable | Convert to lists in `to_dict()` |
| Infinite recursion in Port.value | Circular port references | Check `if connected_port is self` |
| WebSocket cleanup race | Closing CONNECTING socket | Only close OPEN sockets |
| Vite WS proxy flaky | Binary frame handling | Direct `ws://localhost:8000/ws` connection |

---

## Test Suite
- `tests/test_engine.py` — Core engine tests
- `tests/test_graph.py` — Graph management tests
- `tests/test_nodes.py` — Node type tests
- `tests/test_midi.py` — MIDI tests
- `test_streaming.py` — End-to-end audio pipeline test
- **33 tests, all passing**

---

## Dependencies

### Python
```
fastapi
uvicorn[standard]
numpy
sounddevice
mido
pytest
```

### Frontend
```
react: ^18.3.1
@xyflow/react: ^12.6.1
vite: ^6.0.3
typescript: ^5.6.3
```

---

## Legacy Code (Superseded)
- `ui/app.py` — Tkinter GUI (replaced by React frontend)
- `ui/cli.py` — Interactive CLI
- `ui/midi_input.py` — MIDI input handler
- `synthesizers/` — Old synth implementations
- `config/` — Legacy config files
