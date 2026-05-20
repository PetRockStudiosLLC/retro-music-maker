use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

#[derive(Clone, Copy, PartialEq)]
enum Waveform { Square, Sawtooth, Triangle, Sine }

#[derive(Clone, Copy, PartialEq)]
enum EnvPhase { Attack, Decay, Sustain, Release, Off }

pub struct NoteSequencer {
    id: String,
    notes: Vec<u8>,
    bpm: f64,
    /// Fractional sample accumulator for precise step timing
    step_accumulator: f64,
    /// Samples per 16th note step
    samples_per_step: f64,
    current_step: usize,
    waveform: Waveform,
    env_phase: EnvPhase,
    env_value: f32,
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
    current_freq: f32,
    osc_phase: f32,
    amplitude: f32,
    sample_rate: f32,
}

impl NoteSequencer {
    pub fn new(id: String) -> Self {
        let sample_rate = 44100.0_f64;
        let bpm = 140.0_f64;
        Self {
            id,
            notes: vec![60, 64, 67, 72, 67, 64, 60, 0],
            bpm,
            samples_per_step: sample_rate / (bpm / 60.0 * 4.0),
            step_accumulator: 0.0,
            current_step: 0,
            waveform: Waveform::Square,
            env_phase: EnvPhase::Off,
            env_value: 0.0,
            attack: 0.002,
            decay: 0.05,
            sustain: 0.3,
            release: 0.05,
            current_freq: 0.0,
            osc_phase: 0.0,
            amplitude: 0.5,
            sample_rate: 44100.0,
        }
    }

    fn midi_to_freq(note: u8) -> f32 {
        if note == 0 { return 0.0; }
        440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
    }

    fn advance_step(&mut self) {
        self.current_step = (self.current_step + 1) % self.notes.len();
        let note = self.notes[self.current_step];
        let new_freq = Self::midi_to_freq(note);

        if new_freq == 0.0 {
            if self.env_phase != EnvPhase::Off {
                self.env_phase = EnvPhase::Release;
            }
            self.current_freq = 0.0;
        } else {
            self.current_freq = new_freq;
            self.env_phase = EnvPhase::Attack;
            self.env_value = 0.0;
            // Don't reset osc_phase - avoids phase discontinuity clicks
        }
    }

    fn update_envelope(&mut self) {
        let sr = self.sample_rate as f64;
        match self.env_phase {
            EnvPhase::Attack => {
                self.env_value += (1.0 / (self.attack * sr)) as f32;
                if self.env_value >= 1.0 {
                    self.env_value = 1.0;
                    self.env_phase = EnvPhase::Decay;
                }
            }
            EnvPhase::Decay => {
                self.env_value -= ((1.0 - self.sustain) / (self.decay * sr)) as f32;
                if self.env_value <= self.sustain as f32 {
                    self.env_value = self.sustain as f32;
                    self.env_phase = EnvPhase::Sustain;
                }
            }
            EnvPhase::Sustain => {
                self.env_value = self.sustain as f32;
            }
            EnvPhase::Release => {
                self.env_value -= (1.0 / (self.release * sr)) as f32;
                if self.env_value <= 0.0 {
                    self.env_value = 0.0;
                    self.env_phase = EnvPhase::Off;
                }
            }
            EnvPhase::Off => {
                self.env_value = 0.0;
            }
        }
    }

    fn generate_sample(&mut self) -> f32 {
        if self.current_freq == 0.0 || self.env_value == 0.0 {
            return 0.0;
        }

        let phase_inc = self.current_freq / self.sample_rate;
        let sample = match self.waveform {
            Waveform::Square => {
                if self.osc_phase < 0.5 { 1.0 } else { -1.0 }
            }
            Waveform::Sawtooth => {
                2.0 * (self.osc_phase - 0.5)
            }
            Waveform::Triangle => {
                let p = (self.osc_phase * 4.0).rem_euclid(2.0) - 1.0;
                if p < 0.0 { -p * 2.0 + 1.0 } else { -(p * 2.0 - 1.0) + 1.0 }
            }
            Waveform::Sine => {
                (self.osc_phase * std::f32::consts::PI * 2.0).sin()
            }
        };

        self.osc_phase = (self.osc_phase + phase_inc).rem_euclid(1.0);
        sample * self.env_value * self.amplitude
    }

    fn parse_notes(s: &str) -> Vec<u8> {
        s.split(',')
            .filter_map(|p| p.trim().parse::<u8>().ok())
            .filter(|n| *n <= 127)
            .collect()
    }
}

impl Node for NoteSequencer {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "NoteSequencer" }
    fn category(&self) -> &str { "Sequencer" }

    fn inputs(&self) -> Vec<PortInfo> { vec![] }
    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("bpm".to_string(), ParamValue::Float(self.bpm));
        params.insert("notes".to_string(), ParamValue::String(format!("{:?}", self.notes)));
        params.insert("waveform".to_string(), ParamValue::String(match self.waveform {
            Waveform::Square => "square",
            Waveform::Sawtooth => "sawtooth",
            Waveform::Triangle => "triangle",
            Waveform::Sine => "sine",
        }.to_string()));
        params.insert("attack".to_string(), ParamValue::Float(self.attack));
        params.insert("decay".to_string(), ParamValue::Float(self.decay));
        params.insert("sustain".to_string(), ParamValue::Float(self.sustain));
        params.insert("release".to_string(), ParamValue::Float(self.release));
        params.insert("amplitude".to_string(), ParamValue::Float(self.amplitude as f64));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "bpm" => {
                if let ParamValue::Float(f) = value {
                    self.bpm = f.max(20.0).min(300.0);
                    self.samples_per_step = self.sample_rate as f64 / (self.bpm / 60.0 * 4.0);
                }
            }
            "notes" => {
                if let ParamValue::String(s) = value {
                    let parsed = Self::parse_notes(&s);
                    if !parsed.is_empty() {
                        self.notes = parsed;
                        self.current_step = 0;
                    }
                }
            }
            "waveform" => {
                if let ParamValue::String(s) = value {
                    self.waveform = match s.as_str() {
                        "sawtooth" => Waveform::Sawtooth,
                        "triangle" => Waveform::Triangle,
                        "sine" => Waveform::Sine,
                        _ => Waveform::Square,
                    };
                }
            }
            "attack" => { if let ParamValue::Float(f) = value { self.attack = f.max(0.001).min(2.0); } }
            "decay" => { if let ParamValue::Float(f) = value { self.decay = f.max(0.001).min(2.0); } }
            "sustain" => { if let ParamValue::Float(f) = value { self.sustain = f.max(0.0).min(1.0); } }
            "release" => { if let ParamValue::Float(f) = value { self.release = f.max(0.001).min(2.0); } }
            "amplitude" => { if let ParamValue::Float(f) = value { self.amplitude = (f.max(0.0).min(1.0)) as f32; } }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "bpm" => Some(ParamValue::Float(self.bpm)),
            "notes" => Some(ParamValue::String(format!("{:?}", self.notes))),
            "waveform" => Some(ParamValue::String(match self.waveform {
                Waveform::Square => "square",
                Waveform::Sawtooth => "sawtooth",
                Waveform::Triangle => "triangle",
                Waveform::Sine => "sine",
            }.to_string())),
            "attack" => Some(ParamValue::Float(self.attack)),
            "decay" => Some(ParamValue::Float(self.decay)),
            "sustain" => Some(ParamValue::Float(self.sustain)),
            "release" => Some(ParamValue::Float(self.release)),
            "amplitude" => Some(ParamValue::Float(self.amplitude as f64)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, _input: Option<&[f32]>) -> AudioBuffer {
        let mut buffer = AudioBuffer::with_capacity(block_size);

        for _ in 0..block_size {
            self.step_accumulator += 1.0;
            if self.step_accumulator >= self.samples_per_step {
                self.step_accumulator = 0.0;
                self.advance_step();
            }

            self.update_envelope();
            buffer.push(self.generate_sample());
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
