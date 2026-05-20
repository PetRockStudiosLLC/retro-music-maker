"""Sine wave oscillator node."""
import numpy as np
from core.node import Node
from core.port import AudioPort, ControlPort
from core import NodeRegistry


@NodeRegistry.register
class SineOscillator(Node):
    category = "Oscillator"
    description = "Pure sine wave generator"

    def _setup_ports(self):
        self._outputs["audio"] = AudioPort("audio", self.id)

    def _setup_params(self):
        self._params["freq"] = 440.0
        self._params["amplitude"] = 0.5
        self._params["_phase"] = 0.0

    def process(self, block_size: int) -> np.ndarray:
        freq = self._params.get("freq", 440.0)
        amplitude = self._params.get("amplitude", 0.5)
        phase = self._params.get("_phase", 0.0)
        sample_rate = 44100

        t = np.arange(block_size, dtype=np.float32) / sample_rate
        phase_increment = 2.0 * np.pi * freq / sample_rate
        phase = phase + np.arange(block_size, dtype=np.float32) * phase_increment
        phase = phase % (2.0 * np.pi)

        output = amplitude * np.sin(phase)
        self._params["_phase"] = phase[-1] % (2.0 * np.pi) if len(phase) > 0 else 0.0
        self._outputs["audio"].set_signal(output)
        return output
