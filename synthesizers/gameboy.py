"""Gameboy synthesizer.

Emulates the 3-channel Gameboy Audio Processing Unit (APU):
- Channel 1: Square wave with duty cycle and sweep
- Channel 2: Square wave with limited duty
- Channel 3: Wave pattern (16-step sample memory)
- Channel 4: Noise
"""
import numpy as np
from core.node import Node
from core.port import AudioPort, ControlPort
from core import NodeRegistry


@NodeRegistry.register
class GameboySynth(Node):
    category = "Synthesizer"
    description = "Gameboy-style 3-channel synthesizer"

    def _setup_ports(self):
        self._inputs["note"] = ControlPort("note", self.id)
        self._inputs["enable"] = ControlPort("enable", self.id)
        self._outputs["audio"] = AudioPort("audio", self.id)

    def _setup_params(self):
        self._params["amplitude"] = 0.5
        self._params["duty"] = 0.25
        self._params["_channel1_phase"] = 0.0
        self._params["_channel2_phase"] = 0.0
        self._params["_noise_counter"] = 0

    def process(self, block_size: int) -> np.ndarray:
        note_freq = self._inputs["note"].value
        if not isinstance(note_freq, (int, float)):
            note_freq = float(note_freq[0]) if hasattr(note_freq, '__len__') and len(note_freq) > 0 else 440.0

        amplitude = self._params.get("amplitude", 0.5)
        sample_rate = 44100
        duty = self._params.get("duty", 0.25)
        output = np.zeros(block_size, dtype=np.float32)

        # Channel 1: Main square wave with duty cycle
        ch1 = self._channel1(block_size, note_freq, sample_rate, duty)
        output += ch1

        # Channel 2: Secondary square wave (one octave higher, quieter)
        ch2 = self._channel1(block_size, note_freq * 1.5, sample_rate, 0.125)
        output += ch2 * 0.4

        # Channel 3: Wave pattern memory (Gameboy specific)
        ch3 = self._wave_channel(block_size, note_freq * 0.5, sample_rate)
        output += ch3 * 0.3

        # Channel 4: Noise
        ch4 = self._noise_channel(block_size)
        output += ch4 * 0.15

        output = np.clip(output * amplitude * 0.5, -1.0, 1.0)
        self._outputs["audio"].set_signal(output)
        return output

    def _channel1(self, size, freq, sample_rate, duty):
        phase = np.arange(size, dtype=np.float32) * 2.0 * freq / sample_rate
        phase = phase % 2.0
        return np.where(phase < (2.0 * duty), 0.5, -0.5).astype(np.float32)

    def _wave_channel(self, size, freq, sample_rate):
        if freq <= 0:
            return np.zeros(size, dtype=np.float32)
        # Gameboy wave pattern - simplified
        wave_pattern = np.array([
            0.0, 0.0, 0.25, 0.0, 0.5, 0.75, 0.5, 0.0,
            0.0, 0.5, 1.0, 0.5, 0.0, 0.5, 0.25, 0.0
        ])
        phase = np.arange(size, dtype=np.float32) * 2.0 * freq / sample_rate
        indices = (phase * 16 / 2.0).astype(int) % 16
        return wave_pattern[indices].astype(np.float32)

    def _noise_channel(self, size):
        return (np.random.rand(size) * 2.0 - 1.0).astype(np.float32)
