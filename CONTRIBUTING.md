# Contributing to NodeTune

## Quick Reference

| Task | Command |
|------|---------|
| Dev server | `cargo tauri dev` |
| Production build | `cargo tauri build` |
| Frontend only | `cd frontend && npm run build` |
| Type check | `cd frontend && npx tsc --noEmit` |

## Ground Rules

1. **Minimum-viable diffs** — Fix only what's broken. No drive-by refactors.
2. **Exact param names** — Frontend must match Rust backend param names character-for-character.
3. **No background processes** — All state changes go through Tauri IPC commands.
4. **Test before committing** — Run `cargo tauri dev` and verify audio works.

## Adding a New Node

### Builtin Nodes (Rust)
Follow the 5-step process in `NODE_REFERENCE.md`. Key checklist:

- [ ] Rust struct implements `Node` trait
- [ ] Registered in `src/nodes/category/mod.rs`
- [ ] Added to `create_node_from_type()` in `src/commands.rs`
- [ ] Added to `NODE_TYPE_DEFS` in `frontend/src/App.tsx`
- [ ] Param names match exactly between Rust and frontend
- [ ] Category name matches `category()` return value
- [ ] All port names are lowercase

### Custom Nodes (No Code)
For nodes that don't require custom DSP, use the built-in template or script editors:

**Template Nodes**: Compose graphs of builtin nodes into reusable templates. Use the Template Editor in the app to define internal nodes, edges, I/O routing, and exposed params.

**Script Nodes**: Write custom DSP in Lua. Use the Script Editor to define ports, params, and the `process()` function. Scripts run in a sandboxed Lua VM per audio block.

Both appear in the node palette and serialize with full definitions when saving graphs.

## Param Naming Rules

| Rule | Example |
|------|---------|
| lowercase snake_case | `pulse_width`, `duty_cycle` |
| No camelCase | NOT `pulseWidth` |
| No PascalCase | NOT `PulseWidth` |
| Consistent across all nodes | `frequency`, `amplitude`, `cutoff` |

## Port Naming Rules

| Rule | Example |
|------|---------|
| All lowercase | `audio`, `gate`, `envelope` |
| Numbered inputs use underscore | `input_0`, `input_1` |
| Mixer output is `output` | NOT `audio` |

## ParamValue Types

```rust
enum ParamValue {
    Float(f64),    // Continuous: frequency, volume, time
    Int(i64),      // Discrete: steps, channels
    Bool(bool),    // Toggles
    String(String), // Text: paths, note sequences, waveforms
}
```

**Critical**: Effect nodes must accept BOTH `Float` AND `Int` for numeric params:
```rust
let f = match value {
    ParamValue::Float(v) => v,
    ParamValue::Int(v) => v as f64,
    _ => return,
};
```

## Frontend Changes

### Adding to NODE_TYPE_DEFS
```typescript
MyNode: {
  category: "Category",      // Must match Rust category() exactly
  inputs: [...],            // Port definitions
  outputs: [...],           // Port definitions
  params: {                // Parameter definitions
    my_param: {
      type: "number",      // "number", "select", or "text"
      default: 1.0,
      min: 0.0,
      max: 10.0,
    },
  },
  description: "What this node does.",
},
```

### Preset Structure
```typescript
{
  nodes: [
    {
      id: "unique_id",
      name: "NodeTypeName",      // Must match Rust name() exactly
      category: "Category",      // Must match Rust category() exactly
      params: { param_name: value },  // Must match Rust param names
      position: [x, y],
      inputs: [...],
      outputs: [...],
    }
  ],
  edges: [
    {
      source: "node_id",
      source_handle: "output_port",   // lowercase
      target: "node_id",
      target_handle: "input_port",    // lowercase
    }
  ],
}
```

## Code Style

### Rust
- Use `snake_case` for variables and functions
- Prefer `let f = match value { ... }` for ParamValue extraction
- Clamp values in `set_param()` to valid ranges
- Use `AudioBuffer::with_capacity(block_size)` for pre-allocation

### TypeScript
- Use `camelCase` for variables and functions
- Use `snake_case` for param/port names (must match Rust)
- Use `PascalCase` for component names

## Testing Checklist

Before submitting changes:
- [ ] `cargo tauri dev` starts without errors
- [ ] New node appears in palette
- [ ] Node can be added to canvas
- [ ] Parameters can be adjusted in Properties panel
- [ ] Audio routing works (connect nodes, hear sound)
- [ ] Preset with new node loads correctly
- [ ] Save/load graph round-trips correctly
- [ ] Custom nodes (templates/scripts) serialize and deserialize correctly
- [ ] `node_kind` field is set correctly in `NodeInfo`
- [ ] `definition` field contains valid serialized definition for custom nodes

## Common Bugs

| Symptom | Cause | Fix |
|---------|-------|-----|
| Param changes do nothing | Name mismatch | Check exact string in Rust `set_param` |
| Node won't load in preset | Category mismatch | Check `category()` return value |
| Audio silence | Wrong buffer in `process_block` | Ensure upstream→downstream passing |
| Effect reads silence | Hardcoded input buffer | Use `input` parameter, not local buffer |
| Integer params ignored | Missing `Int` variant | Add `ParamValue::Int(v) => v as f64` |

## File Structure

```
src/
├── lib.rs                    # Library entry
├── commands.rs               # Tauri IPC commands
├── core/
│   ├── mod.rs                # Shared types (NodeKind, ParamValue, etc.)
│   ├── node.rs               # Node trait + NodeInfo (with node_kind, definition)
│   ├── port.rs               # Port types
│   ├── graph.rs              # DAG processing + GraphState
│   └── engine.rs             # Audio engine
└── nodes/
    ├── oscillators/          # Sound sources
    ├── filters/              # Frequency shaping
    ├── envelopes/            # Amplitude shaping
    ├── effects/              # Time-domain FX
    ├── mixers/               # Routing + volume
    ├── input/                # MIDI/keyboard
    ├── output/               # Speakers/file
    ├── sequencers/           # Pattern generators
    └── custom/               # Custom nodes system
        ├── template_node.rs  # TemplateNode: JSON-defined node graphs
        └── script_node.rs    # ScriptNode: Lua-based custom DSP

frontend/
├── src/
│   ├── App.tsx               # Main component
│   ├── tauri/useTauriGraph.ts # IPC hook
│   ├── tauri/api.ts          # Tauri API wrappers + types
│   ├── components/           # React Flow nodes
│   ├── components/TemplateEditor.tsx  # Template editor UI
│   ├── components/ScriptEditor.tsx    # Script editor UI
│   └── types/nodes.ts        # TypeScript types
└── package.json
```

## Pull Request Template

```markdown
## What
[Brief description of changes]

## Why
[Reason for the change]

## Testing
- [ ] Audio works correctly
- [ ] Presets load without errors
- [ ] Param names match between Rust and frontend
- [ ] No console errors

## Files Changed
- `src/nodes/...`
- `frontend/src/...`
```
