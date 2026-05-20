use crate::core::{Engine, ParamValue, Node, NodeInfo, PortInfo};
use crate::core::graph::GraphState;
use crate::nodes::oscillators::{sine::SineOscillator, square::SquareOscillator, sawtooth::SawtoothOscillator, triangle::TriangleOscillator, noise::NoiseOscillator, chipsound::ChipSoundOscillator, vco::VCO, waveform::WaveformOsc};
use crate::nodes::filters::{lowpass::LowpassFilter, highpass::HighpassFilter, bitcrush::Bitcrush, bandpass::BandpassFilter};
use crate::nodes::envelopes::adsr::ADSREnvelope;
use crate::nodes::effects::{delay::DelayEffect, distortion::Distortion, reverb::Reverb, duration_gate::DurationGate, loop_node::Loop};
use crate::nodes::mixers::{mixer::Mixer, gain::Gain};
use crate::nodes::input::{keyboard::KeyboardInput, midi_input::MidiInput, wav_file::WavFile};
use crate::nodes::output::{audio_output::AudioOutput, file_output::FileOutput};
use crate::nodes::sequencers::{step_sequencer::StepSequencer, arpeggiator::Arpeggiator, note_sequencer::NoteSequencer, note_mapper::NoteMapper, slot_player::SlotPlayer, clock::Clock, random_trigger::RandomTrigger, trigger_delay::TriggerDelay};
use crate::nodes::controllers::instrument::Instrument;
use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AddNodeRequest {
    pub node_type: String,
    pub position: [f64; 2],
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RemoveNodeRequest {
    pub node_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectNodesRequest {
    pub source: String,
    pub source_handle: String,
    pub target: String,
    pub target_handle: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DisconnectNodesRequest {
    pub source: String,
    pub target: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetNodePositionRequest {
    pub node_id: String,
    pub position: [f64; 2],
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SetParamRequest {
    pub node_id: String,
    pub param_name: String,
    pub value: ParamValue,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GetNodeRequest {
    pub node_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SaveGraphRequest {
    pub path: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LoadGraphRequest {
    pub path: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExportWavRequest {
    pub path: String,
    pub duration: f64,
    pub sample_rate: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NodeCreated {
    pub node_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EngineStatus {
    pub running: bool,
    pub sample_rate: u32,
    pub block_size: usize,
}

pub fn create_node_from_type(node_type: &str, id: String) -> Option<Box<dyn Node>> {
    match node_type {
        "SineOscillator" => Some(Box::new(SineOscillator::new(id))),
        "SquareOscillator" => Some(Box::new(SquareOscillator::new(id))),
        "SawtoothOscillator" => Some(Box::new(SawtoothOscillator::new(id))),
        "TriangleOscillator" => Some(Box::new(TriangleOscillator::new(id))),
        "NoiseOscillator" => Some(Box::new(NoiseOscillator::new(id))),
        "ChipSoundOscillator" => Some(Box::new(ChipSoundOscillator::new(id))),
        "VCO" => Some(Box::new(VCO::new(id))),
        "Waveform" => Some(Box::new(WaveformOsc::new(id))),
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
        "WavFile" => Some(Box::new(WavFile::new(id))),
        "AudioOutput" => Some(Box::new(AudioOutput::new(id))),
        "FileOutput" => Some(Box::new(FileOutput::new(id))),
        "StepSequencer" => Some(Box::new(StepSequencer::new(id))),
        "Arpeggiator" => Some(Box::new(Arpeggiator::new(id))),
        "NoteSequencer" => Some(Box::new(NoteSequencer::new(id))),
        "NoteMapper" => Some(Box::new(NoteMapper::new(id))),
        "SlotPlayer" => Some(Box::new(SlotPlayer::new(id))),
        "Clock" => Some(Box::new(Clock::new(id))),
        "RandomTrigger" => Some(Box::new(RandomTrigger::new(id))),
        "TriggerDelay" => Some(Box::new(TriggerDelay::new(id))),
        "Instrument" => Some(Box::new(Instrument::new(id))),
        _ => None,
    }
}

fn create_node(node_type: &str, id: String) -> Option<Box<dyn Node>> {
    // Try builtin nodes first
    if let Some(node) = create_node_from_type(node_type, id.clone()) {
        return Some(node);
    }

    // Try to load as template
    if let Ok(template_dir) = get_template_dir() {
        let file_name = node_type.to_lowercase().replace(" ", "_");
        let file_path = template_dir.join(format!("{}.json", file_name));
        if let Ok(json) = std::fs::read_to_string(&file_path) {
            if let Ok(def) = serde_json::from_str::<TemplateDefinition>(&json) {
                if let Ok(template_node) = TemplateNode::new(id.clone(), def) {
                    return Some(Box::new(template_node));
                }
            }
        }
    }

    // Try to load as script
    if let Ok(script_dir) = get_script_dir() {
        let file_name = node_type.to_lowercase().replace(" ", "_");
        let file_path = script_dir.join(format!("{}.lua", file_name));
        if let Ok(json) = std::fs::read_to_string(&file_path) {
            if let Ok(def) = serde_json::from_str::<ScriptDefinition>(&json) {
                if let Ok(script_node) = ScriptNode::new(id, def) {
                    return Some(Box::new(script_node));
                }
            }
        }
    }

    None
}

fn get_template_dir() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|e| e.to_string())?;
    Ok(std::path::PathBuf::from(home).join(".nodetune").join("templates"))
}

fn get_script_dir() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|e| e.to_string())?;
    Ok(std::path::PathBuf::from(home).join(".nodetune").join("scripts"))
}

#[tauri::command]
async fn add_node(engine: State<'_, tokio::sync::Mutex<Engine>>, request: AddNodeRequest) -> Result<NodeCreated, String> {
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
async fn remove_node(engine: State<'_, tokio::sync::Mutex<Engine>>, request: RemoveNodeRequest) -> Result<bool, String> {
    let mut engine = engine.lock().await;
    Ok(engine.remove_node(&request.node_id))
}

#[tauri::command]
async fn connect_nodes(engine: State<'_, tokio::sync::Mutex<Engine>>, request: ConnectNodesRequest) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.add_edge(request.source, request.source_handle, request.target, request.target_handle);
    Ok(())
}

#[tauri::command]
async fn disconnect_nodes(engine: State<'_, tokio::sync::Mutex<Engine>>, request: DisconnectNodesRequest) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.remove_edge(&request.source, &request.target);
    Ok(())
}

#[tauri::command]
async fn set_node_position(engine: State<'_, tokio::sync::Mutex<Engine>>, request: SetNodePositionRequest) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.set_node_position(&request.node_id, request.position);
    Ok(())
}

#[tauri::command]
async fn set_param(engine: State<'_, tokio::sync::Mutex<Engine>>, request: SetParamRequest) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.set_param(&request.node_id, &request.param_name, request.value);
    Ok(())
}

#[tauri::command]
async fn get_node(engine: State<'_, tokio::sync::Mutex<Engine>>, request: GetNodeRequest) -> Result<Option<NodeInfo>, String> {
    let engine = engine.lock().await;
    Ok(engine.get_node_info(&request.node_id))
}

#[tauri::command]
async fn get_graph_state(engine: State<'_, tokio::sync::Mutex<Engine>>) -> Result<GraphState, String> {
    let engine = engine.lock().await;
    Ok(engine.get_graph_state())
}

#[tauri::command]
async fn start_audio(engine: State<'_, tokio::sync::Mutex<Engine>>) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.start()?;
    Ok(())
}

#[tauri::command]
async fn stop_audio(engine: State<'_, tokio::sync::Mutex<Engine>>) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.stop();
    Ok(())
}

#[tauri::command]
async fn engine_status(engine: State<'_, tokio::sync::Mutex<Engine>>) -> Result<EngineStatus, String> {
    let engine = engine.lock().await;
    Ok(EngineStatus {
        running: engine.is_running(),
        sample_rate: engine.sample_rate,
        block_size: engine.block_size,
    })
}

#[tauri::command]
async fn save_graph(engine: State<'_, tokio::sync::Mutex<Engine>>, request: SaveGraphRequest) -> Result<(), String> {
    let engine = engine.lock().await;
    let state = engine.get_graph_state();
    let json = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    std::fs::write(&request.path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn load_graph(engine: State<'_, tokio::sync::Mutex<Engine>>, request: LoadGraphRequest) -> Result<(), String> {
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

fn reconstruct_node(node_info: &NodeInfo) -> Result<Box<dyn Node>, String> {
    use crate::core::NodeKind;

    match node_info.node_kind {
        NodeKind::Builtin => {
            create_node_from_type(&node_info.name, node_info.id.clone())
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
async fn export_wav(engine: State<'_, tokio::sync::Mutex<Engine>>, request: ExportWavRequest) -> Result<(), String> {
    let engine = engine.lock().await;
    let wav_data = engine.export_wav(request.duration, request.sample_rate)?;
    std::fs::write(&request.path, wav_data).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn clear_graph(engine: State<'_, tokio::sync::Mutex<Engine>>) -> Result<(), String> {
    let mut engine = engine.lock().await;
    engine.clear();
    Ok(())
}

// --- Template Commands ---

use crate::nodes::custom::template_node::{TemplateDefinition, TemplateInfo, TemplateNode};
use crate::nodes::custom::script_node::{ScriptDefinition, ScriptNode, parse_port_type};

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateTemplateRequest {
    pub definition: TemplateDefinition,
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
        name: request.definition.name,
        category: request.definition.category,
        description: request.definition.description,
        inputs: request.definition.inputs,
        outputs: request.definition.outputs,
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

// --- Script Node Commands ---

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateScriptRequest {
    pub definition: ScriptDefinition,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ScriptInfo {
    pub name: String,
    pub category: String,
    pub description: String,
    pub inputs: Vec<PortInfo>,
    pub outputs: Vec<PortInfo>,
    pub params: Vec<String>,
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
        name: request.definition.name,
        category: request.definition.category,
        description: request.definition.description,
        inputs: request.definition.inputs.iter().map(|p| PortInfo { name: p.name.clone(), port_type: parse_port_type(&p.port_type) }).collect(),
        outputs: request.definition.outputs.iter().map(|p| PortInfo { name: p.name.clone(), port_type: parse_port_type(&p.port_type) }).collect(),
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
                    inputs: def.inputs.iter().map(|p| PortInfo { name: p.name.clone(), port_type: parse_port_type(&p.port_type) }).collect(),
                    outputs: def.outputs.iter().map(|p| PortInfo { name: p.name.clone(), port_type: parse_port_type(&p.port_type) }).collect(),
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
