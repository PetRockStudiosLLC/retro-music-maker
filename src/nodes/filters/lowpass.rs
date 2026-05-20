use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct LowpassFilter {
    id: String,
    cutoff: f64,
    resonance: f64,
    previous: f64,
}

impl LowpassFilter {
    pub fn new(id: String) -> Self {
        Self { id, cutoff: 2000.0, resonance: 1.0, previous: 0.0 }
    }
}

impl Node for LowpassFilter {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "LowpassFilter" }
    fn category(&self) -> &str { "Filter" }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "audio".to_string(), port_type: PortType::Audio },
            PortInfo { name: "cutoff".to_string(), port_type: PortType::Control },
        ]
    }

    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("cutoff".to_string(), ParamValue::Float(self.cutoff));
        params.insert("resonance".to_string(), ParamValue::Float(self.resonance));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        let f = match value { ParamValue::Float(v) => v, ParamValue::Int(v) => v as f64, _ => return };
        match name {
            "cutoff" => { self.cutoff = f.max(20.0).min(22050.0); }
            "resonance" => { self.resonance = f.max(0.1).min(10.0); }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "cutoff" => Some(ParamValue::Float(self.cutoff)),
            "resonance" => Some(ParamValue::Float(self.resonance)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let sample_rate = 44100.0;
        let rc = std::f64::consts::PI * self.cutoff * 2.0 / sample_rate;
        let alpha = rc / (rc + 1.0 + self.resonance * rc);
        let mut buffer = AudioBuffer::with_capacity(block_size);
        let mut prev = self.previous;

        for i in 0..block_size {
            let inp = input.and_then(|b| b.get(i)).copied().unwrap_or(0.0) as f64;
            let output = prev + alpha * (inp - prev);
            buffer.push(output as f32);
            prev = output;
        }

        self.previous = prev;
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
