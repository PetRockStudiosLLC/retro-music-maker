"""Chorus effect node."""
import numpy as np
from core.node import Node
from core.port import AudioPort
from core import NodeRegistry


@NodeRegistry.register
class Chorus(Node):
    category = "Effects"
    description = "Chorus effect with LFO modulation"

    def _setup_ports(self):
        self._inputs["audio"] = AudioPort("audio", self.id)
        self._outputs["audio"] = AudioPort("audio", self.id)

    def _setup_params(self):
        self._params["rate"] = 1.5
        self._params["depth"] = 0.003
        self._params["mix"] = 0.5
        self._params["_phase"] = 0.0
        self._params["_buffer"] = np.zeros(44100, dtype=np.float32)

    def process(self, block_size: int) -> np.ndarray:
        audio_in = self._inputs["audio"].value
        if not hasattr(audio_in, '__len__') or len(audio_in) == 0:
            audio_in = np.zeros(block_size, dtype=np.float32)

        rate = self._params.get("rate", 1.5)
        depth = self._params.get("depth", 0.003)
        mix = self._params.get("mix", 0.5)
        phase = self._params.get("_phase", 0.0)
        buffer = self._params.get("_buffer", np.zeros(44100, dtype=np.float32))
        sample_rate = 44100

        output = np.zeros(block_size, dtype=np.float32)
        for i in range(block_size):
            lfo = np.sin(2.0 * np.pi * rate * phase / sample_rate)
            delay = depth * lfo
            buf_idx = int(phase + delay * sample_rate) % len(buffer)
            output[i] = audio_in[i] * (1.0 - mix) + buffer[int(buf_idx)] * mix
            buffer[int(phase) % len(buffer)] = audio_in[i]
            phase += 1.0

        self._params["_phase"] = phase
        self._params["_buffer"] = buffer
        self._outputs["audio"].set_signal(output)
        return output
