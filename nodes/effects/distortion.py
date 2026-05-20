"""Distortion effect node."""
import numpy as np
from core.node import Node
from core.port import AudioPort
from core import NodeRegistry


@NodeRegistry.register
class Distortion(Node):
    category = "Effects"
    description = "Wave shaper distortion"

    def _setup_ports(self):
        self._inputs["audio"] = AudioPort("audio", self.id)
        self._outputs["audio"] = AudioPort("audio", self.id)

    def _setup_params(self):
        self._params["drive"] = 1.0
        self._params["mix"] = 1.0

    def process(self, block_size: int) -> np.ndarray:
        audio_in = self._inputs["audio"].value
        if not hasattr(audio_in, '__len__') or len(audio_in) == 0:
            audio_in = np.zeros(block_size, dtype=np.float32)

        drive = self._params.get("drive", 1.0)
        mix = self._params.get("mix", 1.0)

        wet = np.tanh(audio_in * drive)
        output = audio_in * (1.0 - mix) + wet * mix
        output = (output * 0.8).astype(np.float32)
        self._outputs["audio"].set_signal(output)
        return output
