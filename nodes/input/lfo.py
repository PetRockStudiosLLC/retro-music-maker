"""LFO (Low Frequency Oscillator) node for modulation."""
import numpy as np
from core.node import Node
from core.port import AudioPort
from core import NodeRegistry


@NodeRegistry.register
class LFO(Node):
    category = "Input"
    description = "Low frequency oscillator for modulation"

    def _setup_ports(self):
        self._outputs["output"] = AudioPort("output", self.id)

    def _setup_params(self):
        self._params["freq"] = 1.0
        self._params["amplitude"] = 0.5
        self._params["waveform"] = "sine"
        self._params["_phase"] = 0.0

    def process(self, block_size: int) -> np.ndarray:
        freq = self._params.get("freq", 1.0)
        amplitude = self._params.get("amplitude", 0.5)
        waveform = self._params.get("waveform", "sine")
        phase = self._params.get("_phase", 0.0)
        sample_rate = 44100

        phase_increment = 2.0 * freq / sample_rate
        phase = phase + np.arange(block_size, dtype=np.float32) * phase_increment
        phase = phase % 2.0

        if waveform == "sine":
            output = amplitude * np.sin(phase * np.pi)
        elif waveform == "square":
            output = amplitude * np.where(phase < 1.0, 1.0, -1.0)
        elif waveform == "sawtooth":
            output = amplitude * (2.0 * (phase / 2.0) - 1.0)
        elif waveform == "triangle":
            output = amplitude * (np.abs(2.0 * phase - 1.0) * 2.0 - 1.0)
        else:
            output = amplitude * np.sin(phase * np.pi)

        output = output.astype(np.float32)
        self._params["_phase"] = phase[-1] if len(phase) > 0 else 0.0
        self._outputs["output"].set_signal(output)
        return output
