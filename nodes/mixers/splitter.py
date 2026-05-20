"""Splitter node - sends one input to multiple outputs."""
import numpy as np
from core.node import Node
from core.port import AudioPort
from core import NodeRegistry


@NodeRegistry.register
class Splitter(Node):
    category = "Mixer"
    description = "Splits one audio input into multiple identical outputs"

    def _setup_ports(self):
        self._inputs["input"] = AudioPort("input", self.id)
        self._outputs["out1"] = AudioPort("out1", self.id)
        self._outputs["out2"] = AudioPort("out2", self.id)

    def process(self, block_size: int) -> np.ndarray:
        audio = self._inputs["input"].value
        if not hasattr(audio, '__len__') or len(audio) == 0:
            audio = np.zeros(block_size, dtype=np.float32)

        output = audio.copy()
        self._outputs["out1"].set_signal(output)
        self._outputs["out2"].set_signal(output)
        return output
