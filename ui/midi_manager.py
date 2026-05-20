"""MIDI integration for Retro Music Maker GUI.

Maps MIDI events to node parameters for real-time control.
"""
import numpy as np
from ui.midi_input import MIDIInput


class MIDIManager:
    """Manages MIDI input and maps it to node parameters."""

    def __init__(self, engine):
        self.engine = engine
        self.midi = None
        self._node_mappings = {}  # midi_note -> (node_id, param_name)

    def start(self, port_name: str = None):
        """Start MIDI input."""
        try:
            self.midi = MIDIInput(port_name=port_name)
            self.midi.on_note_on(self._on_note_on)
            self.midi.on_note_off(self._on_note_off)
            self.midi.start()
            return True
        except Exception as e:
            print(f"MIDI init warning: {e}")
            return False

    def stop(self):
        """Stop MIDI input."""
        if self.midi:
            self.midi.stop()

    def _on_note_on(self, note, velocity, channel):
        """Handle MIDI note_on by setting note freq on oscillator nodes."""
        freq = MIDIInput._note_to_freq(note)
        for nid in self.engine.list_nodes():
            node = self.engine.get_node(nid)
            if node and hasattr(node, 'get_param'):
                if any(kw in node.__class__.__name__.lower() for kw in
                       ['oscillator', 'osc', 'synth']):
                    node.set_param("freq", freq)

    def _on_note_off(self, note, velocity, channel):
        """Handle MIDI note_off."""
        pass

    def get_active_port_names(self):
        """Get list of available MIDI ports."""
        return MIDIInput.list_ports()


def get_midi_port_names():
    """Convenience function to list MIDI ports."""
    return MIDIInput.list_ports()
