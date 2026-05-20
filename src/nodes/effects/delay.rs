use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct DelayEffect {
    id: String,
    time: f64,
    feedback: f64,
    mix: f64,
    buffer: Vec<f32>,
    write_pos: usize,
}

impl DelayEffect {
    pub fn new(id: String) -> Self {
        Self { id, time: 0.3, feedback: 0.4, mix: 0.3, buffer: vec![0.0; 44100 * 2], write_pos: 0 }
    }
}

impl Node for DelayEffect {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "DelayEffect" }
    fn category(&self) -> &str { "Effect" }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("time".to_string(), ParamValue::Float(self.time));
        params.insert("feedback".to_string(), ParamValue::Float(self.feedback));
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
            "time" => { self.time = f.max(0.0).min(2.0); }
            "feedback" => { self.feedback = f.max(0.0).min(0.95); }
            "mix" => { self.mix = f.max(0.0).min(1.0); }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "time" => Some(ParamValue::Float(self.time)),
            "feedback" => Some(ParamValue::Float(self.feedback)),
            "mix" => Some(ParamValue::Float(self.mix)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let delay_samples = (self.time * 44100.0) as usize;
        let feedback = self.feedback as f32;
        let mix = self.mix as f32;
        let mut buffer = AudioBuffer::with_capacity(block_size);

        for i in 0..block_size {
            let inp = input.and_then(|b| b.get(i)).copied().unwrap_or(0.0);
            let read_pos = (self.write_pos + self.buffer.len() - delay_samples) % self.buffer.len();
            let delayed = self.buffer[read_pos];
            self.buffer[self.write_pos] = inp + delayed * feedback;
            self.write_pos = (self.write_pos + 1) % self.buffer.len();
            let output = inp * (1.0 - mix) + delayed * mix;
            buffer.push(output);
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
