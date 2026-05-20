use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct Instrument {
    id: String,
    display_name: String,
    waveform: String,
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
    amplitude: f64,
    triggered: bool,
}

impl Instrument {
    pub fn new(id: String) -> Self {
        Self {
            id,
            display_name: String::new(),
            waveform: "square".to_string(),
            attack: 0.002,
            decay: 0.05,
            sustain: 0.3,
            release: 0.05,
            amplitude: 0.5,
            triggered: false,
        }
    }

    fn instrument_to_json(&self) -> String {
        serde_json::json!({
            "waveform": self.waveform,
            "attack": self.attack,
            "decay": self.decay,
            "sustain": self.sustain,
            "release": self.release,
            "amplitude": self.amplitude,
        }).to_string()
    }
}

impl Node for Instrument {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "Instrument" }
    fn category(&self) -> &str { "Controller" }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "trigger".to_string(), port_type: PortType::Control },
            PortInfo { name: "trigger_in".to_string(), port_type: PortType::Trigger },
        ]
    }
    fn outputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "instrument".to_string(), port_type: PortType::Instrument },
            PortInfo { name: "trigger".to_string(), port_type: PortType::Trigger },
        ]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("display_name".to_string(), ParamValue::String(self.display_name.clone()));
        params.insert("waveform".to_string(), ParamValue::String(self.waveform.clone()));
        params.insert("attack".to_string(), ParamValue::Float(self.attack));
        params.insert("decay".to_string(), ParamValue::Float(self.decay));
        params.insert("sustain".to_string(), ParamValue::Float(self.sustain));
        params.insert("release".to_string(), ParamValue::Float(self.release));
        params.insert("amplitude".to_string(), ParamValue::Float(self.amplitude));
        params.insert("instrument_json".to_string(), ParamValue::String(self.instrument_to_json()));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "display_name" => {
                if let ParamValue::String(s) = value { self.display_name = s; }
            }
            "waveform" => {
                if let ParamValue::String(s) = value { self.waveform = s; }
            }
            "attack" => {
                let f = match value { ParamValue::Float(f) => f, ParamValue::Int(i) => i as f64, _ => return };
                self.attack = f.max(0.001).min(2.0);
            }
            "decay" => {
                let f = match value { ParamValue::Float(f) => f, ParamValue::Int(i) => i as f64, _ => return };
                self.decay = f.max(0.001).min(2.0);
            }
            "sustain" => {
                let f = match value { ParamValue::Float(f) => f, ParamValue::Int(i) => i as f64, _ => return };
                self.sustain = f.max(0.0).min(1.0);
            }
            "release" => {
                let f = match value { ParamValue::Float(f) => f, ParamValue::Int(i) => i as f64, _ => return };
                self.release = f.max(0.001).min(2.0);
            }
            "amplitude" => {
                let f = match value { ParamValue::Float(f) => f, ParamValue::Int(i) => i as f64, _ => return };
                self.amplitude = f.max(0.0).min(1.0);
            }
            "instrument_json" => {
                if let ParamValue::String(s) = value {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&s) {
                        if let Some(w) = data.get("waveform").and_then(|v| v.as_str()) {
                            self.waveform = w.to_string();
                        }
                        if let Some(a) = data.get("attack").and_then(|v| v.as_f64()) {
                            self.attack = a.max(0.001).min(2.0);
                        }
                        if let Some(d) = data.get("decay").and_then(|v| v.as_f64()) {
                            self.decay = d.max(0.001).min(2.0);
                        }
                        if let Some(su) = data.get("sustain").and_then(|v| v.as_f64()) {
                            self.sustain = su.max(0.0).min(1.0);
                        }
                        if let Some(r) = data.get("release").and_then(|v| v.as_f64()) {
                            self.release = r.max(0.001).min(2.0);
                        }
                        if let Some(amp) = data.get("amplitude").and_then(|v| v.as_f64()) {
                            self.amplitude = amp.max(0.0).min(1.0);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "display_name" => Some(ParamValue::String(self.display_name.clone())),
            "waveform" => Some(ParamValue::String(self.waveform.clone())),
            "attack" => Some(ParamValue::Float(self.attack)),
            "decay" => Some(ParamValue::Float(self.decay)),
            "sustain" => Some(ParamValue::Float(self.sustain)),
            "release" => Some(ParamValue::Float(self.release)),
            "amplitude" => Some(ParamValue::Float(self.amplitude)),
            "instrument_json" => Some(ParamValue::String(self.instrument_to_json())),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let mut buffer = AudioBuffer::with_capacity(block_size);
        for i in 0..block_size {
            let val = input.and_then(|b| b.get(i)).copied().unwrap_or(0.0);
            buffer.push(val);
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
