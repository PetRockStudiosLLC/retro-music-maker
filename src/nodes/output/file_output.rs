use std::collections::HashMap;
use std::sync::Mutex;
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType};

pub struct FileOutput {
    id: String,
    filename: String,
    buffer: Mutex<Vec<f32>>,
    recording: bool,
}

impl FileOutput {
    pub fn new(id: String) -> Self {
        Self {
            id,
            filename: "output.wav".to_string(),
            buffer: Mutex::new(Vec::new()),
            recording: true,
        }
    }

    pub fn get_buffer(&self) -> Vec<f32> {
        self.buffer.lock().unwrap().clone()
    }

    pub fn get_buffer_mut(&self) -> std::sync::MutexGuard<Vec<f32>> {
        self.buffer.lock().unwrap()
    }

    pub fn clear_buffer(&self) {
        self.buffer.lock().unwrap().clear();
    }

    pub fn set_recording(&mut self, val: bool) {
        self.recording = val;
    }
}

impl Node for FileOutput {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "FileOutput" }
    fn category(&self) -> &str { "Output" }

    fn inputs(&self) -> Vec<PortInfo> { vec![PortInfo { name: "audio".to_string(), port_type: PortType::Audio }] }
    fn outputs(&self) -> Vec<PortInfo> { vec![] }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        params.insert("filename".to_string(), ParamValue::String(self.filename.clone()));
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        if name == "filename" {
            if let ParamValue::String(s) = value { self.filename = s; }
        }
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        if name == "filename" { Some(ParamValue::String(self.filename.clone())) } else { None }
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        if self.recording {
            if let Some(input_data) = input {
                let mut buf = self.buffer.lock().unwrap();
                for i in 0..block_size {
                    if i < input_data.len() {
                        buf.push(input_data[i]);
                    }
                }
            }
        }
        vec![0.0f32; block_size]
    }

    fn get_recording(&self) -> Option<Vec<f32>> {
        Some(self.buffer.lock().unwrap().clone())
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
