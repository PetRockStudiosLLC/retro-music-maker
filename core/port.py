import numpy as np
from typing import Optional, Callable


class Port:
    """Base class for all node ports."""

    def __init__(self, name: str, node_id: str, default=None):
        self.name = name
        self.node_id = node_id
        self._value = default
        self._connected: list = []  # List of (port, signal) tuples
        self._callbacks: list = []

    @property
    def value(self):
        if self._connected:
            signals = []
            for connected_port, signal in self._connected:
                if connected_port is self:
                    continue  # Avoid circular reference
                if signal is not None and hasattr(signal, '__len__') and len(signal) > 0:
                    signals.append(signal)
                elif hasattr(connected_port, '_value') and connected_port._value is not None:
                    signals.append(connected_port._value)
            if signals:
                combined = signals[0]
                for s in signals[1:]:
                    if hasattr(s, '__len__'):
                        combined = combined + s
                    else:
                        combined = s
                return combined
        return self._value

    @value.setter
    def value(self, val):
        self._value = val

    @property
    def is_connected(self):
        return len(self._connected) > 0

    def connect(self, other_port):
        other_port_id = id(other_port)
        existing_ids = [id(item[0]) for item in self._connected if isinstance(item, tuple)]
        if other_port_id not in existing_ids:
            self._connected.append((other_port, None))
            other_port._connected.append((self, None))
            for callback in self._callbacks:
                callback(True)

    def disconnect(self, other_port=None):
        if other_port is None:
            self._connected.clear()
        else:
            other_port_id = id(other_port)
            self._connected = [
                item for item in self._connected
                if id(item[0]) != other_port_id
            ]

    def on_connect(self, callback: Callable):
        self._callbacks.append(callback)

    def __repr__(self):
        return f"Port({self.type_name}: {self.name})"


class AudioPort(Port):
    """Audio signal port - handles arrays of samples."""

    type_name = "Audio"
    DEFAULT_SAMPLE_RATE = 44100

    def __init__(self, name: str, node_id: str, default: Optional[np.ndarray] = None):
        super().__init__(name, node_id, default)
        if self._value is None:
            self._value = np.array([], dtype=np.float32)

    def process_block(self, block_size: int):
        """Process a block of audio samples."""
        if self.is_connected:
            signals = [signal for _, signal in self._connected]
            if signals:
                combined = np.zeros(block_size, dtype=np.float32)
                for s in signals:
                    if hasattr(s, '__len__') and len(s) > 0:
                        if len(s) < block_size:
                            combined[:len(s)] += s
                        else:
                            combined += s[:block_size]
                return combined
        return self._value if hasattr(self._value, '__len__') else np.zeros(block_size, dtype=np.float32)

    def set_signal(self, signal: np.ndarray):
        """Set the audio signal for this port."""
        self._value = signal.astype(np.float32)

    def __repr__(self):
        return f"AudioPort({self.name}: {len(self._value)} samples)"


class ControlPort(Port):
    """Control signal port - single float value."""

    type_name = "Control"

    def __init__(self, name: str, node_id: str, default: float = 0.0):
        super().__init__(name, node_id, float(default))

    def set_value(self, value: float):
        self._value = float(value)

    def __repr__(self):
        return f"ControlPort({self.name}: {self._value})"


class TriggerPort(Port):
    """Trigger/On-Off port - boolean event."""

    type_name = "Trigger"

    def __init__(self, name: str, node_id: str, default: bool = False):
        super().__init__(name, node_id, default)

    def trigger(self):
        self._value = True
        # Auto-clear after one process cycle
        self._value = False

    def set_value(self, value: bool):
        self._value = bool(value)

    def __repr__(self):
        return f"TriggerPort({self.name}: {self._value})"
