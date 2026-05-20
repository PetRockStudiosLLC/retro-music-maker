import { useCallback, useMemo, useState, useEffect, useRef } from "react";
import { useTauriGraph, getNode, getGraphState } from "../tauri/useTauriGraph";

const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const CELL_W = 36;
const CELL_H = 22;
const NUM_PITCHES = 37;

function midiToName(midi: number): string {
  return NOTE_NAMES[midi % 12] + Math.floor(midi / 12 - 1);
}

function isBlackKey(midi: number): boolean {
  const n = midi % 12;
  return n === 1 || n === 3 || n === 6 || n === 8 || n === 10;
}

interface NoteMapperEditorProps {
  nodeId: string;
  onClose: () => void;
}

export default function NoteMapperEditor({ nodeId: initialNodeId, onClose }: NoteMapperEditorProps) {
  const { handleParamChange } = useTauriGraph();
  const [noteMapperNodes, setNoteMapperNodes] = useState<Array<{ id: string; label: string }>>([]);
  const [selectedNodeId, setSelectedNodeId] = useState(initialNodeId);
  const [displayName, setDisplayName] = useState("");
  const [renaming, setRenaming] = useState(false);
  const [gridStr, setGridStr] = useState("");
  const [numSteps, setNumSteps] = useState(16);
  const [minMidi, setMinMidi] = useState(48);
  const [bpm, setBpm] = useState(140);
  const [waveform, setWaveform] = useState("square");
  const [dragging, setDragging] = useState(false);
  const [dragActivate, setDragActivate] = useState(false);
  const [hoverCell, setHoverCell] = useState<{ col: number; row: number } | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const renameInputRef = useRef<boolean>(false);

  useEffect(() => {
    const fetchGraph = async () => {
      try {
        const state = await getGraphState();
        const mappers = state.nodes
          .filter((n) => n.name === "NoteMapper")
          .map((n) => ({ id: n.id, label: (n.params as any)?.display_name || n.id }));
        setNoteMapperNodes(mappers);
        if (mappers.length > 0 && !mappers.find((n) => n.id === selectedNodeId)) {
          setSelectedNodeId(mappers[0].id);
        }
      } catch { /* ignore */ }
    };
    fetchGraph();
    const interval = setInterval(fetchGraph, 2000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (!selectedNodeId) return;
    const fetchNode = async () => {
      try {
        const node = await getNode(selectedNodeId);
        if (node?.params) {
          setGridStr((node.params.grid as string) || "");
          setNumSteps((node.params.num_steps as number) || 16);
          setMinMidi((node.params.min_midi as number) || 48);
          setBpm((node.params.bpm as number) || 140);
          setWaveform((node.params.waveform as string) || "square");
          if (!renameInputRef.current) setDisplayName((node.params.display_name as string) || "");
        }
      } catch { /* ignore */ }
    };
    fetchNode();
    const interval = setInterval(fetchNode, 1000);
    return () => clearInterval(interval);
  }, [selectedNodeId]);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onClose]);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollLeft = scrollRef.current.scrollWidth;
    }
  }, []);

  const backendCells = useMemo(() => {
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

  const activeCells = backendCells;

  const toggleCell = useCallback((col: number, pitchOffset: number) => {
    const key = `${col},${pitchOffset}`;
    const midi = minMidi + pitchOffset;
    let inner = "";
    try {
      inner = gridStr.trim().startsWith("[") ? gridStr.trim().slice(1, -1) : gridStr.trim();
    } catch { inner = ""; }
    const steps = inner.split(";");
    while (steps.length < numSteps) steps.push("");
    const parts = steps[col]?.split(",").filter(Boolean) || [];
    const currentlyActive = backendCells.has(key);
    if (currentlyActive) {
      const idx = parts.indexOf(String(midi));
      if (idx !== -1) parts.splice(idx, 1);
    } else {
      if (!parts.includes(String(midi))) parts.push(String(midi));
    }
    steps[col] = parts.join(",");
    const newGrid = "[" + steps.join(";") + "]";
    setGridStr(newGrid);
    handleParamChange(selectedNodeId, "grid", newGrid);
  }, [gridStr, numSteps, minMidi, selectedNodeId, handleParamChange, backendCells]);

  const handleCellMouseDown = useCallback((col: number, row: number) => {
    setDragging(true);
    setDragActivate(!activeCells.has(`${col},${row}`));
    toggleCell(col, row);
  }, [activeCells, toggleCell]);

  const handleCellMouseEnter = useCallback((col: number, row: number) => {
    setHoverCell({ col, row });
    if (dragging) {
      if (dragActivate) {
        if (!activeCells.has(`${col},${row}`)) toggleCell(col, row);
      } else {
        if (activeCells.has(`${col},${row}`)) toggleCell(col, row);
      }
    }
  }, [dragging, dragActivate, activeCells, toggleCell]);

  const handleMouseUp = useCallback(() => {
    setDragging(false);
  }, []);

  const handleBpmChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const val = parseFloat(e.target.value);
      if (!isNaN(val) && val >= 40 && val <= 300) {
        setBpm(val);
        handleParamChange(selectedNodeId, "bpm", val);
      }
    },
    [selectedNodeId, handleParamChange]
  );

  const handleStepsChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const val = parseInt(e.target.value, 10);
      if (!isNaN(val) && val >= 4 && val <= 128) {
        setNumSteps(val);
        handleParamChange(selectedNodeId, "num_steps", val);
      }
    },
    [selectedNodeId, handleParamChange]
  );

  const handleMinMidiChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const val = parseInt(e.target.value, 10);
      if (!isNaN(val) && val >= 24 && val <= 72) {
        setMinMidi(val);
        handleParamChange(selectedNodeId, "min_midi", val);
      }
    },
    [selectedNodeId, handleParamChange]
  );

  const handleWaveformChange = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => {
      setWaveform(e.target.value);
      handleParamChange(selectedNodeId, "waveform", e.target.value);
    },
    [selectedNodeId, handleParamChange]
  );

  const handleClear = useCallback(() => {
    const emptyGrid = "[" + ";".repeat(numSteps) + "]";
    setGridStr(emptyGrid);
    handleParamChange(selectedNodeId, "grid", emptyGrid);
  }, [selectedNodeId, numSteps, handleParamChange]);

  const handleNodeSelect = useCallback((e: React.ChangeEvent<HTMLSelectElement>) => {
    setSelectedNodeId(e.target.value);
  }, []);

  const handleRename = useCallback(() => {
    renameInputRef.current = false;
    if (displayName.trim()) {
      handleParamChange(selectedNodeId, "display_name", displayName.trim());
    }
    setRenaming(false);
  }, [displayName, selectedNodeId, handleParamChange]);

  const width = Math.max(600, numSteps * CELL_W + 100);

  return (
    <div
      style={{
        width: "100vw",
        height: "100vh",
        background: "#1e2030",
        display: "flex",
        flexDirection: "column",
        fontFamily: "'Segoe UI', system-ui, sans-serif",
        overflow: "hidden",
      }}
    >
      <div style={{ height: 4, background: "linear-gradient(90deg, #7dcfff, #7aa2f7, #7dcfff)", flexShrink: 0 }} />

      <div
        style={{
          padding: "12px 20px",
          background: "rgba(125,207,255,0.1)",
          borderBottom: "1px solid rgba(255,255,255,0.06)",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          flexShrink: 0,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <span style={{ color: "#e1e4ff", fontSize: 16, fontWeight: 800, letterSpacing: -0.5 }}>
            NoteMapper Editor
          </span>
          {noteMapperNodes.length > 1 && (
            <select value={selectedNodeId} onChange={handleNodeSelect}
              style={{ background: "#151727", border: "1px solid #3b3d57", borderRadius: 4, color: "#7dcfff", fontSize: 11, padding: "4px 8px", minWidth: 120 }}>
              {noteMapperNodes.map((n) => (
                <option key={n.id} value={n.id}>{n.label}</option>
              ))}
            </select>
          )}
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          {renaming ? (
           <input type="text" value={displayName} onChange={(e) => setDisplayName(e.target.value)}
               onFocus={() => (renameInputRef.current = true)}
               onBlur={() => { renameInputRef.current = false; handleRename(); }}
               onKeyDown={(e) => { if (e.key === "Enter") handleRename(); }}
              style={{ background: "#151727", border: "1px solid #7dcfff", borderRadius: 4, color: "#7dcfff", fontSize: 12, padding: "3px 8px", width: 140 }}
              autoFocus />
          ) : (
            <span onClick={() => setRenaming(true)}
              style={{ color: "#a6accd", fontSize: 12, cursor: "pointer", opacity: 0.7 }}>
              {displayName || selectedNodeId}
            </span>
          )}
        </div>
        <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
          <label style={{ color: "#a6accd", fontSize: 12, display: "flex", alignItems: "center", gap: 4 }}>
            BPM
            <input type="number" value={bpm} onChange={handleBpmChange}
              style={{ width: 56, background: "#151727", border: "1px solid #3b3d57", borderRadius: 4, color: "#c0caf5", fontSize: 12, padding: "4px 6px" }}
            />
          </label>
          <label style={{ color: "#a6accd", fontSize: 12, display: "flex", alignItems: "center", gap: 4 }}>
            Steps
            <input type="number" value={numSteps} onChange={handleStepsChange}
              style={{ width: 50, background: "#151727", border: "1px solid #3b3d57", borderRadius: 4, color: "#c0caf5", fontSize: 12, padding: "4px 6px" }}
            />
          </label>
          <label style={{ color: "#a6accd", fontSize: 12, display: "flex", alignItems: "center", gap: 4 }}>
            MIDI
            <input type="number" value={minMidi} onChange={handleMinMidiChange}
              style={{ width: 44, background: "#151727", border: "1px solid #3b3d57", borderRadius: 4, color: "#c0caf5", fontSize: 12, padding: "4px 6px" }}
            />
          </label>
          <select value={waveform} onChange={handleWaveformChange}
            style={{ background: "#151727", border: "1px solid #3b3d57", borderRadius: 4, color: "#c0caf5", fontSize: 12, padding: "4px 8px" }}>
            <option value="square">Square</option>
            <option value="sawtooth">Saw</option>
            <option value="triangle">Tri</option>
            <option value="sine">Sine</option>
          </select>
          <button onClick={handleClear}
            style={{ background: "#e78284", border: "none", borderRadius: 4, color: "#151727", fontSize: 12, padding: "4px 12px", cursor: "pointer", fontWeight: 700 }}>
            Clear All
          </button>
          <div style={{ width: 1, height: 24, background: "#3d4060" }} />
          <button onClick={onClose}
            style={{ background: "transparent", border: "1px solid #565a7e", borderRadius: 4, color: "#a6accd", fontSize: 14, padding: "2px 10px", cursor: "pointer" }}>
            Close
          </button>
        </div>
      </div>

      <div
        ref={scrollRef}
        onMouseUp={handleMouseUp}
        style={{
          flex: 1,
          overflow: "auto",
          background: "#151727",
          padding: "8px 12px",
        }}
      >
        <div style={{ display: "inline-block", position: "relative" }}>
          <div style={{ display: "grid", gridTemplateColumns: `60px repeat(${numSteps}, ${CELL_W}px)`, gap: 0 }}>
            <div style={{ background: "#1a1c2e", borderBottom: "1px solid #2a2c3e", borderRight: "1px solid #2a2c3e", padding: "6px 0" }} />
            {Array.from({ length: numSteps }).map((_, col) => (
              <div key={col} style={{
                background: col % 4 === 0 ? "#1e2030" : "#1a1c2e",
                borderBottom: "1px solid #2a2c3e",
                borderRight: col % 4 === 3 ? "1px solid #2a2c3e" : "1px solid transparent",
                textAlign: "center",
                fontSize: 11,
                fontWeight: col % 4 === 0 ? 700 : 400,
                color: col % 4 === 0 ? "#7aa2f7" : "#565a6e",
                lineHeight: `${CELL_H + 4}px`,
              }}>
                {col + 1}
              </div>
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
                    fontSize: 10,
                    color: black ? "#6a4c93" : "#888ba5",
                    display: "flex",
                    alignItems: "center",
                    paddingLeft: 8,
                    lineHeight: `${CELL_H}px`,
                    fontWeight: midi % 12 === 0 ? 700 : 400,
                  }}>
                    {midiToName(midi)}
                  </div>
                  {Array.from({ length: numSteps }).map((_, col) => {
                    const cellKey = `${col},${NUM_PITCHES - 1 - rowIdx}`;
                    const isActive = activeCells.has(cellKey);
                    const isHover = hoverCell?.col === col && hoverCell?.row === NUM_PITCHES - 1 - rowIdx;
                    return (
                      <div
                        key={col}
                        onMouseDown={() => handleCellMouseDown(col, NUM_PITCHES - 1 - rowIdx)}
                        onMouseEnter={() => handleCellMouseEnter(col, NUM_PITCHES - 1 - rowIdx)}
                        style={{
                          width: CELL_W,
                          height: CELL_H,
                          background: isActive
                            ? "#7dcfff"
                            : isHover
                              ? "rgba(125,207,255,0.25)"
                              : black
                                ? "#1a1520"
                                : "#1e2030",
                          borderBottom: "1px solid #2a2c3e",
                          borderRight: col % 4 === 3 ? "1px solid #3b3d57" : "1px solid #2a2c3e",
                          borderRadius: isActive ? 3 : 0,
                          cursor: "pointer",
                          transition: "background 40ms",
                          boxShadow: isActive ? "inset 0 0 6px rgba(125,207,255,0.6)" : "none",
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
        padding: "6px 16px",
        background: "#1a1c2e",
        borderTop: "1px solid #2a2c3e",
        fontSize: 10,
        color: "#565a7e",
        textAlign: "center",
        flexShrink: 0,
      }}>
        Click to toggle notes &middot; Click &amp; drag to paint/erase &middot; Press Esc to close
      </div>
    </div>
  );
}
