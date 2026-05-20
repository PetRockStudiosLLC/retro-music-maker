use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct KeyboardInput {
    id: String,
    active_notes: Vec<u8>,
}

impl KeyboardInput {
    pub fn new(id: String) -> Self { Self { id, active_notes: Vec::new() } }
}

impl Node for KeyboardInput {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "KeyboardInput" }
    fn category(&self) -> &str { "Input" }

    fn inputs(&self) -> Vec<PortInfo> { vec![] }
    fn outputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "note_on".to_string(), port_type: PortType::Trigger },
            PortInfo { name: "note_off".to_string(), port_type: PortType::Trigger },
            PortInfo { name: "velocity".to_string(), port_type: PortType::Control },
        ]
    }

    fn default_params(&self) -> NodeParams { HashMap::new() }
    fn set_param(&mut self, _name: &str, _value: ParamValue) {}
    fn get_param(&self, _name: &str) -> Option<ParamValue> { None }

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
