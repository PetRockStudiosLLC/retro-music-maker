use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct Loop {
    id: String,
    loop_duration: f64,
    buffer: Vec<f32>,
    write_pos: usize,
    read_pos: usize,
    recording: bool,
    armed: bool,
}

impl Loop {
    pub fn new(id: String) -> Self {
        let buf_len = (44100.0 * 2.0) as usize;
        Self {
            id,
            loop_duration: 2.0,
            buffer: vec![0.0; buf_len],
            write_pos: 0,
            read_pos: 0,
            recording: false,
            armed: true,
        }
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
        self.read_pos = 0;
        self.recording = false;
        self.armed = true;
    }
}

impl Node for Loop {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "Loop" }
    fn category(&self) -> &str { "Effect" }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![
            PortInfo { name: "audio".to_string(), port_type: PortType::Audio },
            PortInfo { name: "trigger".to_string(), port_type: PortType::Trigger },
        ]
    }

    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("loop_duration".to_string(), ParamValue::Float(self.loop_duration));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "loop_duration" => {
                if let ParamValue::Float(f) = value {
                    let dur = f.max(0.1).min(10.0);
                    if (dur - self.loop_duration).abs() > 0.01 {
                        self.loop_duration = dur;
                        self.reset();
                    } else {
                        self.loop_duration = dur;
                    }
                }
            }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "loop_duration" => Some(ParamValue::Float(self.loop_duration)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let buf_len = self.buffer.len();
        let loop_samples = (self.loop_duration * 44100.0) as usize;
        let mut output = AudioBuffer::with_capacity(block_size);

        for i in 0..block_size {
            let inp = input.and_then(|b| b.get(i)).copied().unwrap_or(0.0);

            if self.recording {
                self.buffer[self.write_pos] = inp;
                self.write_pos = (self.write_pos + 1) % buf_len;

                if self.write_pos >= loop_samples {
                    self.recording = false;
                    self.read_pos = 0;
                }

                output.push(inp);
            } else if self.armed {
                if inp.abs() > 0.01 {
                    self.recording = true;
                    self.buffer[self.write_pos] = inp;
                    self.write_pos = (self.write_pos + 1) % buf_len;
                    output.push(inp);
                } else {
                    output.push(0.0);
                }
            } else {
                let sample = self.buffer[self.read_pos];
                self.read_pos = (self.read_pos + 1) % loop_samples;
                output.push(sample);
            }
        }

        output
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
