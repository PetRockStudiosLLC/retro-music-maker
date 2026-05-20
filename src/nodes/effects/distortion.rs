use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct Distortion {
    id: String,
    amount: f64,
}

impl Distortion {
    pub fn new(id: String) -> Self { Self { id, amount: 0.5 } }
}

impl Node for Distortion {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "Distortion" }
    fn category(&self) -> &str { "Effect" }

    fn inputs(&self) -> Vec<PortInfo> { vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }] }
    fn outputs(&self) -> Vec<PortInfo> { vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }] }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("amount".to_string(), ParamValue::Float(self.amount));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        if name == "amount" {
            let f = match value { ParamValue::Float(v) => v, ParamValue::Int(v) => v as f64, _ => return };
            self.amount = f.max(0.0).min(1.0);
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        if name == "amount" { Some(ParamValue::Float(self.amount)) } else { None }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let drive = self.amount as f32 * 10.0;
        (0..block_size).map(|i| {
            let x = input.and_then(|b| b.get(i)).copied().unwrap_or(0.0);
            if drive > 0.0 { (x * drive).tanh() / drive } else { x }
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
