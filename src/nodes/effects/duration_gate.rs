use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct DurationGate {
    id: String,
    duration_samples: usize,
    elapsed: usize,
    active: bool,
    auto_restart: bool,
}

impl DurationGate {
    pub fn new(id: String) -> Self {
        Self {
            id,
            duration_samples: (44100.0 * 1.0) as usize,
            elapsed: 0,
            active: true,
            auto_restart: false,
        }
    }
}

impl Node for DurationGate {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "DurationGate" }
    fn category(&self) -> &str { "Utility" }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        let duration_sec = self.duration_samples as f64 / 44100.0;
        params.insert("duration".to_string(), ParamValue::Float(duration_sec));
        params.insert("auto_restart".to_string(), ParamValue::Bool(false));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "duration" => {
                if let ParamValue::Float(f) = value {
                    self.duration_samples = (f.max(0.01).min(60.0) * 44100.0) as usize;
                }
            }
            "auto_restart" => {
                if let ParamValue::Bool(b) = value {
                    self.auto_restart = b;
                }
            }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "duration" => Some(ParamValue::Float(self.duration_samples as f64 / 44100.0)),
            "auto_restart" => Some(ParamValue::Bool(self.auto_restart)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let mut buffer = AudioBuffer::with_capacity(block_size);

        if self.elapsed >= self.duration_samples {
            self.elapsed = 0;
            self.active = self.auto_restart;
        }

        for s in 0..block_size {
            if self.active && self.elapsed < self.duration_samples {
                let sample = input.map(|buf| buf.get(s).copied().unwrap_or(0.0)).unwrap_or(0.0);
                buffer.push(sample);
                self.elapsed += 1;
            } else {
                buffer.push(0.0);
            }
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
