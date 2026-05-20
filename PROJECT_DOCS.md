# NodeTune - Retro Music Maker

A node-based retro music creation tool built with Rust, Tauri, and React. Connect audio nodes visually to synthesize chiptune sounds, apply effects, sequence melodies, and export your creations.

## Quick Start

### Prerequisites
- **Rust 1.70+** (`rustup install stable`)
- **Node.js 18+** with npm
- **Windows 10/11** (Tauri v2 desktop app)

### Development
```powershell
# Start the Tauri dev server (hot-reload enabled)
cargo tauri dev

# Or build for production
cargo tauri build
```

### Project Structure
```
RetroMusicMaker/
├── src/                          # Rust backend
│   ├── lib.rs                    # Library entry point
│   ├── commands.rs               # Tauri IPC command handlers
│   ├── core/                     # Audio engine core
│   │   ├── mod.rs                # Types: ParamValue, AudioBuffer, BlockSize
│   │   ├── node.rs               # Node trait + SignalRegistry
│   │   ├── port.rs               # PortType, PortInfo, Audio/Control/Trigger ports
│   │   ├── graph.rs              # DAG graph with topological processing
│   │   └── engine.rs             # Audio engine + cpal stream management
│   └── nodes/                    # Audio node implementations
│       ├── oscillators/          # Sine, Square, Sawtooth, Triangle, Noise, ChipSound
│       ├── filters/              # Lowpass, Highpass, Bandpass, Bitcrush
│       ├── envelopes/            # ADSR envelope generator
│       ├── effects/              # Delay, Distortion, Reverb
│       ├── mixers/               # Mixer, Gain
│       ├── input/                # KeyboardInput, MidiInput
│       ├── output/               # AudioOutput, FileOutput
│       ├── sequencers/           # StepSequencer, Arpeggiator, NoteSequencer, NoteMapper
│       └── custom/               # Custom nodes system
│           ├── template_node.rs  # TemplateNode: JSON-defined node graphs
│           └── script_node.rs    # ScriptNode: Lua-based custom DSP
├── frontend/                     # React + Tauri frontend
│   ├── src/
│   │   ├── App.tsx               # Main application component
│   │   ├── tauri/useTauriGraph.ts # Tauri IPC hook (replaces WebSocket)
│   │   ├── components/           # React Flow node components
│   │   └── types/nodes.ts        # TypeScript type definitions
│   └── package.json
├── src-tauri/                    # Tauri configuration
│   └── tauri.conf.json
├── Cargo.toml                    # Rust dependencies
└── presets/                      # JSON preset files
```

## Core Concepts

### Node
The fundamental building block. Every node implements the `Node` trait:
```rust
pub trait Node: Send {
    fn id(&self) -> &str;                      // Unique identifier
    fn name(&self) -> &str;                    // Display name
    fn category(&self) -> &str;                // Category: Oscillator, Filter, etc.
    fn inputs(&self) -> Vec<PortInfo>;         // Input ports
    fn outputs(&self) -> Vec<PortInfo>;        // Output ports
    fn default_params(&self) -> NodeParams;    // Default parameter values
    fn set_param(&mut self, name: &str, value: ParamValue);
    fn get_param(&self, name: &str) -> Option<ParamValue>;
    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer;
    fn to_info(&self, position: [f64; 2]) -> NodeInfo;
    fn kind(&self) -> NodeKind { NodeKind::Builtin } // NodeKind::Template or NodeKind::Script for custom nodes
}
```

### Node Kinds
- **Builtin** — Compiled Rust nodes (oscillators, filters, effects, etc.)
- **Template** — JSON-defined node graphs composed of builtin nodes, serialized with full definition in `NodeInfo`
- **Script** — Lua-based custom DSP nodes, serialized with embedded script definition in `NodeInfo`

### Custom Nodes System
NodeTune supports two types of custom nodes:

**Node Templates** — Compose graphs of builtin nodes into reusable templates. Templates are stored as JSON in `~/.nodetune/templates/` and appear in the node palette. Template nodes instantiate a nested graph and delegate audio processing to internal nodes.

**Script Nodes** — Write custom DSP in Lua. Scripts are stored in `~/.nodetune/scripts/` and define a `process(node, input, output, params)` function. A fresh Lua VM is created per audio block for thread safety.

Both template and script nodes serialize their full definition inline when saving a graph, so loaded graphs are fully self-contained.

### Port Types
- **Audio** (`PortType::Audio`) — Sample data (`f32`), carries audio signals
- **Control** (`PortType::Control`) — Continuous modulation (frequency, cutoff, etc.)
- **Trigger** (`PortType::Trigger`) — Discrete events (note on/off, clock ticks)

### Graph Processing
The audio graph is a **Directed Acyclic Graph (DAG)**. Processing order is determined by topological sort. Each block:
1. Topologically sort nodes (cached until graph changes)
2. For each node in order:
   - Mix upstream buffers from connected edges
   - Call `node.process(block_size, mixed_input)`
   - Store output buffer for downstream nodes
3. Final output nodes mix all active buffers to speakers

### Parameter Values
Parameters use `ParamValue` enum with automatic JSON deserialization:
```rust
enum ParamValue {
    Float(f64),   // Continuous values (frequency, volume, etc.)
    Int(i64),     // Discrete values (steps, channels, etc.)
    Bool(bool),   // Toggle switches
    String(String), // Text paths, note sequences
}
```

## Tauri IPC Commands

All frontend-to-backend communication goes through Tauri commands defined in `src/commands.rs`:

| Command | Request | Response | Description |
|---------|---------|----------|-------------|
| `add_node` | `AddNodeRequest` | `NodeCreated` | Add node to graph |
| `remove_node` | `RemoveNodeRequest` | `bool` | Remove node from graph |
| `connect_nodes` | `ConnectNodesRequest` | `()` | Connect source output to target input |
| `disconnect_nodes` | `DisconnectNodesRequest` | `()` | Disconnect nodes |
| `set_node_position` | `SetNodePositionRequest` | `()` | Move node on canvas |
| `set_param` | `SetParamRequest` | `()` | Change node parameter |
| `get_node` | `GetNodeRequest` | `Option<NodeInfo>` | Get node info |
| `get_graph_state` | `()` | `GraphState` | Get full graph snapshot |
| `start_audio` | `()` | `()` | Start audio engine |
| `stop_audio` | `()` | `()` | Stop audio engine |
| `engine_status` | `()` | `EngineStatus` | Get engine status |
| `save_graph` | `SaveGraphRequest` | `()` | Save graph to JSON file (includes custom node definitions) |
| `load_graph` | `LoadGraphRequest` | `()` | Load graph from JSON file (reconstructs custom nodes) |
| `export_wav` | `ExportWavRequest` | `()` | Export audio as WAV |
| `clear_graph` | `()` | `()` | Clear entire graph |
| `create_template` | `CreateTemplateRequest` | `TemplateInfo` | Save node template to disk |
| `list_templates` | `()` | `Vec<TemplateInfo>` | List available templates |
| `delete_template` | `{ name }` | `bool` | Delete template |
| `load_template` | `{ name }` | `TemplateDefinition` | Load template definition |
| `create_script` | `CreateScriptRequest` | `ScriptInfo` | Save Lua script node to disk |
| `list_scripts` | `()` | `Vec<ScriptInfo>` | List available scripts |
| `delete_script` | `{ name }` | `bool` | Delete script |

## Frontend Architecture

### useTauriGraph Hook
`frontend/src/tauri/useTauriGraph.ts` provides the bridge between React and the Rust backend:
- Polls `get_graph_state` every 500ms for live updates
- Exposes `handleNodeAdd`, `handleConnect`, `handleParamChange`, etc.
- Manages `graphState`, `isPlaying`, and `isLoading` React state

### React Flow Integration
The node editor uses React Flow (`@xyflow/react`):
- Custom node components in `frontend/src/components/`
- Drag-and-drop from palette onto canvas
- Double-click edge to disconnect
- Properties panel for parameter editing

### Preset System
Presets are JSON objects with `nodes` and `edges` arrays. Load via:
```typescript
await tauri.handleLoadPreset({ nodes: [...], edges: [...] });
```

See `NODE_REFERENCE.md` for the complete node catalog with exact param names and port definitions.

## Building

```powershell
# Development mode with hot-reload
cargo tauri dev

# Production build
cargo tauri build

# Frontend only
cd frontend && npm run build

# Lint and type-check frontend
cd frontend && npm run lint && npx tsc --noEmit
```

## Design Principles

1. **Minimum-viable diffs** — Fix only what's broken, don't refactor unless necessary
2. **Exact param names** — Frontend must match Rust backend param names exactly
3. **Category consistency** — Node categories must match `category()` return value
4. **Port naming** — All port names are lowercase (`audio`, `gate`, `envelope`, `input_N`, `output`)
5. **No background processes** — All state changes go through Tauri IPC

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Audio routing silence | Check `process_block` passes buffers upstream→downstream |
| Params not applying | Verify `ParamValue` type matches (Float vs Int) |
| Preset won't load | Check category name matches Rust `category()` exactly |
| Port mismatch | Verify port names are lowercase and match Rust definitions |
| Build fails on Windows | Run `rustup target add x86_64-pc-windows-msvc` |
