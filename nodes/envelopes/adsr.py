"""ADSR envelope generator."""
import numpy as np
from core.node import Node
from core.port import AudioPort, ControlPort, TriggerPort
from core import NodeRegistry


@NodeRegistry.register
class ADSR(Node):
    category = "Envelope"
    description = "Attack-Decay-Sustain-Release envelope"

    def _setup_ports(self):
        self._inputs["trigger"] = TriggerPort("trigger", self.id)
        self._inputs["audio"] = AudioPort("audio", self.id)
        self._outputs["envelope"] = AudioPort("envelope", self.id)

    def _setup_params(self):
        self._params["attack"] = 0.01
        self._params["decay"] = 0.1
        self._params["sustain"] = 0.6
        self._params["release"] = 0.3
        self._params["_state"] = "off"
        self._params["_level"] = 0.0
        self._params["_sample_in_state"] = 0

    def process(self, block_size: int) -> np.ndarray:
        attack = self._params.get("attack", 0.01)
        decay = self._params.get("decay", 0.1)
        sustain = self._params.get("sustain", 0.6)
        release = self._params.get("release", 0.3)
        state = self._params.get("_state", "off")
        level = self._params.get("_level", 0.0)
        sample_in_state = self._params.get("_sample_in_state", 0)
        sample_rate = 44100

        # Check for trigger
        if self._inputs["trigger"].is_connected or self._inputs["audio"].is_connected:
            if self._inputs["trigger"].value:
                state = "attack"
                sample_in_state = 0

        output = np.zeros(block_size, dtype=np.float32)

        for i in range(block_size):
            if state == "attack":
                rate = 1.0 / max(attack, 0.001)
                level += rate / sample_rate
                sample_in_state += 1
                if level >= 1.0:
                    level = 1.0
                    state = "decay"
                    sample_in_state = 0
            elif state == "decay":
                rate = (1.0 - sustain) / max(decay, 0.001)
                level -= rate / sample_rate
                sample_in_state += 1
                if level <= sustain:
                    level = sustain
                    state = "sustain"
                    sample_in_state = 0
            elif state == "sustain":
                sample_in_state += 1
                if self._inputs["trigger"].value:
                    state = "release"
                    sample_in_state = 0
            elif state == "release":
                rate = level / max(release, 0.001)
                level -= rate / sample_rate
                sample_in_state += 1
                if level <= 0.0:
                    level = 0.0
                    state = "off"

        if state == "off":
            level = 0.0

        output = np.full(block_size, level, dtype=np.float32)
        self._params["_state"] = state
        self._params["_level"] = level
        self._params["_sample_in_state"] = sample_in_state
        self._outputs["envelope"].set_signal(output)
        return output
