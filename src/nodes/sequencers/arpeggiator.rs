use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct Arpeggiator {
    id: String,
    bpm: f64,
    pattern: Vec<u8>,
    current: usize,
}

impl Arpeggiator {
    pub fn new(id: String) -> Self {
        Self { id, bpm: 120.0, pattern: vec![0, 4, 7, 12], current: 0 }
    }
}

impl Node for Arpeggiator {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "Arpeggiator" }
    fn category(&self) -> &str { "Sequencer" }

    fn inputs(&self) -> Vec<PortInfo> { vec![PortInfo { name: "trigger".to_string(), port_type: PortType::Trigger }] }
    fn outputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "note".to_string(), port_type: PortType::Control },
            PortInfo { name: "trigger".to_string(), port_type: PortType::Trigger },
        ]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("bpm".to_string(), ParamValue::Float(self.bpm));
        params.insert("pattern".to_string(), ParamValue::String(format!("{:?}", self.pattern)));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        if name == "bpm" { if let ParamValue::Float(f) = value { self.bpm = f.max(20.0).min(300.0); } }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        if name == "bpm" { Some(ParamValue::Float(self.bpm)) } else { None }
    }

    fn process(&mut self, block_size: BlockSize, _input: Option<&[f32]>) -> AudioBuffer {
        let note = self.pattern[self.current] as f32;
        self.current = (self.current + 1) % self.pattern.len();
        vec![note; block_size]
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
