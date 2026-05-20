use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

#[derive(Clone, Copy)]
enum EnvState { Attack, Decay, Sustain, Release, Off }

pub struct ADSREnvelope {
    id: String,
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
    state: EnvState,
    value: f64,
    triggered: bool,
}

impl ADSREnvelope {
    pub fn new(id: String) -> Self {
        Self { id, attack: 0.01, decay: 0.1, sustain: 0.5, release: 0.3, state: EnvState::Off, value: 0.0, triggered: false }
    }

    fn gate(&mut self) {
        self.triggered = true;
        self.state = EnvState::Attack;
        self.value = 0.0;
    }

    fn ungate(&mut self) {
        self.triggered = false;
        if matches!(self.state, EnvState::Sustain) {
            self.state = EnvState::Release;
        }
    }
}

impl Node for ADSREnvelope {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "ADSREnvelope" }
    fn category(&self) -> &str { "Envelope" }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "gate".to_string(), port_type: PortType::Trigger },
            PortInfo { name: "audio".to_string(), port_type: PortType::Audio },
        ]
    }

    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "envelope".to_string(), port_type: PortType::Control }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("attack".to_string(), ParamValue::Float(self.attack));
        params.insert("decay".to_string(), ParamValue::Float(self.decay));
        params.insert("sustain".to_string(), ParamValue::Float(self.sustain));
        params.insert("release".to_string(), ParamValue::Float(self.release));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "attack" => { if let ParamValue::Float(f) = value { self.attack = f.max(0.001).min(5.0); } }
            "decay" => { if let ParamValue::Float(f) = value { self.decay = f.max(0.001).min(5.0); } }
            "sustain" => { if let ParamValue::Float(f) = value { self.sustain = f.max(0.0).min(1.0); } }
            "release" => { if let ParamValue::Float(f) = value { self.release = f.max(0.001).min(5.0); } }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "attack" => Some(ParamValue::Float(self.attack)),
            "decay" => Some(ParamValue::Float(self.decay)),
            "sustain" => Some(ParamValue::Float(self.sustain)),
            "release" => Some(ParamValue::Float(self.release)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, _input: Option<&[f32]>) -> AudioBuffer {
        // Auto-trigger on first call so presets produce sound without explicit gate
        if matches!(self.state, EnvState::Off) && !self.triggered {
            self.gate();
        }
        let sample_rate = 44100.0;
        let mut buffer = AudioBuffer::with_capacity(block_size);

        for _ in 0..block_size {
            match self.state {
                EnvState::Attack => {
                    self.value += 1.0 / (self.attack * sample_rate);
                    if self.value >= 1.0 { self.value = 1.0; self.state = EnvState::Decay; }
                }
                EnvState::Decay => {
                    self.value -= 1.0 / (self.decay * sample_rate) * (1.0 - self.sustain);
                    if self.value <= self.sustain { self.value = self.sustain; self.state = EnvState::Sustain; }
                }
                EnvState::Sustain => {
                    self.value = self.sustain;
                    if !self.triggered { self.state = EnvState::Release; }
                }
                EnvState::Release => {
                    self.value -= 1.0 / (self.release * sample_rate);
                    if self.value <= 0.0 { self.value = 0.0; self.state = EnvState::Off; }
                }
                EnvState::Off => {
                    self.value = 0.0;
                }
            }
            buffer.push(self.value as f32);
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
