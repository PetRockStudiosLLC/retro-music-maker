"""MIDI input handler for Retro Music Maker.

Listens for MIDI events and forwards them to the engine.
Supports note on/off, control change, program change, etc.
"""
import threading
import numpy as np
from typing import Callable, Optional, Dict


class MIDIInput:
    """Handles MIDI input events and routes them to callbacks."""

    def __init__(self, port_name: str = None, sample_rate: int = 44100):
        self.sample_rate = sample_rate
        self._port_name = port_name
        self._input = None
        self._running = False
        self._thread: Optional[threading.Thread] = None
        self._callbacks: Dict[str, list] = {
            "note_on": [],
            "note_off": [],
            "cc": [],
            "program_change": [],
        }
        self._active_notes: Dict[int, float] = {}  # channel -> frequency

    def start(self):
        """Start MIDI input."""
        import mido
        try:
            # List available input ports
            ports = mido.get_input_names()
            if not ports:
                raise RuntimeError("No MIDI inputs found")

            # Select port
            port_name = self._port_name or ports[0]
            if port_name not in ports:
                print(f"Port '{port_name}' not found. Available: {ports}")
                port_name = ports[0]

            self._input = mido.open_input(port_name)
            self._running = True
            self._thread = threading.Thread(target=self._listen_loop, daemon=True)
            self._thread.start()
        except Exception as e:
            raise RuntimeError(f"Failed to start MIDI input: {e}")

    def stop(self):
        """Stop MIDI input."""
        self._running = False
        if self._input:
            self._input.close()
            self._input = None
        if self._thread:
            self._thread.join(timeout=2.0)
            self._thread = None

    @property
    def is_running(self) -> bool:
        return self._running

    def on_note_on(self, callback: Callable):
        """Register a callback for note_on events.

        Callback signature: callback(note_number: int, velocity: int, channel: int)
        """
        self._callbacks["note_on"].append(callback)

    def on_note_off(self, callback: Callable):
        """Register a callback for note_off events."""
        self._callbacks["note_off"].append(callback)

    def on_cc(self, callback: Callable):
        """Register a callback for control change events."""
        self._callbacks["cc"].append(callback)

    def _listen_loop(self):
        """Listen for MIDI messages in a loop."""
        while self._running and self._input:
            for msg in self._input:
                if not self._running:
                    break
                self._handle_message(msg)

    def _handle_message(self, msg):
        """Route MIDI message to appropriate callback."""
        if msg.type == "note_on" and msg.velocity > 0:
            freq = self._note_to_freq(msg.note)
            for cb in self._callbacks["note_on"]:
                try:
                    cb(msg.note, msg.velocity, msg.channel)
                except Exception:
                    pass
        elif msg.type == "note_off" or (msg.type == "note_on" and msg.velocity == 0):
            for cb in self._callbacks["note_off"]:
                try:
                    cb(msg.note, 0, msg.channel)
                except Exception:
                    pass
        elif msg.type == "control_change":
            for cb in self._callbacks["cc"]:
                try:
                    cb(msg.controller, msg.value, msg.channel)
                except Exception:
                    pass
        elif msg.type == "program_change":
            for cb in self._callbacks["program_change"]:
                try:
                    cb(msg.program, msg.channel)
                except Exception:
                    pass

    @staticmethod
    def _note_to_freq(note: int) -> float:
        """Convert MIDI note number to frequency (A4 = 440 Hz)."""
        return 440.0 * (2.0 ** ((note - 69) / 12.0))

    @staticmethod
    def list_ports():
        """List available MIDI input ports."""
        try:
            import mido
            return mido.get_input_names()
        except Exception:
            return []
