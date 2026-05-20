use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct RandomTrigger {
    id: String,
    min_interval: f64,
    max_interval: f64,
    elapsed: usize,
    next_trigger: usize,
}

impl RandomTrigger {
    pub fn new(id: String) -> Self {
        Self {
            id,
            min_interval: 0.5,
            max_interval: 2.0,
            elapsed: 0,
            next_trigger: Self::random_interval(0.5, 2.0),
        }
    }

    fn random_interval(min: f64, max: f64) -> usize {
        let range = max - min;
        let random = fastrand::f64() * range + min;
        (random * 44100.0) as usize
    }
}

impl Node for RandomTrigger {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "RandomTrigger" }
    fn category(&self) -> &str { "Trigger" }

    fn inputs(&self) -> Vec<PortInfo> { vec![] }
    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "trigger".to_string(), port_type: PortType::Trigger }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("min_interval".to_string(), ParamValue::Float(self.min_interval));
        params.insert("max_interval".to_string(), ParamValue::Float(self.max_interval));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        match name {
            "min_interval" => {
                if let ParamValue::Float(f) = value {
                    self.min_interval = f.max(0.01).min(10.0);
                }
            }
            "max_interval" => {
                if let ParamValue::Float(f) = value {
                    self.max_interval = f.max(0.01).min(10.0);
                }
            }
            _ => {}
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        match name {
            "min_interval" => Some(ParamValue::Float(self.min_interval)),
            "max_interval" => Some(ParamValue::Float(self.max_interval)),
            _ => None,
        }
    }

    fn process(&mut self, block_size: BlockSize, _input: Option<&[f32]>) -> AudioBuffer {
        let mut buffer = AudioBuffer::with_capacity(block_size);

        for _ in 0..block_size {
            let is_trigger = self.elapsed == self.next_trigger;
            buffer.push(if is_trigger { 1.0f32 } else { 0.0f32 });

            if self.elapsed >= self.next_trigger {
                self.elapsed = 0;
                self.next_trigger = Self::random_interval(self.min_interval, self.max_interval);
            } else {
                self.elapsed += 1;
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
