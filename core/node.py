from typing import Dict, Any
import numpy as np
from core.port import AudioPort, ControlPort, TriggerPort, Port


class Node:
    """
    Base class for all nodes in the graph.
    Each node has a unique string ID, typed ports, and a process method.
    """

    category = "General"
    description = ""

    def __init__(self, node_id: str = None):
        self.id = node_id or self._generate_id()
        self.name = self.__class__.__name__
        self._inputs: Dict[str, Port] = {}
        self._outputs: Dict[str, Port] = {}
        self._params: Dict[str, Any] = {}
        self._setup_ports()
        self._setup_params()

    def _generate_id(self) -> str:
        """Generate a unique ID based on class name."""
        from core.graph import Graph
        # Will be set by the graph if a counter is available
        prefix = self.__class__.__name__.lower().replace("port", "")
        return f"{prefix}_000"

    def _setup_ports(self):
        """Override to define node ports."""
        pass

    def _setup_params(self):
        """Override to define node parameters."""
        pass

    def process(self, block_size: int) -> np.ndarray:
        """
        Process audio/control data.
        Returns numpy array of output samples.
        """
        return np.zeros(block_size, dtype=np.float32)

    def get_input(self, name: str) -> Port:
        """Get an input port by name."""
        return self._inputs[name]

    def get_output(self, name: str) -> Port:
        """Get an output port by name."""
        return self._outputs[name]

    def get_param(self, name: str):
        """Get a parameter value."""
        return self._params.get(name)

    def set_param(self, name: str, value):
        """Set a parameter value."""
        self._params[name] = value

    @property
    def inputs(self) -> Dict[str, Port]:
        return self._inputs

    @property
    def outputs(self) -> Dict[str, Port]:
        return self._outputs

    @property
    def params(self) -> Dict[str, Any]:
        return self._params

    def to_dict(self) -> dict:
        """Serialize node to dictionary."""
        data = {
            "id": self.id,
            "type": self.__class__.__name__,
            "category": self.category,
            "params": dict(self._params),
            "inputs": [{"name": n, "type": type(p).__name__} for n, p in self._inputs.items()],
            "outputs": [{"name": n, "type": type(p).__name__} for n, p in self._outputs.items()],
        }
        return data

    @classmethod
    def from_dict(cls, data: dict) -> 'Node':
        """Deserialize node from dictionary."""
        node = cls(node_id=data["id"])
        for param_name, value in data.get("params", {}).items():
            node._params[param_name] = value
        return node

    def __repr__(self):
        return f"<{self.__class__.__name__}: {self.id}>"
