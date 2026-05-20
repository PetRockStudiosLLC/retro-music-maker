"""Bitcrush effect - reduces bit depth and sample rate for lo-fi retro sound."""
import numpy as np
from core.node import Node
from core.port import AudioPort, ControlPort
from core import NodeRegistry


@NodeRegistry.register
class Bitcrush(Node):
    category = "Filter"
    description = "Bit depth and sample rate reduction for retro effects"

    def _setup_ports(self):
        self._inputs["audio"] = AudioPort("audio", self.id)
        self._outputs["audio"] = AudioPort("audio", self.id)

    def _setup_params(self):
        self._params["bits"] = 8
        self._params["reduce_ratio"] = 1
        self._params["_sample_counter"] = 0

    def process(self, block_size: int) -> np.ndarray:
        audio_in = self._inputs["audio"].value
        if not hasattr(audio_in, '__len__') or len(audio_in) == 0:
            audio_in = np.zeros(block_size, dtype=np.float32)

        bits = self._params.get("bits", 8)
        reduce_ratio = max(int(self._params.get("reduce_ratio", 1)), 1)
        sample_counter = self._params.get("_sample_counter", 0)

        output = audio_in.copy()
        step = 2 ** (8 - bits) if bits < 8 else 1.0
        if step > 0:
            output = np.round(output / step) * step

        if reduce_ratio > 1:
            downsampled = np.zeros_like(output)
            for i in range(0, block_size, reduce_ratio):
                end = min(i + reduce_ratio, block_size)
                chunk = audio_in[i:end]
                value = float(np.mean(chunk)) if len(chunk) > 0 else 0.0
                downsampled[i:end] = value
            output = downsampled

        sample_counter = (sample_counter + block_size) % (reduce_ratio * 100)
        self._params["_sample_counter"] = sample_counter
        self._outputs["audio"].set_signal(output)
        return output
