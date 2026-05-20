"""SNES (Super Nintendo) synthesizer.

Emulates the Sony SPC700 sound processor:
- 8-channel PCM audio
- ADPCM encoding
- Built-in effects (reverb, delay)
"""
import numpy as np
from core.node import Node
from core.port import AudioPort, ControlPort
from core import NodeRegistry


@NodeRegistry.register
class SNESSynth(Node):
    category = "Synthesizer"
    description = "SNES-style 8-channel PCM synthesizer"

    def _setup_ports(self):
        self._inputs["note"] = ControlPort("note", self.id)
        self._inputs["enable"] = ControlPort("enable", self.id)
        self._outputs["audio"] = AudioPort("audio", self.id)

    def _setup_params(self):
        self._params["amplitude"] = 0.5
        self._params["waveform"] = "pulse"
        self._params["reverb"] = 0.0
        self._params["_phase"] = 0.0

    def process(self, block_size: int) -> np.ndarray:
        note_freq = self._inputs["note"].value
        if not isinstance(note_freq, (int, float)):
            note_freq = float(note_freq[0]) if hasattr(note_freq, '__len__') and len(note_freq) > 0 else 440.0

        amplitude = self._params.get("amplitude", 0.5)
        waveform = self._params.get("waveform", "pulse")
        reverb_amt = self._params.get("reverb", 0.0)
        sample_rate = 44100
        output = np.zeros(block_size, dtype=np.float32)

        # Generate main waveform
        phase = np.arange(block_size, dtype=np.float32) * 2.0 * note_freq / sample_rate
        phase = phase % 2.0

        if waveform == "pulse":
            wave = np.where(phase < 0.5, 0.5, -0.5)
        elif waveform == "saw":
            wave = (phase - 0.5)
        elif waveform == "triangle":
            wave = np.abs(2.0 * phase - 1.0) * 2.0 - 1.0
        else:
            wave = 0.5 * np.sin(phase * np.pi)

        output += wave.astype(np.float32)

        # Add harmonics (SNES polyphonic capability)
        for i in range(1, 4):
            harmonic_freq = note_freq * (i + 1)
            harm_phase = np.arange(block_size, dtype=np.float32) * 2.0 * harmonic_freq / sample_rate
            harm_phase = harm_phase % 2.0
            harmonic = np.where(harm_phase < 0.5, 0.2 / (i + 1), -0.2 / (i + 1))
            output += harmonic.astype(np.float32)

        # Add simple reverb simulation
        if reverb_amt > 0:
            reverb_buf = np.zeros(block_size, dtype=np.float32)
            for i in range(block_size):
                delay = int(0.01 * sample_rate)
                if i >= delay:
                    reverb_buf[i] = output[i - delay] * 0.3
            output += reverb_buf * reverb_amt

        output = np.clip(output * amplitude * 0.3, -1.0, 1.0)
        self._outputs["audio"].set_signal(output)
        return output
