"""Tests for the core engine."""
import numpy as np
from core.engine import Engine
from core.graph import Graph
from nodes.oscillators.square import SquareOscillator
from nodes.oscillators.sine import SineOscillator
from nodes.output.audio_out import AudioOutput


def test_engine_creation():
    """Test engine can be created."""
    engine = Engine()
    assert engine is not None
    assert not engine.is_running


def test_add_node():
    """Test adding a node to the engine."""
    engine = Engine()
    osc = SquareOscillator()
    node_id = engine.add_node(osc)
    assert node_id == osc.id
    assert node_id in engine.list_nodes()


def test_connect_nodes():
    """Test connecting two nodes."""
    engine = Engine()
    osc = SquareOscillator("osc")
    out = AudioOutput("out")
    engine.add_node(osc)
    engine.add_node(out)
    engine.connect("osc", "audio", "out", "audio")
    assert "osc" in engine.list_nodes()
    assert "out" in engine.list_nodes()


def test_process_block():
    """Test processing a single block through the graph."""
    engine = Engine()
    osc = SquareOscillator("osc")
    out = AudioOutput("out")
    engine.add_node(osc)
    engine.add_node(out)
    engine.connect("osc", "audio", "out", "audio")

    # Set a frequency
    osc.set_param("freq", 440.0)
    osc.set_param("amplitude", 0.5)

    # Process a block
    results = engine.graph.process_block(64)
    assert "osc" in results
    assert len(results["osc"]) == 64


def test_graph_serialization():
    """Test serializing and deserializing a graph."""
    engine = Engine()
    osc = SineOscillator("my_osc")
    osc.set_param("freq", 880.0)
    engine.add_node(osc)

    data = engine.to_dict()
    assert len(data["nodes"]) == 1
    assert data["nodes"][0]["type"] == "SineOscillator"
    assert data["nodes"][0]["params"]["freq"] == 880.0


def test_multiple_nodes_process():
    """Test processing multiple connected nodes."""
    engine = Engine()
    osc1 = SquareOscillator("osc1")
    osc2 = SineOscillator("osc2")
    out = AudioOutput("out")
    engine.add_node(osc1)
    engine.add_node(osc2)
    engine.add_node(out)

    results = engine.graph.process_block(128)
    assert len(results) == 3
