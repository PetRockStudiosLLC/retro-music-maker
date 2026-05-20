"""Step sequencer envelope node."""
import numpy as np
from core.node import Node
from core.port import AudioPort, ControlPort, TriggerPort
from core import NodeRegistry


@NodeRegistry.register
class EnvelopeSequencer(Node):
    category = "Envelope"
    description = "Step-by-step envelope pattern sequencer"

    def _setup_ports(self):
        self._inputs["clock"] = TriggerPort("clock", self.id)
        self._outputs["envelope"] = AudioPort("envelope", self.id)

    def _setup_params(self):
        self._params["steps"] = [1.0, 0.0, 0.5, 0.0, 1.0, 0.5, 0.0, 0.0]
        self._params["_current_step"] = 0

    def process(self, block_size: int) -> np.ndarray:
        steps = self._params.get("steps", [1.0] * 8)
        current_step = self._params.get("_current_step", 0)

        if self._inputs["clock"].value:
            current_step = (current_step + 1) % len(steps)
            self._params["_current_step"] = current_step

        level = steps[current_step % len(steps)]
        output = np.full(block_size, level, dtype=np.float32)
        self._outputs["envelope"].set_signal(output)
        return output
