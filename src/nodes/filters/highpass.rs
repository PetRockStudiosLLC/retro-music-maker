use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct HighpassFilter {
    id: String,
    cutoff: f64,
    previous: f64,
}

impl HighpassFilter {
    pub fn new(id: String) -> Self {
        Self { id, cutoff: 500.0, previous: 0.0 }
    }
}

impl Node for HighpassFilter {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "HighpassFilter" }
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
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        if name == "cutoff" {
            let f = match value { ParamValue::Float(v) => v, ParamValue::Int(v) => v as f64, _ => return };
            self.cutoff = f.max(20.0).min(22050.0);
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        if name == "cutoff" { Some(ParamValue::Float(self.cutoff)) } else { None }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let sample_rate = 44100.0;
        let rc = std::f64::consts::PI * self.cutoff / sample_rate;
        let alpha = rc / (rc + 1.0);
        let mut buffer = AudioBuffer::with_capacity(block_size);
        let mut prev = self.previous;

        for i in 0..block_size {
            let inp = input.and_then(|b| b.get(i)).copied().unwrap_or(0.0) as f64;
            let output = prev + alpha * (inp - prev);
            buffer.push((inp - output) as f32);
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
