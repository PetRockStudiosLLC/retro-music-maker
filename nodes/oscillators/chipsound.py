"""Chip sound oscillator - emulates retro game console waveforms."""
import numpy as np
from core.node import Node
from core.port import AudioPort, ControlPort
from core import NodeRegistry


@NodeRegistry.register
class ChipSoundOscillator(Node):
    category = "Oscillator"
    description = "Chip sound generator (NES/Gameboy/SNES style)"

    def _setup_ports(self):
        self._outputs["audio"] = AudioPort("audio", self.id)

    def _setup_params(self):
        self._params["freq"] = 440.0
        self._params["amplitude"] = 0.4
        self._params["waveform"] = "square"
        self._params["_phase"] = 0.0
        self._params["duty_cycle"] = 0.5

    def process(self, block_size: int) -> np.ndarray:
        freq = self._params.get("freq", 440.0)
        amplitude = self._params.get("amplitude", 0.4)
        waveform = self._params.get("waveform", "square")
        phase = self._params.get("_phase", 0.0)
        duty_cycle = self._params.get("duty_cycle", 0.5)
        sample_rate = 44100

        phase_increment = 2.0 * freq / sample_rate
        phase = phase + np.arange(block_size, dtype=np.float32) * phase_increment
        phase = phase % 2.0

        if waveform == "square":
            output = self._square_wave(phase, duty_cycle)
        elif waveform == "pulse":
            output = self._square_wave(phase, duty_cycle)
        elif waveform == "triangle":
            triangle = np.abs(2.0 * phase - 1.0) * 2.0 - 1.0
            output = triangle
        elif waveform == "sawtooth":
            output = 2.0 * (phase / 2.0) - 1.0
        elif waveform == "noise":
            output = np.random.rand(block_size) * 2.0 - 1.0
        else:
            output = np.sin(phase)

        output = (amplitude * output).astype(np.float32)
        self._params["_phase"] = phase[-1] if len(phase) > 0 else 0.0
        self._outputs["audio"].set_signal(output)
        return output

    def _square_wave(self, phase, duty_cycle):
        duty = np.clip(duty_cycle, 0.1, 0.9)
        return np.where(phase < (2.0 * duty), 1.0, -1.0).astype(np.float32)
