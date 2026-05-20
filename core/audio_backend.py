"""Audio backend abstraction layer. Uses sounddevice for output."""
import numpy as np
import sounddevice as sd
from typing import Optional


class AudioBackend:
    """Manages audio output via sounddevice."""

    DEFAULT_SAMPLE_RATE = 44100
    DEFAULT_BLOCK_SIZE = 64
    DEFAULT_DEVICE = None

    def __init__(self, sample_rate: int = DEFAULT_SAMPLE_RATE,
                 block_size: int = DEFAULT_BLOCK_SIZE,
                 device: int = None):
        self.sample_rate = sample_rate
        self.block_size = block_size
        self._device = device or AudioBackend.DEFAULT_DEVICE
        self._stream = None
        self._running = False
        self._callback = None

    def start(self, callback=None):
        """Start the audio stream."""
        self._callback = callback
        self._stream = sd.OutputStream(
            samplerate=self.sample_rate,
            channels=1,
            blocksize=self.block_size,
            device=self._device,
            callback=self._audio_callback
        )
        self._stream.start()
        self._running = True

    def stop(self):
        """Stop the audio stream."""
        if self._stream:
            self._stream.stop()
            self._stream.close()
            self._stream = None
        self._running = False

    @property
    def is_running(self) -> bool:
        return self._running

    def _audio_callback(self, outdata: np.ndarray, frames: int,
                        time_info, status):
        """Audio callback for sounddevice."""
        if self._callback:
            try:
                audio_data = self._callback(frames)
                if audio_data is not None:
                    if isinstance(audio_data, np.ndarray):
                        if len(audio_data) < frames:
                            audio_data = np.pad(
                                audio_data, (0, frames - len(audio_data))
                            )
                        outdata[:, 0] = audio_data[:frames]
                    else:
                        outdata[:, 0] = 0
                else:
                    outdata.fill(0)
            except Exception:
                outdata.fill(0)
        else:
            outdata.fill(0)

    def get_device_info(self, index: int = None) -> dict:
        """Get info about an audio device."""
        devices = sd.query_devices()
        if index is not None:
            return devices[index]
        return devices

    def list_devices(self):
        """Print available audio devices."""
        devices = sd.query_devices()
        for i, dev in enumerate(devices):
            name = dev['name']
            inputs = dev['max_input_channels']
            outputs = dev['max_output_channels']
            print(f"  [{i}] {name} (in:{inputs}, out:{outputs})")

    def __enter__(self):
        self.start()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.stop()
        return False
