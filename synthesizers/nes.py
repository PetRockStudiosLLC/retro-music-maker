"""NES (Nintendo Entertainment System) synthesizer.

Emulates the 5-channel NES Audio Processing Unit (APU):
- 2 Pulse waves (channel 1, 2)
- 1 Triangle wave (channel 3 - fixed pattern)
- 1 Noise channel (channel 4)
- 1 DMC channel (channel 5 - not emulated in basic version)
"""
import numpy as np
from core.node import Node
from core.port import AudioPort, ControlPort
from core import NodeRegistry


@NodeRegistry.register
class NESSynth(Node):
    category = "Synthesizer"
    description = "NES-style 5-channel synthesizer"

    def _setup_ports(self):
        self._inputs["note"] = ControlPort("note", self.id)
        self._inputs["enable"] = ControlPort("enable", self.id)
        self._outputs["audio"] = AudioPort("audio", self.id)

    def _setup_params(self):
        self._params["pulse_duty"] = 0.5
        self._params["amplitude"] = 0.4
        self._params["_pulse_phase"] = 0.0
        self._params["_triangle_phase"] = 0.0
        self._params["_noise_phase"] = 0

    def process(self, block_size: int) -> np.ndarray:
        note_freq = self._inputs["note"].value
        if not isinstance(note_freq, (int, float)):
            note_freq = float(note_freq[0]) if hasattr(note_freq, '__len__') and len(note_freq) > 0 else 440.0

        amplitude = self._params.get("amplitude", 0.4)
        sample_rate = 44100
        output = np.zeros(block_size, dtype=np.float32)

        # Channel 1: Pulse wave (duty cycle configurable)
        pulse_output = self._pulse_wave(block_size, note_freq, sample_rate)
        output += pulse_output

        # Channel 2: Triangle wave (fixed pattern, quarter frequency)
        tri_output = self._triangle_wave(block_size, note_freq * 0.25, sample_rate)
        output += tri_output * 0.5

        # Channel 3: Noise
        noise_output = self._noise_channel(block_size)
        output += noise_output * 0.3

        output = np.clip(output * amplitude * 0.3, -1.0, 1.0)
        self._outputs["audio"].set_signal(output)
        return output

    def _pulse_wave(self, size, freq, sample_rate):
        phase_inc = 2.0 * freq / sample_rate
        phase = np.arange(size, dtype=np.float32) * phase_inc
        phase = phase % 2.0
        duty = self._params.get("pulse_duty", 0.5)
        return np.where(phase < (2.0 * duty), 0.5, -0.5).astype(np.float32)

    def _triangle_wave(self, size, freq, sample_rate):
        if freq <= 0:
            return np.zeros(size, dtype=np.float32)
        phase = np.arange(size, dtype=np.float32) * 2.0 * freq / sample_rate
        phase = phase % 2.0
        return (np.abs(2.0 * phase - 1.0) * 2.0 - 1.0).astype(np.float32)

    def _noise_channel(self, size):
        return (np.random.rand(size) * 2.0 - 1.0).astype(np.float32)
