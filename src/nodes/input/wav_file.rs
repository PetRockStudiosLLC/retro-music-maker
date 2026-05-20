use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct WavFile {
    id: String,
    path: String,
    samples: Vec<f32>,
    position: usize,
    playing: bool,
    loop_enabled: bool,
}

impl WavFile {
    pub fn new(id: String) -> Self {
        Self {
            id,
            path: String::new(),
            samples: Vec::new(),
            position: 0,
            playing: false,
            loop_enabled: false,
        }
    }

    fn load_wav(&mut self, path: &str) {
        self.position = 0;
        self.samples.clear();
        match hound::WavReader::open(path) {
            Ok(mut reader) => {
                let channels = reader.spec().channels as usize;
                let _sample_rate = reader.spec().sample_rate;
                let _bits_per_sample = reader.spec().bits_per_sample;

                let mut raw_samples = Vec::new();
                for sample in reader.samples::<i32>() {
                    if let Ok(s) = sample {
                        raw_samples.push(s);
                    }
                }

                // Normalize to f32 and downmix to mono if needed
                let max_val = (i32::MAX) as f32;
                if channels == 1 {
                    self.samples = raw_samples.iter().map(|s| *s as f32 / max_val).collect();
                } else {
                    // Downmix stereo/multichannel to mono
                    let mut mono = Vec::new();
                    for chunk in raw_samples.chunks(channels) {
                        let sum: f32 = chunk.iter().map(|s| *s as f32 / max_val).sum();
                        mono.push(sum / channels as f32);
                    }
                    self.samples = mono;
                }
            }
            Err(e) => {
                log::warn!("Failed to load WAV file '{}': {}", path, e);
            }
        }
    }
}

impl Node for WavFile {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "WavFile" }
    fn category(&self) -> &str { "Input" }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "trigger".to_string(), port_type: PortType::Trigger },
            PortInfo { name: "volume".to_string(), port_type: PortType::Control },
        ]
    }

    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("path".to_string(), ParamValue::String(self.path.clone()));
        params.insert("loop_enabled".to_string(), ParamValue::Bool(self.loop_enabled));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "path" => {
                if let ParamValue::String(new_path) = value {
                    if new_path != self.path {
                        self.path = new_path.clone();
                        self.load_wav(&new_path);
                    }
                }
            }
            "loop_enabled" => {
                if let ParamValue::Bool(b) = value {
                    self.loop_enabled = b;
                }
            }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "path" => Some(ParamValue::String(self.path.clone())),
            "loop_enabled" => Some(ParamValue::Bool(self.loop_enabled)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let mut buffer = AudioBuffer::with_capacity(block_size);

        // Check for trigger on first sample
        if let Some(inp) = input {
            if !self.playing && inp.first().map_or(false, |&s| s > 0.5) {
                self.playing = true;
                self.position = 0;
            }
        }

        if !self.playing || self.samples.is_empty() {
            for _ in 0..block_size {
                buffer.push(0.0);
            }
            return buffer;
        }

        // Apply volume from control input
        let volume = if let Some(inp) = input {
            inp.first().copied().unwrap_or(1.0)
        } else {
            1.0
        };

        for _ in 0..block_size {
            if self.position >= self.samples.len() {
                if self.loop_enabled {
                    self.position = 0;
                } else {
                    self.playing = false;
                    buffer.push(0.0);
                    continue;
                }
            }
            buffer.push(self.samples[self.position] * volume);
            self.position += 1;
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
