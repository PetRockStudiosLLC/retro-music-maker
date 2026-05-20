"""Noise oscillator node."""
import numpy as np
from core.node import Node
from core.port import AudioPort
from core import NodeRegistry


@NodeRegistry.register
class NoiseOscillator(Node):
    category = "Oscillator"
    description = "White noise generator"

    def _setup_ports(self):
        self._outputs["audio"] = AudioPort("audio", self.id)

    def _setup_params(self):
        self._params["amplitude"] = 0.3
        self._params["color"] = "white"

    def process(self, block_size: int) -> np.ndarray:
        amplitude = self._params.get("amplitude", 0.3)
        color = self._params.get("color", "white")
        noise = (np.random.rand(block_size) * 2.0 - 1.0).astype(np.float32)

        if color == "pink":
            # Simple pink noise approximation
            noise = self._pink_noise(block_size)
        elif color == "brown":
            # Simple brown noise approximation
            noise = self._brown_noise(block_size)

        output = (amplitude * noise).astype(np.float32)
        self._outputs["audio"].set_signal(output)
        return output

    def _pink_noise(self, size):
        """Generate pink noise using Paul's algorithm."""
        noise = np.random.randn(size).astype(np.float32)
        b = np.array([0.99886, 0.06564], dtype=np.float32)
        y = np.zeros(size, dtype=np.float32)
        w0 = 0.0
        w1 = 0.0
        for i in range(size):
            w0 = 0.99886 * w0 + noise[i] * 0.06564
            w1 = 0.99886 * w1 + noise[i] * 0.06564
            y[i] = w0 + w1 * 0.5
        return y

    def _brown_noise(self, size):
        """Generate brown noise."""
        noise = np.random.randn(size).astype(np.float32)
        last = 0.0
        output = np.zeros(size, dtype=np.float32)
        for i in range(size):
            last = (last + (0.02 * noise[i])) / 1.02
            output[i] = last * 3.5
        return output.astype(np.float32)
