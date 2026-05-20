from core.port import AudioPort, ControlPort, TriggerPort, Port
from core.node import Node
from core.graph import Graph
from core.engine import Engine
from core.scheduler import Scheduler
from core.audio_backend import AudioBackend


class NodeRegistry:
    """Registry for all known node types."""

    _registry: dict = {}

    @classmethod
    def register(cls, node_class):
        """Register a node class."""
        cls._registry[node_class.__name__] = node_class
        return node_class

    @classmethod
    def unregister(cls, node_name: str):
        """Unregister a node class."""
        cls._registry.pop(node_name, None)

    @classmethod
    def get_node_class(cls, node_name: str):
        """Get a node class by name."""
        if node_name in cls._registry:
            return cls._registry[node_name]
        raise KeyError(f"Unknown node type: {node_name}. Available: {list(cls._registry.keys())}")

    @classmethod
    def get_all_nodes(cls) -> dict:
        """Get all registered node classes."""
        return dict(cls._registry)

    @classmethod
    def load_plugins(cls, plugin_dir: str):
        """Load nodes from a plugin directory."""
        import os
        import importlib.util
        if not os.path.isdir(plugin_dir):
            return
        for filename in os.listdir(plugin_dir):
            if filename.endswith('.py'):
                filepath = os.path.join(plugin_dir, filename)
                spec = importlib.util.spec_from_file_location(filename[:-3], filepath)
                if spec and spec.loader:
                    module = importlib.util.module_from_spec(spec)
                    spec.loader.exec_module(module)
                    for attr_name in dir(module):
                        attr = getattr(module, attr_name)
                        if (isinstance(attr, type) and
                                hasattr(attr, '_is_node') and
                                attr.__name__ not in cls._registry):
                            cls.register(attr)
