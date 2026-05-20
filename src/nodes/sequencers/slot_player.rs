use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct SlotPlayer {
    id: String,
    num_slots: usize,
    current_slot: usize,
    silence_counter: usize,
    has_signal: bool,
}

impl SlotPlayer {
    pub fn new(id: String) -> Self {
        Self {
            id,
            num_slots: 4,
            current_slot: 0,
            silence_counter: 0,
            has_signal: false,
        }
    }
}

impl Node for SlotPlayer {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "SlotPlayer" }
    fn category(&self) -> &str { "Sequencer" }

    fn inputs(&self) -> Vec<PortInfo> {
        (0..self.num_slots)
            .map(|i| PortInfo { name: format!("input_{}", i), port_type: PortType::Audio })
            .collect()
    }

    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("num_slots".to_string(), ParamValue::Int(self.num_slots as i64));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "num_slots" => {
                if let ParamValue::Int(n) = value {
                    self.num_slots = (n as usize).max(1).min(16);
                    if self.current_slot >= self.num_slots {
                        self.current_slot = 0;
                    }
                }
            }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "num_slots" => Some(ParamValue::Int(self.num_slots as i64)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, _input: Option<&[f32]>) -> AudioBuffer {
        vec![0.0f32; block_size]
    }

    fn process_multi(&mut self, block_size: BlockSize, inputs: &[Option<AudioBuffer>]) -> AudioBuffer {
        let mut buffer = AudioBuffer::with_capacity(block_size);
        let silence_threshold = 0.001;
        let advance_after_silence = 64;

        for s in 0..block_size {
            let sample = if let Some(Some(ref buf)) = inputs.get(self.current_slot) {
                if s < buf.len() {
                    buf[s].clamp(-1.0, 1.0)
                } else {
                    0.0
                }
            } else {
                0.0
            };

            buffer.push(sample);

            if sample.abs() > silence_threshold {
                self.has_signal = true;
                self.silence_counter = 0;
            } else if self.has_signal {
                self.silence_counter += 1;
                if self.silence_counter >= advance_after_silence {
                    self.current_slot = (self.current_slot + 1) % self.num_slots;
                    self.has_signal = false;
                    self.silence_counter = 0;
                }
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
