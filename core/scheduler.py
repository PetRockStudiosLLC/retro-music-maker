"""Sample-accurate timing and scheduling for the audio engine."""
import time
import threading
import numpy as np
from typing import Callable, Optional


class Scheduler:
    """Manages timing for sample-accurate audio processing."""

    def __init__(self, sample_rate: int = 44100):
        self.sample_rate = sample_rate
        self._current_sample = 0
        self._timer_events: list = []
        self._lock = threading.Lock()

    def reset(self):
        """Reset the scheduler."""
        self._current_sample = 0

    @property
    def current_sample(self) -> int:
        return self._current_sample

    @property
    def current_time(self) -> float:
        """Get current time in seconds."""
        return self._current_sample / self.sample_rate

    def add_callback(self, callback: Callable, delay_blocks: int,
                     block_size: int = 64, iterations: int = -1):
        """Schedule a callback to be called after a delay (in blocks)."""
        trigger_sample = self._current_sample + (delay_blocks * block_size)
        self._timer_events.append({
            'callback': callback,
            'trigger_sample': trigger_sample,
            'block_size': block_size,
            'iterations': iterations,
            'remaining': iterations,
            'active': True,
        })

    def add_timer(self, callback: Callable, interval_seconds: float,
                  iterations: int = -1) -> int:
        """Add a repeating timer. Returns timer id."""
        interval_blocks = int((interval_seconds * self.sample_rate) / 64)
        timer_id = len(self._timer_events)
        self.add_callback(callback, interval_blocks, 64, iterations)
        return timer_id

    def remove_timer(self, timer_id: int):
        """Remove a timer by id."""
        if timer_id < len(self._timer_events):
            self._timer_events[timer_id]['active'] = False

    def update(self, blocks_processed: int, block_size: int = 64):
        """Update scheduler state after processing blocks."""
        self._current_sample += blocks_processed * block_size
        with self._lock:
            for event in self._timer_events:
                if event['active'] and self._current_sample >= event['trigger_sample']:
                    event['callback']()
                    if event['iterations'] > 0:
                        event['remaining'] -= 1
                        if event['remaining'] <= 0:
                            event['active'] = False
                        else:
                            interval_blocks = event['trigger_sample'] // max(
                                self._current_sample, 1
                            ) * event['block_size']
                            event['trigger_sample'] += interval_blocks

    def get_active_timers(self) -> int:
        """Count active timers."""
        return sum(1 for e in self._timer_events if e['active'])
