use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

#[derive(Clone, Copy, PartialEq)]
enum Waveform { Square, Sawtooth, Triangle, Sine, Pulse, PulseHalf, Noise, SampleHold }

#[derive(Clone, Copy, PartialEq)]
enum EnvPhase { Attack, Decay, Sustain, Release, Off }

#[derive(Clone, Copy, PartialEq)]
enum Instrument {
    SquareLead,
    SquareBass,
    SawLead,
    SawBass,
    TrianglePad,
    SinePad,
    ChipPunch,
    ChipArp,
}

impl Instrument {
    fn waveform(&self) -> Waveform {
        match self {
            Instrument::SquareLead | Instrument::SquareBass | Instrument::ChipPunch | Instrument::ChipArp => Waveform::Square,
            Instrument::SawLead | Instrument::SawBass => Waveform::Sawtooth,
            Instrument::TrianglePad => Waveform::Triangle,
            Instrument::SinePad => Waveform::Sine,
        }
    }
    fn attack(&self) -> f64 {
        match self {
            Instrument::SquareLead => 0.002,
            Instrument::SquareBass => 0.001,
            Instrument::SawLead => 0.005,
            Instrument::SawBass => 0.002,
            Instrument::TrianglePad => 0.3,
            Instrument::SinePad => 0.5,
            Instrument::ChipPunch => 0.001,
            Instrument::ChipArp => 0.001,
        }
    }
    fn decay(&self) -> f64 {
        match self {
            Instrument::SquareLead => 0.05,
            Instrument::SquareBass => 0.02,
            Instrument::SawLead => 0.08,
            Instrument::SawBass => 0.04,
            Instrument::TrianglePad => 0.8,
            Instrument::SinePad => 1.0,
            Instrument::ChipPunch => 0.15,
            Instrument::ChipArp => 0.03,
        }
    }
    fn sustain(&self) -> f64 {
        match self {
            Instrument::SquareLead => 0.6,
            Instrument::SquareBass => 0.4,
            Instrument::SawLead => 0.5,
            Instrument::SawBass => 0.3,
            Instrument::TrianglePad => 0.7,
            Instrument::SinePad => 0.8,
            Instrument::ChipPunch => 0.2,
            Instrument::ChipArp => 0.5,
        }
    }
    fn release(&self) -> f64 {
        match self {
            Instrument::SquareLead => 0.05,
            Instrument::SquareBass => 0.02,
            Instrument::SawLead => 0.1,
            Instrument::SawBass => 0.03,
            Instrument::TrianglePad => 1.5,
            Instrument::SinePad => 2.0,
            Instrument::ChipPunch => 0.01,
            Instrument::ChipArp => 0.02,
        }
    }
    fn to_str(&self) -> &'static str {
        match self {
            Instrument::SquareLead => "square_lead",
            Instrument::SquareBass => "square_bass",
            Instrument::SawLead => "saw_lead",
            Instrument::SawBass => "saw_bass",
            Instrument::TrianglePad => "triangle_pad",
            Instrument::SinePad => "sine_pad",
            Instrument::ChipPunch => "chip_punch",
            Instrument::ChipArp => "chip_arp",
        }
    }
    fn from_str(s: &str) -> Self {
        match s {
            "square_bass" => Instrument::SquareBass,
            "saw_lead" => Instrument::SawLead,
            "saw_bass" => Instrument::SawBass,
            "triangle_pad" => Instrument::TrianglePad,
            "sine_pad" => Instrument::SinePad,
            "chip_punch" => Instrument::ChipPunch,
            "chip_arp" => Instrument::ChipArp,
            _ => Instrument::SquareLead,
        }
    }
}

pub struct NoteMapper {
    id: String,
    display_name: String,
    grid: Vec<Vec<bool>>,
    num_steps: usize,
    min_midi: u8,
    bpm: f64,
    step_accumulator: f64,
    samples_per_step: f64,
    current_step: usize,
    instrument: Instrument,
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
    sh_value: f32,
    sh_timer: u32,
}

impl NoteMapper {
    pub fn new(id: String) -> Self {
        let sample_rate = 44100.0_f64;
        let bpm = 140.0_f64;
        let num_steps = 16;
        let min_midi = 48;
        let num_pitches = 37;
        let grid = vec![vec![false; num_pitches]; num_steps];
       Self {
            id,
            display_name: String::new(),
            grid,
            num_steps,
            min_midi,
            bpm,
            samples_per_step: sample_rate / (bpm / 60.0 * 4.0),
            step_accumulator: 0.0,
            current_step: 0,
            instrument: Instrument::SquareLead,
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
            sh_value: 0.0,
            sh_timer: 0,
       }
    }

    fn instrument_to_json(&self) -> String {
        serde_json::json!({
            "waveform": match self.waveform {
                Waveform::Square => "square",
                Waveform::Sawtooth => "sawtooth",
                Waveform::Triangle => "triangle",
                Waveform::Sine => "sine",
                Waveform::Pulse => "pulse",
                Waveform::PulseHalf => "pulse_half",
                Waveform::Noise => "noise",
                Waveform::SampleHold => "sample_hold",
            },
            "attack": self.attack,
            "decay": self.decay,
            "sustain": self.sustain,
            "release": self.release,
            "amplitude": self.amplitude,
        }).to_string()
    }

    fn midi_to_freq(note: u8) -> f32 {
        if note == 0 { return 0.0; }
        440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
    }

    fn grid_to_json(&self) -> String {
        let rows: Vec<String> = self.grid.iter().map(|row| {
            row.iter().enumerate()
                .filter(|(_, &active)| active)
                .map(|(pitch_idx, _)| (self.min_midi as usize + pitch_idx).to_string())
                .collect::<Vec<_>>()
                .join(",")
        }).collect::<Vec<_>>();
        format!("[{}]", rows.join(";"))
    }


    fn advance_step(&mut self) {
        self.current_step = (self.current_step + 1) % self.num_steps;
        let step = &self.grid[self.current_step];
        let mut freq = 0.0_f32;
        for (pitch_idx, &active) in step.iter().enumerate() {
            if active {
                let midi = (self.min_midi as usize + pitch_idx) as u8;
                freq = Self::midi_to_freq(midi);
                break;
            }
        }
        if freq == 0.0 {
            if self.env_phase != EnvPhase::Off {
                self.env_phase = EnvPhase::Release;
            }
            self.current_freq = 0.0;
        } else {
            self.current_freq = freq;
            self.env_phase = EnvPhase::Attack;
            self.env_value = 0.0;
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
            EnvPhase::Off => {}
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
            Waveform::Sawtooth => 2.0 * (self.osc_phase - 0.5),
            Waveform::Triangle => {
                let p = (self.osc_phase * 4.0).rem_euclid(2.0) - 1.0;
                if p < 0.0 { -p * 2.0 + 1.0 } else { -(p * 2.0 - 1.0) + 1.0 }
            }
            Waveform::Sine => (self.osc_phase * std::f32::consts::PI * 2.0).sin(),
            Waveform::Pulse => {
                if self.osc_phase < 0.25 { 1.0 } else { -1.0 }
            }
            Waveform::PulseHalf => {
                let p = self.osc_phase.rem_euclid(2.0);
                if p < 1.0 { 1.0 } else { -1.0 }
            }
            Waveform::Noise => {
                self.sh_timer += 1;
                let seed = (self.sh_timer.wrapping_mul(1103515245).wrapping_add(12345) & 0x7FFF) as f32;
                (seed / 32767.0) * 2.0 - 1.0
            }
            Waveform::SampleHold => {
                self.sh_timer += 1;
                if self.sh_timer % 88 == 0 {
                    let seed = (self.sh_timer.wrapping_mul(1103515245).wrapping_add(12345) & 0x7FFF) as f32;
                    self.sh_value = (seed / 32767.0) * 2.0 - 1.0;
                }
                self.sh_value
            }
        };
        self.osc_phase = (self.osc_phase + phase_inc).rem_euclid(1.0);
        sample * self.env_value * self.amplitude
    }
}

impl Node for NoteMapper {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "NoteMapper" }
    fn category(&self) -> &str { "Sequencer" }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "instrument".to_string(), port_type: PortType::Instrument }]
    }
    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("display_name".to_string(), ParamValue::String(self.display_name.clone()));
        params.insert("bpm".to_string(), ParamValue::Float(self.bpm));
        params.insert("grid".to_string(), ParamValue::String(self.grid_to_json()));
        params.insert("num_steps".to_string(), ParamValue::Float(self.num_steps as f64));
       params.insert("min_midi".to_string(), ParamValue::Float(self.min_midi as f64));
        params.insert("instrument".to_string(), ParamValue::String(self.instrument.to_str().to_string()));
        params.insert("waveform".to_string(), ParamValue::String(match self.waveform {
            Waveform::Square => "square",
            Waveform::Sawtooth => "sawtooth",
            Waveform::Triangle => "triangle",
            Waveform::Sine => "sine",
            Waveform::Pulse => "pulse",
            Waveform::PulseHalf => "pulse_half",
            Waveform::Noise => "noise",
            Waveform::SampleHold => "sample_hold",
        }.to_string()));
        params.insert("attack".to_string(), ParamValue::Float(self.attack));
        params.insert("decay".to_string(), ParamValue::Float(self.decay));
        params.insert("sustain".to_string(), ParamValue::Float(self.sustain));
        params.insert("release".to_string(), ParamValue::Float(self.release));
      params.insert("amplitude".to_string(), ParamValue::Float(self.amplitude as f64));
        params.insert("instrument_json".to_string(), ParamValue::String(self.instrument_to_json()));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "display_name" => {
                if let ParamValue::String(s) = value {
                    self.display_name = s;
                }
            }
            "bpm" => {
                let f = match value {
                    ParamValue::Float(f) => f,
                    ParamValue::Int(i) => i as f64,
                    _ => return,
                };
                self.bpm = f.max(20.0).min(300.0);
                self.samples_per_step = self.sample_rate as f64 / (self.bpm / 60.0 * 4.0);
            }
            "grid" => {
                if let ParamValue::String(s) = value {
                    let steps: Vec<&str> = s.trim_matches(|c| c == '[' || c == ']')
                        .split(';').collect();
                    self.grid = vec![vec![false; 37]; self.num_steps];
                    for (step_idx, step_str) in steps.iter().take(self.num_steps).enumerate() {
                        for note_str in step_str.split(',') {
                            if let Ok(midi) = note_str.trim().parse::<usize>() {
                                if let Some(idx) = midi.checked_sub(self.min_midi as usize) {
                                    if idx < 37 {
                                        self.grid[step_idx][idx] = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "num_steps" => {
                let new_len = match value {
                    ParamValue::Float(f) => f as usize,
                    ParamValue::Int(i) => i as usize,
                    _ => return,
                };
                if new_len > 0 && new_len <= 128 {
                    let old_grid = std::mem::take(&mut self.grid);
                    self.num_steps = new_len;
                    self.grid = vec![vec![false; 37]; new_len];
                    for step in 0..new_len.min(old_grid.len()) {
                        for pitch in 0..37 {
                            self.grid[step][pitch] = old_grid[step][pitch];
                        }
                    }
                }
            }
            "min_midi" => {
                self.min_midi = match value {
                    ParamValue::Float(f) => f as u8,
                    ParamValue::Int(i) => i as u8,
           _ => return,
                };
            }
            "instrument" => {
                if let ParamValue::String(s) = value {
                    let inst = Instrument::from_str(s.as_str());
                    self.instrument = inst;
                    self.waveform = inst.waveform();
                    self.attack = inst.attack();
                    self.decay = inst.decay();
                    self.sustain = inst.sustain();
                    self.release = inst.release();
                }
            }
            "waveform" => {
                if let ParamValue::String(s) = value {
                    self.waveform = match s.as_str() {
                        "sawtooth" => Waveform::Sawtooth,
                        "triangle" => Waveform::Triangle,
                        "sine" => Waveform::Sine,
                        "pulse" => Waveform::Pulse,
                        "pulse_half" => Waveform::PulseHalf,
                        "noise" => Waveform::Noise,
                        "sample_hold" => Waveform::SampleHold,
                        _ => Waveform::Square,
                    };
                }
            }
            "attack" => { let f = match value { ParamValue::Float(f) => f, ParamValue::Int(i) => i as f64, _ => return }; self.attack = f.max(0.001).min(2.0); }
            "decay" => { let f = match value { ParamValue::Float(f) => f, ParamValue::Int(i) => i as f64, _ => return }; self.decay = f.max(0.001).min(2.0); }
            "sustain" => { let f = match value { ParamValue::Float(f) => f, ParamValue::Int(i) => i as f64, _ => return }; self.sustain = f.max(0.0).min(1.0); }
            "release" => { let f = match value { ParamValue::Float(f) => f, ParamValue::Int(i) => i as f64, _ => return }; self.release = f.max(0.001).min(2.0); }
"amplitude" => { let f = match value { ParamValue::Float(f) => f, ParamValue::Int(i) => i as f64, _ => return }; self.amplitude = (f.max(0.0).min(1.0)) as f32; }
            "instrument_json" => {
                if let ParamValue::String(s) = value {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&s) {
                        if let Some(w) = data.get("waveform").and_then(|v| v.as_str()) {
                            self.waveform = match w {
                                "sawtooth" => Waveform::Sawtooth,
                                "triangle" => Waveform::Triangle,
                                "sine" => Waveform::Sine,
                                "pulse" => Waveform::Pulse,
                                "pulse_half" => Waveform::PulseHalf,
                                "noise" => Waveform::Noise,
                                "sample_hold" => Waveform::SampleHold,
                                _ => Waveform::Square,
                            };
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
                            self.amplitude = (amp.max(0.0).min(1.0)) as f32;
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
            "bpm" => Some(ParamValue::Float(self.bpm)),
            "grid" => Some(ParamValue::String(self.grid_to_json())),
            "num_steps" => Some(ParamValue::Float(self.num_steps as f64)),
          "min_midi" => Some(ParamValue::Float(self.min_midi as f64)),
            "instrument" => Some(ParamValue::String(self.instrument.to_str().to_string())),
            "waveform" => Some(ParamValue::String(match self.waveform {
                Waveform::Square => "square",
                Waveform::Sawtooth => "sawtooth",
                Waveform::Triangle => "triangle",
                Waveform::Sine => "sine",
                Waveform::Pulse => "pulse",
                Waveform::PulseHalf => "pulse_half",
                Waveform::Noise => "noise",
                Waveform::SampleHold => "sample_hold",
            }.to_string())),
            "attack" => Some(ParamValue::Float(self.attack)),
            "decay" => Some(ParamValue::Float(self.decay)),
            "sustain" => Some(ParamValue::Float(self.sustain)),
            "release" => Some(ParamValue::Float(self.release)),
"amplitude" => Some(ParamValue::Float(self.amplitude as f64)),
            "instrument_json" => Some(ParamValue::String(self.instrument_to_json())),
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
