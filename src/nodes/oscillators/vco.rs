use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

#[derive(Clone, Copy)]
enum Waveform { Sine, Square, Sawtooth, Triangle }

pub struct VCO {
    id: String,
    waveform: Waveform,
    amplitude: f64,
    phase: f64,
    current_freq: f64,
    default_note: f64,
}

impl VCO {
    pub fn new(id: String) -> Self {
        Self {
            id,
            waveform: Waveform::Sine,
            amplitude: 0.3,
            phase: 0.0,
            current_freq: 440.0,
            default_note: 69.0,
        }
    }

    fn midi_to_freq(note: f64) -> f64 {
        440.0 * 2.0_f64.powf((note - 69.0) / 12.0)
    }

    fn waveform_sample(&self, wf: Waveform, phase: f64) -> f64 {
        match wf {
            Waveform::Sine => (phase * std::f64::consts::PI * 2.0).sin(),
            Waveform::Square => if phase.fract() < 0.5 { 1.0 } else { -1.0 },
            Waveform::Sawtooth => 2.0 * (phase.fract() - 0.5),
            Waveform::Triangle => {
                let p = phase.fract() * 2.0;
                if p < 1.0 { 2.0 * p - 1.0 } else { 3.0 - 2.0 * p }
            }
        }
    }
}

impl Node for VCO {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "VCO" }
    fn category(&self) -> &str { "Oscillator" }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "note".to_string(), port_type: PortType::Control },
        ]
    }

    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("waveform".to_string(), ParamValue::String(match self.waveform {
            Waveform::Sine => "sine",
            Waveform::Square => "square",
            Waveform::Sawtooth => "sawtooth",
            Waveform::Triangle => "triangle",
        }.to_string()));
        params.insert("amplitude".to_string(), ParamValue::Float(self.amplitude));
        params.insert("note".to_string(), ParamValue::Float(self.default_note));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "waveform" => {
                if let ParamValue::String(s) = value {
                    self.waveform = match s.as_str() {
                        "square" => Waveform::Square,
                        "sawtooth" => Waveform::Sawtooth,
                        "triangle" => Waveform::Triangle,
                        _ => Waveform::Sine,
                    };
                }
            }
            "amplitude" => {
                if let ParamValue::Float(f) = value {
                    self.amplitude = f.max(0.0).min(1.0);
                }
            }
            "note" => {
                if let ParamValue::Float(f) = value {
                    self.default_note = f;
                    self.current_freq = Self::midi_to_freq(f);
                }
            }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "waveform" => Some(ParamValue::String(match self.waveform {
                Waveform::Sine => "sine",
                Waveform::Square => "square",
                Waveform::Sawtooth => "sawtooth",
                Waveform::Triangle => "triangle",
            }.to_string())),
            "amplitude" => Some(ParamValue::Float(self.amplitude)),
            "note" => Some(ParamValue::Float(self.default_note)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let sample_rate = 44100.0;
        let mut buffer = AudioBuffer::with_capacity(block_size);

        // Read control input for MIDI note number
        if let Some(input_buf) = input {
            if !input_buf.is_empty() {
                let midi_note = input_buf[0] as f64;
                let freq = if midi_note > 0.0 {
                    Self::midi_to_freq(midi_note)
                } else {
                    0.0
                };
                self.current_freq = freq;
            }
        } else {
            self.current_freq = Self::midi_to_freq(self.default_note);
        }

        for _ in 0..block_size {
            if self.current_freq > 0.0 {
                let sample = self.waveform_sample(self.waveform, self.phase);
                buffer.push((sample * self.amplitude) as f32);
                self.phase += self.current_freq / sample_rate;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
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
