# Retro Music Maker v3 — Tauri Migration Plan

## Why Tauri?

### Problems with Current Architecture
1. **Two processes** — Backend (Python) + Frontend (Node) must both be running
2. **Network layer** — REST + WebSocket adds latency, complexity, failure modes
3. **Deployment** — User must install Python + Node, manage two servers
4. **No file system access** — Browser sandbox limits save/load, MIDI, audio devices
5. **Backend dies** — Shell timeout kills server, frontend shows "offline"
6. **Proxy flakiness** — Vite WebSocket proxy unreliable with binary frames

### Tauri Benefits
1. **Single binary** — One install, one process, no server management
2. **Direct IPC** — Rust ↔ Python via Tauri commands (no HTTP/WebSocket)
3. **Native audio** — Direct `sounddevice` access, no Web Audio API bridge
4. **File system** — Native save/load, presets, project management
5. **MIDI** — Direct `mido` access, no browser permissions
6. **Small bundle** — ~3MB vs Electron's ~150MB
7. **System tray** — Background audio, minimize to tray

---

## Architecture: v3 (Tauri)

```
┌─────────────────────────────────────────────────────────────┐
│  Tauri Window (Single Process)                               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ React Frontend (Same as v2, no network code)           │  │
│  │  ┌──────────┐  ┌────────────┐  ┌──────────────────┐  │  │
│  │  │ Palette  │  │  Canvas    │  │ Properties Panel │  │  │
│  │  └──────────┘  └────────────┘  └──────────────────┘  │  │
│  │  ┌──────────────────────────────────────────────────┐  │  │
│  │  │ Console + Status Bar                             │  │  │
│  │  └──────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
│         │ Tauri Commands (IPC)                              │
│         ▼                                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Rust Backend (Tauri Commands)                          │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │  Command Handlers:                              │  │  │
│  │  │  - get_node_types()                             │  │  │
│  │  │  - get_graph_state()                            │  │  │
│  │  │  - add_node(type) → node_id                     │  │  │
│  │  │  - connect_nodes(src, dst)                      │  │  │
│  │  │  - start_audio() / stop_audio()                 │  │  │
│  │  │  - export_wav(duration) → bytes                 │  │  │
│  │  │  - save_graph(path) / load_graph(path)          │  │  │
│  │  │  - list_midi_ports()                            │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │  Python Engine (subprocess or PyO3)             │  │  │
│  │  │  - Same core/ code, runs in-process             │  │  │
│  │  │  - Direct function calls, no serialization      │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Migration Strategy

### Option A: Python Subprocess (Recommended)
**Keep Python engine as-is, communicate via stdin/stdout JSON**

```
Rust Tauri Commands → JSON over pipe → Python subprocess → JSON response
```

**Pros:**
- Zero changes to Python engine code
- Fast to implement
- Python handles audio natively (sounddevice works fine)
- Easy debugging (can run Python separately)

**Cons:**
- Process overhead (negligible for our use case)
- Need to install Python on target machine (or bundle it)

### Option B: PyO3 (Rust → Python FFI)
**Call Python directly from Rust**

```
Rust Tauri Commands → PyO3 bindings → Python engine (in-process)
```

**Pros:**
- In-process, no serialization overhead
- Tighter integration
- Can embed Python binary

**Cons:**
- More complex build
- GIL management
- Harder to debug

### Option C: Full Rust Rewrite
**Port engine to Rust**

```
Rust Tauri Commands → Rust engine (cpal for audio)
```

**Pros:**
- No Python dependency
- Best performance
- Single language

**Cons:**
- Months of work
- Rewrite 22 node types
- Rewrite graph, scheduler, audio backend

**Recommendation: Option A** — Fastest path to v3, preserves working engine.

---

## Implementation Plan

### Phase 1: Tauri Skeleton (Week 1)
**Goal:** Blank Tauri window with React frontend

```
retro-music-maker/
  .cargo/
  src/                          ← Rust source
    main.rs                    ← Tauri entry point
    commands.rs                ← Tauri command handlers
  src-tauri/
    tauri.conf.json            ← Tauri config
    Cargo.toml                 ← Rust dependencies
  frontend/                    ← React app (migrated from v2)
    package.json
    src/
      App.tsx
      ...
```

**Tasks:**
1. `npm create tauri-app@latest` — Initialize Tauri project
2. Copy React frontend from v2
3. Remove all network code (WebSocket, REST fetches)
4. Replace with Tauri command calls
5. Verify blank window opens

**New Dependencies:**
```toml
# Cargo.toml
[dependencies]
tauri = { version = "2", features = ["tray"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

```json
// frontend/package.json
"dependencies": {
  "@tauri-apps/api": "^2.0.0",
  "@tauri-apps/plugin-shell": "^2.0.0",
  ...
}
```

### Phase 2: Python Bridge (Week 2)
**Goal:** Rust commands → Python engine via JSON pipe

**Rust Side (`src/commands.rs`):**
```rust
#[tauri::command]
async fn get_node_types(python: State<'_, PythonBridge>) -> Result<JsonValue, String> {
    python.send("get_node_types", json!({}))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_node(python: State<'_, PythonBridge>, node_type: String) -> Result<JsonValue, String> {
    python.send("add_node", json!({"node_type": node_type}))
        .await
        .map_err(|e| e.to_string())
}
```

**Python Side (`python_bridge.py`):**
```python
import sys, json

def main():
    engine = Engine()
    while True:
        try:
            line = input()
            msg = json.loads(line)
            cmd = msg["cmd"]
            data = msg.get("data", {})
            
            if cmd == "get_node_types":
                result = get_node_types(engine)
            elif cmd == "add_node":
                result = add_node(engine, data["node_type"])
            # ... etc
            
            print(json.dumps({"ok": True, "data": result}))
            sys.stdout.flush()
        except Exception as e:
            print(json.dumps({"ok": False, "error": str(e)}))
            sys.stdout.flush()
```

**Tasks:**
1. Create `python_bridge.py` — JSON pipe protocol
2. Implement Rust `PythonBridge` — manages subprocess, sends/receives JSON
3. Wire up all commands: node_types, graph_state, add_node, connect, etc.
4. Test: frontend calls command → Rust → Python → response

### Phase 3: Audio Pipeline (Week 3)
**Goal:** Real-time audio playback

**Approach:** Python engine runs audio callback directly (sounddevice works in subprocess)

**Tasks:**
1. `start_audio()` command — Python starts sounddevice stream
2. `stop_audio()` command — Python stops stream
3. Audio plays directly from Python (no Web Audio bridge needed!)
4. `export_wav()` command — Render to WAV, return bytes via IPC
5. Test: play test graph, verify audio output

**Key insight:** Since Python runs in the same process tree, `sounddevice` works natively. No need to stream audio to frontend at all!

### Phase 4: File System + Presets (Week 4)
**Goal:** Native save/load, project management

**Tasks:**
1. `save_graph(path)` — Write graph JSON to disk
2. `load_graph(path)` — Read graph JSON, reconstruct engine state
3. File dialog via Tauri shell plugin
4. Preset browser — List/load saved presets
5. Auto-save on window close
6. Recent projects list

### Phase 5: MIDI Support (Week 5)
**Goal:** Native MIDI input

**Tasks:**
1. `list_midi_ports()` — Return available MIDI inputs
2. `start_midi(port_name)` — Begin MIDI capture
3. `stop_midi()` — Stop MIDI capture
4. MIDI note → trigger nodes in graph
5. MIDI velocity → control parameters
6. Visual MIDI indicator in UI

### Phase 6: UI Polish (Week 6)
**Goal:** Production-ready interface

**Tasks:**
1. System tray icon — Minimize to tray, keep audio running
2. Window management — Resize, fullscreen, multi-monitor
3. Keyboard shortcuts — Play/pause, save, undo/redo
4. Node search — Filter palette by name
5. Undo/redo stack — Graph operations
6. Error handling — Graceful Python crash recovery
7. Loading states — Skeleton screens
8. Dark/light theme toggle

### Phase 7: Packaging + Distribution (Week 7)
**Goal:** Installable binaries

**Tasks:**
1. `tauri build` — Windows .exe, macOS .dmg, Linux .deb
2. Bundle Python with engine (or require Python install)
3. Installer with Python check
4. Auto-update via Tauri updater
5. App icon, splash screen
6. README with install instructions

---

## Frontend Changes (v2 → v3)

### Remove
- `useWebSocket.ts` — Entire file (replaced by Tauri commands)
- `useAudioPlayer.ts` — Audio handled by Python directly
- `audio-stream-processor.js` — No Web Audio bridge needed
- Vite proxy config — No backend server
- All `fetch("/api/...")` calls

### Add
- `src-tauri/commands.ts` — TypeScript bindings for Tauri commands
- `src/hooks/useEngine.ts` — State management + command calls
- `src/hooks/useAudio.ts` — Start/stop audio via commands

### Replace
```typescript
// v2: REST call
const resp = await fetch("/api/node-types");
const data = await resp.json();

// v3: Tauri command
import { invoke } from "@tauri-apps/api/core";
const data = await invoke("get_node_types");
```

```typescript
// v2: WebSocket
ws.send(JSON.stringify({ type: "start_audio" }));

// v3: Tauri command
await invoke("start_audio");
```

---

## Rust Command API

```rust
// All Tauri commands (src/commands.rs)

#[tauri::command]
async fn get_node_types(python: State<PythonBridge>) -> Result<JsonValue, String>

#[tauri::command]
async fn get_graph_state(python: State<PythonBridge>) -> Result<JsonValue, String>

#[tauri::command]
async fn add_node(
    python: State<PythonBridge>,
    node_type: String,
    position: (f64, f64),
) -> Result<JsonValue, String>

#[tauri::command]
async fn remove_node(
    python: State<PythonBridge>,
    node_id: String,
) -> Result<(), String>

#[tauri::command]
async fn connect_nodes(
    python: State<PythonBridge>,
    src_id: String,
    src_port: String,
    dst_id: String,
    dst_port: String,
) -> Result<(), String>

#[tauri::command]
async fn disconnect_nodes(
    python: State<PythonBridge>,
    src_id: String,
    dst_id: String,
) -> Result<(), String>

#[tauri::command]
async fn set_param(
    python: State<PythonBridge>,
    node_id: String,
    param_name: String,
    value: JsonValue,
) -> Result<(), String>

#[tauri::command]
async fn start_audio(python: State<PythonBridge>) -> Result<(), String>

#[tauri::command]
async fn stop_audio(python: State<PythonBridge>) -> Result<(), String>

#[tauri::command]
async fn export_wav(
    python: State<PythonBridge>,
    duration: f64,
) -> Result<Vec<u8>, String>

#[tauri::command]
async fn save_graph(
    python: State<PythonBridge>,
    path: String,
) -> Result<(), String>

#[tauri::command]
async fn load_graph(
    python: State<PythonBridge>,
    path: String,
) -> Result<JsonValue, String>

#[tauri::command]
async fn new_graph(python: State<PythonBridge>) -> Result<(), String>

#[tauri::command]
async fn list_midi_ports(python: State<PythonBridge>) -> Result<Vec<String>, String>

#[tauri::command]
async fn start_midi(
    python: State<PythonBridge>,
    port_name: String,
) -> Result<(), String>

#[tauri::command]
async fn stop_midi(python: State<PythonBridge>) -> Result<(), String>
```

---

## Python Bridge Protocol

```python
# Request (Rust → Python via stdin)
{
    "cmd": "add_node",
    "data": {"node_type": "SineOscillator", "position": [100, 200]},
    "id": 1  # Request ID for matching responses
}

# Response (Python → Rust via stdout)
{
    "ok": true,
    "id": 1,
    "data": {"node_id": "sineosc_000", "node_type": "SineOscillator"}
}

# Error response
{
    "ok": false,
    "id": 1,
    "error": "Unknown node type: FooBar"
}
```

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Python subprocess crashes | Medium | High | Auto-restart, error recovery |
| sounddevice latency | Low | Medium | Test on target hardware |
| Rust learning curve | Medium | Medium | Use proven patterns, keep Rust thin |
| Python bundling | Low | Medium | Require Python install OR use PyO3 |
| Audio popping/clicking | Low | Medium | Proper buffer management |
| Tauri 2.0 stability | Low | Low | Use stable features only |

---

## Timeline

| Phase | Duration | Deliverable |
|-------|----------|------------|
| 1. Tauri Skeleton | 1 week | Blank window + React frontend |
| 2. Python Bridge | 1 week | All commands working |
| 3. Audio Pipeline | 1 week | Play/stop audio |
| 4. File System | 1 week | Save/load presets |
| 5. MIDI Support | 1 week | MIDI input |
| 6. UI Polish | 1 week | Production UI |
| 7. Packaging | 1 week | Installable binaries |
| **Total** | **7 weeks** | **v3 Release** |

---

## Decision Log

- **[DECIDED]** Option A (Python subprocess) — Fastest path, preserves engine
- **[DECIDED]** Tauri 2.0 — Latest stable, better DX
- **[DECIDED]** Keep React frontend — Works well, minimal changes needed
- **[DECIDED]** Drop WebSocket entirely — Direct IPC is simpler
- **[DECIDED]** Drop Web Audio bridge — Python handles audio natively
- **[OPEN]** Python bundling vs install — Depends on target audience
