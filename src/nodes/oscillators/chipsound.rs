use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

#[derive(Clone, Copy, PartialEq, Debug)]
enum ChipWave { Square, Triangle, Noise, DPCM }

pub struct ChipSoundOscillator {
    id: String,
    frequency: f64,
    amplitude: f64,
    wave: ChipWave,
    phase: f64,
    duty_cycle: f64,
}

impl ChipSoundOscillator {
    pub fn new(id: String) -> Self {
        Self { id, frequency: 440.0, amplitude: 0.3, wave: ChipWave::Square, phase: 0.0, duty_cycle: 0.5 }
    }
}

impl Node for ChipSoundOscillator {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "ChipSoundOscillator" }
    fn category(&self) -> &str { "Oscillator" }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "frequency".to_string(), port_type: PortType::Control },
            PortInfo { name: "amplitude".to_string(), port_type: PortType::Control },
            PortInfo { name: "duty_cycle".to_string(), port_type: PortType::Control },
        ]
    }

    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("frequency".to_string(), ParamValue::Float(self.frequency));
        params.insert("amplitude".to_string(), ParamValue::Float(self.amplitude));
        params.insert("duty_cycle".to_string(), ParamValue::Float(self.duty_cycle));
        params.insert("wave".to_string(), ParamValue::String(format!("{:?}", self.wave)));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "frequency" => { if let ParamValue::Float(f) = value { self.frequency = f.max(20.0).min(20000.0); } }
            "amplitude" => { if let ParamValue::Float(f) = value { self.amplitude = f.max(0.0).min(1.0); } }
            "duty_cycle" => { if let ParamValue::Float(f) = value { self.duty_cycle = f.max(0.0).min(1.0); } }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "frequency" => Some(ParamValue::Float(self.frequency)),
            "amplitude" => Some(ParamValue::Float(self.amplitude)),
            "duty_cycle" => Some(ParamValue::Float(self.duty_cycle)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, _input: Option<&[f32]>) -> AudioBuffer {
        let sample_rate = 44100.0;
        let mut buffer = AudioBuffer::with_capacity(block_size);

        for _ in 0..block_size {
            let sample = match self.wave {
                ChipWave::Square => {
                    if self.phase < self.duty_cycle { 1.0 } else { -1.0 }
                }
                ChipWave::Triangle => {
                    4.0 * (self.phase - (self.phase as i32) as f64 - 0.5).abs() - 1.0
                }
                ChipWave::Noise => {
                    (rand::random::<f64>() * 2.0) - 1.0
                }
                ChipWave::DPCM => {
                    let delta = (rand::random::<f64>() * 2.0) - 1.0;
                    self.phase = (self.phase + delta * 0.1).clamp(-1.0, 1.0);
                    self.phase
                }
            };
            buffer.push((sample * self.amplitude) as f32);
            self.phase += self.frequency / sample_rate;
            if self.phase >= 1.0 { self.phase -= 1.0; }
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
