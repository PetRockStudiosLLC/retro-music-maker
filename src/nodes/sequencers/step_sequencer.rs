use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct StepSequencer {
    id: String,
    steps: Vec<bool>,
    bpm: f64,
    current_step: usize,
}

impl StepSequencer {
    pub fn new(id: String) -> Self {
        Self { id, steps: vec![false; 16], bpm: 120.0, current_step: 0 }
    }
}

impl Node for StepSequencer {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "StepSequencer" }
    fn category(&self) -> &str { "Sequencer" }

    fn inputs(&self) -> Vec<PortInfo> { vec![] }
    fn outputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "trigger".to_string(), port_type: PortType::Trigger },
            PortInfo { name: "step".to_string(), port_type: PortType::Control },
        ]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("bpm".to_string(), ParamValue::Float(self.bpm));
        params.insert("steps".to_string(), ParamValue::String(format!("{:?}", self.steps)));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        if name == "bpm" { if let ParamValue::Float(f) = value { self.bpm = f.max(20.0).min(300.0); } }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        if name == "bpm" { Some(ParamValue::Float(self.bpm)) } else { None }
    }

    fn process(&mut self, block_size: BlockSize, _input: Option<&[f32]>) -> AudioBuffer {
        let mut buffer = AudioBuffer::with_capacity(block_size);
        for _ in 0..block_size {
            if self.steps[self.current_step] {
                buffer.push(1.0f32);
            } else {
                buffer.push(0.0f32);
            }
        }
        self.current_step = (self.current_step + 1) % self.steps.len();
        buffer
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
