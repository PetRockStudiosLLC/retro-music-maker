/** Node palette sidebar with drag-and-drop to add nodes. */
import { memo, useState } from "react";
import type { NodeTypeInfo } from "../types/nodes";

interface PaletteProps {
  nodeTypes: Record<string, NodeTypeInfo>;
  onNodeDragStart: (nodeType: string) => void;
  placingNodeType?: string | null;
  onNodeClick?: (nodeType: string) => void;
}

const CATEGORY_COLORS: Record<string, string> = {
  Oscillator: "#7aa2f7",
  Synthesizer: "#e0af68",
  Filter: "#73daca",
  Envelope: "#9ece6a",
  Effects: "#e0af68",
  Mixer: "#bb9af7",
  Input: "#bb9af7",
  Output: "#c0caf5",
  Sequencer: "#7dcfff",
  Trigger: "#f7768e",
  Controller: "#e0af68",
};

const NODE_DESCRIPTIONS: Record<string, string> = {
  SineOscillator: "Pure sine wave tone. Smooth and clean.",
  SquareOscillator: "Hollow, retro 8-bit sound. Classic chiptune lead.",
  SawtoothOscillator: "Bright, buzzy tone. Great for bass and leads.",
  TriangleOscillator: "Soft, mellow tone. Good for bass lines.",
  NoiseOscillator: "Random noise. Use for drums, percussion, FX.",
  ChipSoundOscillator: "Multi-waveform retro synth. Square, triangle, noise, DPCM.",
  VCO: "Takes MIDI note numbers from control input, outputs audio waveform.",
  LowpassFilter: "Removes high frequencies. Warm, muffled sound.",
  HighpassFilter: "Removes low frequencies. Thin, bright sound.",
  BandpassFilter: "Only passes a frequency band. Telephone/robot FX.",
  Bitcrush: "Reduces bit depth & sample rate. Lo-fi retro crunch.",
  ADSREnvelope: "Shapes volume over time: Attack, Decay, Sustain, Release.",
  DelayEffect: "Echo/repeat effect. Creates spatial depth.",
  Distortion: "Adds grit and aggression. Overdrive/distortion.",
  Reverb: "Simulates room/space acoustics. Adds ambiance.",
  DurationGate: "Lets audio pass for a set duration, then mutes.",
  Loop: "Records incoming audio, then loops the recorded segment.",
  Mixer: "Combines up to 4 audio inputs into one output.",
  Gain: "Adjusts volume level. Amplify or attenuate.",
  KeyboardInput: "Computer keyboard as MIDI input. Play notes with keys.",
  MidiInput: "External MIDI device input for note control.",
  WavFile: "Load and play a WAV audio file. Trigger to play, loop to repeat.",
  AudioOutput: "Final audio output to your speakers/headphones.",
  FileOutput: "Record audio to a WAV file on disk.",
  StepSequencer: "Rhythm sequencer. Triggers beats on a grid.",
  Arpeggiator: "Plays notes in ascending/descending patterns.",
  NoteSequencer: "Self-contained melody player. Set notes as MIDI numbers (0=rest).",
  NoteMapper: "FL Studio-style piano roll. Click cells to place notes on a visual grid.",
  SlotPlayer: "Routes audio from connected inputs sequentially. Advances when input goes silent.",
  Clock: "Fires beat and tick triggers at a set BPM. Like a metronome.",
  RandomTrigger: "Fires triggers at random intervals between min and max seconds.",
  TriggerDelay: "Delays an incoming trigger by a set amount of time.",
  Instrument: "Defines instrument sound (waveform + ADSR). Connect to NoteMapper to change its sound.",
};

function Palette({ nodeTypes, onNodeDragStart, placingNodeType, onNodeClick }: PaletteProps) {
  const [tooltip, setTooltip] = useState<{ text: string; x: number; y: number } | null>(null);
  const [search, setSearch] = useState("");

  const query = search.toLowerCase().trim();

  const categories = Object.entries(
    Object.values(nodeTypes).reduce(
      (acc, nt) => {
        const cat = nt.category;
        if (query && !nt.name.toLowerCase().includes(query) && !cat.toLowerCase().includes(query)) return acc;
        if (!acc[cat]) acc[cat] = [];
        acc[cat].push(nt);
        return acc;
      },
      {} as Record<string, NodeTypeInfo[]>
    )
  ).sort(([a], [b]) => a.localeCompare(b));

  return (
    <div
      style={{
        width: 240,
        height: "100%",
        background: "#1e2030",
        borderRight: "1px solid #3d4060",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          padding: "16px 16px 8px",
          fontSize: 11,
          fontWeight: 700,
          color: "#565a7e",
          textTransform: "uppercase",
          letterSpacing: 1.5,
        }}
      >
        Node Types
      </div>
      <div style={{ padding: "0 16px 8px", fontSize: 10, color: "#565a7e" }}>
        Click to place • Drag to drop
      </div>

      <div style={{ padding: "0 12px 8px" }}>
        <input
          type="text"
          placeholder="Search nodes..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          style={{
            width: "100%",
            padding: "6px 10px",
            background: "#13141e",
            border: "1px solid #3d4060",
            borderRadius: 4,
            color: "#c0caf5",
            fontSize: 12,
            outline: "none",
            boxSizing: "border-box",
          }}
        />
      </div>

      <div
        style={{
          flex: 1,
          overflowY: "auto",
          padding: "4px 8px 16px",
        }}
      >
        {categories.map(([catName, nodes]) => (
          <div key={catName} style={{ marginBottom: 8 }}>
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 6,
                padding: "4px 8px",
                marginBottom: 2,
              }}
            >
              <div
                style={{
                  width: 6,
                  height: 6,
                  borderRadius: "50%",
                  background: CATEGORY_COLORS[catName] ?? "#c0caf5",
                }}
              />
              <span
                style={{
                  fontSize: 10,
                  fontWeight: 700,
                  color: CATEGORY_COLORS[catName] ?? "#c0caf5",
                  textTransform: "uppercase",
                  letterSpacing: 1,
                }}
              >
                {catName}
              </span>
            </div>

          {nodes.map((nt) => {
              const isSelected = placingNodeType === nt.name;
              const description = NODE_DESCRIPTIONS[nt.name] ?? "";
              return (
                <div
                  key={nt.name}
                  draggable={true}
                  onDragStart={(e: React.DragEvent) => {
                    e.dataTransfer.setData("application/nodeType", nt.name);
                    e.dataTransfer.setData("text/plain", nt.name);
                    e.dataTransfer.effectAllowed = "copy";
                    onNodeDragStart(nt.name);
                  }}
                  onClick={(e) => {
                    e.stopPropagation();
                    onNodeClick?.(nt.name);
                  }}
                  onMouseMove={(e) => {
                    if (description) {
                      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
                      setTooltip({ text: description, x: rect.right + 8, y: rect.top + rect.height / 2 });
                    }
                  }}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                    padding: "7px 12px",
                    background: isSelected ? "#7aa2f7" : "transparent",
                    borderRadius: 4,
                    color: isSelected ? "#13141e" : "#c0caf5",
                    fontSize: 12,
                    fontFamily: "'Segoe UI', system-ui, sans-serif",
                    cursor: "pointer",
                    transition: "background 0.15s",
                    userSelect: "none",
                    position: "relative",
                  }}
                  onMouseEnter={(e) => {
                    if (!isSelected) (e.currentTarget as HTMLElement).style.background = "#333654";
                  }}
                  onMouseLeave={(e) => {
                    if (!isSelected) (e.currentTarget as HTMLElement).style.background = "transparent";
                    setTooltip(null);
                  }}
                >
                  <svg width="12" height="12" viewBox="0 0 12 12" fill="none" style={{ flexShrink: 0 }}>
                    <rect x="1" y="1" width="4" height="4" rx="1" fill={isSelected ? "#13141e" : "#565a7e"} />
                    <rect x="7" y="1" width="4" height="4" rx="1" fill={isSelected ? "#13141e" : "#565a7e"} />
                    <rect x="1" y="7" width="4" height="4" rx="1" fill={isSelected ? "#13141e" : "#565a7e"} />
                    <rect x="7" y="7" width="4" height="4" rx="1" fill={isSelected ? "#13141e" : "#565a7e"} />
                  </svg>
                  <span style={{ fontSize: 9, color: isSelected ? "#13141e" : "#565a7e", marginLeft: 4 }}>
                    {nt.inputs?.length || 0}/{nt.outputs?.length || 0}
                  </span>
                  <span
                    style={{
                      whiteSpace: "nowrap",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                    }}
                  >
                    {nt.name
                      .split(/(?=[A-Z])/)
                      .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
                      .join(" ")}
                  </span>
                </div>
              );
            })}
          </div>
        ))}
      </div>

      {tooltip && (
        <div
          style={{
            position: "fixed",
            left: tooltip.x,
            top: tooltip.y,
            transform: "translate(0, -50%)",
            background: "#181926",
            border: "1px solid #7aa2f7",
            borderRadius: 4,
            padding: "5px 10px",
            fontSize: 11,
            color: "#c0caf5",
            fontFamily: "'Segoe UI', system-ui, sans-serif",
            boxShadow: "0 4px 12px rgba(0,0,0,0.5)",
            whiteSpace: "normal",
            maxWidth: 200,
            lineHeight: 1.4,
            pointerEvents: "none",
            zIndex: 9999,
          }}
        >
          {tooltip.text}
        </div>
      )}
    </div>
  );
}

export default memo(Palette);
