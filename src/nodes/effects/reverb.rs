use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct Reverb {
    id: String,
    decay: f64,
    mix: f64,
    buffer: Vec<f32>,
    pos: usize,
}

impl Reverb {
    pub fn new(id: String) -> Self {
        Self { id, decay: 0.5, mix: 0.2, buffer: vec![0.0; 44100], pos: 0 }
    }
}

impl Node for Reverb {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "Reverb" }
    fn category(&self) -> &str { "Effect" }

    fn inputs(&self) -> Vec<PortInfo> { vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }] }
    fn outputs(&self) -> Vec<PortInfo> { vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }] }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("decay".to_string(), ParamValue::Float(self.decay));
        params.insert("mix".to_string(), ParamValue::Float(self.mix));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        let f = match value {
            ParamValue::Float(v) => v,
            ParamValue::Int(v) => v as f64,
            _ => return,
        };
        match name {
            "decay" => { self.decay = f.max(0.0).min(0.99); }
            "mix" => { self.mix = f.max(0.0).min(1.0); }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "decay" => Some(ParamValue::Float(self.decay)),
            "mix" => Some(ParamValue::Float(self.mix)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let decay = self.decay as f32;
        let mix = self.mix as f32;
        let mut buffer = AudioBuffer::with_capacity(block_size);
        for i in 0..block_size {
            let inp = input.and_then(|b| b.get(i)).copied().unwrap_or(0.0);
            let wet = self.buffer[self.pos];
            self.buffer[self.pos] = wet * decay + inp;
            self.pos = (self.pos + 1) % self.buffer.len();
            buffer.push(inp * (1.0 - mix) + wet * mix);
        }
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
