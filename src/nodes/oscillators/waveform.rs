use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

#[derive(Clone, Copy, PartialEq)]
enum Waveform { Sine, Square, Sawtooth, Triangle, Pulse, PulseHalf, Noise, SampleHold }

pub struct WaveformOsc {
    id: String,
    waveform: Waveform,
    amplitude: f64,
    phase: f64,
    current_freq: f64,
    default_note: f64,
    sh_value: f64,
    sh_timer: u64,
}

impl WaveformOsc {
    pub fn new(id: String) -> Self {
        Self {
            id,
            waveform: Waveform::Sine,
            amplitude: 0.3,
            phase: 0.0,
            current_freq: 440.0,
            default_note: 69.0,
            sh_value: 0.0,
            sh_timer: 0,
        }
    }

    fn midi_to_freq(note: f64) -> f64 {
        440.0 * 2.0_f64.powf((note - 69.0) / 12.0)
    }

    fn waveform_sample(&mut self, wf: Waveform, phase: f64) -> f64 {
        match wf {
            Waveform::Sine => (phase * std::f64::consts::PI * 2.0).sin(),
            Waveform::Square => if phase.fract() < 0.5 { 1.0 } else { -1.0 },
            Waveform::Sawtooth => 2.0 * (phase.fract() - 0.5),
            Waveform::Triangle => {
                let p = phase.fract() * 2.0;
                if p < 1.0 { 2.0 * p - 1.0 } else { 3.0 - 2.0 * p }
            }
            Waveform::Pulse => if phase.fract() < 0.25 { 1.0 } else { -1.0 },
            Waveform::PulseHalf => {
                let p = phase.fract();
                if p < 0.5 { 1.0 } else { -1.0 }
            }
            Waveform::Noise => {
                self.sh_timer += 1;
                let seed = (self.sh_timer.wrapping_mul(1103515245).wrapping_add(12345) & 0x7FFF) as f64;
                (seed / 32767.0) * 2.0 - 1.0
            }
            Waveform::SampleHold => {
                self.sh_timer += 1;
                if self.sh_timer % 88 == 0 {
                    let seed = (self.sh_timer.wrapping_mul(1103515245).wrapping_add(12345) & 0x7FFF) as f64;
                    self.sh_value = (seed / 32767.0) * 2.0 - 1.0;
                }
                self.sh_value
            }
        }
    }

    fn wf_name(wf: Waveform) -> &'static str {
        match wf {
            Waveform::Sine => "sine",
            Waveform::Square => "square",
            Waveform::Sawtooth => "sawtooth",
            Waveform::Triangle => "triangle",
            Waveform::Pulse => "pulse",
            Waveform::PulseHalf => "pulse_half",
            Waveform::Noise => "noise",
            Waveform::SampleHold => "sample_hold",
        }
    }

    fn parse_wf(s: &str) -> Waveform {
        match s {
            "square" => Waveform::Square,
            "sawtooth" => Waveform::Sawtooth,
            "triangle" => Waveform::Triangle,
            "pulse" => Waveform::Pulse,
            "pulse_half" => Waveform::PulseHalf,
            "noise" => Waveform::Noise,
            "sample_hold" => Waveform::SampleHold,
            _ => Waveform::Sine,
        }
    }
}

impl Node for WaveformOsc {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "Waveform" }
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
        params.insert("waveform".to_string(), ParamValue::String(Self::wf_name(self.waveform).to_string()));
        params.insert("amplitude".to_string(), ParamValue::Float(self.amplitude));
        params.insert("note".to_string(), ParamValue::Float(self.default_note));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "waveform" => {
                if let ParamValue::String(s) = value {
                    self.waveform = Self::parse_wf(s.as_str());
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
            "waveform" => Some(ParamValue::String(Self::wf_name(self.waveform).to_string())),
            "amplitude" => Some(ParamValue::Float(self.amplitude)),
            "note" => Some(ParamValue::Float(self.default_note)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let sample_rate = 44100.0;
        let mut buffer = AudioBuffer::with_capacity(block_size);

        if let Some(input_buf) = input {
            if !input_buf.is_empty() {
                let midi_note = input_buf[0] as f64;
                self.current_freq = if midi_note > 0.0 {
                    Self::midi_to_freq(midi_note)
                } else {
                    0.0
                };
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
