# NodeTune - Retro Music Maker

<div align="center">

![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)
![React](https://img.shields.io/badge/React-18+-blue?logo=react)
![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri)
![License](https://img.shields.io/badge/License-PixVox%20Hybrid-blue)

A node-based retro music creation tool for synthesizing chiptune sounds, applying effects, sequencing melodies, and exporting your creations. Connect audio nodes visually to build complex soundscapes with an intuitive drag-and-drop interface.

[Features](#features) • [Quick Start](#quick-start) • [Documentation](#documentation) • [Examples](#examples)

</div>

## Screenshots

**Main Interface** - Drag-and-drop node editor with sidebar palette, toolbar, and console
![Main Interface](screenshots/Screenshot%202026-05-20%20101748.png)

**Preset Example** - Snake Nokia Classic Tune loaded via the Demos menu
![Preset](screenshots/Screenshot%202026-05-20%20174735.png)

**Node Details** - Hover tooltips show parameter descriptions and values
![Node Details](screenshots/Screenshot%202026-05-20%20084931.png)

## Features

### Audio Engine
- **Node-based architecture** - Build complex audio graphs with a directed acyclic graph (DAG) processing system
- **Topological sorting** - Automatic processing order determination with caching
- **Real-time audio** - Low-latency audio processing via cpal with configurable block sizes
- **Multi-format export** - Export compositions as WAV files

### Nodes & Effects
- **Oscillators**: Sine, Square, Sawtooth, Triangle, Noise, ChipSound, VCO, Waveform
- **Filters**: Lowpass, Highpass, Bandpass, Bitcrush
- **Envelopes**: ADSR envelope generator, LFO modulation
- **Effects**: Delay, Distortion, Reverb, Chorus, Flanger, Loop, Duration Gate
- **Mixers**: 4-channel Mixer, Gain, Crossfader, Splitter
- **Sequencers**: Step Sequencer, Arpeggiator, Clock, Random Trigger, Note Mapper
- **Input**: Keyboard Input, MIDI Input, WAV File player
- **Output**: Audio Output, File Output

### Custom Nodes
- **Template Nodes** - Compose graphs of builtin nodes into reusable templates (JSON-based)
- **Script Nodes** - Write custom DSP in Lua for unlimited creative possibilities

### Frontend
- **React Flow editor** - Drag-and-drop node canvas with pan and zoom
- **Palette browser** - Filterable node categories for easy discovery
- **Properties panel** - Real-time parameter editing with visual controls
- **Piano Roll** - Visual melody composition and editing
- **Note Mapper Editor** - Advanced mapping interface for MIDI controllers
- **Template/Script editors** - Create and manage custom nodes visually
- **Export modal** - WAV export with configuration options
- **Console** - Debug and system messages
- **Undo/Redo support** - Full graph state history

### Presets
- Built-in retro presets for quick starting
- Save and load custom presets as JSON
- Share presets with the community

### Python API
- **Modular node system** - Create custom nodes in Python with decorators
- **Graph engine** - Build and process audio graphs programmatically
- **Synthesizers** - NES, SNES, and Game Boy synthesizer implementations
- **MIDI integration** - Full MIDI input and control support
- **CLI interface** - Command-line audio processing

## Tech Stack

| Layer | Technology |
|-------|------------|
| **Desktop App** | Tauri v2 (Rust backend + React frontend) |
| **Audio Engine** | Rust with cpal for audio streaming |
| **Node System** | Custom DAG with topological sort |
| **Frontend** | React 18, TypeScript, Vite |
| **Node Editor** | React Flow (@xyflow/react) |
| **Custom DSP** | Lua scripting via rlua |
| **Python Backend** | Modular node system with decorators |
| **Testing** | Rust tests, Python pytest |

## Quick Start

### Prerequisites
- **Rust 1.70+** (`rustup install stable`)
- **Node.js 18+** with npm
- **Windows 10/11** (Tauri v2 desktop app)

### Install Dependencies

```powershell
# Install Rust
rustup install stable

# Install Node.js
# Download from https://nodejs.org/

# Clone and install
git clone https://github.com/PetRockStudiosLLC/retro-music-maker.git
cd retro-music-maker

# Install frontend dependencies
cd frontend
npm install
cd ..
```

### Development

```powershell
# Start the Tauri dev server (hot-reload enabled)
cargo tauri dev

# Or start frontend development server separately
cd frontend
npm run dev
```

### Build

```powershell
# Install Tauri CLI (once)
npm install -g @tauri-apps/cli

# Build for production (from project root)
cd src-tauri
tauri build
```

**Release outputs** are placed in `src-tauri/target/release/bundle/`:
- **NSIS Installer** (`bundle/nsis/NodeTune_3.0.0_x64-setup.exe`) — ~2.7MB, recommended for distribution
- **MSI Installer** (`bundle/msi/NodeTune_3.0.0_x64_en-US.msi`) — standard Windows installer

**Frontend only:**
```powershell
cd frontend
npm run build
```

**Lint and type-check:**
```powershell
cd frontend
npm run lint
npx tsc --noEmit
```

## Usage

### Node Editor Basics

1. **Add nodes** - Drag nodes from the palette onto the canvas, or double-click to add at center
2. **Connect nodes** - Click an output pin and drag to an input pin to create a connection
3. **Edit parameters** - Click a node to open the properties panel and adjust parameters
4. **Move nodes** - Drag nodes to rearrange your graph layout
5. **Disconnect** - Double-click an edge to remove the connection
6. **Delete nodes** - Select a node and press Delete, or right-click and choose "Remove"

### Audio Playback

1. Build your node graph by connecting oscillators to effects and outputs
2. Click "Start Audio" to begin playback
3. Use keyboard input or MIDI controller to play notes
4. Adjust parameters in real-time while audio is playing

### Saving & Loading

- **Save Graph** - Export your node graph as a JSON file for later editing
- **Load Graph** - Import a previously saved graph
- **Export WAV** - Render your composition to a WAV file for sharing

### Custom Nodes

#### Template Nodes
1. Build a node graph in the editor
2. Click "Create Template" in the palette
3. Give your template a name and save
4. Your template now appears in the palette for reuse

#### Script Nodes
1. Click "Create Script" in the palette
2. Write Lua code defining your custom DSP logic
3. The script executes per audio block for real-time processing

## Project Structure

```
RetroMusicMaker/
├── src/                          # Rust backend (Tauri)
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
│       ├── effects/              # Delay, Distortion, Reverb, Chorus, Flanger
│       ├── mixers/               # Mixer, Gain, Crossfader, Splitter
│       ├── input/                # KeyboardInput, MidiInput, WAV File
│       ├── output/               # AudioOutput, FileOutput
│       ├── sequencers/           # StepSequencer, Arpeggiator, NoteMapper, etc.
│       └── custom/               # Custom nodes system
│           ├── template_node.rs  # JSON-defined node graphs
│           └── script_node.rs    # Lua-based custom DSP
├── frontend/                     # React + Tauri frontend
│   ├── src/
│   │   ├── App.tsx               # Main application component
│   │   ├── components/           # React Flow node components
│   │   ├── hooks/                # useAudioPlayer, useToast, useWebSocket
│   │   ├── tauri/                # Tauri IPC hooks
│   │   └── types/                # TypeScript type definitions
│   ├── package.json
│   └── vite.config.ts
├── src-tauri/                    # Tauri configuration
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   └── icons/
├── core/                         # Python audio engine core
│   ├── engine.py
│   ├── graph.py
│   ├── node.py
│   ├── port.py
│   └── scheduler.py
├── nodes/                        # Python node implementations
│   ├── oscillators/
│   ├── filters/
│   ├── envelopes/
│   ├── effects/
│   ├── mixers/
│   ├── input/
│   ├── output/
│   └── sequencers/
├── synthesizers/                 # Chiptune synthesizers
│   ├── nes.py
│   ├── snes.py
│   └── gameboy.py
├── python_api/                   # Python API with decorators
│   ├── decorators.py
│   └── examples/
├── ui/                           # Python UI layer
│   ├── app.py
│   ├── cli.py
│   ├── midi_input.py
│   └── retro_theme.py
├── presets/                      # JSON preset files
├── tests/                        # Python tests
├── api/                          # Python API server
├── Cargo.toml                    # Rust dependencies
├── requirements.txt              # Python dependencies
├── start-dev.ps1                 # Development startup script
└── PROJECT_DOCS.md               # Detailed architecture docs
```

## Core Concepts

### Node Trait

Every node implements the `Node` trait:

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
    fn kind(&self) -> NodeKind { NodeKind::Builtin }
}
```

### Port Types

- **Audio** (`PortType::Audio`) - Sample data (`f32`), carries audio signals
- **Control** (`PortType::Control`) - Continuous modulation (frequency, cutoff, etc.)
- **Trigger** (`PortType::Trigger`) - Discrete events (note on/off, clock ticks)

### Graph Processing

The audio graph is a **Directed Acyclic Graph (DAG)**. Processing order is determined by topological sort:

1. Topologically sort nodes (cached until graph changes)
2. For each node in order:
   - Mix upstream buffers from connected edges
   - Call `node.process(block_size, mixed_input)`
   - Store output buffer for downstream nodes
3. Final output nodes mix all active buffers to speakers

### Parameter Values

```rust
enum ParamValue {
    Float(f64),   // Continuous values (frequency, volume, etc.)
    Int(i64),     // Discrete values (steps, channels, etc.)
    Bool(bool),   // Toggle switches
    String(String), // Text paths, note sequences
}
```

## Tauri IPC Commands

All frontend-to-backend communication goes through Tauri commands:

| Command | Description |
|---------|-------------|
| `add_node` | Add node to graph |
| `remove_node` | Remove node from graph |
| `connect_nodes` | Connect source output to target input |
| `disconnect_nodes` | Disconnect nodes |
| `set_node_position` | Move node on canvas |
| `set_param` | Change node parameter |
| `get_graph_state` | Get full graph snapshot |
| `start_audio` | Start audio engine |
| `stop_audio` | Stop audio engine |
| `save_graph` | Save graph to JSON file |
| `load_graph` | Load graph from JSON file |
| `export_wav` | Export audio as WAV |
| `clear_graph` | Clear entire graph |
| `create_template` | Save node template to disk |
| `list_templates` | List available templates |
| `create_script` | Save Lua script node to disk |
| `list_scripts` | List available scripts |

## Python Node Creation

Create custom nodes in Python using decorators:

```python
from python_api import audio_node, Input, Output, Param

@audio_node("My Synth", "Synthesizer")
class MySynth:
    frequency = Param(float, 440.0, "Frequency", 20.0, 20000.0)
    waveform = Param(str, "sine", "Waveform", choices=["sine", "square", "sawtooth"])
    
    input = Input("audio", "Input signal")
    output = Output("audio", "Output signal")
    
    def process(self, input: list[float], block_size: int) -> list[float]:
        # Your DSP code here
        return output_buffer
```

## Preset System

Presets are JSON objects with `nodes` and `edges` arrays. Load via:

```typescript
await tauri.handleLoadPreset({ nodes: [...], edges: [...] });
```

Built-in presets include:
- **Chiptune Bass** - Classic retro bass sound
- **Retro Adventure** - Adventure game-style theme starter

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Audio routing silence | Check `process_block` passes buffers upstream→downstream |
| Params not applying | Verify `ParamValue` type matches (Float vs Int) |
| Preset won't load | Check category name matches Rust `category()` exactly |
| Port mismatch | Verify port names are lowercase and match Rust definitions |
| Build fails on Windows | Run `rustup target add x86_64-pc-windows-msvc` |
| Frontend not hot-reloading | Ensure no other Vite dev server is running on the port |

## Design Principles

1. **Minimum-viable diffs** - Fix only what's broken, don't refactor unless necessary
2. **Exact param names** - Frontend must match Rust backend param names exactly
3. **Category consistency** - Node categories must match `category()` return value
4. **Port naming** - All port names are lowercase (`audio`, `gate`, `envelope`, `input_N`, `output`)
5. **No background processes** - All state changes go through Tauri IPC

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

### Development Workflow

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the [PixVox Hybrid License](LICENSE) (BSL 1.1 + MIT).

Production use is restricted to entities with **$100,000 USD or less** in gross revenue/funding. On **January 1st, 2030**, this license converts to MIT.

## Acknowledgments

- Built with [Tauri](https://tauri.app/) for the desktop application
- Node editor powered by [React Flow](https://reactflow.dev/)
- Audio processing via [cpal](https://crates.io/crates/cpal)
- Custom DSP enabled by [Lua](https://www.lua.org/) scripting

---

<div align="center">
  <sub>Built with ❤️ by <a href="https://github.com/PetRockStudiosLLC">PetRockStudiosLLC</a></sub>
</div>
