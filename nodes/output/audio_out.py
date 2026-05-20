"""Audio output node - sends audio to speakers."""
import numpy as np
from core.node import Node
from core.port import AudioPort
from core import NodeRegistry


@NodeRegistry.register
class AudioOutput(Node):
    category = "Output"
    description = "Outputs audio to system speakers"

    def _setup_ports(self):
        self._inputs["audio"] = AudioPort("audio", self.id)

    def process(self, block_size: int) -> np.ndarray:
        audio = self._inputs["audio"].value
        if not hasattr(audio, '__len__') or len(audio) == 0:
            audio = np.zeros(block_size, dtype=np.float32)
        return audio.astype(np.float32)
