use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortType {
    Audio,
    Control,
    Trigger,
    Instrument,
}

impl std::fmt::Display for PortType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortType::Audio => write!(f, "Audio"),
            PortType::Control => write!(f, "Control"),
            PortType::Trigger => write!(f, "Trigger"),
            PortType::Instrument => write!(f, "Instrument"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub name: String,
    pub port_type: PortType,
}

#[derive(Debug, Clone)]
pub struct AudioPort {
    pub name: String,
    pub buffer: crate::core::AudioBuffer,
}

impl AudioPort {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            buffer: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn set_buffer(&mut self, buffer: crate::core::AudioBuffer) {
        self.buffer = buffer;
    }

    pub fn get_buffer(&self) -> &[crate::core::Sample] {
        &self.buffer
    }

    pub fn info(&self) -> PortInfo {
        PortInfo {
            name: self.name.clone(),
            port_type: PortType::Audio,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ControlPort {
    pub name: String,
    pub value: f64,
}

impl ControlPort {
    pub fn new(name: &str, default: f64) -> Self {
        Self {
            name: name.to_string(),
            value: default,
        }
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value;
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }

    pub fn info(&self) -> PortInfo {
        PortInfo {
            name: self.name.clone(),
            port_type: PortType::Control,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TriggerPort {
    pub name: String,
    pub triggered: bool,
}

impl TriggerPort {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            triggered: false,
        }
    }

    pub fn trigger(&mut self) {
        self.triggered = true;
    }

    pub fn is_triggered(&self) -> bool {
        self.triggered
    }

    pub fn clear(&mut self) {
        self.triggered = false;
    }

    pub fn info(&self) -> PortInfo {
        PortInfo {
            name: self.name.clone(),
            port_type: PortType::Trigger,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstrumentPort {
    pub name: String,
    pub instrument_json: String,
}

impl InstrumentPort {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            instrument_json: String::new(),
        }
    }

    pub fn set_instrument(&mut self, json: String) {
        self.instrument_json = json;
    }

    pub fn get_instrument(&self) -> &str {
        &self.instrument_json
    }

    pub fn info(&self) -> PortInfo {
        PortInfo {
            name: self.name.clone(),
            port_type: PortType::Instrument,
        }
    }
}
