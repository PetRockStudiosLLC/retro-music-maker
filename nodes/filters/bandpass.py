"""Bandpass filter node."""
import numpy as np
from core.node import Node
from core.port import AudioPort
from core import NodeRegistry


@NodeRegistry.register
class BandpassFilter(Node):
    category = "Filter"
    description = "Simple bandpass filter"

    def _setup_ports(self):
        self._inputs["audio"] = AudioPort("audio", self.id)
        self._outputs["audio"] = AudioPort("audio", self.id)

    def _setup_params(self):
        self._params["cutoff"] = 1000.0
        self._params["q"] = 1.0
        self._params["_prev_out1"] = 0.0
        self._params["_prev_out2"] = 0.0

    def process(self, block_size: int) -> np.ndarray:
        audio_in = self._inputs["audio"].value
        if not hasattr(audio_in, '__len__') or len(audio_in) == 0:
            audio_in = np.zeros(block_size, dtype=np.float32)

        cutoff = self._params.get("cutoff", 1000.0)
        q = self._params.get("q", 1.0)
        prev_out1 = self._params.get("_prev_out1", 0.0)
        prev_out2 = self._params.get("_prev_out2", 0.0)
        sample_rate = 44100

        fc = max(min(cutoff, sample_rate / 2), 1.0)
        alpha = float(np.exp(-2.0 * np.pi * fc / sample_rate))
        beta = alpha / q

        output = np.zeros(block_size, dtype=np.float32)
        for i in range(block_size):
            sample = audio_in[i] if i < len(audio_in) else 0.0
            out1 = alpha * sample + beta * prev_out1 - alpha * prev_out2
            out2 = prev_out1
            prev_out1 = out1
            prev_out2 = out2
            output[i] = (out1 - out2) * 0.5

        self._params["_prev_out1"] = prev_out1
        self._params["_prev_out2"] = prev_out2
        self._outputs["audio"].set_signal(output)
        return output
