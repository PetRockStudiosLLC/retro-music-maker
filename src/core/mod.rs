pub mod port;
pub mod node;
pub mod graph;
pub mod engine;

pub use port::{PortType, AudioPort, ControlPort, TriggerPort, InstrumentPort, PortInfo};
pub use node::{Node, NodeInfo, NodeParams, SignalRegistry, NodeKind};
pub use graph::{Graph, GraphState, Edge};
pub use engine::Engine;

pub type Sample = f32;
pub type AudioBuffer = Vec<Sample>;
pub type BlockSize = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    Float(f64),
    Int(i64),
    Bool(bool),
    String(String),
}

impl serde::Serialize for ParamValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ParamValue::Float(f) => f.serialize(serializer),
            ParamValue::Int(i) => i.serialize(serializer),
            ParamValue::Bool(b) => b.serialize(serializer),
            ParamValue::String(s) => s.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ParamValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct ParamValueVisitor;

        impl<'de> Visitor<'de> for ParamValueVisitor {
            type Value = ParamValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a number, boolean, or string")
            }

            fn visit_f64<E>(self, value: f64) -> Result<ParamValue, E> {
                Ok(ParamValue::Float(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<ParamValue, E> {
                Ok(ParamValue::Int(value))
            }

            fn visit_i32<E>(self, value: i32) -> Result<ParamValue, E> {
                Ok(ParamValue::Int(value as i64))
            }

            fn visit_u64<E>(self, value: u64) -> Result<ParamValue, E> {
                Ok(ParamValue::Int(value as i64))
            }

            fn visit_u32<E>(self, value: u32) -> Result<ParamValue, E> {
                Ok(ParamValue::Int(value as i64))
            }

            fn visit_bool<E>(self, value: bool) -> Result<ParamValue, E> {
                Ok(ParamValue::Bool(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<ParamValue, E>
            where
                E: de::Error,
            {
                Ok(ParamValue::String(value.to_string()))
            }
        }

        deserializer.deserialize_any(ParamValueVisitor)
    }
}
