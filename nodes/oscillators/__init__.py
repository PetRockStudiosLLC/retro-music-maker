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

    def process(self, block_size: int) -> np.ndarray:
        freq = self._params.get("freq", 440.0)
        amplitude = self._params.get("amplitude", 0.5)
        t = np.arange(block_size, dtype=np.float32) / self._params.get("_sr", 44100)
        # Store sample rate for reuse
        if "_sr" not in self._params:
            self._params["_sr"] = 44100
        phase = 2.0 * np.pi * freq * t
        output = amplitude * np.sin(phase)
        self._outputs["audio"].set_signal(output)
        return output
