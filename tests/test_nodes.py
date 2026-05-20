"""Tests for individual node types."""
import numpy as np
from nodes.oscillators.square import SquareOscillator
from nodes.oscillators.sine import SineOscillator
from nodes.oscillators.sawtooth import SawtoothOscillator
from nodes.oscillators.triangle import TriangleOscillator
from nodes.oscillators.noise import NoiseOscillator
from nodes.oscillators.chipsound import ChipSoundOscillator
from nodes.filters.lowpass import LowpassFilter
from nodes.filters.highpass import HighpassFilter
from nodes.filters.bitcrush import Bitcrush
from nodes.envelopes.adsr import ADSR
from nodes.effects.delay import Delay
from nodes.effects.distortion import Distortion
from nodes.mixers.mixer import Mixer
from nodes.mixers.splitter import Splitter
from nodes.input.lfo import LFO
from nodes.output.audio_out import AudioOutput


def test_square_oscillator():
    """Test square oscillator output."""
    osc = SquareOscillator()
    osc.set_param("freq", 440.0)
    osc.set_param("amplitude", 0.5)
    output = osc.process(64)
    assert len(output) == 64
    assert np.all(np.isfinite(output))
    assert np.max(np.abs(output)) <= 0.5


def test_sine_oscillator():
    """Test sine oscillator output."""
    osc = SineOscillator()
    osc.set_param("freq", 440.0)
    osc.set_param("amplitude", 0.5)
    output = osc.process(64)
    assert len(output) == 64
    assert np.all(np.isfinite(output))


def test_sawtooth_oscillator():
    """Test sawtooth oscillator."""
    osc = SawtoothOscillator()
    output = osc.process(64)
    assert len(output) == 64


def test_triangle_oscillator():
    """Test triangle oscillator."""
    osc = TriangleOscillator()
    output = osc.process(64)
    assert len(output) == 64


def test_noise_oscillator():
    """Test noise oscillator."""
    noise = NoiseOscillator()
    output = noise.process(64)
    assert len(output) == 64
    assert np.all(np.isfinite(output))


def test_chipsound_oscillator():
    """Test chip sound oscillator."""
    chip = ChipSoundOscillator()
    output = chip.process(64)
    assert len(output) == 64


def test_lowpass_filter():
    """Test lowpass filter."""
    filt = LowpassFilter()
    filt.set_param("cutoff", 1000.0)
    output = filt.process(64)
    assert len(output) == 64


def test_highpass_filter():
    """Test highpass filter."""
    filt = HighpassFilter()
    filt.set_param("cutoff", 1000.0)
    output = filt.process(64)
    assert len(output) == 64


def test_bitcrush():
    """Test bitcrush effect."""
    bc = Bitcrush()
    bc.set_param("bits", 8)
    output = bc.process(64)
    assert len(output) == 64


def test_adsr():
    """Test ADSR envelope."""
    adsr = ADSR()
    adsr.set_param("attack", 0.01)
    adsr.set_param("decay", 0.1)
    adsr.set_param("sustain", 0.6)
    adsr.set_param("release", 0.3)
    output = adsr.process(64)
    assert len(output) == 64
    assert np.all(np.isfinite(output))


def test_delay():
    """Test delay effect."""
    delay = Delay()
    delay.set_param("time", 0.3)
    delay.set_param("feedback", 0.4)
    output = delay.process(64)
    assert len(output) == 64


def test_distortion():
    """Test distortion effect."""
    dist = Distortion()
    dist.set_param("drive", 1.0)
    output = dist.process(64)
    assert len(output) == 64


def test_lfo():
    """Test LFO."""
    lfo = LFO()
    lfo.set_param("freq", 1.0)
    output = lfo.process(64)
    assert len(output) == 64


def test_mixer():
    """Test mixer."""
    mixer = Mixer()
    mixer.set_param("vol1", 0.5)
    mixer.set_param("vol2", 0.5)
    output = mixer.process(64)
    assert len(output) == 64


def test_splitter():
    """Test splitter."""
    splitter = Splitter()
    output = splitter.process(64)
    assert len(output) == 64


def test_audio_output():
    """Test audio output node."""
    out = AudioOutput()
    output = out.process(64)
    assert len(output) == 64


def test_node_serialization():
    """Test node serialization."""
    osc = SineOscillator("test_osc")
    osc.set_param("freq", 880.0)
    data = osc.to_dict()
    assert data["id"] == "test_osc"
    assert data["type"] == "SineOscillator"
    assert data["params"]["freq"] == 880.0

    restored = SineOscillator.from_dict(data)
    assert restored.id == "test_osc"
    assert restored._params["freq"] == 880.0
