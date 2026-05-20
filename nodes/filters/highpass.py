"""Highpass filter node."""
import numpy as np
from core.node import Node
from core.port import AudioPort
from core import NodeRegistry


@NodeRegistry.register
class HighpassFilter(Node):
    category = "Filter"
    description = "Single-pole highpass filter"

    def _setup_ports(self):
        self._inputs["audio"] = AudioPort("audio", self.id)
        self._outputs["audio"] = AudioPort("audio", self.id)

    def _setup_params(self):
        self._params["cutoff"] = 1000.0
        self._params["resonance"] = 0.0
        self._params["_prev_input"] = 0.0
        self._params["_prev_output"] = 0.0

    def process(self, block_size: int) -> np.ndarray:
        audio_in = self._inputs["audio"].value
        if not hasattr(audio_in, '__len__') or len(audio_in) == 0:
            audio_in = np.zeros(block_size, dtype=np.float32)

        cutoff = self._params.get("cutoff", 1000.0)
        resonance = self._params.get("resonance", 0.0)
        prev_input = self._params.get("_prev_input", 0.0)
        prev_output = self._params.get("_prev_output", 0.0)
        sample_rate = 44100

        fc = max(min(cutoff, sample_rate / 2), 1.0)
        alpha = float(np.exp(-2.0 * np.pi * fc / sample_rate))

        output = np.zeros(block_size, dtype=np.float32)
        for i in range(block_size):
            sample = audio_in[i] if i < len(audio_in) else 0.0
            filtered = alpha * (prev_input + sample)
            hp = sample - filtered
            output[i] = hp
            prev_input = sample
            prev_output = hp

        self._params["_prev_input"] = prev_input
        self._params["_prev_output"] = prev_output
        self._outputs["audio"].set_signal(output)
        return output
