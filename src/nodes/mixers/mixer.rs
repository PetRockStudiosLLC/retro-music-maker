use std::collections::HashMap;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct Mixer {
    id: String,
    channels: usize,
    volumes: Vec<f64>,
    pans: Vec<f64>,
}

impl Mixer {
    pub fn new(id: String, channels: usize) -> Self {
        Self { id, channels, volumes: vec![0.5; channels], pans: vec![0.0; channels] }
    }

    fn pan_l(&self, idx: usize) -> f32 {
        let p = self.pans.get(idx).copied().unwrap_or(0.0);
        (1.0 - p).max(0.0).min(2.0) as f32 * 0.7071
    }

    fn pan_r(&self, idx: usize) -> f32 {
        let p = self.pans.get(idx).copied().unwrap_or(0.0);
        (1.0 + p).max(0.0).min(2.0) as f32 * 0.7071
    }
}

impl Node for Mixer {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "Mixer" }
    fn category(&self) -> &str { "Mixer" }

    fn inputs(&self) -> Vec<PortInfo> {
        (0..self.channels).map(|i| PortInfo { name: format!("input_{}", i), port_type: PortType::Audio }).collect()
    }

    fn outputs(&self) -> Vec<PortInfo> {
        vec![PortInfo { name: "output".to_string(), port_type: PortType::Audio }]
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("channels".to_string(), ParamValue::Int(self.channels as i64));
        for (i, vol) in self.volumes.iter().enumerate() {
            params.insert(format!("volume_{}", i), ParamValue::Float(*vol));
        }
        for (i, pan) in self.pans.iter().enumerate() {
            params.insert(format!("pan_{}", i), ParamValue::Float(*pan));
        }
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        let f = match value { ParamValue::Float(v) => v, ParamValue::Int(v) => v as f64, _ => return };
        if name == "channels" {
            let new_channels = (f as usize).max(1).min(16);
            self.channels = new_channels;
            while self.volumes.len() < new_channels { self.volumes.push(0.5); }
            while self.pans.len() < new_channels { self.pans.push(0.0); }
            self.volumes.truncate(new_channels);
            self.pans.truncate(new_channels);
        } else if let Some(idx) = name.strip_prefix("volume_") {
            if let Ok(i) = idx.parse::<usize>() {
                if i < self.volumes.len() { self.volumes[i] = f.max(0.0).min(1.0); }
            }
        } else if let Some(idx) = name.strip_prefix("pan_") {
            if let Ok(i) = idx.parse::<usize>() {
                if i < self.pans.len() { self.pans[i] = f.max(-1.0).min(1.0); }
            }
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        if let Some(idx) = name.strip_prefix("volume_") {
            if let Ok(i) = idx.parse::<usize>() {
                if i < self.volumes.len() { return Some(ParamValue::Float(self.volumes[i])); }
            }
        } else if let Some(idx) = name.strip_prefix("pan_") {
            if let Ok(i) = idx.parse::<usize>() {
                if i < self.pans.len() { return Some(ParamValue::Float(self.pans[i])); }
            }
        }
        None
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        let avg_vol = if self.volumes.is_empty() { 0.5 } else { self.volumes.iter().sum::<f64>() / self.volumes.len() as f64 };
        (0..block_size).map(|i| {
            let inp = input.and_then(|b| b.get(i)).copied().unwrap_or(0.0);
            inp * avg_vol as f32
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
