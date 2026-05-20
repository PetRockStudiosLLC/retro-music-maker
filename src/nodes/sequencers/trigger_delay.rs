use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct TriggerDelay {
    id: String,
    delay_time: f64,
    delay_samples: usize,
    buffer: Vec<f32>,
    write_pos: usize,
}

impl TriggerDelay {
    pub fn new(id: String) -> Self {
        let delay_time = 0.5;
        let delay_samples = (delay_time * 44100.0) as usize;
        Self {
            id,
            delay_time,
            delay_samples,
            buffer: vec![0.0; delay_samples],
            write_pos: 0,
        }
    }

    fn update_delay(&mut self) {
        self.delay_samples = (self.delay_time * 44100.0) as usize;
        self.buffer = vec![0.0; self.delay_samples.max(1)];
        self.write_pos = 0;
    }
}

impl Node for TriggerDelay {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "TriggerDelay" }
    fn category(&self) -> &str { "Trigger" }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "trigger".to_string(), port_type: PortType::Trigger }]
    }
    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "trigger".to_string(), port_type: PortType::Trigger }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("delay_time".to_string(), ParamValue::Float(self.delay_time));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "delay_time" => {
                if let ParamValue::Float(f) = value {
                    self.delay_time = f.max(0.01).min(10.0);
                    self.update_delay();
                }
            }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "delay_time" => Some(ParamValue::Float(self.delay_time)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let mut buffer = AudioBuffer::with_capacity(block_size);
        let delay = self.delay_samples;

        for i in 0..block_size {
            let input_val = input.and_then(|b| b.get(i)).copied().unwrap_or(0.0);

            // Read from delay position
            let read_pos = (self.write_pos + self.buffer.len() - delay) % self.buffer.len();
            let output = self.buffer[read_pos];

            // Write input to buffer
            self.buffer[self.write_pos] = input_val;
            self.write_pos = (self.write_pos + 1) % self.buffer.len();

            buffer.push(output);
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
