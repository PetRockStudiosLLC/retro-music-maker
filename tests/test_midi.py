"""Tests for MIDI input functionality."""
from ui.midi_input import MIDIInput
from ui.midi_manager import MIDIManager
from core.engine import Engine
import numpy as np


def test_midi_note_to_freq():
    """Test MIDI note to frequency conversion."""
    assert abs(MIDIInput._note_to_freq(69) - 440.0) < 0.01
    assert abs(MIDIInput._note_to_freq(60) - 261.63) < 0.1
    assert abs(MIDIInput._note_to_freq(72) - 523.25) < 0.1


def test_midi_manager_creation():
    """Test MIDI manager initialization."""
    engine = Engine()
    manager = MIDIManager(engine)
    assert manager.engine is engine
    assert manager.midi is None


def test_midi_manager_get_ports():
    """Test getting MIDI port names."""
    ports = MIDIInput.list_ports()
    assert isinstance(ports, list)


def test_midi_manager_not_running():
    """Test that MIDI manager starts stopped."""
    engine = Engine()
    manager = MIDIManager(engine)
    assert manager.midi is None
