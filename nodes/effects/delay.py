"""Delay effect node."""
import numpy as np
from core.node import Node
from core.port import AudioPort, ControlPort
from core import NodeRegistry


@NodeRegistry.register
class Delay(Node):
    category = "Effects"
    description = "Digital delay with feedback"

    def _setup_ports(self):
        self._inputs["audio"] = AudioPort("audio", self.id)
        self._outputs["audio"] = AudioPort("audio", self.id)

    def _setup_params(self):
        self._params["time"] = 0.3
        self._params["feedback"] = 0.4
        self._params["mix"] = 0.5
        self._params["_delay_buffer"] = np.zeros(44100 * 2, dtype=np.float32)
        self._params["_write_pos"] = 0

    def process(self, block_size: int) -> np.ndarray:
        audio_in = self._inputs["audio"].value
        if not hasattr(audio_in, '__len__') or len(audio_in) == 0:
            audio_in = np.zeros(block_size, dtype=np.float32)

        time_samples = int(self._params.get("time", 0.3) * 44100)
        feedback = self._params.get("feedback", 0.4)
        mix = self._params.get("mix", 0.5)
        delay_buffer = self._params.get("_delay_buffer", np.zeros(44100 * 2, dtype=np.float32))
        write_pos = self._params.get("_write_pos", 0)

        output = np.zeros(block_size, dtype=np.float32)
        for i in range(block_size):
            sample = audio_in[i] if i < len(audio_in) else 0.0
            read_pos = (write_pos - time_samples) % len(delay_buffer)
            delayed = delay_buffer[int(read_pos) % len(delay_buffer)]
            output[i] = sample * (1.0 - mix) + delayed * mix
            delay_buffer[int(write_pos) % len(delay_buffer)] = (
                sample + delayed * feedback
            )
            write_pos = (write_pos + 1) % len(delay_buffer)

        self._params["_delay_buffer"] = delay_buffer
        self._params["_write_pos"] = write_pos
        self._outputs["audio"].set_signal(output)
        return output
