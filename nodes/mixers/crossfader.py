"""Crossfader node."""
import numpy as np
from core.node import Node
from core.port import AudioPort, ControlPort
from core import NodeRegistry


@NodeRegistry.register
class Crossfader(Node):
    category = "Mixer"
    description = "Crossfades between two audio inputs"

    def _setup_ports(self):
        self._inputs["input1"] = AudioPort("input1", self.id)
        self._inputs["input2"] = AudioPort("input2", self.id)
        self._outputs["output"] = AudioPort("output", self.id)

    def _setup_params(self):
        self._params["position"] = 0.5

    def process(self, block_size: int) -> np.ndarray:
        audio1 = self._inputs["input1"].value
        audio2 = self._inputs["input2"].value
        position = self._params.get("position", 0.5)

        if not hasattr(audio1, '__len__') or len(audio1) == 0:
            audio1 = np.zeros(block_size, dtype=np.float32)
        if not hasattr(audio2, '__len__') or len(audio2) == 0:
            audio2 = np.zeros(block_size, dtype=np.float32)

        position = np.clip(position, 0.0, 1.0)
        output = (audio1 * (1.0 - position) + audio2 * position).astype(np.float32)
        output = np.clip(output, -1.0, 1.0)
        self._outputs["output"].set_signal(output)
        return output
