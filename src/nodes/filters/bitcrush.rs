use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct Bitcrush {
    id: String,
    bit_depth: i32,
    sample_rate: f64,
}

impl Bitcrush {
    pub fn new(id: String) -> Self {
        Self { id, bit_depth: 8, sample_rate: 22050.0 }
    }
}

impl Node for Bitcrush {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "Bitcrush" }
    fn category(&self) -> &str { "Filter" }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("bit_depth".to_string(), ParamValue::Int(self.bit_depth as i64));
        params.insert("sample_rate".to_string(), ParamValue::Float(self.sample_rate));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "bit_depth" => {
                let i = match value { ParamValue::Int(v) => v, ParamValue::Float(v) => v as i64, _ => return };
                self.bit_depth = i.max(1).min(16) as i32;
            }
            "sample_rate" => {
                let f = match value { ParamValue::Float(v) => v, ParamValue::Int(v) => v as f64, _ => return };
                self.sample_rate = f.max(100.0).min(44100.0);
            }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "bit_depth" => Some(ParamValue::Int(self.bit_depth as i64)),
            "sample_rate" => Some(ParamValue::Float(self.sample_rate)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let steps = 1 << self.bit_depth;
        let downsample = (44100.0 / self.sample_rate).max(1.0) as usize;
        let mut buffer = AudioBuffer::with_capacity(block_size);
        let mut last = 0.0f32;

        for i in 0..block_size {
            let inp = input.and_then(|b| b.get(i)).copied().unwrap_or(0.0);
            if i % downsample == 0 {
                last = (inp * steps as f32).round() / steps as f32;
            }
            buffer.push(last);
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
