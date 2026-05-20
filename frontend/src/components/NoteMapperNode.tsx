import { memo, useMemo, useState, useEffect, useRef } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";

const PORT_COLORS: Record<string, string> = {
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

const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const CELL_W = 6;
const CELL_H = 4;
const NUM_PITCHES = 37;

const waveColors: Record<string, string> = {
  square: "#7aa2f7",
  sawtooth: "#e0af68",
  triangle: "#9ece6a",
  sine: "#7dcfff",
  pulse: "#f7768e",
  pulse_half: "#bb9af7",
  noise: "#c0caf5",
  sample_hold: "#ff9e64",
};

function waveformFromInstrument(inst: string): string {
  const key = inst.toLowerCase().replace(/[_\s]/g, "");
  if (key.includes("square")) return "square";
  if (key.includes("saw")) return "sawtooth";
  if (key.includes("triangle")) return "triangle";
  if (key.includes("sine")) return "sine";
  if (key.includes("pulse")) return "pulse";
  if (key.includes("noise")) return "noise";
  if (key.includes("sample") || key.includes("hold")) return "sample_hold";
  return "square";
}

interface NoteMapperData {
  id: string;
  type: string;
  category: string;
  params: Record<string, number | string | boolean>;
  inputs: Array<{ name: string; type: string }>;
  outputs: Array<{ name: string; type: string }>;
}

function midiToName(midi: number): string {
  return NOTE_NAMES[midi % 12] + Math.floor(midi / 12 - 1);
}

function isBlackKey(midi: number): boolean {
  const n = midi % 12;
  return n === 1 || n === 3 || n === 6 || n === 8 || n === 10;
}

function NoteMapperNode({ data: raw }: NodeProps) {
  const data = raw as unknown as NoteMapperData;

  const gridStr = (data.params.grid as string) || "";
  const numSteps = (data.params.num_steps as number) || 16;
  const minMidi = (data.params.min_midi as number) || 48;
  const bpm = (data.params.bpm as number) || 140;
  const displayName = (data.params.display_name as string) || "";
  const instrument = (data.params.instrument as string) || "square_lead";
  const instrumentJson = (data.params.instrument_json as string) || "";

  const waveform = waveformFromInstrument(instrument);
  const color = waveColors[waveform] || "#7aa2f7";

  const [glow, setGlow] = useState(false);
  const prevInst = useRef(instrumentJson);
  useEffect(() => {
    if (instrumentJson && instrumentJson !== prevInst.current) {
      setGlow(true);
      prevInst.current = instrumentJson;
      const t = setTimeout(() => setGlow(false), 1200);
      return () => clearTimeout(t);
    }
    if (!instrumentJson) {
      prevInst.current = "";
    }
  }, [instrumentJson]);

  const activeCells = useMemo(() => {
    const cells: Set<string> = new Set();
    try {
      const inner = gridStr.trim().startsWith("[") ? gridStr.trim().slice(1, -1) : gridStr.trim();
      const steps = inner.split(";");
      for (let s = 0; s < Math.min(steps.length, numSteps); s++) {
        const parts = steps[s].split(",").filter(Boolean);
        for (const p of parts) {
          const midi = parseInt(p.trim(), 10);
          if (!isNaN(midi) && midi >= minMidi && midi < minMidi + NUM_PITCHES) {
            cells.add(`${s},${midi - minMidi}`);
          }
        }
      }
    } catch { /* ignore */ }
    return cells;
  }, [gridStr, numSteps, minMidi]);

  const width = Math.max(260, numSteps * CELL_W + 50);

  return (
    <div
      style={{
        width,
        background: "#1e2030",
        border: `1px solid ${color}`,
        borderRadius: 6,
        boxShadow: glow
          ? `0 0 16px ${color}80, 0 0 32px ${color}40, 0 2px 12px rgba(0,0,0,0.5)`
          : `0 0 8px ${color}30, 0 2px 12px rgba(0,0,0,0.5)`,
        overflow: "hidden",
        fontFamily: "'Segoe UI', system-ui, sans-serif",
        transition: "box-shadow 0.3s ease",
      }}
    >
      <div style={{ height: 4, background: color, borderRadius: "6px 6px 0 0" }} />

      <div style={{
        padding: "4px 8px",
        background: `rgba(${waveform === "square" ? "122,162,247" : waveform === "sawtooth" ? "224,175,104" : waveform === "triangle" ? "158,206,106" : waveform === "pulse" ? "247,118,142" : waveform === "pulse_half" ? "187,154,247" : waveform === "noise" ? "192,202,245" : waveform === "sample_hold" ? "255,158,100" : "125,207,255"}, 0.15)`,
        borderBottom: "1px solid rgba(255,255,255,0.05)",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
      }}>
        <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
          <div style={{
            width: 8,
            height: 8,
            borderRadius: "50%",
            background: color,
            boxShadow: glow ? `0 0 6px ${color}` : `0 0 3px ${color}60`,
            transition: "box-shadow 0.3s ease",
          }} />
          <span style={{ color: "#e1e4ff", fontSize: 10, fontWeight: 700 }}>
            {displayName || "NoteMapper"}
          </span>
        </div>
        <span style={{ fontSize: 8, color: "#a6accd" }}>
          {bpm} BPM &middot; {numSteps} steps &middot; {instrument.replace(/_/g, " ")}
        </span>
      </div>

      <div style={{
        padding: "2px 4px",
        background: "#151727",
        overflow: "hidden",
      }}>
        <div style={{ display: "inline-block" }}>
          <div style={{
            display: "grid",
            gridTemplateColumns: `30px repeat(${numSteps}, ${CELL_W}px)`,
            gap: 0,
          }}>
            <div style={{ background: "#1a1c2e", borderBottom: "1px solid #2a2c3e", borderRight: "1px solid #2a2c3e" }} />
            {Array.from({ length: numSteps }).map((_, col) => (
              <div key={col} style={{
                background: col % 4 === 0 ? "#1e2030" : "#1a1c2e",
                borderBottom: "1px solid #2a2c3e",
                borderRight: "1px solid transparent",
              }} />
            ))}

            {Array.from({ length: NUM_PITCHES }).map((_, rowIdx) => {
              const midi = minMidi + (NUM_PITCHES - 1 - rowIdx);
              const black = isBlackKey(midi);
              return (
                <div key={rowIdx} style={{ display: "contents" }}>
                  <div style={{
                    background: black ? "#1a1520" : "#1e2030",
                    borderBottom: "1px solid #2a2c3e",
                    borderRight: "1px solid #2a2c3e",
                    fontSize: 6,
                    color: midi % 12 === 0 ? "#888ba5" : "transparent",
                    display: "flex",
                    alignItems: "center",
                    paddingLeft: 2,
                    lineHeight: `${CELL_H}px`,
                    fontWeight: midi % 12 === 0 ? 700 : 400,
                  }}>
                    {midiToName(midi)}
                  </div>
                  {Array.from({ length: numSteps }).map((_, col) => {
                    const cellKey = `${col},${NUM_PITCHES - 1 - rowIdx}`;
                    const isActive = activeCells.has(cellKey);
                    return (
                      <div
                        key={col}
                        style={{
                          width: CELL_W,
                          height: CELL_H,
                          background: isActive
                            ? "#7dcfff"
                            : black
                              ? "#1a1520"
                              : "#1e2030",
                          borderBottom: "1px solid #2a2c3e",
                          borderRight: "1px solid #2a2c3e",
                          borderRadius: isActive ? 1 : 0,
                          boxShadow: isActive ? "inset 0 0 2px rgba(125,207,255,0.6)" : "none",
                        }}
                      />
                    );
                  })}
                </div>
              );
            })}
          </div>
        </div>
      </div>

      <div style={{
        padding: "3px 8px",
        background: "#1a1c2e",
        borderTop: "1px solid #2a2c3e",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
      }}>
        <span style={{ fontSize: 8, color: "#565a7e" }}>
          {activeCells.size} note{activeCells.size !== 1 ? "s" : ""}
        </span>
        <span style={{ fontSize: 8, color: "#7dcfff", opacity: 0.7 }}>
          Double-click to open editor
        </span>
      </div>

      <Handle
        type="target"
        position={Position.Left}
        id="instrument"
        style={{
          left: -5,
          top: "30%",
          width: 12,
          height: 12,
          background: PORT_COLORS.InstrumentPort,
          border: `2px solid ${PORT_COLORS.InstrumentPort}`,
          borderRadius: "50%",
          boxShadow: glow ? `0 0 8px ${color}` : `0 0 6px ${PORT_COLORS.InstrumentPort}66`,
          transition: "background 0.3s ease, box-shadow 0.3s ease",
        }}
      />
      <Handle
        type="source"
        position={Position.Right}
        id="audio"
        style={{
          right: -5,
          top: "50%",
          width: 12,
          height: 12,
          background: PORT_COLORS.AudioPort,
          border: `2px solid ${PORT_COLORS.AudioPort}`,
          borderRadius: "50%",
          boxShadow: `0 0 6px ${PORT_COLORS.AudioPort}66`,
        }}
      />
    </div>
  );
}

export default memo(NoteMapperNode);
