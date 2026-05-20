use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::core::{AudioBuffer, BlockSize, ParamValue};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeKind {
    Builtin,
    Template,
    Script,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    pub category: String,
    pub inputs: Vec<crate::core::PortInfo>,
    pub outputs: Vec<crate::core::PortInfo>,
    pub params: HashMap<String, ParamValue>,
    pub position: [f64; 2],
    #[serde(default = "default_node_kind")]
    pub node_kind: NodeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<serde_json::Value>,
}

impl NodeInfo {
    /// Build NodeInfo with builtin defaults for node_kind and definition.
    /// Use struct update syntax: NodeInfo::builtin { id, name, .. }
    pub fn builtin(id: String, name: String, category: String, inputs: Vec<crate::core::PortInfo>, outputs: Vec<crate::core::PortInfo>, params: HashMap<String, ParamValue>, position: [f64; 2]) -> Self {
        Self {
            id, name, category, inputs, outputs, params, position,
            node_kind: NodeKind::Builtin,
            definition: None,
        }
    }
}

fn default_node_kind() -> NodeKind {
    NodeKind::Builtin
}

pub type NodeParams = HashMap<String, ParamValue>;

pub trait Node: Send {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn category(&self) -> &str;
    fn inputs(&self) -> Vec<crate::core::PortInfo>;
    fn outputs(&self) -> Vec<crate::core::PortInfo>;
    fn default_params(&self) -> NodeParams;
    fn set_param(&mut self, name: &str, value: ParamValue);
    fn get_param(&self, name: &str) -> Option<ParamValue>;
    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer;
    /// Process with per-input buffers. Default implementation mixes inputs like process().
    fn process_multi(&mut self, block_size: BlockSize, inputs: &[Option<AudioBuffer>]) -> AudioBuffer {
        let has_input = inputs.iter().any(|b| b.is_some());
        if !has_input {
            return self.process(block_size, None);
        }
        let mut mixed = AudioBuffer::with_capacity(block_size);
        for buf in inputs {
            if let Some(ref b) = buf {
                for i in 0..block_size {
                    let val = b.get(i).copied().unwrap_or(0.0);
                    if i < mixed.len() { mixed[i] += val; }
                    else { mixed.push(val); }
                }
            }
        }
        self.process(block_size, Some(&mixed))
    }
    fn to_info(&self, position: [f64; 2]) -> NodeInfo;
    fn kind(&self) -> NodeKind {
        NodeKind::Builtin
    }
    /// Returns accumulated samples for recording nodes (FileOutput), None otherwise.
    fn get_recording(&self) -> Option<Vec<f32>> { None }
    fn get_recording_mut(&mut self) -> Option<&mut Vec<f32>> { None }
}

pub struct SignalRegistry {
    signals: HashMap<String, AudioBuffer>,
}

impl SignalRegistry {
    pub fn new() -> Self {
        Self {
            signals: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, buffer: AudioBuffer) {
        self.signals.insert(key, buffer);
    }

    pub fn get(&self, key: &str) -> Option<&AudioBuffer> {
        self.signals.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut AudioBuffer> {
        self.signals.get_mut(key)
    }

    pub fn clear(&mut self) {
        self.signals.clear();
    }
}
