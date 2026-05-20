"""Main audio engine that ties the graph, scheduler, and backend together."""
import threading
import numpy as np
from typing import Dict, Optional
from core.graph import Graph
from core.audio_backend import AudioBackend
from core.scheduler import Scheduler


class Engine:
    """Central audio engine managing graph processing and audio output."""

    def __init__(self, sample_rate: int = 44100, block_size: int = 64):
        self.sample_rate = sample_rate
        self.block_size = block_size
        self.graph = Graph()
        self.backend = AudioBackend(
            sample_rate=sample_rate,
            block_size=block_size
        )
        self.scheduler = Scheduler(sample_rate=sample_rate)
        self._running = False
        self._thread: Optional[threading.Thread] = None
        self._lock = threading.Lock()

    def start(self):
        """Start the audio engine."""
        if self._running:
            return
        self._running = True
        self.graph.start()
        self.backend.start(callback=self._audio_callback)

    def stop(self):
        """Stop the audio engine."""
        self._running = False
        self.graph.stop()
        self.backend.stop()
        if self._thread:
            self._thread.join(timeout=2.0)
            self._thread = None

    @property
    def is_running(self) -> bool:
        return self._running

    def _audio_callback(self, frames: int) -> np.ndarray:
        """Called by the audio backend for each audio block."""
        if not self._running:
            return np.zeros(frames, dtype=np.float32)

        try:
            results = self.graph.process_block(frames)
            self.scheduler.update(1, self.block_size)

            # Collect output from all nodes - sum all audio ports
            output = np.zeros(frames, dtype=np.float32)
            for node_id in results:
                node = self.graph.nodes.get(node_id)
                if not node:
                    continue
                for port_name, port in node.outputs.items():
                    if hasattr(port, 'value'):
                        port_signal = port.value
                        if hasattr(port_signal, '__len__') and len(port_signal) > 0:
                            if len(port_signal) < frames:
                                output[:len(port_signal)] += port_signal
                            else:
                                output += port_signal[:frames]

            # Apply gentle gain to prevent clipping
            output = np.clip(output * 0.5, -1.0, 1.0)
            return output.astype(np.float32)

        except Exception:
            return np.zeros(frames, dtype=np.float32)

    def add_node(self, node) -> str:
        """Add a node to the graph. Returns the node's id."""
        return self.graph.add_node(node)

    def remove_node(self, node_id: str) -> bool:
        """Remove a node from the graph."""
        return self.graph.remove_node(node_id)

    def connect(self, source_id: str, source_port: str,
                target_id: str, target_port: str):
        """Connect two nodes."""
        self.graph.connect(source_id, source_port, target_id, target_port)

    def disconnect(self, source_id: str, source_port: str,
                   target_id: str, target_port: str):
        """Disconnect two nodes."""
        self.graph.disconnect(source_id, source_port, target_id, target_port)

    def get_node(self, node_id: str):
        """Get a node by id."""
        return self.graph.nodes.get(node_id)

    def list_nodes(self) -> list:
        """List all node ids."""
        return list(self.graph.nodes.keys())

    def to_dict(self) -> dict:
        """Serialize the engine state."""
        return self.graph.to_dict()

    @classmethod
    def from_dict(cls, data: dict, sample_rate: int = 44100,
                  block_size: int = 64) -> 'Engine':
        """Create engine from serialized data."""
        engine = cls(sample_rate=sample_rate, block_size=block_size)
        engine.graph = Graph.from_dict(data)
        return engine
