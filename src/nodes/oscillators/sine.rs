use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct SineOscillator {
    id: String,
    frequency: f64,
    amplitude: f64,
    phase: f64,
}

impl SineOscillator {
    pub fn new(id: String) -> Self {
        Self {
            id,
            frequency: 440.0,
            amplitude: 0.3,
            phase: 0.0,
        }
    }
}

impl Node for SineOscillator {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "SineOscillator"
    }

    fn category(&self) -> &str {
        "Oscillator"
    }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "frequency".to_string(), port_type: PortType::Control },
            PortInfo { name: "amplitude".to_string(), port_type: PortType::Control },
        ]
    }

    fn outputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "audio".to_string(), port_type: PortType::Audio },
        ]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("frequency".to_string(), ParamValue::Float(self.frequency));
        params.insert("amplitude".to_string(), ParamValue::Float(self.amplitude));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "frequency" => {
                if let ParamValue::Float(f) = value {
                    self.frequency = f.max(20.0).min(20000.0);
                }
            }
            "amplitude" => {
                if let ParamValue::Float(f) = value {
                    self.amplitude = f.max(0.0).min(1.0);
                }
            }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "frequency" => Some(ParamValue::Float(self.frequency)),
            "amplitude" => Some(ParamValue::Float(self.amplitude)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, _input: Option<&[f32]>) -> AudioBuffer {
        let sample_rate = 44100.0;
        let mut buffer = AudioBuffer::with_capacity(block_size);

        for _ in 0..block_size {
            let sample = (self.phase * std::f64::consts::PI * 2.0).sin();
            buffer.push((sample * self.amplitude) as f32);
            self.phase += self.frequency / sample_rate;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }

        buffer
    }

    fn to_info(&self, position: [f64; 2]) -> NodeInfo {
        NodeInfo {
            id: self.id.clone(),
            name: self.name().to_string(),
            category: self.category().to_string(),
            inputs: self.inputs(),
            outputs: self.outputs(),
            params: self.default_params(),
            position,
        node_kind: crate::core::NodeKind::Builtin,
        definition: None,
        }
    }
}
