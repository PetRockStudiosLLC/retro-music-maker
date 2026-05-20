import numpy as np
from typing import Dict, List, Optional, Set, Tuple


class Graph:
    """
    Node graph management using directed acyclic graph (DAG).
    Handles topological sorting for correct processing order.
    """

    def __init__(self):
        self.nodes: Dict[str, object] = {}  # id -> Node
        self._counters: Dict[str, int] = {}
        self._edges: List[Tuple[str, str]] = []  # (source_id, target_id)
        self._running = False

    def add_node(self, node) -> str:
        """Add a node to the graph. Returns the node's id."""
        if node.id in self.nodes:
            node.id = self._generate_unique_id(node.__class__.__name__)
            node.name = node.id
        self.nodes[node.id] = node
        return node.id

    def remove_node(self, node_id: str) -> bool:
        """Remove a node from the graph."""
        if node_id not in self.nodes:
            return False
        del self.nodes[node_id]
        self._edges = [(s, t) for s, t in self._edges if s != node_id and t != node_id]
        return True

    def connect(self, source_id: str, source_port: str, target_id: str, target_port: str):
        """Connect two nodes via their ports."""
        if source_id not in self.nodes or target_id not in self.nodes:
            raise ValueError(f"Node not found in graph: {source_id} or {target_id}")

        source_node = self.nodes[source_id]
        target_node = self.nodes[target_id]

        if source_port not in source_node.outputs:
            raise ValueError(
                f"Output port '{source_port}' not found on node '{source_id}'. "
                f"Available outputs: {list(source_node.outputs.keys())}"
            )
        if target_port not in target_node.inputs:
            raise ValueError(
                f"Input port '{target_port}' not found on node '{target_id}'. "
                f"Available inputs: {list(target_node.inputs.keys())}"
            )

        source_output = source_node.get_output(source_port)
        target_input = target_node.get_input(target_port)

        source_output.connect(target_input)
        self._edges.append((source_id, target_id))

    def disconnect(self, source_id: str, source_port: str, target_id: str, target_port: str):
        """Disconnect two nodes via their ports."""
        if source_id not in self.nodes or target_id not in self.nodes:
            return

        source_node = self.nodes[source_id]
        target_node = self.nodes[target_id]

        source_output = source_node.get_output(source_port)
        target_input = target_node.get_input(target_port)

        source_output.disconnect(target_input)
        self._edges = [(s, t) for s, t in self._edges
                       if not (s == source_id and t == target_id)]

    def get_topological_order(self) -> List[str]:
        """Get nodes in topological order for processing."""
        in_degree: Dict[str, int] = {nid: 0 for nid in self.nodes}
        adj: Dict[str, List[str]] = {nid: [] for nid in self.nodes}

        for source_id, target_id in self._edges:
            if target_id in adj:
                adj[source_id].append(target_id)
                in_degree[target_id] += 1

        queue = [nid for nid, deg in in_degree.items() if deg == 0]
        result = []

        while queue:
            queue.sort()
            node_id = queue.pop(0)
            result.append(node_id)
            for neighbor in adj[node_id]:
                in_degree[neighbor] -= 1
                if in_degree[neighbor] == 0:
                    queue.append(neighbor)

        if len(result) != len(self.nodes):
            raise RuntimeError("Graph contains a cycle!")

        return result

    def process_block(self, block_size: int) -> Dict[str, np.ndarray]:
        """Process one block of audio through the graph."""
        order = self.get_topological_order()
        results = {}

        for node_id in order:
            node = self.nodes[node_id]
            output = node.process(block_size)
            results[node_id] = output

        return results

    def get_outputs(self, block_size: int) -> Dict[str, np.ndarray]:
        """Get all output port values for external access."""
        self.process_block(block_size)
        outputs = {}
        for node_id, node in self.nodes.items():
            node_outputs = {}
            for port_name, port in node.outputs.items():
                node_outputs[port_name] = port.value.copy() if hasattr(port.value, 'copy') else np.array([port.value])
            outputs[node_id] = node_outputs
        return outputs

    def to_dict(self) -> dict:
        """Serialize the entire graph."""
        return {
            "nodes": [node.to_dict() for node in self.nodes.values()],
            "edges": list(self._edges),
        }

    @classmethod
    def from_dict(cls, data: dict) -> 'Graph':
        """Deserialize a graph from dictionary."""
        from core import NodeRegistry

        graph = cls()

        node_instances = {}
        for node_data in data["nodes"]:
            node_class = NodeRegistry.get_node_class(node_data["type"])
            node = node_class.from_dict(node_data)
            node_instances[node.id] = node
            graph.nodes[node.id] = node

        for edge in data.get("edges", []):
            if isinstance(edge, dict):
                source_id = edge["source"]
                target_id = edge["target"]
                source_port = edge.get("source_port", None)
                target_port = edge.get("target_port", None)
            else:
                source_id, target_id = edge[0], edge[1]
                source_port = edge[1] if len(edge) > 3 else None
                target_port = edge[3] if len(edge) > 3 else None

            source_node = graph.nodes.get(source_id)
            target_node = graph.nodes.get(target_id)
            if source_node is None or target_node is None:
                continue

            if source_port is None:
                source_port = next(iter(source_node.outputs.keys()), None)
            if target_port is None:
                target_port = next(iter(target_node.inputs.keys()), None)

            if source_port and target_port:
                try:
                    graph.connect(source_id, source_port, target_id, target_port)
                except (ValueError, KeyError):
                    pass

        return graph

    def _generate_unique_id(self, class_name: str) -> str:
        """Generate a unique ID with incrementing suffix."""
        prefix = class_name.lower().replace("node", "").replace("_", "")
        if prefix not in self._counters:
            self._counters[prefix] = 0
        else:
            self._counters[prefix] += 1

        counter = self._counters[prefix]
        return f"{prefix}_{counter:03d}"

    def start(self):
        """Mark graph as running."""
        self._running = True

    def stop(self):
        """Mark graph as stopped."""
        self._running = False

    @property
    def is_running(self) -> bool:
        return self._running

    def clear(self):
        """Clear all nodes and edges."""
        for node in self.nodes.values():
            for port in node.outputs.values():
                port.disconnect()
        self.nodes.clear()
        self._edges.clear()
        self._counters.clear()
