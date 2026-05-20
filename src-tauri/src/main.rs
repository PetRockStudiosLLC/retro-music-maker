use tauri::{Manager, WebviewWindow};
use nodetune_lib::core::{Engine, ParamValue, NodeInfo, GraphState};
use nodetune_lib::nodes::oscillators::{sine::SineOscillator, square::SquareOscillator, sawtooth::SawtoothOscillator, triangle::TriangleOscillator, noise::NoiseOscillator, chipsound::ChipSoundOscillator, vco::VCO};
use nodetune_lib::nodes::filters::{lowpass::LowpassFilter, highpass::HighpassFilter, bitcrush::Bitcrush, bandpass::BandpassFilter};
use nodetune_lib::nodes::envelopes::adsr::ADSREnvelope;
use nodetune_lib::nodes::effects::{delay::DelayEffect, distortion::Distortion, reverb::Reverb, duration_gate::DurationGate, loop_node::Loop};
use nodetune_lib::nodes::mixers::{mixer::Mixer, gain::Gain};
use nodetune_lib::nodes::input::{keyboard::KeyboardInput, midi_input::MidiInput, wav_file::WavFile};
use nodetune_lib::nodes::output::{audio_output::AudioOutput, file_output::FileOutput};
use nodetune_lib::nodes::sequencers::{step_sequencer::StepSequencer, arpeggiator::Arpeggiator, note_sequencer::NoteSequencer, note_mapper::NoteMapper, slot_player::SlotPlayer, clock::Clock, random_trigger::RandomTrigger, trigger_delay::TriggerDelay};
use nodetune_lib::nodes::custom::template_node::{TemplateDefinition, TemplateNode, TemplateInfo};
use nodetune_lib::nodes::custom::script_node::{ScriptDefinition, ScriptNode, ScriptInfo, parse_port_type};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct AddNodeRequest {
    node_type: String,
    position: [f64; 2],
}

#[derive(Clone, Serialize, Deserialize)]
struct RemoveNodeRequest {
    node_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct ConnectNodesRequest {
    source: String,
    source_handle: String,
    target: String,
    target_handle: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct DisconnectNodesRequest {
    source: String,
    target: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct SetNodePositionRequest {
    node_id: String,
    position: [f64; 2],
}

#[derive(Clone, Serialize, Deserialize)]
struct SetParamRequest {
    node_id: String,
    param_name: String,
    value: ParamValue,
}

#[derive(Clone, Serialize, Deserialize)]
struct GetNodeRequest {
    node_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct SaveGraphRequest {
    path: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct LoadGraphRequest {
    path: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct ExportWavRequest {
    path: String,
    duration: f64,
    sample_rate: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SaveFileOutputRequest {
    node_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct NodeCreated {
    node_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct EngineStatus {
    running: bool,
    sample_rate: u32,
    block_size: usize,
}

fn create_node(node_type: &str, id: String) -> Option<Box<dyn nodetune_lib::core::Node>> {
    match node_type {
        "SineOscillator" => Some(Box::new(SineOscillator::new(id))),
        "SquareOscillator" => Some(Box::new(SquareOscillator::new(id))),
        "SawtoothOscillator" => Some(Box::new(SawtoothOscillator::new(id))),
        "TriangleOscillator" => Some(Box::new(TriangleOscillator::new(id))),
        "NoiseOscillator" => Some(Box::new(NoiseOscillator::new(id))),
        "ChipSoundOscillator" => Some(Box::new(ChipSoundOscillator::new(id))),
        "VCO" => Some(Box::new(VCO::new(id))),
        "LowpassFilter" => Some(Box::new(LowpassFilter::new(id))),
        "HighpassFilter" => Some(Box::new(HighpassFilter::new(id))),
        "BandpassFilter" => Some(Box::new(BandpassFilter::new(id))),
        "Bitcrush" => Some(Box::new(Bitcrush::new(id))),
        "ADSREnvelope" => Some(Box::new(ADSREnvelope::new(id))),
        "DelayEffect" => Some(Box::new(DelayEffect::new(id))),
        "Distortion" => Some(Box::new(Distortion::new(id))),
        "Reverb" => Some(Box::new(Reverb::new(id))),
        "DurationGate" => Some(Box::new(DurationGate::new(id))),
        "Loop" => Some(Box::new(Loop::new(id))),
        "Mixer" => Some(Box::new(Mixer::new(id, 4))),
        "Gain" => Some(Box::new(Gain::new(id))),
        "KeyboardInput" => Some(Box::new(KeyboardInput::new(id))),
        "MidiInput" => Some(Box::new(MidiInput::new(id))),
        "AudioOutput" => Some(Box::new(AudioOutput::new(id))),
        "FileOutput" => Some(Box::new(FileOutput::new(id))),
        "StepSequencer" => Some(Box::new(StepSequencer::new(id))),
        "Arpeggiator" => Some(Box::new(Arpeggiator::new(id))),
        "NoteSequencer" => Some(Box::new(NoteSequencer::new(id))),
        "NoteMapper" => Some(Box::new(NoteMapper::new(id))),
        "SlotPlayer" => Some(Box::new(SlotPlayer::new(id))),
        "WavFile" => Some(Box::new(WavFile::new(id))),
        "Clock" => Some(Box::new(Clock::new(id))),
        "RandomTrigger" => Some(Box::new(RandomTrigger::new(id))),
        "TriggerDelay" => Some(Box::new(TriggerDelay::new(id))),
        "Instrument" => Some(Box::new(nodetune_lib::nodes::controllers::instrument::Instrument::new(id))),
        _ => {
            if let Ok(def) = load_template_sync(node_type) {
                match TemplateNode::new(id, def) {
                    Ok(node) => Some(Box::new(node)),
                    Err(e) => {
                        eprintln!("Failed to create template node: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        }
    }
}

fn load_template_sync(name: &str) -> Result<TemplateDefinition, String> {
    let template_dir = get_template_dir()?;
    let file_name = name.to_lowercase().replace(" ", "_");
    let file_path = template_dir.join(format!("{}.json", file_name));
    let json = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_node(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>, request: AddNodeRequest) -> Result<NodeCreated, String> {
    let mut engine = engine.lock().await;
    let id = format!("{}_{}", request.node_type.to_lowercase().replace("oscillator", "osc").replace("filter", "flt").replace("effect", "fx"), engine.node_count());
    if let Some(node) = create_node(&request.node_type, id.clone()) {
        engine.add_node(node, request.position);
        Ok(NodeCreated { node_id: id })
    } else {
        Err(format!("Unknown node type: {}", request.node_type))
    }
}

#[tauri::command]
async fn remove_node(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>, request: RemoveNodeRequest) -> Result<bool, String> {
    let mut engine = engine.lock().await;
    Ok(engine.remove_node(&request.node_id))
}

#[tauri::command]
async fn connect_nodes(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>, request: ConnectNodesRequest) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.add_edge(request.source.clone(), request.source_handle.clone(), request.target.clone(), request.target_handle.clone());

    // Sync Instrument -> NoteMapper when instrument port is connected
    if request.target_handle == "instrument" {
        if let Some(info) = engine.get_node_info(&request.source) {
            if let Some(ParamValue::String(json)) = info.params.get("instrument_json").map(|v| v.clone()) {
                engine.set_node_param(&request.target, "instrument_json", ParamValue::String(json));
            }
        }
    }
    Ok(())
}

#[tauri::command]
async fn disconnect_nodes(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>, request: DisconnectNodesRequest) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.remove_edge(&request.source, &request.target);
    Ok(())
}

#[tauri::command]
async fn set_node_position(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>, request: SetNodePositionRequest) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.set_node_position(&request.node_id, request.position);
    Ok(())
}

#[tauri::command]
async fn set_param(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>, request: SetParamRequest) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.set_param(&request.node_id, &request.param_name, request.value);
    Ok(())
}

#[tauri::command]
async fn get_node(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>, request: GetNodeRequest) -> Result<Option<NodeInfo>, String> {
    let engine = engine.lock().await;
    Ok(engine.get_node_info(&request.node_id))
}

#[tauri::command]
async fn get_graph_state(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>) -> Result<GraphState, String> {
    let engine = engine.lock().await;
    Ok(engine.get_graph_state())
}

#[tauri::command]
async fn start_audio(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.start()?;
    Ok(())
}

#[tauri::command]
async fn stop_audio(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.stop();
    Ok(())
}

#[tauri::command]
async fn engine_status(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>) -> Result<EngineStatus, String> {
    let engine = engine.lock().await;
    Ok(EngineStatus {
        running: engine.is_running(),
        sample_rate: engine.sample_rate,
        block_size: engine.block_size,
    })
}

#[tauri::command]
async fn save_graph(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>, request: SaveGraphRequest) -> Result<(), String> {
    let engine = engine.lock().await;
    let state = engine.get_graph_state();
    let json = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    std::fs::write(&request.path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn load_graph(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>, request: LoadGraphRequest) -> Result<(), String> {
    use nodetune_lib::core::graph::GraphState;

    let json = std::fs::read_to_string(&request.path).map_err(|e| e.to_string())?;
    let state: GraphState = serde_json::from_str(&json).map_err(|e| e.to_string())?;

    let mut engine = engine.lock().await;
    engine.clear();

    for node_info in &state.nodes {
        let node = reconstruct_node(node_info)?;
        engine.add_node(node, node_info.position);
    }

    for edge in &state.edges {
        engine.add_edge(edge.source.clone(), edge.source_handle.clone(), edge.target.clone(), edge.target_handle.clone());
    }

    Ok(())
}

fn reconstruct_node(node_info: &nodetune_lib::core::NodeInfo) -> Result<Box<dyn nodetune_lib::core::Node>, String> {
    use nodetune_lib::core::NodeKind;

    match node_info.node_kind {
        NodeKind::Builtin => {
            nodetune_lib::commands::create_node_from_type(&node_info.name, node_info.id.clone())
                .ok_or_else(|| format!("Unknown builtin node type: {}", node_info.name))
        }
        NodeKind::Template => {
            let def: TemplateDefinition = node_info.definition
                .as_ref()
                .and_then(|v| serde_json::from_value::<TemplateDefinition>(v.clone()).ok())
                .ok_or_else(|| "Template definition missing".to_string())?;
            let node = TemplateNode::new(node_info.id.clone(), def)
                .map_err(|e| format!("Failed to create template node: {}", e))?;
            Ok(Box::new(node))
        }
        NodeKind::Script => {
            let def: ScriptDefinition = node_info.definition
                .as_ref()
                .and_then(|v| serde_json::from_value::<ScriptDefinition>(v.clone()).ok())
                .ok_or_else(|| "Script definition missing".to_string())?;
            let node = ScriptNode::new(node_info.id.clone(), def)
                .map_err(|e| format!("Failed to create script node: {}", e))?;
            Ok(Box::new(node))
        }
    }
}

#[tauri::command]
async fn export_wav(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>, request: ExportWavRequest) -> Result<(), String> {
    let engine = engine.lock().await;
    let wav_data = engine.export_wav(request.duration, request.sample_rate)?;
    std::fs::write(&request.path, wav_data).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn save_file_output(
    engine: tauri::State<'_, tokio::sync::Mutex<Engine>>,
    dialog: tauri::State<'_, tauri_plugin_dialog::Dialog<tauri::Wry>>,
    request: SaveFileOutputRequest,
) -> Result<(), String> {
    let node_id = request.node_id;
    let path = dialog.file()
        .set_title("Save Recording")
        .set_file_name("recording.wav")
        .add_filter("WAV Audio", &["wav"])
        .blocking_save_file();
    let final_path = match path {
        Some(p) => {
            let s = p.as_path().map(|p| p.display().to_string()).ok_or("Invalid path")?;
            if s.ends_with(".wav") { s } else { format!("{}.wav", s) }
        }
        None => return Err("No file selected".to_string()),
    };
    let engine = engine.lock().await;
    let samples = engine.get_file_output_buffer(&node_id)?;
    let sr = engine.sample_rate;
    let nchannels = 1u16;
    let bits = 16u16;
    let byte_rate = sr * nchannels as u32 * bits as u32 / 8;
    let block_align = nchannels * bits / 8;
    let data_size = (samples.len() * bits as usize / 8) as u32;
    let chunk_size = 4u32 + (8 + 24 + data_size) as u32;
    let mut wav = Vec::with_capacity(44 + samples.len() * 2);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&chunk_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&nchannels.to_le_bytes());
    wav.extend_from_slice(&sr.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    for s in &samples {
        let pcm = (*s * 32767.0) as i16;
        wav.extend_from_slice(&pcm.to_le_bytes());
    }
    std::fs::write(&final_path, wav).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn clear_graph(engine: tauri::State<'_, tokio::sync::Mutex<Engine>>) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.clear();
    Ok(())
}

// --- Template Commands ---

#[derive(Clone, Serialize, Deserialize)]
struct CreateTemplateRequest {
    definition: TemplateDefinition,
}

#[tauri::command]
async fn create_template(request: CreateTemplateRequest) -> Result<TemplateInfo, String> {
    let template_dir = get_template_dir()?;
    std::fs::create_dir_all(&template_dir).map_err(|e| e.to_string())?;

    let file_name = request.definition.name.to_lowercase().replace(" ", "_");
    let file_path = template_dir.join(format!("{}.json", file_name));

    let json = serde_json::to_string_pretty(&request.definition).map_err(|e| e.to_string())?;
    std::fs::write(&file_path, json).map_err(|e| e.to_string())?;

    Ok(TemplateInfo {
        name: request.definition.name.clone(),
        category: request.definition.category.clone(),
        description: request.definition.description.clone(),
        inputs: request.definition.inputs.clone(),
        outputs: request.definition.outputs.clone(),
        exposed_params: request.definition.exposed_params.iter().map(|p| p.param.clone()).collect(),
    })
}

#[tauri::command]
async fn list_templates() -> Result<Vec<TemplateInfo>, String> {
    let template_dir = get_template_dir()?;
    if !template_dir.exists() {
        return Ok(Vec::new());
    }

    let mut templates = Vec::new();
    for entry in std::fs::read_dir(&template_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            let json = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            if let Ok(def) = serde_json::from_str::<TemplateDefinition>(&json) {
                templates.push(TemplateInfo {
                    name: def.name,
                    category: def.category,
                    description: def.description,
                    inputs: def.inputs,
                    outputs: def.outputs,
                    exposed_params: def.exposed_params.iter().map(|p| p.param.clone()).collect(),
                });
            }
        }
    }
    Ok(templates)
}

#[tauri::command]
async fn delete_template(name: String) -> Result<bool, String> {
    let template_dir = get_template_dir()?;
    let file_name = name.to_lowercase().replace(" ", "_");
    let file_path = template_dir.join(format!("{}.json", file_name));
    if file_path.exists() {
        std::fs::remove_file(&file_path).map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
async fn load_template(name: String) -> Result<TemplateDefinition, String> {
    let template_dir = get_template_dir()?;
    let file_name = name.to_lowercase().replace(" ", "_");
    let file_path = template_dir.join(format!("{}.json", file_name));
    let json = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

fn get_template_dir() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|e| e.to_string())?;
    Ok(std::path::PathBuf::from(home).join(".nodetune").join("templates"))
}

// --- Script Node Commands ---

#[derive(Clone, Serialize, Deserialize)]
struct CreateScriptRequest {
    definition: ScriptDefinition,
}

#[tauri::command]
async fn create_script(request: CreateScriptRequest) -> Result<ScriptInfo, String> {
    let script_dir = get_script_dir()?;
    std::fs::create_dir_all(&script_dir).map_err(|e| e.to_string())?;

    let file_name = request.definition.name.to_lowercase().replace(" ", "_");
    let file_path = script_dir.join(format!("{}.lua", file_name));

    let json = serde_json::to_string_pretty(&request.definition).map_err(|e| e.to_string())?;
    std::fs::write(&file_path, json).map_err(|e| e.to_string())?;

    Ok(ScriptInfo {
        name: request.definition.name.clone(),
        category: request.definition.category.clone(),
        description: request.definition.description.clone(),
        inputs: request.definition.inputs.iter().map(|p| nodetune_lib::core::PortInfo { name: p.name.clone(), port_type: parse_port_type(&p.port_type) }).collect(),
        outputs: request.definition.outputs.iter().map(|p| nodetune_lib::core::PortInfo { name: p.name.clone(), port_type: parse_port_type(&p.port_type) }).collect(),
        params: request.definition.params.iter().map(|p| p.name.clone()).collect(),
    })
}

#[tauri::command]
async fn list_scripts() -> Result<Vec<ScriptInfo>, String> {
    let script_dir = get_script_dir()?;
    if !script_dir.exists() {
        return Ok(Vec::new());
    }

    let mut scripts = Vec::new();
    for entry in std::fs::read_dir(&script_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "lua") {
            let json = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            if let Ok(def) = serde_json::from_str::<ScriptDefinition>(&json) {
                scripts.push(ScriptInfo {
                    name: def.name,
                    category: def.category,
                    description: def.description,
                    inputs: def.inputs.iter().map(|p| nodetune_lib::core::PortInfo { name: p.name.clone(), port_type: parse_port_type(&p.port_type) }).collect(),
                    outputs: def.outputs.iter().map(|p| nodetune_lib::core::PortInfo { name: p.name.clone(), port_type: parse_port_type(&p.port_type) }).collect(),
                    params: def.params.iter().map(|p| p.name.clone()).collect(),
                });
            }
        }
    }
    Ok(scripts)
}

#[tauri::command]
async fn delete_script(name: String) -> Result<bool, String> {
    let script_dir = get_script_dir()?;
    let file_name = name.to_lowercase().replace(" ", "_");
    let file_path = script_dir.join(format!("{}.lua", file_name));
    if file_path.exists() {
        std::fs::remove_file(&file_path).map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn get_script_dir() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|e| e.to_string())?;
    Ok(std::path::PathBuf::from(home).join(".nodetune").join("scripts"))
}

#[tauri::command]
async fn open_wav_file_dialog(dialog: tauri::State<'_, tauri_plugin_dialog::Dialog<tauri::Wry>>) -> Result<Option<String>, String> {
    let file = dialog.file()
        .set_title("Select WAV File")
        .add_filter("WAV files", &["wav"])
        .add_filter("All files", &["*"])
        .blocking_pick_file();
    match file {
        Some(path) => Ok(path.as_path().map(|p| p.display().to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
async fn save_wav_file_dialog(dialog: tauri::State<'_, tauri_plugin_dialog::Dialog<tauri::Wry>>) -> Result<Option<String>, String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let file = dialog.file()
        .set_title("Save WAV File")
        .set_file_name(format!("export_{}.wav", timestamp))
        .add_filter("WAV Audio", &["wav"])
        .blocking_save_file();
    match file {
        Some(path) => {
            let path_str = path.as_path().map(|p| p.display().to_string());
            if let Some(ref s) = path_str {
                if !s.ends_with(".wav") {
                    return Ok(Some(format!("{}.wav", s)));
                }
            }
            Ok(path_str)
        }
        None => Ok(None),
    }
}

#[tauri::command]
async fn open_note_mapper_editor(window: tauri::WebviewWindow, node_id: String) -> Result<(), String> {
    let app = window.app_handle();
    let existing = app.get_webview_window("note-mapper-editor");
    if let Some(win) = existing {
        let _ = win.set_focus();
        return Ok(());
    }
    let win = WebviewWindow::builder(app, "note-mapper-editor", tauri::WebviewUrl::App("note_mapper_editor.html".into()))
        .title("NoteMapper Editor")
        .inner_size(900.0, 650.0)
        .min_inner_size(700.0, 500.0)
        .build()
        .map_err(|e| e.to_string())?;
    let _ = win.eval(&format!(
        "window.__NOTE_MAPPER_NODE_ID__ = '{}';",
        node_id.replace('\'', "\\'")
    ));
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let engine = Engine::new(44100, 512);
            app.manage(tokio::sync::Mutex::new(engine));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_node,
            remove_node,
            connect_nodes,
            disconnect_nodes,
            set_node_position,
            set_param,
            get_node,
            get_graph_state,
            start_audio,
            stop_audio,
            engine_status,
            save_graph,
            load_graph,
            export_wav,
            save_file_output,
            clear_graph,
            open_wav_file_dialog,
            save_wav_file_dialog,
            open_note_mapper_editor,
            create_template,
            list_templates,
            delete_template,
            load_template,
            create_script,
            list_scripts,
            delete_script,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
