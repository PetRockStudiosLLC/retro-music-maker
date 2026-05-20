"""Step sequencer - classic step-based pattern sequencer."""
import numpy as np
from core.node import Node
from core.port import AudioPort, ControlPort, TriggerPort
from core import NodeRegistry


@NodeRegistry.register
class StepSequencer(Node):
    category = "Sequencer"
    description = "Step-based pattern sequencer with note control"

    def _setup_ports(self):
        self._inputs["clock"] = TriggerPort("clock", self.id)
        self._outputs["note"] = ControlPort("note", self.id)
        self._outputs["trigger"] = TriggerPort("trigger", self.id)

    def _setup_params(self):
        self._params["steps"] = [
            440.0, 0.0, 440.0, 0.0,
            523.25, 0.0, 523.25, 0.0,
        ]
        self._params["_current_step"] = 0
        self._params["speed"] = 4.0

    def process(self, block_size: int) -> np.ndarray:
        steps = self._params.get("steps", [440.0] * 8)
        current_step = self._params.get("_current_step", 0)

        if self._inputs["clock"].value:
            current_step = (current_step + 1) % len(steps)
            self._params["_current_step"] = current_step

        freq = steps[current_step % len(steps)]
        self._outputs["note"].set_value(freq)
        self._outputs["trigger"].trigger()

        return np.zeros(block_size, dtype=np.float32)
