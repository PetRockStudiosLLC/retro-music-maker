use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, PortType, NodeKind};

pub fn parse_port_type(s: &str) -> PortType {
    match s.to_lowercase().as_str() {
        "audio" => PortType::Audio,
        "control" => PortType::Control,
        "trigger" => PortType::Trigger,
        "instrument" => PortType::Instrument,
        _ => PortType::Audio,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptParamDef {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub default: ParamValue,
    pub min: Option<ParamValue>,
    pub max: Option<ParamValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptPortDef {
    pub name: String,
    #[serde(rename = "type")]
    pub port_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInfo {
    pub name: String,
    pub category: String,
    pub description: String,
    pub inputs: Vec<PortInfo>,
    pub outputs: Vec<PortInfo>,
    pub params: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptDefinition {
    pub name: String,
    pub category: String,
    pub description: String,
    pub params: Vec<ScriptParamDef>,
    pub inputs: Vec<ScriptPortDef>,
    pub outputs: Vec<ScriptPortDef>,
    pub script: String,
}

pub struct ScriptNode {
    id: String,
    name: String,
    category: String,
    inputs: Vec<PortInfo>,
    outputs: Vec<PortInfo>,
    param_defs: Vec<ScriptParamDef>,
    current_params: NodeParams,
    script: String,
    position: [f64; 2],
}

impl ScriptNode {
    pub fn new(id: String, definition: ScriptDefinition) -> Result<Self, String> {
        let inputs: Vec<PortInfo> = definition.inputs.iter().map(|p| PortInfo {
            name: p.name.clone(),
            port_type: parse_port_type(&p.port_type),
        }).collect();

        let outputs: Vec<PortInfo> = definition.outputs.iter().map(|p| PortInfo {
            name: p.name.clone(),
            port_type: parse_port_type(&p.port_type),
        }).collect();

        let mut current_params = HashMap::new();
        for param in &definition.params {
            current_params.insert(param.name.clone(), param.default.clone());
        }

        let lua = mlua::Lua::new();
        if let Err(e) = lua.load(&definition.script).eval::<()>() {
            return Err(format!("Lua compile error: {}", e));
        }

        Ok(Self {
            id,
            name: definition.name,
            category: definition.category,
            inputs,
            outputs,
            param_defs: definition.params,
            current_params,
            script: definition.script,
            position: [0.0, 0.0],
        })
    }
}

impl Node for ScriptNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn category(&self) -> &str {
        &self.category
    }

    fn inputs(&self) -> Vec<PortInfo> {
        self.inputs.clone()
    }

    fn outputs(&self) -> Vec<PortInfo> {
        self.outputs.clone()
    }

    fn default_params(&self) -> NodeParams {
        self.current_params.clone()
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        self.current_params.insert(name.to_string(), value);
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        self.current_params.get(name).cloned()
    }

    fn process(&mut self, block_size: BlockSize, _input: Option<&[f32]>) -> AudioBuffer {
        let mut output = vec![0.0f32; block_size];

        let lua = mlua::Lua::new();

        if let Err(e) = (|| -> Result<(), String> {
            lua.load(&self.script).eval::<()>().map_err(|e| e.to_string())?;

            let process_fn = lua.globals().get::<_, mlua::Function>("process")
                .map_err(|e| format!("No process function: {}", e))?;

            let input_buf = vec![0.0f32; block_size];
            let input_table = lua.create_table().map_err(|e| e.to_string())?;
            for (i, sample) in input_buf.iter().enumerate() {
                input_table.set(i + 1, *sample).map_err(|e| e.to_string())?;
            }

            let output_table = lua.create_table().map_err(|e| e.to_string())?;
            output_table.set("size", block_size).map_err(|e| e.to_string())?;
            for i in 0..block_size {
                output_table.set(i + 1, 0.0f32).map_err(|e| e.to_string())?;
            }

            let params_table = lua.create_table().map_err(|e| e.to_string())?;
            for (key, value) in &self.current_params {
                match value {
                    ParamValue::Float(f) => { params_table.set(key.as_str(), *f).ok(); }
                    ParamValue::Int(i) => { params_table.set(key.as_str(), *i).ok(); }
                    ParamValue::Bool(b) => { params_table.set(key.as_str(), *b).ok(); }
                    ParamValue::String(s) => { params_table.set(key.as_str(), s.as_str()).ok(); }
                }
            }

            let _ = process_fn.call::<_, ()>((
                lua.create_table().map_err(|e| e.to_string())?,
                input_table,
                output_table.clone(),
                params_table,
            )).map_err(|e| e.to_string())?;

            for i in 0..block_size {
                if let Ok(sample) = output_table.get::<_, f32>(i + 1) {
                    output[i] = sample;
                }
            }

            Ok(())
        })() {
            eprintln!("ScriptNode process error: {}", e);
        }

        output
    }

    fn to_info(&self, position: [f64; 2]) -> NodeInfo {
        let def_value = serde_json::to_value(&self.script).unwrap_or_default();
        NodeInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            inputs: self.inputs(),
            outputs: self.outputs(),
            params: self.current_params.clone(),
            position,
            node_kind: NodeKind::Script,
            definition: Some(def_value),
        }
    }

    fn kind(&self) -> NodeKind {
        NodeKind::Script
    }
}
