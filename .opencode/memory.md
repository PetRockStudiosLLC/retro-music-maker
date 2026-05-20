## Goal
- Plan and develop a hybrid custom nodes system (Node Templates + Script Nodes) with full features from day one.

## Constraints & Preferences
- Hybrid approach: JSON-based node templates for composition + embedded scripting (Lua) for custom DSP.
- Full feature set from the start, not MVP.
- Must integrate seamlessly with existing Rust/Tauri v2 backend and React Flow frontend.
- Maintain exact naming conventions and param types across Rust and frontend.

## Progress
### Done (All 6 Phases Complete)
- Phase 1: Node Templates backend (`TemplateNode`, JSON schema, Tauri commands, auto-loading).
- Phase 2: Template frontend (`TemplateEditor.tsx`, toolbar, palette sync).
- Phase 3: Script Nodes backend (`mlua`, `ScriptNode`, Lua per-block VM).
- Phase 4: Script editor frontend (`ScriptEditor.tsx`, Lua code editor, palette sync).
- Phase 5: Save/load persistence — `NodeKind` enum, `NodeInfo` with `node_kind`/`definition`, `load_graph` reconstructs custom nodes, `create_node` tries builtin→template→script chain.
- Phase 6: Documentation updated (`PROJECT_DOCS.md`, `NODE_REFERENCE.md`, `CONTRIBUTING.md`) + build verification.

### In Progress
- (none)

### Blocked
- (none)

## Key Decisions
- Chose Hybrid custom nodes system over pure templates or pure scripting.
- Prioritized full feature implementation from day one over iterative MVP.
- Backend uses `Box<dyn Node>` trait objects in `Graph`, requiring `TemplateNode` and `ScriptNode` to implement the `Node` trait.
- Templates stored in `~/.nodetune/templates/` as JSON files.
- Scripts stored in `~/.nodetune/scripts/` as JSON files with `.lua` extension.
- Lua VM created fresh per audio block to avoid thread-safety issues with `mlua`.
- Custom nodes serialize full definition inline in `NodeInfo.definition` for self-contained graph saves.
- `NodeKind` enum (`Builtin`, `Template`, `Script`) tracks node type for reconstruction.

## Next Steps
- Consider adding: template/script versioning, hot-reload for scripts, Lua sandbox security hardening, graph validation before load.

## Critical Context
- **Backend**: Rust + Tauri v2, `Graph` stores `HashMap<String, Box<dyn Node>>`.
- **Audio Engine**: DAG-based topological processing, 44100 Hz sample rate, block-based processing.
- **Param Types**: `Float`, `Int`, `Bool`, `String` via `ParamValue` enum.
- **Node Trait**: Requires `id`, `name`, `category`, `inputs`, `outputs`, `default_params`, `set_param`, `get_param`, `process`, `to_info`, `kind`.
- **Frontend**: React + `@xyflow/react`, `NODE_TYPE_DEFS` drives palette/properties, `useTauriGraph` handles IPC.
- **Template System**: `TemplateDefinition` holds internal nodes, edges, I/O routing, and exposed params. `TemplateNode` instantiates a nested `Graph` and delegates processing.
- **Script System**: `ScriptDefinition` holds Lua code, ports, params. `ScriptNode` creates fresh Lua VM per block, calls `process(node, input, output, params)` function.
- **Cargo.toml**: Added `mlua = { version = "0.9", features = ["lua54", "vendored"] }` for Lua runtime.

## Relevant Files
- `src/core/node.rs`: `NodeKind` enum, `NodeInfo` with `node_kind`/`definition`, `Node` trait with `kind()` method.
- `src/nodes/custom/template_node.rs`: `TemplateDefinition`, `TemplateInfo`, `TemplateNode` struct and `Node` trait impl.
- `src/nodes/custom/script_node.rs`: `ScriptDefinition`, `ScriptInfo`, `ScriptNode` struct, `parse_port_type()`, Lua processing.
- `src/commands.rs`: Tauri IPC handlers for templates and scripts, `create_node()` with builtin→template→script lookup chain.
- `src-tauri/src/main.rs`: Duplicate command handlers, `reconstruct_node()` for graph loading.
- `frontend/src/App.tsx`: Main app shell, `NODE_TYPE_DEFS`, template/script loading on mount, palette merge.
- `frontend/src/components/TemplateEditor.tsx`: Full UI for composing templates.
- `frontend/src/components/ScriptEditor.tsx`: Full UI for composing Lua script nodes.
- `frontend/src/tauri/api.ts`: TypeScript interfaces for template/script schema + `invoke` wrappers.
- `frontend/src/tauri/useTauriGraph.ts`: IPC hook with save/load graph support.
- `frontend/src/types/nodes.ts`: Extended `EngineState` with `node_kind`/`definition`.
- `PROJECT_DOCS.md`: Updated with custom nodes architecture, IPC commands table, project structure.
- `NODE_REFERENCE.md`: Added custom nodes section with template/script schemas and Lua API docs.
- `CONTRIBUTING.md`: Updated with custom nodes workflow, testing checklist, file structure.