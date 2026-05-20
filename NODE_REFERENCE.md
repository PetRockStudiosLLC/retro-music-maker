# Node Reference

Complete catalog of all audio nodes with exact parameter names, types, port definitions, and value ranges.

## Naming Conventions
- **Param names**: lowercase snake_case (e.g., `pulse_width`, `duty_cycle`)
- **Port names**: lowercase (e.g., `audio`, `gate`, `envelope`, `input_0`, `output`)
- **Categories**: PascalCase (e.g., `Oscillator`, `Effect`, `Filter`)

---

## Oscillators

### SineOscillator
Pure sine wave oscillator.
- **Category**: `Oscillator`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `frequency` | Float | 440.0 | 20.0–20000.0 |
  | `amplitude` | Float | 0.3 | 0.0–1.0 |
- **Inputs**: `frequency` (Control), `amplitude` (Control)
- **Outputs**: `audio` (Audio)

### SquareOscillator
Square wave with adjustable pulse width.
- **Category**: `Oscillator`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `frequency` | Float | 220.0 | 20.0–20000.0 |
  | `amplitude` | Float | 0.3 | 0.0–1.0 |
  | `pulse_width` | Float | 0.5 | 0.0–1.0 |
- **Inputs**: `frequency` (Control), `amplitude` (Control), `pulse_width` (Control)
- **Outputs**: `audio` (Audio)

### SawtoothOscillator
Sawtooth wave oscillator.
- **Category**: `Oscillator`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `frequency` | Float | 440.0 | 20.0–20000.0 |
  | `amplitude` | Float | 0.3 | 0.0–1.0 |
- **Inputs**: `frequency` (Control), `amplitude` (Control)
- **Outputs**: `audio` (Audio)

### TriangleOscillator
Triangle wave oscillator.
- **Category**: `Oscillator`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `frequency` | Float | 440.0 | 20.0–20000.0 |
  | `amplitude` | Float | 0.3 | 0.0–1.0 |
- **Inputs**: `frequency` (Control), `amplitude` (Control)
- **Outputs**: `audio` (Audio)

### NoiseOscillator
White noise generator.
- **Category**: `Oscillator`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `amplitude` | Float | 0.3 | 0.0–1.0 |
- **Inputs**: `amplitude` (Control)
- **Outputs**: `audio` (Audio)

### ChipSoundOscillator
Multi-waveform retro synth (Square, Triangle, Noise, DPCM).
- **Category**: `Oscillator`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `frequency` | Float | 440.0 | 20.0–20000.0 |
  | `amplitude` | Float | 0.3 | 0.0–1.0 |
  | `duty_cycle` | Float | 0.5 | 0.0–1.0 |
  | `wave` | String | "Square" | "Square", "Triangle", "Noise", "DPCM" |
- **Inputs**: `frequency` (Control), `amplitude` (Control), `duty_cycle` (Control)
- **Outputs**: `audio` (Audio)

---

## Filters

### LowpassFilter
One-pole lowpass with resonance.
- **Category**: `Filter`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `cutoff` | Float | 2000.0 | 20.0–22050.0 |
  | `resonance` | Float | 1.0 | 0.1–10.0 |
- **Inputs**: `audio` (Audio), `cutoff` (Control)
- **Outputs**: `audio` (Audio)

### HighpassFilter
Highpass filter.
- **Category**: `Filter`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `cutoff` | Float | 500.0 | 20.0–22050.0 |
  | `resonance` | Float | 1.0 | 0.1–10.0 |
- **Inputs**: `audio` (Audio), `cutoff` (Control)
- **Outputs**: `audio` (Audio)

### BandpassFilter
Bandpass with Q control.
- **Category**: `Filter`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `cutoff` | Float | 1000.0 | 20.0–22050.0 |
  | `q` | Float | 1.0 | 0.1–10.0 |
- **Inputs**: `audio` (Audio), `cutoff` (Control)
- **Outputs**: `audio` (Audio)

### Bitcrush
Bit depth and sample rate reduction.
- **Category**: `Filter`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `bit_depth` | Float | 8.0 | 1.0–16.0 |
  | `sample_rate` | Float | 44100.0 | 100.0–44100.0 |
- **Inputs**: `audio` (Audio)
- **Outputs**: `audio` (Audio)

---

## Envelopes

### ADSREnvelope
Attack-Decay-Sustain-Release envelope generator.
- **Category**: `Envelope`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `attack` | Float | 0.01 | 0.001–5.0 |
  | `decay` | Float | 0.1 | 0.001–5.0 |
  | `sustain` | Float | 0.5 | 0.0–1.0 |
  | `release` | Float | 0.3 | 0.001–5.0 |
- **Inputs**: `gate` (Trigger), `audio` (Audio)
- **Outputs**: `envelope` (Control)
- **Note**: Auto-triggers on first `process()` call if no gate received

---

## Effects

### DelayEffect
Delay/echo with feedback and wet-dry mix.
- **Category**: `Effect`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `time` | Float | 0.3 | 0.0–2.0 |
  | `feedback` | Float | 0.4 | 0.0–0.95 |
  | `mix` | Float | 0.3 | 0.0–1.0 |
- **Inputs**: `audio` (Audio)
- **Outputs**: `audio` (Audio)

### Distortion
Tanh-based waveshaping distortion.
- **Category**: `Effect`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `amount` | Float | 0.5 | 0.0–1.0 |
- **Inputs**: `audio` (Audio)
- **Outputs**: `audio` (Audio)

### Reverb
Simple convolution-style reverb.
- **Category**: `Effect`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `decay` | Float | 0.5 | 0.0–0.99 |
  | `mix` | Float | 0.2 | 0.0–1.0 |
- **Inputs**: `audio` (Audio)
- **Outputs**: `audio` (Audio)

---

## Mixers

### Mixer
Multi-channel audio mixer.
- **Category**: `Mixer`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `channels` | Int | 4 | 1–16 |
  | `volume_0`...`volume_N` | Float | 0.5 | 0.0–1.0 |
- **Inputs**: `input_0`...`input_N` (Audio) — count = `channels`
- **Outputs**: `output` (Audio)

### Gain
Volume amplifier/attenuator.
- **Category**: `Mixer`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `volume` | Float | 0.5 | 0.0–2.0 |
- **Inputs**: `audio` (Audio)
- **Outputs**: `audio` (Audio)

---

## Input

### KeyboardInput
Computer keyboard as MIDI input.
- **Category**: `Input`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `velocity` | Float | 0.7 | 0.0–1.0 |
- **Inputs**: (none)
- **Outputs**: `audio` (Audio)

### MidiInput
External MIDI device input.
- **Category**: `Input`
- **Params**: (none)
- **Inputs**: (none)
- **Outputs**: `audio` (Audio)

---

## Output

### AudioOutput
Final audio output to speakers.
- **Category**: `Output`
- **Params**: (none)
- **Inputs**: `audio` (Audio)
- **Outputs**: (none)

### FileOutput
Record audio to WAV file.
- **Category**: `Output`
- **Params**:
  | Name | Type | Default |
  |------|------|---------|
  | `path` | String | "output.wav" |
- **Inputs**: `audio` (Audio)
- **Outputs**: (none)

---

## Sequencers

### StepSequencer
Rhythm sequencer with 16-step grid.
- **Category**: `Sequencer`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `bpm` | Float | 120.0 | 20.0–300.0 |
  | `steps` | String | "[false; 16]" | JSON bool array |
- **Inputs**: (none)
- **Outputs**: `trigger` (Trigger), `step` (Control)

### Arpeggiator
Note pattern arpeggiator.
- **Category**: `Sequencer`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `bpm` | Float | 120.0 | 20.0–300.0 |
  | `pattern` | String | "[0, 4, 7, 12]" | JSON u8 array (MIDI note offsets) |
- **Inputs**: `trigger` (Trigger)
- **Outputs**: `note` (Control), `trigger` (Trigger)

### NoteSequencer
Self-contained melody player with built-in oscillator and envelope.
- **Category**: `Sequencer`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `bpm` | Float | 140.0 | 20.0–300.0 |
  | `notes` | String | "[60, 64, 67, 72, 67, 64, 60, 0]" | Comma-separated MIDI notes (0=rest) |
  | `waveform` | String | "square" | "square", "sawtooth", "triangle", "sine" |
  | `attack` | Float | 0.002 | 0.001–2.0 |
  | `decay` | Float | 0.05 | 0.001–2.0 |
  | `sustain` | Float | 0.3 | 0.0–1.0 |
  | `release` | Float | 0.05 | 0.001–2.0 |
  | `amplitude` | Float | 0.5 | 0.0–1.0 |
- **Inputs**: (none)
- **Outputs**: `audio` (Audio)

### NoteMapper
FL Studio-style piano roll grid sequencer.
- **Category**: `Sequencer`
- **Params**:
  | Name | Type | Default | Range |
  |------|------|---------|-------|
  | `bpm` | Float | 140.0 | 20.0–300.0 |
  | `grid` | String | "[;;;;;;;;;;;;;;;;]" | 16-char string (;=off, A-G=notes) |
  | `num_steps` | Int | 16 | 4–128 |
  | `min_midi` | Int | 48 | 24–72 |
  | `waveform` | String | "square" | "square", "sawtooth", "triangle", "sine" |
  | `attack` | Float | 0.002 | 0.001–2.0 |
  | `decay` | Float | 0.05 | 0.001–2.0 |
  | `sustain` | Float | 0.3 | 0.0–1.0 |
  | `release` | Float | 0.05 | 0.001–2.0 |
  | `amplitude` | Float | 0.5 | 0.0–1.0 |
- **Inputs**: (none)
- **Outputs**: `audio` (Audio)

---

## How to Add a New Node

### 1. Create the Rust struct
```rust
// src/nodes/category/my_node.rs
use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct MyNode {
    id: String,
    my_param: f64,
}

impl MyNode {
    pub fn new(id: String) -> Self {
        Self { id, my_param: 1.0 }
    }
}
```

### 2. Implement the Node trait
```rust
impl Node for MyNode {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "MyNode" }
    fn category(&self) -> &str { "Category" }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("my_param".to_string(), ParamValue::Float(self.my_param));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        if name == "my_param" {
            let f = match value {
                ParamValue::Float(v) => v,
                ParamValue::Int(v) => v as f64,
                _ => return,
            };
            self.my_param = f.max(0.0).min(10.0);
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        if name == "my_param" { Some(ParamValue::Float(self.my_param)) } else { None }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        (0..block_size).map(|i| {
            let inp = input.and_then(|b| b.get(i)).copied().unwrap_or(0.0);
            inp * self.my_param as f32
        }).collect()
    }

    fn to_info(&self, position: [f64; 2]) -> NodeInfo {
        NodeInfo {
            id: self.id.clone(),
            name: self.name().to_string(),
            category: self.category().to_string(),
            inputs: self.inputs(),
            outputs: self.outputs(),
            params: self.default_params(),
            position,
        }
    }
}
```

### 3. Register in mod.rs
```rust
// src/nodes/category/mod.rs
pub mod my_node;
```

### 4. Register in commands.rs
```rust
// Add to imports
use crate::nodes::category::my_node::MyNode;

// Add to create_node() match arm
"MyNode" => Some(Box::new(MyNode::new(id))),
```

### 5. Add to frontend NODE_TYPE_DEFS
```typescript
// frontend/src/App.tsx
MyNode: {
  category: "Category",
  inputs: [{ name: "audio", port_type: "audio" }],
  outputs: [{ name: "audio", port_type: "audio" }],
  params: { my_param: { type: "number", default: 1, min: 0, max: 10 } },
  description: "My custom node description.",
},
```

## Common Pitfalls

1. **Param name mismatch**: Frontend `set_param` calls use exact string names. If Rust has `pulse_width`, frontend must use `pulse_width` (not `pulseWidth` or `duty`).

2. **Category mismatch**: Node category in `NODE_TYPE_DEFS` must match `category()` return value exactly (case-sensitive).

3. **Port name case**: All port names are lowercase. Use `audio`, not `Audio`.

4. **ParamValue type**: Effects accept both `Float` and `Int` via the `match` pattern. Always handle both:
   ```rust
   let f = match value {
       ParamValue::Float(v) => v,
       ParamValue::Int(v) => v as f64,
       _ => return,
   };
   ```

5. **Sample rate**: Hardcoded to 44100 Hz in all nodes. Do not change without updating all nodes.

6. **Block processing**: `process()` receives `block_size` samples. Always iterate exactly `block_size` times.

---

## Custom Nodes

### Node Templates
JSON-defined node graphs composed of builtin nodes. Stored in `~/.nodetune/templates/` as `.json` files.

**Template Definition Schema**:
```json
{
  "name": "MyTemplate",
  "category": "Custom",
  "description": "A reusable node graph",
  "inputs": [{"name": "audio", "port_type": "Audio"}],
  "outputs": [{"name": "audio", "port_type": "Audio"}],
  "exposed_params": [{"param": "frequency", "label": "Freq"}],
  "internal_nodes": [
    {"id": "osc_0", "node_type": "SineOscillator", "position": [100, 100], "params": {"frequency": 440.0}}
  ],
  "internal_edges": [
    {"source": "osc_0", "source_handle": "audio", "target": "mix_0", "target_handle": "input_0"}
  ],
  "input_routing": [
    {"external": "audio", "internal_node": "osc_0", "internal_port": "frequency"}
  ],
  "output_routing": [
    {"internal_node": "mix_0", "internal_port": "output", "external": "audio"}
  ]
}
```

**How Templates Work**:
- Template nodes instantiate a nested `Graph` with the internal nodes
- Input/output routing connects external ports to internal nodes
- Exposed params forward parameter changes to internal nodes
- Audio processing delegates to the internal graph's topological pipeline

### Script Nodes
Lua-based custom DSP nodes. Stored in `~/.nodetune/scripts/` as `.lua` files.

**Script Definition Schema**:
```json
{
  "name": "MyScript",
  "category": "Custom",
  "description": "Custom DSP in Lua",
  "params": [
    {"name": "gain", "type": "Float", "default": 0.5, "min": 0.0, "max": 1.0}
  ],
  "inputs": [{"name": "audio", "type": "Audio"}],
  "outputs": [{"name": "audio", "type": "Audio"}],
  "script": "function process(node, input, output, params)\n  for i = 1, #input do\n    output[i] = input[i] * params.gain\n  end\nend"
}
```

**Lua API**:
- `process(node, input, output, params)` — Called per audio block
  - `input` — Array of input samples (1-indexed)
  - `output` — Array to write output samples (1-indexed)
  - `params` — Table of current parameter values
  - `node` — Table with `id`, `name`, `category` fields
- Lua tables are 1-indexed (not 0-indexed like Rust)
- A fresh Lua VM is created per audio block for thread safety

**Port Types** (for scripts):
- `"Audio"` — Sample data (f32)
- `"Control"` — Continuous modulation
- `"Trigger"` — Discrete events

### Save/Load Persistence
When saving a graph, custom nodes serialize their full definition inline in `NodeInfo.definition`. When loading, the backend reads `node_kind` to determine reconstruction strategy:
- `Builtin` — Instantiates from `create_node_from_type()`
- `Template` — Deserializes `TemplateDefinition` and creates `TemplateNode`
- `Script` — Deserializes `ScriptDefinition` and creates `ScriptNode`

This ensures saved graphs are fully self-contained and portable.
