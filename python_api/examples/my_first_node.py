"""Example: Creating your first custom node.

This demonstrates how easy it is to create a new node.
Copy this file to plugins/ and it will be auto-loaded!
"""
import numpy as np
from core.node import Node
from core.port import AudioPort, ControlPort
from core import NodeRegistry


@NodeRegistry.register
class MyFirstNode(Node):
    """A simple amplitude modulator - multiply audio by a control signal."""

    category = "Effects"
    description = "Amplitude modulation effect"

    def _setup_ports(self):
        self._inputs["audio"] = AudioPort("audio", self.id)
        self._inputs["modulation"] = ControlPort("modulation", self.id)
        self._outputs["audio"] = AudioPort("audio", self.id)

    def _setup_params(self):
        self._params["mix"] = 0.5

    def process(self, block_size: int) -> np.ndarray:
        audio = self._inputs["audio"].value
        modulation = self._inputs["modulation"].value
        mix = self._params.get("mix", 0.5)

        if not hasattr(audio, '__len__') or len(audio) == 0:
            audio = np.zeros(block_size, dtype=np.float32)

        if not hasattr(modulation, '__len__') or len(modulation) == 0:
            modulation = np.full(block_size, 0.5, dtype=np.float32)
        elif np.isscalar(modulation):
            modulation = np.full(block_size, float(modulation), dtype=np.float32)

        if len(audio) < block_size:
            audio = np.pad(audio, (0, block_size - len(audio)))
        if len(modulation) < block_size:
            modulation = np.pad(modulation, (0, block_size - len(modulation)))

        output = (audio * modulation).astype(np.float32)
        output = np.clip(output, -1.0, 1.0)
        self._outputs["audio"].set_signal(output)
        return output


@NodeRegistry.register
class RetroWahwah(Node):
    """A retro-styled wah-wah effect using a moving bandpass filter."""

    category = "Effects"
    description = "Retro wah-wah effect"

    def _setup_ports(self):
        self._inputs["audio"] = AudioPort("audio", self.id)
        self._outputs["audio"] = AudioPort("audio", self.id)

    def _setup_params(self):
        self._params["rate"] = 2.0
        self._params["depth"] = 2000.0
        self._params["center"] = 1000.0
        self._params["_phase"] = 0.0

    def process(self, block_size: int) -> np.ndarray:
        audio = self._inputs["audio"].value
        if not hasattr(audio, '__len__') or len(audio) == 0:
            audio = np.zeros(block_size, dtype=np.float32)

        rate = self._params.get("rate", 2.0)
        depth = self._params.get("depth", 2000.0)
        center = self._params.get("center", 1000.0)
        phase = self._params.get("_phase", 0.0)
        sample_rate = 44100

        output = np.zeros(block_size, dtype=np.float32)
        alpha = float(np.exp(-2.0 * np.pi * center / sample_rate))

        prev = self._params.get("_prev_output", 0.0)
        for i in range(block_size):
            lfo = 0.5 + 0.5 * np.sin(2.0 * np.pi * rate * phase / sample_rate)
            cutoff = center + depth * lfo
            a = float(np.exp(-2.0 * np.pi * cutoff / sample_rate))
            prev = prev + a * (audio[i] - prev)
            output[i] = prev
            prev = prev
            phase += 1.0

        self._params["_phase"] = phase
        self._params["_prev_output"] = prev
        self._outputs["audio"].set_signal(output)
        return output
