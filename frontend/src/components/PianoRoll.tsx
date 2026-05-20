import { memo, useCallback, useMemo, useState } from "react";

interface PianoRollProps {
  notesStr: string;
  bpm: number;
  onNotesChange: (notesStr: string) => void;
  onBpmChange: (bpm: number) => void;
}

const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const CELL_W = 28;
const CELL_H = 16;
const MIN_NOTE = 36;
const MAX_NOTE = 84;

function midiToName(n: number) {
  if (n === 0) return "---";
  return `${NOTE_NAMES[n % 12]}${Math.floor(n / 12) - 1}`;
}

function parseNotes(str: string): number[] {
  try {
    const cleaned = str.replace(/^\[/, "").replace(/\]$/, "");
    const parsed = cleaned.split(",").map((s) => parseInt(s.trim(), 10)).filter((n) => !isNaN(n));
    return parsed;
  } catch {
    return [60, 64, 67, 72, 67, 64, 60, 0];
  }
}

function PianoRoll({ notesStr, bpm, onNotesChange, onBpmChange }: PianoRollProps) {
  const notes = useMemo(() => parseNotes(notesStr), [notesStr]);
  const [placeNote, setPlaceNote] = useState(60);
  const [hoveredCell, setHoveredCell] = useState<{ col: number; row: number } | null>(null);

  const numSteps = notes.length;
  const numNotes = MAX_NOTE - MIN_NOTE + 1;

  const isNoteActive = useCallback(
    (col: number, midi: number) => notes[col] === midi,
    [notes]
  );

  const handleCellClick = useCallback(
    (col: number) => {
      const newNotes = [...notes];
      if (newNotes[col] === placeNote) {
        newNotes[col] = 0;
      } else {
        newNotes[col] = placeNote;
      }
      onNotesChange(JSON.stringify(newNotes));
    },
    [notes, placeNote, onNotesChange]
  );

  const handleCellDrag = useCallback(
    (col: number, midi: number, e: React.MouseEvent) => {
      if (!e.buttons) return;
      const newNotes = [...notes];
      if (newNotes[col] === midi) {
        newNotes[col] = 0;
      } else {
        newNotes[col] = midi;
      }
      onNotesChange(JSON.stringify(newNotes));
    },
    [notes, onNotesChange]
  );

  const handleClear = useCallback(() => {
    onNotesChange(JSON.stringify(notes.map(() => 0)));
  }, [notes, onNotesChange]);

  const handleAddStep = useCallback(() => {
    onNotesChange(JSON.stringify([...notes, 0]));
  }, [notes, onNotesChange]);

  const handleRemoveStep = useCallback(() => {
    if (notes.length <= 1) return;
    onNotesChange(JSON.stringify(notes.slice(0, -1)));
  }, [notes, onNotesChange]);

  return (
    <div
      style={{
        background: "#13141e",
        border: "1px solid #3d4060",
        borderRadius: 4,
        padding: 8,
      }}
    >
      {/* Controls */}
      <div style={{ display: "flex", gap: 6, marginBottom: 8, flexWrap: "wrap", alignItems: "center" }}>
        <button onClick={handleClear} style={btnStyle}>Clear</button>
        <button onClick={handleAddStep} style={btnStyle}>+ Step</button>
        <button onClick={handleRemoveStep} style={btnStyle}>- Step</button>
        <div style={{ width: 1, height: 20, background: "#3d4060" }} />
        <span style={{ color: "#565a7e", fontSize: 10 }}>Place:</span>
        <select
          value={placeNote}
          onChange={(e) => setPlaceNote(Number(e.target.value))}
          style={{
            ...selectStyle,
          }}
        >
          {Array.from({ length: 73 }, (_, i) => MIN_NOTE + i).map((n) => (
            <option key={n} value={n}>
              {midiToName(n)}
            </option>
          ))}
        </select>
        <div style={{ width: 1, height: 20, background: "#3d4060" }} />
        <span style={{ color: "#565a7e", fontSize: 10 }}>BPM:</span>
        <input
          type="number"
          value={bpm}
          onChange={(e) => onBpmChange(Number(e.target.value))}
          style={{ ...inputStyle, width: 50 }}
          min={20}
          max={300}
        />
      </div>

      {/* Grid */}
      <div
        style={{
          maxHeight: 320,
          overflow: "auto",
          border: "1px solid #2a2c45",
          borderRadius: 2,
        }}
      >
        <div
          style={{
            display: "inline-block",
            minWidth: `${CELL_W * numSteps + 36}px`,
          }}
        >
          {/* Note name column */}
          <div
            style={{
              display: "flex",
              flexDirection: "column-reverse",
              position: "sticky",
              left: 0,
              zIndex: 1,
            }}
          >
            {Array.from({ length: numNotes }, (_, i) => MAX_NOTE - i).map((midi) => (
              <div
                key={midi}
                style={{
                  height: CELL_H,
                  width: 36,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "flex-end",
                  paddingRight: 4,
                  fontSize: 8,
                  fontFamily: "Consolas, monospace",
                  color: midi % 12 === 0 ? "#7aa2f7" : "#3d4060",
                  background: "#181926",
                  boxSizing: "border-box",
                  borderRight: "1px solid #2a2c45",
                  flexShrink: 0,
                }}
              >
                {midiToName(midi)}
              </div>
            ))}
          </div>

          {/* Grid cells */}
          <div style={{ display: "inline-block", verticalAlign: "top" }}>
            {Array.from({ length: numNotes }, (_, i) => MAX_NOTE - i).map((midi) => (
              <div key={midi} style={{ display: "flex", height: CELL_H }}>
                {Array.from({ length: numSteps }, (_, j) => j).map((col) => {
                  const active = isNoteActive(col, midi);
                  const isC = midi % 12 === 0;
                  return (
                    <div
                      key={col}
                      onClick={() => handleCellClick(col)}
                      onMouseEnter={() => setHoveredCell({ col, row: midi })}
                      onMouseLeave={() => setHoveredCell(null)}
                      style={{
                        width: CELL_W,
                        height: CELL_H,
                        border: "1px solid #1e2030",
                        boxSizing: "border-box",
                        background: active
                          ? "#7aa2f7"
                          : hoveredCell?.col === col && hoveredCell?.row === midi
                          ? "#2a2c45"
                          : isC
                          ? "#1a1b2e"
                          : "#181926",
                        cursor: "pointer",
                        transition: "background 0.05s",
                      }}
                    />
                  );
                })}
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Step count */}
      <div
        style={{
          marginTop: 4,
          fontSize: 9,
          color: "#3d4060",
          textAlign: "right",
          fontFamily: "Consolas, monospace",
        }}
      >
        {numSteps} steps
      </div>
    </div>
  );
}

const btnStyle: React.CSSProperties = {
  padding: "3px 8px",
  background: "#1e2030",
  border: "1px solid #3d4060",
  borderRadius: 3,
  color: "#9999a5",
  fontSize: 10,
  cursor: "pointer",
  fontFamily: "'Segoe UI', system-ui, sans-serif",
};

const selectStyle: React.CSSProperties = {
  padding: "2px 4px",
  background: "#13141e",
  border: "1px solid #3d4060",
  borderRadius: 3,
  color: "#c0caf5",
  fontSize: 10,
  outline: "none",
  fontFamily: "Consolas, monospace",
};

const inputStyle: React.CSSProperties = {
  padding: "2px 4px",
  background: "#13141e",
  border: "1px solid #3d4060",
  borderRadius: 3,
  color: "#c0caf5",
  fontSize: 10,
  outline: "none",
  fontFamily: "Consolas, monospace",
};

export default memo(PianoRoll);
