use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use crate::core::{AudioBuffer, BlockSize, Node, NodeInfo, ParamValue, PortType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source: String,
    pub source_handle: String,
    pub target: String,
    pub target_handle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphState {
    pub nodes: Vec<NodeInfo>,
    pub edges: Vec<Edge>,
}

pub struct Graph {
    pub nodes: HashMap<String, Box<dyn Node>>,
    edges: Vec<Edge>,
    cached_order: Vec<String>,
    order_valid: bool,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            cached_order: Vec::new(),
            order_valid: false,
        }
    }

    fn invalidate_cache(&mut self) {
        self.order_valid = false;
    }

    pub fn add_node(&mut self, node: Box<dyn Node>) -> String {
        let id = node.id().to_string();
        self.nodes.insert(id.clone(), node);
        self.invalidate_cache();
        id
    }

    pub fn remove_node(&mut self, node_id: &str) -> bool {
        if self.nodes.remove(node_id).is_some() {
            self.edges.retain(|e| e.source != node_id && e.target != node_id);
            self.invalidate_cache();
            true
        } else {
            false
        }
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
        self.invalidate_cache();
    }

    pub fn remove_edge(&mut self, source: &str, target: &str) {
        self.edges.retain(|e| !(e.source == source && e.target == target));
        self.invalidate_cache();
    }

    pub fn get_node(&self, id: &str) -> Option<&dyn Node> {
        self.nodes.get(id).map(|n| n.as_ref())
    }

    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut (dyn Node + 'static)> {
        self.nodes.get_mut(id).map(|n| n.as_mut())
    }

    pub fn node_ids(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }

    

    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for id in self.nodes.keys() {
            in_degree.entry(id.clone()).or_insert(0);
            adj.entry(id.clone()).or_insert_with(Vec::new);
        }

        for edge in &self.edges {
            if let Some(neighbors) = adj.get_mut(&edge.source) {
                neighbors.push(edge.target.clone());
            }
            *in_degree.entry(edge.target.clone()).or_insert(0) += 1;
        }

        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut result = Vec::new();
        while let Some(node_id) = queue.pop_front() {
            result.push(node_id);
            if let Some(neighbors) = adj.get(result.last().unwrap()) {
                for neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            return Err("Graph contains a cycle".to_string());
        }

        Ok(result)
    }

    pub fn process_block(&mut self, block_size: BlockSize) -> Result<HashMap<String, AudioBuffer>, String> {
        if !self.order_valid {
            self.cached_order = self.topological_sort()?;
            self.order_valid = true;
        }
        let mut results: HashMap<String, AudioBuffer> = HashMap::new();

        for node_id in &self.cached_order {
            // Propagate instrument_json from upstream Instrument nodes via Instrument ports
            {
                let input_ports = self.nodes.get(node_id).map(|n| n.inputs());
                if let Some(ports) = &input_ports {
                    for (idx, port) in ports.iter().enumerate() {
                        if port.port_type == PortType::Instrument {
                            for edge in &self.edges {
                                if edge.target == *node_id && edge.target_handle == port.name {
                                    if let Some(src_node) = self.nodes.get(&edge.source) {
                                        let src_outputs = src_node.outputs();
                                        let src_port = src_outputs.iter().find(|p| p.name == edge.source_handle);
                                        if src_port.map_or(false, |p| p.port_type == PortType::Instrument) {
                                            if let Some(instrument_json) = src_node.get_param("instrument_json") {
                                                if let ParamValue::String(json_str) = &instrument_json {
                                                    if let Some(target_node) = self.nodes.get_mut(node_id) {
                                                        target_node.set_param("instrument_json", instrument_json.clone());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Read input ports first (immutable access)
            let num_inputs = self.nodes.get(node_id)
                .map(|n| n.inputs().len())
                .unwrap_or(0);

            // Build per-input buffers by routing edges to matching target_handle
            let mut per_input: Vec<Option<AudioBuffer>> = vec![None; num_inputs];
            let input_ports = self.nodes.get(node_id).map(|n| n.inputs());
            if let Some(ports) = &input_ports {
                for edge in &self.edges {
                    if edge.target == *node_id {
                        if let Some(upstream_buf) = results.get(&edge.source) {
                            let port_idx = ports.iter().position(|p| p.name == edge.target_handle);
                            if let Some(idx) = port_idx {
                                if per_input[idx].is_none() {
                                    per_input[idx] = Some(vec![0.0f32; block_size]);
                                }
                                if let Some(ref mut buf) = per_input[idx] {
                                    for i in 0..block_size {
                                        let val = upstream_buf.get(i).copied().unwrap_or(0.0);
                                        if i < buf.len() { buf[i] += val; }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Process with mutable access
            if let Some(node) = self.nodes.get_mut(node_id) {
                let output = node.process_multi(block_size, &per_input);
                results.insert(node_id.clone(), output);
            }
        }

        Ok(results)
    }

    pub fn to_state(&self, positions: &HashMap<String, [f64; 2]>) -> GraphState {
        GraphState {
            nodes: self.nodes.iter().map(|(id, node)| {
                let pos = positions.get(id).copied().unwrap_or([0.0, 0.0]);
                node.to_info(pos)
            }).collect(),
            edges: self.edges.clone(),
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.cached_order.clear();
        self.order_valid = false;
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}
