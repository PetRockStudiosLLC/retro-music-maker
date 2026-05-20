use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct BandpassFilter {
    id: String,
    cutoff: f64,
    q: f64,
    prev_in: f64,
    prev_out: f64,
}

impl BandpassFilter {
    pub fn new(id: String) -> Self {
        Self { id, cutoff: 1000.0, q: 1.0, prev_in: 0.0, prev_out: 0.0 }
    }
}

impl Node for BandpassFilter {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "BandpassFilter" }
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
        params.insert("q".to_string(), ParamValue::Float(self.q));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        let f = match value { ParamValue::Float(v) => v, ParamValue::Int(v) => v as f64, _ => return };
        match name {
            "cutoff" => { self.cutoff = f.max(20.0).min(22050.0); }
            "q" => { self.q = f.max(0.1).min(10.0); }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "cutoff" => Some(ParamValue::Float(self.cutoff)),
            "q" => Some(ParamValue::Float(self.q)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let sample_rate = 44100.0;
        let wc = 2.0 * std::f64::consts::PI * self.cutoff / sample_rate;
        let cos = wc.cos();
        let sin = wc.sin();
        let alpha = sin / (2.0 * self.q);
        let b1 = alpha;
        let a1 = -2.0 * cos;
        let a2 = 1.0 - alpha;
        let mut buffer = AudioBuffer::with_capacity(block_size);
        let mut x1 = self.prev_in;
        let mut y1 = self.prev_out;

        for i in 0..block_size {
            let x0 = input.and_then(|b| b.get(i)).copied().unwrap_or(0.0) as f64;
            let y0 = b1 * x0 + y1;
            buffer.push(y0 as f32);
            y1 = a1 * y0 + a2 * x1;
            x1 = x0;
        }

        self.prev_in = x1;
        self.prev_out = y1;
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
