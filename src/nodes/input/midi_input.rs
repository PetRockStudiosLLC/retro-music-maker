use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct MidiInput {
    id: String,
    port: String,
}

impl MidiInput {
    pub fn new(id: String) -> Self {
        Self { id, port: "default".to_string() }
    }
}

impl Node for MidiInput {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "MidiInput" }
    fn category(&self) -> &str { "Input" }

    fn inputs(&self) -> Vec<PortInfo> { vec![] }
    fn outputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "note_on".to_string(), port_type: PortType::Trigger },
            PortInfo { name: "note_off".to_string(), port_type: PortType::Trigger },
            PortInfo { name: "pitch".to_string(), port_type: PortType::Control },
            PortInfo { name: "velocity".to_string(), port_type: PortType::Control },
        ]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("port".to_string(), ParamValue::String(self.port.clone()));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        if name == "port" { if let ParamValue::String(s) = value { self.port = s; } }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        if name == "port" { Some(ParamValue::String(self.port.clone())) } else { None }
    }

    fn process(&mut self, block_size: BlockSize, _input: Option<&[f32]>) -> AudioBuffer {
        vec![0.0f32; block_size]
    }

    fn to_info(&self, position: [f64; 2]) -> NodeInfo {
        NodeInfo {
            id: self.id.clone(), name: self.name().to_string(), category: self.category().to_string(),
            inputs: self.inputs(), outputs: self.outputs(), params: self.default_params(), position,
        node_kind: crate::core::NodeKind::Builtin,
        definition: None,
        }
    }
}
