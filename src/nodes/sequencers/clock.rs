use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct Clock {
    id: String,
    bpm: f64,
    samples_per_beat: usize,
    elapsed: usize,
    swing: f64,
}

impl Clock {
    pub fn new(id: String) -> Self {
        let bpm = 120.0;
        let samples_per_beat = (44100.0 / (bpm / 4.0)) as usize;
        Self { id, bpm, samples_per_beat, elapsed: 0, swing: 0.0 }
    }

    fn update_samples_per_beat(&mut self) {
        self.samples_per_beat = (44100.0 / (self.bpm / 4.0)) as usize;
    }
}

impl Node for Clock {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "Clock" }
    fn category(&self) -> &str { "Trigger" }

    fn inputs(&self) -> Vec<PortInfo> { vec![] }
    fn outputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "beat".to_string(), port_type: PortType::Trigger },
            PortInfo { name: "tick".to_string(), port_type: PortType::Trigger },
        ]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("bpm".to_string(), ParamValue::Float(self.bpm));
        params.insert("swing".to_string(), ParamValue::Float(self.swing));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "bpm" => {
                if let ParamValue::Float(f) = value {
                    self.bpm = f.max(20.0).min(300.0);
                    self.update_samples_per_beat();
                }
            }
            "swing" => {
                if let ParamValue::Float(f) = value {
                    self.swing = f.max(0.0).min(1.0);
                }
            }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "bpm" => Some(ParamValue::Float(self.bpm)),
            "swing" => Some(ParamValue::Float(self.swing)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, _input: Option<&[f32]>) -> AudioBuffer {
        let mut buffer = AudioBuffer::with_capacity(block_size);
        let swing = self.swing as f32;

        for _ in 0..block_size {
            let is_beat = self.elapsed == 0;
            let is_tick = self.elapsed == 0 || (self.elapsed == self.samples_per_beat / 2 && swing < 0.5);

            if is_beat {
                buffer.push(1.0f32);
            } else if is_tick {
                buffer.push(0.5f32);
            } else {
                buffer.push(0.0f32);
            }

            self.elapsed = if self.elapsed >= self.samples_per_beat {
                0
            } else {
                self.elapsed + 1
            };
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
