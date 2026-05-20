"""Tests for the node graph."""
from core.graph import Graph
from nodes.oscillators.square import SquareOscillator
from nodes.oscillators.sine import SineOscillator
from nodes.output.audio_out import AudioOutput
from nodes.mixers.mixer import Mixer


def test_create_graph():
    """Test creating a new graph."""
    graph = Graph()
    assert len(graph.nodes) == 0
    assert len(graph._edges) == 0


def test_add_remove_node():
    """Test adding and removing nodes."""
    graph = Graph()
    osc = SquareOscillator("test_osc")
    graph.add_node(osc)
    assert len(graph.nodes) == 1
    assert "test_osc" in graph.nodes

    graph.remove_node("test_osc")
    assert len(graph.nodes) == 0


def test_topological_order():
    """Test topological sorting of nodes."""
    graph = Graph()
    osc1 = SquareOscillator("osc1")
    osc2 = SineOscillator("osc2")
    mixer = Mixer("mixer")
    out = AudioOutput("out")

    graph.add_node(osc1)
    graph.add_node(osc2)
    graph.add_node(mixer)
    graph.add_node(out)

    graph.connect("osc1", "audio", "mixer", "input1")
    graph.connect("osc2", "audio", "mixer", "input2")
    graph.connect("mixer", "master", "out", "audio")

    order = graph.get_topological_order()
    assert "osc1" in order
    assert "osc2" in order
    assert "mixer" in order
    assert "out" in order
    # Mixer must come after both oscillators
    assert order.index("mixer") > order.index("osc1")
    assert order.index("mixer") > order.index("osc2")


def test_cycle_detection():
    """Test that cycles are detected."""
    graph = Graph()
    osc1 = SquareOscillator("osc1")
    osc2 = SquareOscillator("osc2")
    graph.add_node(osc1)
    graph.add_node(osc2)
    graph._edges = [("osc1", "osc2"), ("osc2", "osc1")]

    try:
        graph.get_topological_order()
        assert False, "Should have raised RuntimeError"
    except RuntimeError:
        pass  # Expected


def test_graph_serialization():
    """Test serializing and deserializing a graph."""
    graph = Graph()
    osc = SineOscillator("my_osc")
    osc.set_param("freq", 440.0)
    graph.add_node(osc)
    graph._edges.append(("my_osc", "out"))

    data = graph.to_dict()
    assert len(data["nodes"]) == 1

    restored = Graph.from_dict(data)
    assert len(restored.nodes) == 1
    assert "my_osc" in restored.nodes


def test_clear_graph():
    """Test clearing a graph."""
    graph = Graph()
    osc = SquareOscillator("osc")
    graph.add_node(osc)
    graph._edges.append(("osc", "out"))
    graph.clear()
    assert len(graph.nodes) == 0
    assert len(graph._edges) == 0
