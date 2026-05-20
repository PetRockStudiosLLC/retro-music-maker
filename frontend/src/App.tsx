/** Main application component - Tauri native */
/** Main application component - Tauri native */
import { useCallback, useMemo, useRef, useState, useEffect, Fragment } from "react";
import { createPortal } from "react-dom";
import { invoke } from '@tauri-apps/api/core';
import { ToastProvider, useToast } from "./hooks/useToast";

const PORT_EDGE_COLORS: Record<string, string> = {
  AudioPort: "#9ece6a",
  ControlPort: "#e0af68",
  TriggerPort: "#f7768e",
  InstrumentPort: "#bb9af7",
  Audio: "#9ece6a",
  Control: "#e0af68",
  Trigger: "#f7768e",
  Instrument: "#bb9af7",
  audio: "#9ece6a",
  control: "#e0af68",
  trigger: "#f7768e",
  instrument: "#bb9af7",
};

import {
  ReactFlow,
  Controls,
  Background,
  Connection,
  addEdge,
  applyNodeChanges,
  applyEdgeChanges,
  type Node,
  type Edge,
  type NodeChange,
  type EdgeChange,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import CustomNode from "./components/CustomNode";
import NoteMapperNode from "./components/NoteMapperNode";
import InstrumentNode from "./components/InstrumentNode";
import Palette from "./components/Palette";
import Console from "./components/Console";
import TemplateEditor from "./components/TemplateEditor";
import ScriptEditor from "./components/ScriptEditor";
import ExportWavModal from "./components/ExportWavModal";
import type { NodeData, ConsoleMessage, AudioStatus } from "./types/nodes";
import { useTauriGraph, addNode as apiAddNode, removeNode as apiRemoveNode, connectNodes as apiConnect, disconnectNodes as apiDisconnect, setParam as apiSetParam } from "./tauri/useTauriGraph";
import { listTemplates, listScripts } from "./tauri/api";
import { setPushUndoCallback, getPushUndoCallback } from "./undoBridge";

const nodeTypes = { custom: CustomNode as any, noteMapper: NoteMapperNode as any, instrument: InstrumentNode as any };

const NODE_TYPE_DEFS: Record<string, { inputs: any[]; outputs: any[]; params: Record<string, any>; category: string; description: string }> = {
  SineOscillator: { category: "Oscillator", inputs: [], outputs: [{ name: "audio", port_type: "audio" }], params: { frequency: { type: "number", default: 440, min: 20, max: 8000 }, detune: { type: "number", default: 0, min: -100, max: 100 } }, description: "Pure sine wave tone. Smooth and clean." },
  SquareOscillator: { category: "Oscillator", inputs: [], outputs: [{ name: "audio", port_type: "audio" }], params: { frequency: { type: "number", default: 440, min: 20, max: 8000 }, duty: { type: "number", default: 0.5, min: 0, max: 1 } }, description: "Hollow, retro 8-bit sound. Classic chiptune lead." },
  SawtoothOscillator: { category: "Oscillator", inputs: [], outputs: [{ name: "audio", port_type: "audio" }], params: { frequency: { type: "number", default: 440, min: 20, max: 8000 } }, description: "Bright, buzzy tone. Great for bass and leads." },
  TriangleOscillator: { category: "Oscillator", inputs: [], outputs: [{ name: "audio", port_type: "audio" }], params: { frequency: { type: "number", default: 440, min: 20, max: 8000 } }, description: "Soft, mellow tone. Good for bass lines." },
  NoiseOscillator: { category: "Oscillator", inputs: [], outputs: [{ name: "audio", port_type: "audio" }], params: {}, description: "Random noise. Use for drums, percussion, FX." },
  ChipSoundOscillator: { category: "Oscillator", inputs: [], outputs: [{ name: "audio", port_type: "audio" }], params: { frequency: { type: "number", default: 440, min: 20, max: 8000 }, wave: { type: "select", default: "square", options: ["square", "triangle", "noise", "dpcm"] } }, description: "Multi-waveform retro synth. Switch between square, triangle, noise, DPCM." },
  VCO: { category: "Oscillator", inputs: [{ name: "note", port_type: "control" }], outputs: [{ name: "audio", port_type: "audio" }], params: { waveform: { type: "select", default: "sine", options: ["sine", "square", "sawtooth", "triangle"] }, amplitude: { type: "number", default: 0.3, min: 0, max: 1 }, note: { type: "number", default: 69, min: 0, max: 127 } }, description: "Voltage-controlled oscillator. Takes MIDI note numbers from control input, outputs audio." },
  Waveform: { category: "Oscillator", inputs: [{ name: "note", port_type: "control" }], outputs: [{ name: "audio", port_type: "audio" }], params: { waveform: { type: "select", default: "sine", options: ["sine", "square", "sawtooth", "triangle", "pulse", "pulse_half", "noise", "sample_hold"] }, amplitude: { type: "number", default: 0.3, min: 0, max: 1 }, note: { type: "number", default: 69, min: 0, max: 127 } }, description: "Full-featured waveform oscillator with 8 waveforms including noise and sample-and-hold. Outputs audio." },
  LowpassFilter: { category: "Filter", inputs: [{ name: "audio", port_type: "audio" }, { name: "cutoff", port_type: "control" }], outputs: [{ name: "audio", port_type: "audio" }], params: { cutoff: { type: "number", default: 2000, min: 20, max: 20000 }, resonance: { type: "number", default: 1, min: 0.1, max: 20 } }, description: "Removes high frequencies. Warm, muffled sound." },
  HighpassFilter: { category: "Filter", inputs: [{ name: "audio", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }], params: { cutoff: { type: "number", default: 500, min: 20, max: 20000 }, resonance: { type: "number", default: 1, min: 0.1, max: 20 } }, description: "Removes low frequencies. Thin, bright sound." },
  BandpassFilter: { category: "Filter", inputs: [{ name: "audio", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }], params: { cutoff: { type: "number", default: 1000, min: 20, max: 20000 }, resonance: { type: "number", default: 1, min: 0.1, max: 20 } }, description: "Only passes a frequency band. Telephone/robot FX." },
  Bitcrush: { category: "Filter", inputs: [{ name: "audio", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }], params: { bitDepth: { type: "number", default: 8, min: 1, max: 16 }, sampleRate: { type: "number", default: 44100, min: 100, max: 44100 } }, description: "Reduces bit depth & sample rate. Lo-fi retro crunch." },
  ADSREnvelope: { category: "Envelope", inputs: [{ name: "audio", port_type: "audio" }, { name: "gate", port_type: "trigger" }], outputs: [{ name: "envelope", port_type: "control" }], params: { attack: { type: "number", default: 0.01, min: 0.001, max: 5 }, decay: { type: "number", default: 0.1, min: 0.001, max: 5 }, sustain: { type: "number", default: 0.7, min: 0, max: 1 }, release: { type: "number", default: 0.3, min: 0.001, max: 10 } }, description: "Shapes volume over time: Attack, Decay, Sustain, Release." },
  DelayEffect: { category: "Effects", inputs: [{ name: "audio", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }], params: { delayTime: { type: "number", default: 0.3, min: 0.001, max: 2 }, feedback: { type: "number", default: 0.4, min: 0, max: 0.95 }, mix: { type: "number", default: 0.3, min: 0, max: 1 } }, description: "Echo/repeat effect. Creates spatial depth." },
  Distortion: { category: "Effects", inputs: [{ name: "audio", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }], params: { amount: { type: "number", default: 0.5, min: 0, max: 1 } }, description: "Adds grit and aggression. Overdrive/ distortion." },
  Reverb: { category: "Effects", inputs: [{ name: "audio", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }], params: { decay: { type: "number", default: 2, min: 0.1, max: 10 }, mix: { type: "number", default: 0.3, min: 0, max: 1 } }, description: "Simulates room/space acoustics. Adds ambiance." },
  DurationGate: { category: "Utility", inputs: [{ name: "audio", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }], params: { duration: { type: "number", default: 1, min: 0.01, max: 60 }, auto_restart: { type: "bool", default: false } }, description: "Lets audio pass for a set duration, then mutes. Controls how long each slot plays." },
  Loop: { category: "Effect", inputs: [{ name: "audio", port_type: "audio" }, { name: "trigger", port_type: "trigger" }], outputs: [{ name: "audio", port_type: "audio" }], params: { loop_duration: { type: "number", default: 2, min: 0.1, max: 10 } }, description: "Records incoming audio, then loops the recorded segment." },
  Mixer: { category: "Mixer", inputs: [{ name: "input_0", port_type: "audio" }, { name: "input_1", port_type: "audio" }, { name: "input_2", port_type: "audio" }, { name: "input_3", port_type: "audio" }], outputs: [{ name: "output", port_type: "audio" }], params: {}, description: "Combines up to 4 audio inputs into one output." },
  Gain: { category: "Mixer", inputs: [{ name: "audio", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }], params: { gain: { type: "number", default: 1, min: 0, max: 10 } }, description: "Adjusts volume level. Amplify or attenuate." },
  KeyboardInput: { category: "Input", inputs: [], outputs: [{ name: "audio", port_type: "audio" }], params: { velocity: { type: "number", default: 0.7, min: 0, max: 1 } }, description: "Computer keyboard as MIDI input. Play notes with keys." },
  MidiInput: { category: "Input", inputs: [], outputs: [{ name: "audio", port_type: "audio" }], params: {}, description: "External MIDI device input for note control." },
  WavFile: { category: "Input", inputs: [{ name: "trigger", port_type: "trigger" }, { name: "volume", port_type: "control" }], outputs: [{ name: "audio", port_type: "audio" }], params: { path: { type: "text", default: "" }, loop_enabled: { type: "bool", default: false } }, description: "Load and play a WAV audio file. Trigger to play, loop to repeat." },
  AudioOutput: { category: "Output", inputs: [{ name: "audio", port_type: "audio" }], outputs: [], params: { volume: { type: "number", default: 0.8, min: 0, max: 1 } }, description: "Final audio output to your speakers/headphones." },
  FileOutput: { category: "Output", inputs: [{ name: "audio", port_type: "audio" }], outputs: [], params: { path: { type: "text", default: "output.wav" } }, description: "Record audio to a WAV file on disk." },
  StepSequencer: { category: "Sequencer", inputs: [], outputs: [{ name: "trigger", port_type: "trigger" }], params: { tempo: { type: "number", default: 120, min: 40, max: 300 }, steps: { type: "number", default: 16, min: 4, max: 32 } }, description: "Rhythm sequencer. Triggers beats on a grid." },
  Arpeggiator: { category: "Sequencer", inputs: [{ name: "audio", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }], params: { tempo: { type: "number", default: 120, min: 40, max: 300 }, octaveRange: { type: "number", default: 2, min: 1, max: 4 } }, description: "Plays notes in ascending/descending patterns." },
  NoteSequencer: { category: "Sequencer", inputs: [], outputs: [{ name: "audio", port_type: "audio" }], params: { bpm: { type: "number", default: 140, min: 40, max: 300 }, notes: { type: "text", default: "[60, 64, 67, 72, 67, 64, 60, 0]" }, waveform: { type: "select", default: "square", options: ["square", "sawtooth", "triangle", "sine"] }, attack: { type: "number", default: 0.002, min: 0.001, max: 2 }, decay: { type: "number", default: 0.05, min: 0.001, max: 2 }, sustain: { type: "number", default: 0.3, min: 0, max: 1 }, release: { type: "number", default: 0.05, min: 0.001, max: 2 }, amplitude: { type: "number", default: 0.5, min: 0, max: 1 } }, description: "Self-contained melody player. Set notes as MIDI numbers (0=rest)." },
  NoteMapper: { category: "Sequencer", inputs: [{ name: "instrument", port_type: "instrument" }], outputs: [{ name: "audio", port_type: "audio" }], params: { bpm: { type: "number", default: 140, min: 40, max: 300 }, grid: { type: "text", default: "[;;;;;;;;;;;;;;;;]" }, num_steps: { type: "number", default: 16, min: 4, max: 128 }, min_midi: { type: "number", default: 48, min: 24, max: 72 }, instrument: { type: "select", default: "square_lead", options: ["square_lead", "square_bass", "saw_lead", "saw_bass", "triangle_pad", "sine_pad", "chip_punch", "chip_arp"] }, waveform: { type: "select", default: "square", options: ["square", "sawtooth", "triangle", "sine", "pulse", "pulse_half", "noise", "sample_hold"] }, attack: { type: "number", default: 0.002, min: 0.001, max: 2 }, decay: { type: "number", default: 0.05, min: 0.001, max: 2 }, sustain: { type: "number", default: 0.3, min: 0, max: 1 }, release: { type: "number", default: 0.05, min: 0.001, max: 2 }, amplitude: { type: "number", default: 0.5, min: 0, max: 1 } }, description: "FL Studio-style piano roll. Click cells to place notes on a visual grid." },
  SlotPlayer: { category: "Sequencer", inputs: [{ name: "input_0", port_type: "audio" }, { name: "input_1", port_type: "audio" }, { name: "input_2", port_type: "audio" }, { name: "input_3", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }], params: { num_slots: { type: "number", default: 4, min: 1, max: 16 } }, description: "Routes audio from connected inputs sequentially. Advances to next slot when current input goes silent." },
  Clock: { category: "Trigger", inputs: [], outputs: [{ name: "beat", port_type: "trigger" }, { name: "tick", port_type: "trigger" }], params: { bpm: { type: "number", default: 120, min: 20, max: 300 }, swing: { type: "number", default: 0, min: 0, max: 1 } }, description: "Fires beat and tick triggers at a set BPM. Like a metronome." },
  RandomTrigger: { category: "Trigger", inputs: [], outputs: [{ name: "trigger", port_type: "trigger" }], params: { min_interval: { type: "number", default: 0.5, min: 0.01, max: 10 }, max_interval: { type: "number", default: 2, min: 0.01, max: 10 } }, description: "Fires triggers at random intervals between min and max seconds." },
  TriggerDelay: { category: "Trigger", inputs: [{ name: "trigger", port_type: "trigger" }], outputs: [{ name: "trigger", port_type: "trigger" }], params: { delay_time: { type: "number", default: 0.5, min: 0.01, max: 10 } }, description: "Delays an incoming trigger by a set amount of time." },
  Instrument: { category: "Controller", inputs: [{ name: "trigger", port_type: "control" }, { name: "trigger_in", port_type: "trigger" }], outputs: [{ name: "instrument", port_type: "instrument" }, { name: "trigger", port_type: "trigger" }], params: { waveform: { type: "select", default: "square", options: ["square", "sawtooth", "triangle", "sine", "pulse", "pulse_half", "noise", "sample_hold"] }, attack: { type: "number", default: 0.002, min: 0.001, max: 2 }, decay: { type: "number", default: 0.05, min: 0.001, max: 2 }, sustain: { type: "number", default: 0.3, min: 0, max: 1 }, release: { type: "number", default: 0.05, min: 0.001, max: 2 }, amplitude: { type: "number", default: 0.5, min: 0, max: 1 } }, description: "Defines instrument sound (waveform + ADSR). Connect to NoteMapper to change its sound." },
};

const paletteNodeTypes = Object.fromEntries(
  Object.entries(NODE_TYPE_DEFS).map(([name, def]) => [
    name,
    { name, category: def.category, inputs: def.inputs, outputs: def.outputs, params: def.params },
  ])
) as Record<string, any>;

const demoPresets: Record<string, { label: string; category: string; nodes: any[]; edges: any[] }> = {
  // --- SONG PRESETS (NoteSequencer-based) ---
  mario: {
    category: "🎵 Songs",
    label: "Super Mario Bros - Overworld Theme",
    nodes: [
      { id: "melody", name: "NoteSequencer", category: "Sequencer", params: {
        bpm: 200,
        notes: "[76,76,0,76,0,72,76,0,79,67,0,0,0,0,64,0,69,71,71,0,71,0,69,0,67,0,76,72,73,71,0,67,0,69,71,0,69,0,67,0,76,79,80,0,76,72,73,71,0,67,0,69,71,0,69,0,67,0,76,79,80,0,76,72,73,71,0,67,0,69,71,0,69,0,67,0,76,79,80,0,76,72,73,71,0,67,64,72,73,71,0,67]",
        waveform: "square", attack: 0.001, decay: 0.04, sustain: 0.2, release: 0.02, amplitude: 0.35,
      }, position: [40, 40], inputs: [], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "bass", name: "NoteSequencer", category: "Sequencer", params: {
        bpm: 200,
        notes: "[52,52,52,52,55,55,55,55,60,60,60,60,59,59,59,59,52,52,52,52,55,55,55,55,60,60,60,60,59,59,59,59,52,52,52,52,55,55,55,55,60,60,60,60,59,59,59,59,52,52,52,52,55,55,55,55,60,60,60,60,59,59,59,59,52,52,52,52,55,55,55,55,60,60,60,60,59,59,59,59]",
        waveform: "triangle", attack: 0.002, decay: 0.08, sustain: 0.4, release: 0.03, amplitude: 0.3,
      }, position: [40, 240], inputs: [], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "mix_001", name: "Mixer", category: "Mixer", params: { channels: 4 }, position: [400, 140], inputs: [{ name: "input_0", port_type: "audio" }, { name: "input_1", port_type: "audio" }, { name: "input_2", port_type: "audio" }, { name: "input_3", port_type: "audio" }], outputs: [{ name: "output", port_type: "audio" }] },
      { id: "delay_001", name: "DelayEffect", category: "Effect", params: { time: 0.15, feedback: 0.25, mix: 0.3 }, position: [660, 140], inputs: [{ name: "audio", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "audio_001", name: "AudioOutput", category: "Output", params: {}, position: [920, 140], inputs: [{ name: "audio", port_type: "audio" }], outputs: [] },
    ],
    edges: [
      { source: "melody", source_handle: "audio", target: "mix_001", target_handle: "input_0" },
      { source: "bass", source_handle: "audio", target: "mix_001", target_handle: "input_1" },
      { source: "mix_001", source_handle: "output", target: "delay_001", target_handle: "audio" },
      { source: "delay_001", source_handle: "audio", target: "audio_001", target_handle: "audio" },
    ],
  },
  tetris: {
    category: "🎵 Songs",
    label: "Tetris - Theme K (Disco Remix)",
    nodes: [
      { id: "melody", name: "NoteSequencer", category: "Sequencer", params: {
        bpm: 160,
        notes: "[72,72,72,72,69,69,69,69,67,67,67,67,64,64,64,64,67,67,67,67,69,69,69,69,72,72,72,72,69,69,69,69,67,67,67,67,64,64,64,64,60,60,60,60,64,64,64,64,67,67,67,67,69,69,69,69,72,72,72,72,69,69,69,69,67,67,67,67,64,64,64,64,60,60,60,60,64,64,64,64]",
        waveform: "square", attack: 0.001, decay: 0.06, sustain: 0.25, release: 0.02, amplitude: 0.35,
      }, position: [40, 40], inputs: [], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "bass", name: "NoteSequencer", category: "Sequencer", params: {
        bpm: 160,
        notes: "[48,48,48,48,48,48,48,48,50,50,50,50,50,50,50,50,53,53,53,53,53,53,53,53,48,48,48,48,48,48,48,48,50,50,50,50,50,50,50,50,48,48,48,48,48,48,48,48,50,50,50,50,50,50,50,50,53,53,53,53,53,53,53,53,48,48,48,48,48,48,48,48,50,50,50,50]",
        waveform: "triangle", attack: 0.002, decay: 0.1, sustain: 0.5, release: 0.03, amplitude: 0.25,
      }, position: [40, 240], inputs: [], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "mix_001", name: "Mixer", category: "Mixer", params: { channels: 4 }, position: [400, 140], inputs: [{ name: "input_0", port_type: "audio" }, { name: "input_1", port_type: "audio" }, { name: "input_2", port_type: "audio" }, { name: "input_3", port_type: "audio" }], outputs: [{ name: "output", port_type: "audio" }] },
      { id: "delay_001", name: "DelayEffect", category: "Effect", params: { time: 0.2, feedback: 0.3, mix: 0.25 }, position: [660, 140], inputs: [{ name: "audio", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "audio_001", name: "AudioOutput", category: "Output", params: {}, position: [920, 140], inputs: [{ name: "audio", port_type: "audio" }], outputs: [] },
    ],
    edges: [
      { source: "melody", source_handle: "audio", target: "mix_001", target_handle: "input_0" },
      { source: "bass", source_handle: "audio", target: "mix_001", target_handle: "input_1" },
      { source: "mix_001", source_handle: "output", target: "delay_001", target_handle: "audio" },
      { source: "delay_001", source_handle: "audio", target: "audio_001", target_handle: "audio" },
    ],
  },
  snake: {
    category: "🎵 Songs",
    label: "Snake - Nokia Classic Tune",
    nodes: [
      { id: "melody", name: "NoteSequencer", category: "Sequencer", params: {
        bpm: 180,
        notes: "[67,65,64,62,64,65,67,69,67,65,64,62,64,65,67,69,71,69,67,65,64,62,64,65,67,69,67,65,64,62,64,65,67,69,71,69,67,65,64,62,64,65,67,69,67,65,64,62,64,65,67,69,67,65,64,62,64,65,67,69,71,69,67,65,64,62,64,65,67,69,67,65,64,62,64,65,67,69]",
        waveform: "square", attack: 0.001, decay: 0.05, sustain: 0.15, release: 0.01, amplitude: 0.4,
      }, position: [40, 40], inputs: [], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "bass", name: "NoteSequencer", category: "Sequencer", params: {
        bpm: 180,
        notes: "[48,48,48,48,48,48,48,48,48,48,48,48,48,48,48,48,43,43,43,43,43,43,43,43,43,43,43,43,43,43,43,43,48,48,48,48,48,48,48,48,48,48,48,48,48,48,48,48,43,43,43,43,43,43,43,43,43,43,43,43,43,43,43,43,48,48,48,48,48,48,48,48,43,43,43,43]",
        waveform: "triangle", attack: 0.002, decay: 0.1, sustain: 0.5, release: 0.03, amplitude: 0.2,
      }, position: [40, 240], inputs: [], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "mix_001", name: "Mixer", category: "Mixer", params: { channels: 4 }, position: [400, 140], inputs: [{ name: "input_0", port_type: "audio" }, { name: "input_1", port_type: "audio" }, { name: "input_2", port_type: "audio" }, { name: "input_3", port_type: "audio" }], outputs: [{ name: "output", port_type: "audio" }] },
      { id: "delay_001", name: "DelayEffect", category: "Effect", params: { time: 0.1, feedback: 0.2, mix: 0.2 }, position: [660, 140], inputs: [{ name: "audio", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "audio_001", name: "AudioOutput", category: "Output", params: {}, position: [920, 140], inputs: [{ name: "audio", port_type: "audio" }], outputs: [] },
    ],
    edges: [
      { source: "melody", source_handle: "audio", target: "mix_001", target_handle: "input_0" },
      { source: "bass", source_handle: "audio", target: "mix_001", target_handle: "input_1" },
      { source: "mix_001", source_handle: "output", target: "delay_001", target_handle: "audio" },
      { source: "delay_001", source_handle: "audio", target: "audio_001", target_handle: "audio" },
    ],
  },
  pacmania: {
    category: "🎵 Songs",
    label: "Pac-Man - In-Game Theme",
    nodes: [
      { id: "melody", name: "NoteSequencer", category: "Sequencer", params: {
        bpm: 200,
        notes: "[67,67,67,67,67,67,67,67,67,69,67,65,67,69,71,69,67,69,67,65,67,69,71,69,67,67,67,67,67,67,67,67,67,69,67,65,67,69,71,69,67,69,67,65,67,69,71,69,67,67,67,67,67,67,67,67,67,69,67,65,67,69,71,69,67,69,67,65,67,69,71,69,67,67,67,67]",
        waveform: "square", attack: 0.001, decay: 0.04, sustain: 0.2, release: 0.015, amplitude: 0.35,
      }, position: [40, 40], inputs: [], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "bass", name: "NoteSequencer", category: "Sequencer", params: {
        bpm: 200,
        notes: "[48,48,48,48,48,48,48,48,48,48,48,48,48,48,48,48,43,43,43,43,43,43,43,43,43,43,43,43,43,43,43,43,48,48,48,48,48,48,48,48,48,48,48,48,48,48,48,48,43,43,43,43,43,43,43,43,43,43,43,43,43,43,43,43,48,48,48,48,48,48,48,48,43,43,43,43]",
        waveform: "triangle", attack: 0.002, decay: 0.08, sustain: 0.4, release: 0.02, amplitude: 0.25,
      }, position: [40, 240], inputs: [], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "mix_001", name: "Mixer", category: "Mixer", params: { channels: 4 }, position: [400, 140], inputs: [{ name: "input_0", port_type: "audio" }, { name: "input_1", port_type: "audio" }, { name: "input_2", port_type: "audio" }, { name: "input_3", port_type: "audio" }], outputs: [{ name: "output", port_type: "audio" }] },
      { id: "delay_001", name: "DelayEffect", category: "Effect", params: { time: 0.12, feedback: 0.2, mix: 0.2 }, position: [660, 140], inputs: [{ name: "audio", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "audio_001", name: "AudioOutput", category: "Output", params: {}, position: [920, 140], inputs: [{ name: "audio", port_type: "audio" }], outputs: [] },
    ],
    edges: [
      { source: "melody", source_handle: "audio", target: "mix_001", target_handle: "input_0" },
      { source: "bass", source_handle: "audio", target: "mix_001", target_handle: "input_1" },
      { source: "mix_001", source_handle: "output", target: "delay_001", target_handle: "audio" },
      { source: "delay_001", source_handle: "audio", target: "audio_001", target_handle: "audio" },
    ],
  },
 
  // --- MENU THEME ---
  menu_theme: {
    category: "🎵 Songs",
    label: "Retro Menu Theme (1 min loop)",
    nodes: [
      { id: "melody", name: "NoteSequencer", category: "Sequencer", params: {
        bpm: 140,
        notes: "[72,0,76,0,79,0,84,0,81,0,79,0,76,0,72,0,77,0,81,0,84,0,86,0,84,0,81,0,79,0,76,0,0,0,0,0,72,0,76,0,79,0,84,0,81,0,79,0,76,0,72,0,0,0,0,0,77,0,81,0,84,0,86,0,84,0,81,0,79,0,76,0]",
        waveform: "square", attack: 0.002, decay: 0.08, sustain: 0.25, release: 0.03, amplitude: 0.32,
      }, position: [30, 30], inputs: [], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "bass", name: "NoteSequencer", category: "Sequencer", params: {
        bpm: 140,
        notes: "[48,0,0,0,55,0,0,0,48,0,0,0,55,0,0,0,53,0,0,0,57,0,0,0,53,0,0,0,57,0,0,0,48,0,0,0,55,0,0,0,48,0,0,0,55,0,0,0,53,0,0,0,57,0,0,0,53,0,0,0,57,0,0,0,0]",
        waveform: "triangle", attack: 0.003, decay: 0.12, sustain: 0.4, release: 0.04, amplitude: 0.28,
      }, position: [30, 230], inputs: [], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "mix_001", name: "Mixer", category: "Mixer", params: {}, position: [380, 130], inputs: [{ name: "input_0", port_type: "audio" }, { name: "input_1", port_type: "audio" }, { name: "input_2", port_type: "audio" }, { name: "input_3", port_type: "audio" }], outputs: [{ name: "output", port_type: "audio" }] },
      { id: "delay_001", name: "DelayEffect", category: "Effects", params: { delayTime: 0.3, feedback: 0.25, mix: 0.2 }, position: [640, 130], inputs: [{ name: "audio", port_type: "audio" }], outputs: [{ name: "audio", port_type: "audio" }] },
      { id: "audio_001", name: "AudioOutput", category: "Output", params: { volume: 0.8 }, position: [900, 130], inputs: [{ name: "audio", port_type: "audio" }], outputs: [] },
    ],
    edges: [
      { source: "melody", source_handle: "audio", target: "mix_001", target_handle: "input_0" },
      { source: "bass", source_handle: "audio", target: "mix_001", target_handle: "input_1" },
      { source: "mix_001", source_handle: "output", target: "delay_001", target_handle: "audio" },
      { source: "delay_001", source_handle: "audio", target: "audio_001", target_handle: "audio" },
    ],
  },
  };

function AppContent() {
  const { toast } = useToast();
  const tauri = useTauriGraph();
  const reactFlowInstance = useRef<any>(null);
  const reactFlowWrapper = useRef<HTMLDivElement>(null);
  const [selectedNode, setSelectedNode] = useState<NodeData | null>(null);
  const [consoleVisible, setConsoleVisible] = useState(true);
  const [localConsole, setLocalConsole] = useState<ConsoleMessage[]>([]);
  const [audioStatus, setAudioStatus] = useState<AudioStatus>("stopped");
  const [exporting, setExporting] = useState(false);
  const [placingNodeType, setPlacingNodeType] = useState<string | null>(null);
  const [presetMenuOpen, setPresetMenuOpen] = useState(false);
  const demoBtnRef = useRef<HTMLDivElement | null>(null);
  const [hoveredEdge, setHoveredEdge] = useState<string | null>(null);
  const [showTemplateEditor, setShowTemplateEditor] = useState(false);
  const [showScriptEditor, setShowScriptEditor] = useState(false);
  const [showExportModal, setShowExportModal] = useState(false);
  const [templateNodeTypes, setTemplateNodeTypes] = useState<Record<string, any>>({});
  const [scriptNodeTypes, setScriptNodeTypes] = useState<Record<string, any>>({});
  const importInputRef = useRef<HTMLInputElement>(null);

  // Undo/redo stacks
  const [undoStack, setUndoStack] = useState<any[]>([]);
  const [redoStack, setRedoStack] = useState<any[]>([]);
  const pushUndoRef = useRef<(action: any) => void>();
  const [pushUndoReady, setPushUndoReady] = useState(false);

  const pushUndo = useCallback((action: any) => {
    setUndoStack(prev => {
      const next = [...prev, action];
      return next.length > 50 ? next.slice(-50) : next;
    });
    setRedoStack([]);
  }, []);

  useEffect(() => {
    setPushUndoCallback(pushUndo);
  }, [pushUndo]);

  useEffect(() => {
    pushUndoRef.current = pushUndo;
    setPushUndoReady(true);
  }, [pushUndo]);

  const handleUndo = useCallback(async () => {
    if (undoStack.length === 0) return;
    const action = undoStack[undoStack.length - 1];
    setUndoStack(prev => prev.slice(0, -1));

    try {
      switch (action.type) {
        case "ADD_NODE":
          await apiRemoveNode(action.nodeId);
          break;
        case "REMOVE_NODE":
          const nid = await apiAddNode(action.nodeType, action.position);
          setRedoStack(prev => [...prev, { type: "REMOVE_NODE", nodeId: nid, nodeType: action.nodeType, position: action.position }]);
          return;
        case "CONNECT":
          await apiDisconnect(action.source, action.target);
          break;
        case "DISCONNECT":
          await apiConnect(action.source, action.sourceHandle, action.target, action.targetHandle);
          break;
        case "SET_PARAM":
          await apiSetParam(action.nodeId, action.paramName, action.prevValue);
          break;
        case "MOVE_NODE":
          await invoke('set_node_position', { request: { node_id: action.nodeId, position: action.oldPosition } });
          break;
      }
      await tauri.refreshGraph();
      setRedoStack(prev => [...prev, action]);
      toast("Undo", "info");
    } catch (err: any) {
      toast(`Undo failed: ${err.message}`, "error");
    }
  }, [undoStack, tauri, toast]);

  const handleRedo = useCallback(async () => {
    if (redoStack.length === 0) return;
    const action = redoStack[redoStack.length - 1];
    setRedoStack(prev => prev.slice(0, -1));

    try {
      switch (action.type) {
        case "ADD_NODE":
          await apiAddNode(action.nodeType, action.position);
          break;
        case "REMOVE_NODE":
          await apiRemoveNode(action.nodeId);
          break;
        case "CONNECT":
          await apiConnect(action.source, action.sourceHandle, action.target, action.targetHandle);
          break;
        case "DISCONNECT":
          await apiDisconnect(action.source, action.target);
          break;
        case "SET_PARAM":
          await apiSetParam(action.nodeId, action.paramName, action.newValue);
          break;
        case "MOVE_NODE":
          await invoke('set_node_position', { request: { node_id: action.nodeId, position: action.newPosition } });
          break;
      }
      await tauri.refreshGraph();
      setUndoStack(prev => [...prev, action]);
      toast("Redo", "info");
    } catch (err: any) {
      toast(`Redo failed: ${err.message}`, "error");
    }
  }, [redoStack, tauri, toast]);

  useEffect(() => {
    setAudioStatus(tauri.isPlaying ? "running" : "stopped");
  }, [tauri.isPlaying]);

  useEffect(() => {
    listTemplates().then((templates) => {
      const types: Record<string, any> = {};
      for (const t of templates) {
        types[t.name] = {
          name: t.name,
          category: t.category,
          inputs: t.inputs,
          outputs: t.outputs,
          params: Object.fromEntries(t.exposed_params.map((p) => [p, { type: "number", default: 0, min: 0, max: 1 }])),
        };
      }
      setTemplateNodeTypes(types);
    }).catch(() => {});

    listScripts().then((scripts) => {
      const types: Record<string, any> = {};
      for (const s of scripts) {
        types[s.name] = {
          name: s.name,
          category: s.category,
          inputs: s.inputs,
          outputs: s.outputs,
          params: Object.fromEntries(s.params.map((p) => [p, { type: "number", default: 0, min: 0, max: 1 }])),
        };
      }
      setScriptNodeTypes(types);
    }).catch(() => {});
  }, []);

  const setReactFlowRef = useCallback((instance: any) => {
    reactFlowInstance.current = instance;
  }, []);

  const nodes: Node[] = useMemo(
    () =>
      tauri.graphState.nodes.map((n, idx) => ({
        id: n.id,
        type: n.name === "NoteMapper" ? "noteMapper" : n.name === "Instrument" ? "instrument" : "custom",
        position: n.position ? { x: n.position[0], y: n.position[1] } : { x: 100 + (idx % 3) * 250, y: 100 + Math.floor(idx / 3) * 200 },
        data: {
          id: n.id,
          type: n.name,
          category: n.category,
          params: n.params || {},
          inputs: n.inputs || [],
          outputs: n.outputs || [],
        } as any,
      })),
    [tauri.graphState.nodes]
  );

  const edges: Edge[] = useMemo(
    () => {
      const nodeMap = new Map(tauri.graphState.nodes.map((n: any) => [n.id, n]));
      return tauri.graphState.edges.map((e: any, idx: number) => {
        const edgeId = `edge-${idx}`;
        const sourceNode = nodeMap.get(e.source);
        const sourcePort = sourceNode?.outputs?.find((p: any) => p.name === e.source_handle);
        const portType = sourcePort?.port_type ?? sourcePort?.type ?? "AudioPort";
        const baseColor = PORT_EDGE_COLORS[portType] ?? "#7aa2f7";
        return {
          id: edgeId,
          source: e.source,
          target: e.target,
          sourceHandle: e.source_handle,
          targetHandle: e.target_handle,
          style: {
            stroke: hoveredEdge === edgeId ? "#ffffff" : baseColor,
            strokeWidth: hoveredEdge === edgeId ? 3 : 2,
            cursor: "pointer",
          },
          markerEnd: { type: "arrowclosed", strokeWidth: 2, markerUnits: "userSpaceOnUse" },
        };
      });
    },
    [tauri.graphState.edges, hoveredEdge]
  );

  const [rfNodes, setRfNodes] = useState<Node[]>(nodes);
  const [rfEdges, setRfEdges] = useState<Edge[]>(edges);

  // Sync React Flow state when Tauri graph state changes
  useEffect(() => {
    setRfNodes(nodes);
  }, [nodes]);
  useEffect(() => {
    setRfEdges(edges);
  }, [edges]);

  const onNodesChange = useCallback((changes: NodeChange[]) => {
    setRfNodes((nds) => applyNodeChanges(changes, nds));
  }, []);

  const onNodeDragStop = useCallback(async (_event: React.MouseEvent, node: Node) => {
    const tauriNode = tauri.graphState.nodes.find(n => n.id === node.id);
    const oldPosition = tauriNode?.position ?? [node.position.x, node.position.y];
    const newPosition: [number, number] = [node.position.x, node.position.y];
    if (oldPosition[0] !== newPosition[0] || oldPosition[1] !== newPosition[1]) {
      pushUndo({ type: "MOVE_NODE", nodeId: node.id, oldPosition, newPosition });
    }
    tauri.handleNodePositionChange(node.id, newPosition);
  }, [tauri, pushUndo]);
  const onEdgesChange = useCallback((changes: EdgeChange[]) => {
    setRfEdges((eds) => applyEdgeChanges(changes, eds));
  }, []);

  const onConnect = useCallback(
    (connection: Connection) => {
      setRfEdges((eds) => addEdge(connection, eds));
      tauri.handleConnect(
        connection.source!,
        connection.sourceHandle!,
        connection.target!,
        connection.targetHandle!
      );
      pushUndo({ type: "CONNECT", source: connection.source!, sourceHandle: connection.sourceHandle!, target: connection.target!, targetHandle: connection.targetHandle! });
    },
    [tauri, pushUndo]
  );

  const onEdgeDoubleClick = useCallback((_event: React.MouseEvent, edge: Edge) => {
    pushUndo({ type: "DISCONNECT", source: edge.source, sourceHandle: edge.sourceHandle, target: edge.target, targetHandle: edge.targetHandle });
    tauri.handleDisconnect(edge.source, edge.target);
    setRfEdges((eds) => eds.filter((e) => e.id !== edge.id));
    setLocalConsole((prev) => [
      ...prev,
      { id: `${Date.now()}`, text: `Disconnected: ${edge.source} → ${edge.target}`, type: "info", timestamp: Date.now() },
    ]);
  }, [tauri, pushUndo]);

  const onEdgeMouseEnter = useCallback((_event: React.MouseEvent, edge: Edge) => {
    setHoveredEdge(edge.id);
  }, []);

  const onEdgeMouseLeave = useCallback(() => {
    setHoveredEdge(null);
  }, []);

  const onNodeClick = useCallback((_: React.MouseEvent, node: Node) => {
    if (placingNodeType) {
      setPlacingNodeType(null);
      return;
    }
    setSelectedNode(node.data as any as NodeData);
  }, [placingNodeType]);

  const onNodeDoubleClick = useCallback(async (_: React.MouseEvent, node: Node) => {
    const nodeData = node.data as any as NodeData;
    if (nodeData?.type === "NoteMapper") {
      try {
        await invoke('open_note_mapper_editor', { nodeId: nodeData.id });
      } catch (err) {
        console.error('Failed to open NoteMapper editor:', err);
      }
    }
  }, []);

  const onPaneClick = useCallback(async (e: React.MouseEvent) => {
    if (placingNodeType) {
      const instance = reactFlowInstance.current;
      if (instance && instance.screenToFlowPosition) {
        const position = instance.screenToFlowPosition({
          x: e.clientX,
          y: e.clientY,
        });
        const nodeId = await apiAddNode(placingNodeType, [position.x, position.y]);
        await tauri.refreshGraph();
        pushUndo({ type: "ADD_NODE", nodeId, nodeType: placingNodeType, position: [position.x, position.y] });
        toast(`Added ${placingNodeType}`, "success");
        setLocalConsole((prev) => [
          ...prev,
          { id: `${Date.now()}`, text: `Added: ${placingNodeType}`, type: "success", timestamp: Date.now() },
        ]);
        setPlacingNodeType(null);
      }
      return;
    }
    setSelectedNode(null);
  }, [placingNodeType, tauri, setLocalConsole, toast, pushUndo]);

  useEffect(() => {
    const handleDragOver = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
    };

    const handleDragEnter = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
    };

    const handleDrop = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();

      const nodeType = e.dataTransfer?.getData("application/nodeType") || e.dataTransfer?.getData("text/plain");
      if (!nodeType) return;

      const wrapper = reactFlowWrapper.current;
      if (!wrapper) return;
      const target = e.target as EventTarget | null;
      if (target && !wrapper.contains(target as any)) return;

      const instance = reactFlowInstance.current;
      if (!instance || !instance.screenToFlowPosition) return;

      const position = instance.screenToFlowPosition({
        x: e.clientX,
        y: e.clientY,
      });

      (async () => {
        try {
          const nodeId = await apiAddNode(nodeType, [position.x, position.y]);
          await tauri.refreshGraph();
          pushUndo({ type: "ADD_NODE", nodeId, nodeType, position: [position.x, position.y] });
          toast(`Added ${nodeType}`, "success");
          setLocalConsole((prev) => [
            ...prev,
            { id: `${Date.now()}`, text: `Added: ${nodeType}`, type: "success", timestamp: Date.now() },
          ]);
        } catch (err: any) {
          toast(`Error adding node: ${err.message}`, "error");
          setLocalConsole((prev) => [
            ...prev,
            { id: `${Date.now()}`, text: `Error adding node: ${err.message}`, type: "error", timestamp: Date.now() },
          ]);
        }
      })();
    };

    document.addEventListener("dragover", handleDragOver, true);
    document.addEventListener("dragenter", handleDragEnter, true);
    document.addEventListener("drop", handleDrop, true);

    return () => {
      document.removeEventListener("dragover", handleDragOver, true);
      document.removeEventListener("dragenter", handleDragEnter, true);
      document.removeEventListener("drop", handleDrop, true);
    };
  }, [tauri, setLocalConsole, toast, pushUndo]);

  const handleParamChange = useCallback(
    (nodeId: string, paramName: string, value: number | string | boolean) => {
      tauri.handleParamChange(nodeId, paramName, value);
    },
    [tauri]
  );

  const handleStartAudio = useCallback(() => {
    tauri.handlePlay();
    setAudioStatus("running");
  }, [tauri]);

  const handleStopAudio = useCallback(() => {
    tauri.handleStop();
    setAudioStatus("stopped");
  }, [tauri]);

  const handleExportWav = useCallback(() => {
    setShowExportModal(true);
  }, []);

  const handleDoExport = useCallback(async (duration: number, sampleRate: number) => {
    setExporting(true);
    setAudioStatus("exporting");
    try {
      const path = await invoke<string | null>('save_wav_file_dialog');
      if (!path) {
        toast("Export cancelled", "info");
        return;
      }
      await tauri.handleExport(path, duration, sampleRate);
      toast(`Exported to ${path.split("\\").pop()}`, "success");
      setLocalConsole((prev) => [
        ...prev,
        {
          id: `${Date.now()}`,
          text: `Exported WAV to: ${path}`,
          type: "success",
          timestamp: Date.now(),
        },
      ]);
    } catch (err: any) {
      toast(`Export failed: ${err.message}`, "error");
      setLocalConsole((prev) => [
        ...prev,
        {
          id: `${Date.now()}`,
          text: `Export failed: ${err.message}`,
          type: "error",
          timestamp: Date.now(),
        },
      ]);
      throw err;
    } finally {
      setExporting(false);
      setAudioStatus(tauri.isPlaying ? "running" : "stopped");
    }
  }, [tauri, toast]);

  const handleNewGraph = useCallback(() => {
    tauri.handleClear();
    toast("Graph cleared", "info");
    setLocalConsole((prev) => [
      ...prev,
      {
        id: `${Date.now()}`,
        text: "Graph cleared",
        type: "info",
        timestamp: Date.now(),
      },
    ]);
  }, [tauri, toast]);

  const handleLoadPresetByKey = useCallback(async (key: string) => {
    const preset = demoPresets[key];
    if (!preset) return;
    setPresetMenuOpen(false);
    try {
      await tauri.handleLoadPreset({ nodes: preset.nodes, edges: preset.edges });
      toast(`Loaded: ${preset.label}`, "success");
      setLocalConsole((prev) => [
        ...prev,
        {
          id: `${Date.now()}`,
          text: `Loaded: ${preset.label}`,
          type: "success",
          timestamp: Date.now(),
        },
      ]);
    } catch (err: any) {
      toast(`Preset load failed: ${err.message}`, "error");
      setLocalConsole((prev) => [
        ...prev,
        {
          id: `${Date.now()}`,
          text: `Preset load failed: ${err.message}`,
          type: "error",
          timestamp: Date.now(),
        },
      ]);
    }
  }, [tauri, toast]);

  useEffect(() => {
    if (!presetMenuOpen) return;
    const close = () => setPresetMenuOpen(false);
    document.addEventListener("mousedown", close);
    document.addEventListener("keydown", (e) => { if (e.key === "Escape") close(); });
    return () => { document.removeEventListener("mousedown", close); document.removeEventListener("keydown", (e) => { if (e.key === "Escape") close(); }); };
  }, [presetMenuOpen]);

  const handleImportGraph = useCallback(async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;
    event.target.value = "";
    try {
      const text = await file.text();
      const graphData = JSON.parse(text);
      await tauri.handleLoadPreset(graphData);
      toast(`Imported: ${graphData.nodes.length} nodes, ${graphData.edges.length} edges`, "success");
      setLocalConsole((prev) => [
        ...prev,
        {
          id: `${Date.now()}`,
          text: `Graph imported: ${graphData.nodes.length} nodes, ${graphData.edges.length} edges`,
          type: "success",
          timestamp: Date.now(),
        },
      ]);
    } catch (err: any) {
      toast(`Import failed: ${err.message}`, "error");
      setLocalConsole((prev) => [
        ...prev,
        {
          id: `${Date.now()}`,
          text: `Error importing graph: ${err.message}`,
          type: "error",
          timestamp: Date.now(),
        },
      ]);
    }
  }, [tauri, toast]);

  const handleSaveGraph = useCallback(async () => {
    try {
      const path = `C:\\Users\\Scotty\\OneDrive\\Desktop\\RetroMusicMaker\\save_${Date.now()}.json`;
      await tauri.handleSave(path);
      toast(`Saved to ${path.split("\\").pop()}`, "success");
      setLocalConsole((prev) => [
        ...prev,
        {
          id: `${Date.now()}`,
          text: `Graph saved to: ${path}`,
          type: "success",
          timestamp: Date.now(),
        },
      ]);
    } catch (err: any) {
      toast(`Save failed: ${err.message}`, "error");
      setLocalConsole((prev) => [
        ...prev,
        {
          id: `${Date.now()}`,
          text: `Save failed: ${err.message}`,
          type: "error",
          timestamp: Date.now(),
        },
      ]);
    }
  }, [tauri, toast]);

  const handleDeleteSelected = useCallback(async () => {
    if (selectedNode) {
      const node = tauri.graphState.nodes.find(n => n.id === selectedNode.id);
      if (node) {
        pushUndo({ type: "REMOVE_NODE", nodeId: node.id, nodeType: node.name, position: node.position });
      }
      await apiRemoveNode(selectedNode.id);
      await tauri.refreshGraph();
      setSelectedNode(null);
    }
  }, [tauri, selectedNode, pushUndo]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      const inInput = document.activeElement?.tagName === "INPUT";

      if ((e.key === "Delete" || e.key === "Backspace") && selectedNode && !inInput) {
        handleDeleteSelected();
      }
      if (e.key === " " && !inInput) {
        e.preventDefault();
        if (audioStatus === "running") {
          handleStopAudio();
        } else {
          handleStartAudio();
        }
      }
      if (e.key === "Escape" && placingNodeType) {
        setPlacingNodeType(null);
      }
      if (e.ctrlKey && e.key === "z" && !e.shiftKey && !inInput) {
        e.preventDefault();
        handleUndo();
      }
      if (e.ctrlKey && (e.key === "y" || (e.key === "z" && e.shiftKey)) && !inInput) {
        e.preventDefault();
        handleRedo();
      }
      if (e.ctrlKey && e.key === "s" && !inInput) {
        e.preventDefault();
        handleSaveGraph();
      }
      if (e.ctrlKey && e.key === "e" && !inInput) {
        e.preventDefault();
        handleExportWav();
      }
    },
    [selectedNode, audioStatus, placingNodeType, handleDeleteSelected, handleStartAudio, handleStopAudio, handleSaveGraph, handleExportWav, handleUndo, handleRedo]
  );

  return (
    <div
      style={{
        width: "100vw",
        height: "100vh",
        display: "flex",
        flexDirection: "column",
        background: "#13141e",
        color: "#c0caf5",
        fontFamily: "'Segoe UI', system-ui, sans-serif",
        overflow: "hidden",
      }}
      onKeyDown={handleKeyDown}
      tabIndex={0}
    >
      {/* Top Toolbar */}
      <div
        style={{
          height: 48,
          background: "#1a1b26",
          borderBottom: "1px solid #3d4060",
          display: "flex",
          alignItems: "center",
          padding: "0 16px",
          gap: 8,
          flexShrink: 0,
        }}
      >
        <span style={{ fontSize: 14, fontWeight: 800, color: "#e1e4ff", marginRight: 16, letterSpacing: -0.5 }}>
          <span style={{ color: "#7aa2f7" }}>◆</span> Retro Music Maker
        </span>

        <div style={{ width: 1, height: 24, background: "#3d4060", margin: "0 8px" }} />

        <input
          ref={importInputRef}
          type="file"
          accept=".json,application/json"
          style={{ display: "none" }}
          onChange={handleImportGraph}
        />
        <ToolbarButton onClick={() => importInputRef.current?.click()} tooltip="Load a saved graph from a JSON file">Import JSON</ToolbarButton>
        <ToolbarButton onClick={handleNewGraph} tooltip="Clear the graph and start fresh">New</ToolbarButton>
        <ToolbarButton onClick={handleSaveGraph} tooltip="Save current graph to JSON file">Save</ToolbarButton>
        <ToolbarButton onClick={handleExportWav} disabled={exporting} tooltip="Export audio to a WAV file">
          {exporting ? "Exporting..." : "Export WAV"}
        </ToolbarButton>
        <div style={{ position: "relative" }}>
          <div ref={demoBtnRef} style={{ display: "inline-block" }}>
             <ToolbarButton onClick={() => setPresetMenuOpen(!presetMenuOpen)} tooltip="Load a demo preset graph">Demos ▾</ToolbarButton>
           </div>
          {presetMenuOpen && (() => {
            const rect = demoBtnRef.current?.getBoundingClientRect();
            return createPortal(
              <div style={{
                position: "fixed", top: rect ? rect.bottom + 4 : 0, left: rect ? rect.left : 0,
                background: "#181926", border: "1px solid #3d4060", borderRadius: 4,
                boxShadow: "0 8px 24px rgba(0,0,0,0.5)", zIndex: 9999, minWidth: 280,
                padding: "4px 0",
              }}>
              {(() => {
                  const groups: Record<string, Array<[string, any]>> = {};
                  for (const [key, preset] of Object.entries(demoPresets)) {
                    const cat = preset.category || "Other";
                    if (!groups[cat]) groups[cat] = [];
                    groups[cat].push([key, preset]);
                  }
                  return Object.entries(groups).map(([cat, entries]) => (
                    <div key={cat}>
                      <div style={{ padding: "6px 14px 2px", fontSize: 9, fontWeight: 700, color: "#787cb8", textTransform: "uppercase", letterSpacing: 1, fontFamily: "'Segoe UI', system-ui, sans-serif" }}>{cat}</div>
                      {entries.map(([key, preset]) => (
                        <div
                          key={key}
                          onClick={(e) => { e.stopPropagation(); handleLoadPresetByKey(key); }}
                          style={{
                            padding: "6px 14px", cursor: "pointer", fontSize: 11,
                            color: "#c0caf5", fontFamily: "'Segoe UI', system-ui, sans-serif",
                            whiteSpace: "nowrap",
                          }}
                          onMouseEnter={(e) => (e.currentTarget.style.background = "#262840")}
                          onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
                        >
                          {preset.label}
                        </div>
                      ))}
                    </div>
                  ));
                })()}
           </div>,
            document.body
          )}
        )()}
        </div>

        <div style={{ width: 1, height: 24, background: "#3d4060", margin: "0 8px" }} />
        <ToolbarButton
          onClick={handleUndo}
          disabled={undoStack.length === 0}
          tooltip="Undo last action (Ctrl+Z)"
        >
          ↩ Undo{undoStack.length > 0 ? ` (${undoStack.length})` : ""}
        </ToolbarButton>
        <ToolbarButton
          onClick={handleRedo}
          disabled={redoStack.length === 0}
          tooltip="Redo last action (Ctrl+Y)"
        >
          ↪ Redo{redoStack.length > 0 ? ` (${redoStack.length})` : ""}
        </ToolbarButton>
        <ToolbarButton onClick={() => setShowTemplateEditor(true)} tooltip="Create custom template nodes">
          🧩 Templates
        </ToolbarButton>
        <ToolbarButton onClick={() => setShowScriptEditor(true)} tooltip="Create custom script nodes with Lua">
          📜 Scripts
        </ToolbarButton>
        <ToolbarButton onClick={handleStartAudio} disabled={audioStatus === "running"} tooltip="Start audio engine (Space)">
          ▶ Start
        </ToolbarButton>
        <ToolbarButton onClick={handleStopAudio} disabled={audioStatus === "stopped"} tooltip="Stop audio engine (Space)">
          ■ Stop
        </ToolbarButton>

        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            marginLeft: 16,
            fontSize: 11,
            color:
              audioStatus === "running"
                ? "#9ece6a"
                : audioStatus === "exporting"
                ? "#e0af68"
                : "#565a7e",
          }}
        >
          <div
            style={{
              width: 6,
              height: 6,
              borderRadius: "50%",
              background:
                audioStatus === "running"
                  ? "#9ece6a"
                  : audioStatus === "exporting"
                  ? "#e0af68"
                  : "#565a7e",
            }}
          />
          {audioStatus.toUpperCase()}
        </div>

        {placingNodeType && (
          <div style={{ display: "flex", alignItems: "center", gap: 6, marginLeft: 16, fontSize: 11, color: "#7aa2f7" }}>
            <span>🎯 Place: {placingNodeType}</span>
            <span style={{ cursor: "pointer", opacity: 0.7 }} onClick={() => setPlacingNodeType(null)}>✕</span>
          </div>
        )}

        <div style={{ display: "flex", alignItems: "center", gap: 10, fontSize: 10, color: "#787cb8" }}>
          <span style={{ textTransform: "uppercase", letterSpacing: 1, opacity: 0.6 }}>Ports</span>
          <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
            <div style={{ width: 8, height: 8, borderRadius: "50%", background: "#9ece6a", boxShadow: "0 0 4px #9ece6a" }} />
            <span>Audio</span>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
            <div style={{ width: 8, height: 8, borderRadius: "50%", background: "#e0af68", boxShadow: "0 0 4px #e0af68" }} />
            <span>Control</span>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
            <div style={{ width: 8, height: 8, borderRadius: "50%", background: "#f7768e", boxShadow: "0 0 4px #f7768e" }} />
            <span>Trigger</span>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
            <div style={{ width: 8, height: 8, borderRadius: "50%", background: "#bb9af7", boxShadow: "0 0 4px #bb9af7" }} />
            <span>Instrument</span>
          </div>
        </div>

        <div style={{ flex: 1 }} />

        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            fontSize: 11,
            color: "#9ece6a",
          }}
        >
          <div
            style={{
              width: 6,
              height: 6,
              borderRadius: "50%",
              background: "#9ece6a",
              boxShadow: "0 0 4px #9ece6a",
            }}
          />
          Native
        </div>
      </div>

      {/* Main content area */}
      <div style={{ flex: 1, display: "flex", overflow: "hidden" }}>
        <Palette
          nodeTypes={{ ...paletteNodeTypes, ...templateNodeTypes, ...scriptNodeTypes }}
          onNodeDragStart={() => {}}
          placingNodeType={placingNodeType}
          onNodeClick={(type) => setPlacingNodeType(placingNodeType === type ? null : type)}
        />

        <div
          ref={reactFlowWrapper}
          style={{ flex: 1, position: "relative" }}
        >
          <ReactFlow
            nodes={rfNodes}
            edges={rfEdges}
            onNodesChange={onNodesChange}
            onNodeDragStop={onNodeDragStop}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onEdgeDoubleClick={onEdgeDoubleClick}
            onEdgeMouseEnter={onEdgeMouseEnter}
            onEdgeMouseLeave={onEdgeMouseLeave}
            onNodeClick={onNodeClick}
            onNodeDoubleClick={onNodeDoubleClick}
            onPaneClick={onPaneClick}
            onInit={(view) => setReactFlowRef(view)}
            nodeTypes={nodeTypes}
            defaultViewport={{ x: 100, y: 100, zoom: 1 }}
            minZoom={0.2}
            maxZoom={4}
            fitView
            fitViewOptions={{ padding: 0.2 }}
            proOptions={{ hideAttribution: true }}
          >
            <Background color="#3d4060" gap={24} size={1} />
            <Controls />
         </ReactFlow>
         </div>

      </div>

      {/* Bottom: Console */}
      <Console
        messages={localConsole}
        onClear={() => setLocalConsole([])}
        visible={consoleVisible}
        onToggle={() => setConsoleVisible((v) => !v)}
      />

      {/* Template Editor Modal */}
      {showTemplateEditor && (
        <TemplateEditor
          onClose={() => setShowTemplateEditor(false)}
          onSave={() => {
            setShowTemplateEditor(false);
          }}
        />
      )}

      {/* Script Editor Modal */}
      {showScriptEditor && (
        <ScriptEditor
          onClose={() => setShowScriptEditor(false)}
          onSave={() => {
            setShowScriptEditor(false);
          }}
        />
      )}

      {/* Export WAV Modal */}
      {showExportModal && (
        <ExportWavModal
          onClose={() => setShowExportModal(false)}
          onExport={handleDoExport}
        />
      )}
    </div>
  );
}

export default function App() {
  return (
    <ToastProvider>
      <AppContent />
    </ToastProvider>
  );
}

function ToolbarButton({
  children,
  onClick,
  disabled,
  style,
  tooltip,
}: {
  children: React.ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  style?: React.CSSProperties;
  tooltip?: string;
}) {
  const [showTooltip, setShowTooltip] = useState(false);
  const [pos, setPos] = useState({ x: 0, y: 0 });
  const btnRef = useRef<HTMLButtonElement>(null);

  const handleEnter = (e: React.MouseEvent) => {
    if (!disabled && tooltip) {
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      setPos({ x: rect.left + rect.width / 2, y: rect.bottom + 6 });
      setShowTooltip(true);
    }
    if (!disabled) {
      (e.target as HTMLElement).style.background = "#333654";
    }
  };

  const handleLeave = () => {
    setShowTooltip(false);
    if (!disabled) {
      (btnRef.current as HTMLElement).style.background = "#1e2030";
    }
  };

  return (
    <div style={{ position: "relative" }}>
      <button
        ref={btnRef}
        onClick={onClick}
        disabled={disabled}
        style={{
          padding: "6px 14px",
          background: disabled ? "#13141e" : "#1e2030",
          border: "1px solid #3d4060",
          borderRadius: 4,
          color: disabled ? "#565a7e" : "#c0caf5",
          fontSize: 12,
          fontFamily: "'Segoe UI', system-ui, sans-serif",
          cursor: disabled ? "not-allowed" : "pointer",
          opacity: disabled ? 0.5 : 1,
          transition: "all 0.15s",
          ...style,
        }}
        onMouseEnter={handleEnter}
        onMouseLeave={handleLeave}
      >
        {children}
      </button>
      {showTooltip && tooltip && (
        <div
          style={{
            position: "fixed",
            left: pos.x,
            top: pos.y,
            transform: "translateX(-50%)",
            background: "#181926",
            border: "1px solid #7aa2f7",
            borderRadius: 4,
            padding: "5px 10px",
            fontSize: 11,
            color: "#c0caf5",
            fontFamily: "'Segoe UI', system-ui, sans-serif",
            boxShadow: "0 4px 12px rgba(0,0,0,0.5)",
            whiteSpace: "nowrap",
            pointerEvents: "none",
            zIndex: 9999,
          }}
        >
          {tooltip}
        </div>
      )}
    </div>
  );
}
