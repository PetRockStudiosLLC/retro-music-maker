use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct Gain {
    id: String,
    volume: f64,
}

impl Gain {
    pub fn new(id: String) -> Self { Self { id, volume: 0.5 } }
}

impl Node for Gain {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "Gain" }
    fn category(&self) -> &str { "Mixer" }

    fn inputs(&self) -> Vec<PortInfo> { vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }] }
    fn outputs(&self) -> Vec<PortInfo> { vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }] }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("volume".to_string(), ParamValue::Float(self.volume));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        if name == "volume" {
            let f = match value { ParamValue::Float(v) => v, ParamValue::Int(v) => v as f64, _ => return };
            self.volume = f.max(0.0).min(2.0);
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        if name == "volume" { Some(ParamValue::Float(self.volume)) } else { None }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        (0..block_size).map(|i| {
            let inp = input.and_then(|b| b.get(i)).copied().unwrap_or(0.0);
            inp * self.volume as f32
        }).collect()
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
