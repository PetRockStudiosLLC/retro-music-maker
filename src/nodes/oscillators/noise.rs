use std::collections::HashMap;
use rand::Rng;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct NoiseOscillator {
    id: String,
    amplitude: f64,
}

impl NoiseOscillator {
    pub fn new(id: String) -> Self { Self { id, amplitude: 0.3 } }
}

impl Node for NoiseOscillator {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "NoiseOscillator" }
    fn category(&self) -> &str { "Oscillator" }

    fn inputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "amplitude".to_string(), port_type: PortType::Control }]
    }

    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("amplitude".to_string(), ParamValue::Float(self.amplitude));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        if name == "amplitude" { if let ParamValue::Float(f) = value { self.amplitude = f.max(0.0).min(1.0); } }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        if name == "amplitude" { Some(ParamValue::Float(self.amplitude)) } else { None }
    }

    fn process(&mut self, block_size: BlockSize, _input: Option<&[f32]>) -> AudioBuffer {
        let mut rng = rand::thread_rng();
        (0..block_size).map(|_| (rng.gen_range(-1.0..1.0) * self.amplitude) as f32).collect()
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
