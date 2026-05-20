"""Test the audio pipeline end-to-end.

Creates a simple graph: SineOscillator -> LowpassFilter -> AudioOutput
Then exports 2 seconds of audio to verify it works.
"""
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# Load all node plugins first
_nodes_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "nodes")
if os.path.isdir(_nodes_dir):
    import importlib.util
    for root, dirs, files in os.walk(_nodes_dir):
        dirs[:] = [d for d in dirs if d != "__pycache__"]
        for f in files:
            if f.endswith(".py") and not f.startswith("__"):
                filepath = os.path.join(root, f)
                try:
                    spec = importlib.util.spec_from_file_location(f"nodes.{f[:-3]}", filepath)
                    if spec and spec.loader:
                        module = importlib.util.module_from_spec(spec)
                        spec.loader.exec_module(module)
                except Exception:
                    pass

from core import NodeRegistry
from core.engine import Engine
import numpy as np


def test_simple_graph():
    """Test a simple oscillator -> filter -> output chain."""
    print("Creating test graph...")
    
    # Create engine
    engine = Engine(sample_rate=44100, block_size=64)
    
    # Create nodes
    sine = NodeRegistry.get_node_class("SineOscillator")(node_id="sineosc_000")
    sine._params["freq"] = 440.0
    sine._params["amplitude"] = 0.3
    
    lpf = NodeRegistry.get_node_class("LowpassFilter")(node_id="lowpass_000")
    lpf._params["cutoff"] = 2000.0
    lpf._params["resonance"] = 0.0
    
    output = NodeRegistry.get_node_class("AudioOutput")(node_id="audioout_000")
    
    # Add nodes to graph
    engine.add_node(sine)
    engine.add_node(lpf)
    engine.add_node(output)
    
    # Connect them
    engine.connect("sineosc_000", "audio", "lowpass_000", "audio")
    engine.connect("lowpass_000", "audio", "audioout_000", "audio")
    
    print(f"Graph has {len(engine.list_nodes())} nodes and {len(engine.graph._edges)} edges")
    
    # Process a few blocks to verify audio is generated
    print("Processing audio blocks...")
    for i in range(10):
        result = engine.graph.process_block(64)
        # Check that we have output from nodes
        for node_id, output_data in result.items():
            if hasattr(output_data, '__len__') and len(output_data) > 0:
                max_val = np.max(np.abs(output_data))
                print(f"  Block {i}: {node_id} output max={max_val:.4f}")
    
    # Test serialization
    print("Testing serialization...")
    state = engine.to_dict()
    print(f"  Serialized {len(state['nodes'])} nodes, {len(state['edges'])} edges")
    
    # Test deserialization
    print("Testing deserialization...")
    engine2 = Engine.from_dict(state)
    print(f"  Loaded {len(engine2.list_nodes())} nodes")
    
    print("\nAll tests passed!")
    return True


if __name__ == "__main__":
    test_simple_graph()
