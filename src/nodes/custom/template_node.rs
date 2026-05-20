use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::core::{Node, NodeInfo, NodeParams, AudioBuffer, BlockSize, ParamValue, PortInfo, Graph, NodeKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateExposedParam {
    pub param: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInternalNode {
    pub id: String,
    pub node_type: String,
    pub position: [f64; 2],
    pub params: HashMap<String, ParamValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateEdge {
    pub source: String,
    pub source_handle: String,
    pub target: String,
    pub target_handle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInputRouting {
    pub external: String,
    pub internal_node: String,
    pub internal_port: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateOutputRouting {
    pub internal_node: String,
    pub internal_port: String,
    pub external: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDefinition {
    pub name: String,
    pub category: String,
    pub description: String,
    pub inputs: Vec<PortInfo>,
    pub outputs: Vec<PortInfo>,
    pub exposed_params: Vec<TemplateExposedParam>,
    pub internal_nodes: Vec<TemplateInternalNode>,
    pub internal_edges: Vec<TemplateEdge>,
    pub input_routing: Vec<TemplateInputRouting>,
    pub output_routing: Vec<TemplateOutputRouting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub name: String,
    pub category: String,
    pub description: String,
    pub inputs: Vec<PortInfo>,
    pub outputs: Vec<PortInfo>,
    pub exposed_params: Vec<String>,
}

pub struct TemplateNode {
    id: String,
    definition: TemplateDefinition,
    internal_graph: Graph,
    param_map: HashMap<String, String>,
    output_map: Vec<String>,
    position: [f64; 2],
}

impl TemplateNode {
    pub fn new(id: String, definition: TemplateDefinition) -> Result<Self, String> {
        let mut internal_graph = Graph::new();
        let mut param_map = HashMap::new();
        let mut output_map = Vec::new();

        for internal_node in &definition.internal_nodes {
            use crate::commands::create_node_from_type;
            if let Some(node) = create_node_from_type(&internal_node.node_type, format!("{}_{}", id, internal_node.id)) {
                internal_graph.add_node(node);
            } else {
                return Err(format!("Unknown internal node type: {}", internal_node.node_type));
            }
        }

        for edge in &definition.internal_edges {
            use crate::core::graph::Edge;
            internal_graph.add_edge(Edge {
                source: format!("{}_{}", id, edge.source),
                source_handle: edge.source_handle.clone(),
                target: format!("{}_{}", id, edge.target),
                target_handle: edge.target_handle.clone(),
            });
        }

        for exposed in &definition.exposed_params {
            let parts: Vec<&str> = exposed.param.split('.').collect();
            if parts.len() == 2 {
                let internal_id = format!("{}_{}", id, parts[0]);
                param_map.insert(parts[1].to_string(), internal_id);
            }
        }

        for routing in &definition.output_routing {
            output_map.push(format!("{}_{}", id, routing.internal_node));
        }

        Ok(Self {
            id,
            definition,
            internal_graph,
            param_map,
            output_map,
            position: [0.0, 0.0],
        })
    }

    fn set_internal_param(&mut self, param_path: &str, value: ParamValue) {
        let parts: Vec<&str> = param_path.split('.').collect();
        if parts.len() == 2 {
            let internal_id = format!("{}_{}", self.id, parts[0]);
            if let Some(node) = self.internal_graph.get_node_mut(&internal_id) {
                node.set_param(parts[1], value);
            }
        }
    }
}

impl Node for TemplateNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.definition.name
    }

    fn category(&self) -> &str {
        &self.definition.category
    }

    fn inputs(&self) -> Vec<PortInfo> {
        self.definition.inputs.clone()
    }

    fn outputs(&self) -> Vec<PortInfo> {
        self.definition.outputs.clone()
    }

    fn default_params(&self) -> NodeParams {
        let mut params = HashMap::new();
        for exposed in &self.definition.exposed_params {
            let parts: Vec<&str> = exposed.param.split('.').collect();
            if parts.len() == 2 {
                let internal_id = format!("{}_{}", self.id, parts[0]);
                if let Some(node) = self.internal_graph.get_node(&internal_id) {
                    if let Some(val) = node.get_param(parts[1]) {
                        params.insert(exposed.param.clone(), val);
                    }
                }
            }
        }
        params
    }

    fn set_param(&mut self, name: &str, value: ParamValue) {
        self.set_internal_param(name, value);
    }

    fn get_param(&self, name: &str) -> Option<ParamValue> {
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() == 2 {
            let internal_id = format!("{}_{}", self.id, parts[0]);
            if let Some(node) = self.internal_graph.get_node(&internal_id) {
                return node.get_param(parts[1]);
            }
        }
        None
    }

    fn process(&mut self, block_size: BlockSize, input: Option<&[f32]>) -> AudioBuffer {
        if let Some(input_buf) = input {
            for routing in &self.definition.input_routing {
                let internal_id = format!("{}_{}", self.id, routing.internal_node);
                if let Some(node) = self.internal_graph.get_node_mut(&internal_id) {
                    let buf = AudioBuffer::from(input_buf.to_vec());
                    let _ = node.process(block_size, Some(&buf));
                }
            }
        }

        if let Ok(results) = self.internal_graph.process_block(block_size) {
            let mut output = vec![0.0f32; block_size];
            let mut count = 0usize;

            for node_id in &self.output_map {
                if let Some(buf) = results.get(node_id) {
                    if !buf.is_empty() {
                        count += 1;
                        for i in 0..block_size {
                            if i < output.len() && i < buf.len() {
                                output[i] += buf[i];
                            }
                        }
                    }
                }
            }

            if count > 0 {
                let divisor = count as f32;
                for sample in &mut output {
                    *sample = (*sample / divisor).min(1.0).max(-1.0);
                }
            }

            output
        } else {
            vec![0.0f32; block_size]
        }
    }

    fn to_info(&self, position: [f64; 2]) -> NodeInfo {
        NodeInfo {
            id: self.id.clone(),
            name: self.name().to_string(),
            category: self.category().to_string(),
            inputs: self.inputs(),
            outputs: self.outputs(),
            params: self.default_params(),
            position,
            node_kind: NodeKind::Template,
            definition: Some(serde_json::to_value(&self.definition).unwrap_or_default()),
        }
    }

    fn kind(&self) -> NodeKind {
        NodeKind::Template
    }
}
