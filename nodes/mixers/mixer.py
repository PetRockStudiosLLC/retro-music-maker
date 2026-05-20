"""N-channel mixer node."""
import numpy as np
from core.node import Node
from core.port import AudioPort
from core import NodeRegistry


@NodeRegistry.register
class Mixer(Node):
    category = "Mixer"
    description = "Mixes multiple audio inputs into one output"

    def _setup_ports(self):
        self._inputs["input1"] = AudioPort("input1", self.id)
        self._inputs["input2"] = AudioPort("input2", self.id)
        self._inputs["input3"] = AudioPort("input3", self.id)
        self._inputs["input4"] = AudioPort("input4", self.id)
        self._outputs["master"] = AudioPort("master", self.id)

    def _setup_params(self):
        self._params["vol1"] = 0.5
        self._params["vol2"] = 0.5
        self._params["vol3"] = 0.5
        self._params["vol4"] = 0.5

    def process(self, block_size: int) -> np.ndarray:
        v1 = self._params.get("vol1", 0.5)
        v2 = self._params.get("vol2", 0.5)
        v3 = self._params.get("vol3", 0.5)
        v4 = self._params.get("vol4", 0.5)

        signals = []
        for port in [self._inputs["input1"], self._inputs["input2"],
                     self._inputs["input3"], self._inputs["input4"]]:
            val = port.value
            if hasattr(val, '__len__') and len(val) > 0:
                signals.append(val)

        if not signals:
            output = np.zeros(block_size, dtype=np.float32)
        else:
            output = np.zeros(block_size, dtype=np.float32)
            for i, sig in enumerate(signals):
                vol = [v1, v2, v3, v4][i]
                if len(sig) < block_size:
                    padded = np.zeros(block_size, dtype=np.float32)
                    padded[:len(sig)] = sig
                    output += padded * vol
                else:
                    output += sig[:block_size] * vol

        output = np.clip(output, -1.0, 1.0)
        self._outputs["master"].set_signal(output)
        return output
