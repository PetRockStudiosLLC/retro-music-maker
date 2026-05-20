use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct AudioOutput {
    id: String,
    volume: f64,
    pan: f64,
    duration: f64,
    loop_enabled: bool,
    elapsed: f64,
    sample_rate: f64,
}

impl AudioOutput {
    pub fn new(id: String) -> Self {
        Self { id, volume: 1.0, pan: 0.0, duration: 0.0, loop_enabled: false, elapsed: 0.0, sample_rate: 48000.0 }
    }

    fn pan_l(&self) -> f32 { (1.0 - self.pan).max(0.0).min(2.0) as f32 * 0.7071 }
    fn pan_r(&self) -> f32 { (1.0 + self.pan).max(0.0).min(2.0) as f32 * 0.7071 }
}

impl Node for AudioOutput {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "AudioOutput" }
    fn category(&self) -> &str { "Output" }

    fn inputs(&self) -> Vec<PortInfo> { vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }] }
    fn outputs(&self) -> Vec<PortInfo> { vec![] }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("volume".to_string(), ParamValue::Float(self.volume));
        params.insert("pan".to_string(), ParamValue::Float(self.pan));
        params.insert("duration".to_string(), ParamValue::Float(self.duration));
        params.insert("loop_enabled".to_string(), ParamValue::Bool(self.loop_enabled));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "loop_enabled" => {
                if let ParamValue::Bool(b) = value { self.loop_enabled = b; }
            }
            _ => {
                let f = match value { ParamValue::Float(v) => v, ParamValue::Int(v) => v as f64, _ => return };
                match name {
                    "volume" => self.volume = f.max(0.0).min(2.0),
                    "pan" => self.pan = f.max(-1.0).min(1.0),
                    "duration" => self.duration = f.max(0.0),
                    _ => {}
                }
            }
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "volume" => Some(ParamValue::Float(self.volume)),
            "pan" => Some(ParamValue::Float(self.pan)),
            "duration" => Some(ParamValue::Float(self.duration)),
            "loop_enabled" => Some(ParamValue::Bool(self.loop_enabled)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        self.elapsed += block_size as f64 / self.sample_rate;
        if self.duration > 0.0 && self.elapsed >= self.duration {
            if self.loop_enabled {
                self.elapsed = 0.0;
            } else {
                return vec![0.0f32; block_size];
            }
        }
        match input {
            Some(buf) => buf.iter().take(block_size).copied().map(|s| {
                s * self.volume as f32
            }).collect(),
            None => vec![0.0f32; block_size],
        }
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
