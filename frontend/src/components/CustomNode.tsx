/** Custom node component for React Flow. */
import { memo, useState, useCallback, useRef, useEffect } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import { setParam } from "../tauri/useTauriGraph";
import { getPushUndoCallback } from "../undoBridge";
import { openWavFileDialog, saveFileOutput } from "../tauri/api";

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
  Utility: "#7dcfff",
  Effect: "#e0af68",
};

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

const PARAM_RANGES: Record<string, { min: number; max: number; step: number }> = {
  frequency: { min: 20, max: 20000, step: 0.5 },
  amplitude: { min: 0, max: 1, step: 0.01 },
  pulse_width: { min: 0, max: 1, step: 0.01 },
  duty_cycle: { min: 0, max: 1, step: 0.01 },
  duty: { min: 0, max: 1, step: 0.01 },
  attack: { min: 0.001, max: 5, step: 0.001 },
  decay: { min: 0.001, max: 5, step: 0.001 },
  sustain: { min: 0, max: 1, step: 0.01 },
  release: { min: 0.001, max: 10, step: 0.001 },
  time: { min: 0, max: 2, step: 0.001 },
  delayTime: { min: 0.001, max: 2, step: 0.001 },
  feedback: { min: 0, max: 0.95, step: 0.01 },
  mix: { min: 0, max: 1, step: 0.01 },
  cutoff: { min: 20, max: 22050, step: 1 },
  resonance: { min: 0.1, max: 20, step: 0.1 },
  volume: { min: 0, max: 2, step: 0.01 },
  volume_0: { min: 0, max: 1, step: 0.01 },
  volume_1: { min: 0, max: 1, step: 0.01 },
  volume_2: { min: 0, max: 1, step: 0.01 },
  volume_3: { min: 0, max: 1, step: 0.01 },
  pan: { min: -1, max: 1, step: 0.01 },
  pan_0: { min: -1, max: 1, step: 0.01 },
  pan_1: { min: -1, max: 1, step: 0.01 },
  pan_2: { min: -1, max: 1, step: 0.01 },
  pan_3: { min: -1, max: 1, step: 0.01 },
  gain: { min: 0, max: 10, step: 0.01 },
  amount: { min: 0, max: 1, step: 0.01 },
  bitDepth: { min: 1, max: 16, step: 1 },
  sampleRate: { min: 100, max: 44100, step: 100 },
  bpm: { min: 20, max: 300, step: 1 },
  tempo: { min: 20, max: 300, step: 1 },
  note: { min: 0, max: 127, step: 1 },
  detune: { min: -100, max: 100, step: 1 },
  duration: { min: 0, max: 300, step: 0.1 },
  loop_duration: { min: 0.1, max: 10, step: 0.1 },
  num_slots: { min: 1, max: 16, step: 1 },
  octaveRange: { min: 1, max: 4, step: 1 },
  steps: { min: 4, max: 32, step: 1 },
  velocity: { min: 0, max: 1, step: 0.01 },
  num_steps: { min: 4, max: 128, step: 1 },
  min_midi: { min: 24, max: 72, step: 1 },
};

interface NodeData {
  id: string;
  type: string;
  category: string;
  params: Record<string, number | string | boolean>;
  inputs: Array<{ name: string; type?: string; port_type?: string }>;
  outputs: Array<{ name: string; type?: string; port_type?: string }>;
  [key: string]: unknown;
}

function getPortType(ports: Array<{ name: string; type?: string; port_type?: string }>, name: string): string {
  const port = ports?.find((p) => p.name === name);
  return (port?.port_type ?? port?.type ?? "AudioPort") as string;
}

function getPortColor(ports: Array<{ name: string; type?: string; port_type?: string }>, name: string): string {
  return PORT_COLORS[getPortType(ports, name)] ?? "#c0caf5";
}

function getParamRange(name: string) {
  if (PARAM_RANGES[name]) return PARAM_RANGES[name];
  if (/^volume_\d+$/.test(name)) return { min: 0, max: 1, step: 0.01 };
  if (/^pan_\d+$/.test(name)) return { min: -1, max: 1, step: 0.01 };
  if (/^channels$/.test(name)) return { min: 1, max: 16, step: 1 };
  return { min: 0, max: 100, step: 0.01 };
}

function ParamRowInline({
  nodeId,
  name,
  value,
}: {
  nodeId: string;
  name: string;
  value: number | string | boolean;
}) {
  const [local, setLocal] = useState<number | string | boolean>(value);

  const handleChange = useCallback(
    async (v: number | string | boolean) => {
      const prevValue = local;
      setLocal(v);
      try {
        await setParam(nodeId, name, v);
        const pushUndo = getPushUndoCallback();
        if (pushUndo) {
          pushUndo({ type: "SET_PARAM", nodeId, paramName: name, prevValue, newValue: v });
        }
      } catch (err) {
        console.error("Failed to set param:", err);
      }
    },
    [nodeId, name, local]
  );

  const isBool = typeof value === "boolean";
  const isNum = typeof value === "number";
  const range = getParamRange(name);

  const label = name.replace(/_/g, " ");

  if (isBool) {
    return (
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "2px 0" }}>
        <span style={{ color: "#9999a5", fontSize: 10, textTransform: "capitalize" }}>{label}</span>
        <input
          type="checkbox"
          checked={local as boolean}
          onChange={(e) => handleChange(e.target.checked)}
          style={{ accentColor: "#7aa2f7", cursor: "pointer" }}
        />
      </div>
    );
  }

  if (isNum) {
    return (
      <div style={{ padding: "2px 0" }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 2 }}>
          <span style={{ color: "#9999a5", fontSize: 10, textTransform: "capitalize" }}>{label}</span>
          <span style={{ color: "#c0caf5", fontSize: 10, fontFamily: "Consolas, monospace" }}>
            {typeof local === "number" && local % 1 !== 0 ? local.toFixed(3) : local}
          </span>
        </div>
        <input
          type="range"
          min={range.min}
          max={range.max}
          step={range.step}
          value={typeof local === "number" ? local : parseFloat(local as string)}
          onChange={(e) => handleChange(parseFloat(e.target.value))}
          style={{ width: "100%", accentColor: "#7aa2f7" }}
        />
      </div>
    );
  }

  const isPath = name === "path";

  return (
    <div style={{ padding: "2px 0" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 2 }}>
        <span style={{ color: "#9999a5", fontSize: 10, textTransform: "capitalize" }}>{label}</span>
        {isPath && (
          <button
            onClick={async () => {
              try {
                const selected = await openWavFileDialog();
                if (selected) {
                  handleChange(selected);
                }
              } catch (err) {
                console.error("Failed to open file dialog:", err);
              }
            }}
            style={{
              background: "#7aa2f7",
              border: "none",
              borderRadius: 3,
              color: "#1e2030",
              fontSize: 9,
              fontWeight: 700,
              cursor: "pointer",
              padding: "1px 5px",
              lineHeight: 1,
            }}
          >
            BROWSE
          </button>
        )}
      </div>
      <input
        type="text"
        value={String(local)}
        onChange={(e) => handleChange(e.target.value)}
        style={{
          width: "100%",
          padding: "2px 6px",
          background: "#13141e",
          border: "1px solid #3d4060",
          borderRadius: 3,
          color: "#c0caf5",
          fontSize: 10,
          fontFamily: "'Segoe UI', system-ui, sans-serif",
          outline: "none",
          boxSizing: "border-box",
        }}
      />
    </div>
  );
}

function CustomNode({ data: raw, selected }: NodeProps) {
  const data = raw as unknown as NodeData;
  const color = CATEGORY_COLORS[data.category] ?? "#c0caf5";
  const bgDark = data.category
    ? `rgb(${parseInt(color.slice(1, 3), 16) * 0.3}, ${parseInt(color.slice(3, 5), 16) * 0.3}, ${parseInt(color.slice(5, 7), 16) * 0.3})`
    : "#1e2030";

  const inputNames = data.inputs?.length ? data.inputs : [];
  const outputNames = data.outputs?.length ? data.outputs : [];
  const params = data.params || {};
  const paramEntries = Object.entries(params)
    .filter(([k]) => !k.startsWith("_"))
    .sort(([a], [b]) => a.localeCompare(b));

  const displayName = (() => {
    const id = (data as any).id ?? "";
    if (!id) return data.type ?? "Node";
    return id
      .split("_")
      .map((w: string) => w.charAt(0).toUpperCase() + w.slice(1))
      .join(" ")
      .slice(0, 18);
  })();

  const isFileOutput = data.type === "FileOutput";

  const handleSave = useCallback(async () => {
    try {
      await saveFileOutput(data.id);
      console.log("FileOutput saved successfully");
    } catch (err) {
      console.error("Failed to save FileOutput:", err);
    }
  }, [data.id]);

  const [saving, setSaving] = useState(false);

  return (
    <div
      className="rf-node-wrapper"
      style={{
        width: 220,
        background: "#1e2030",
        border: `1px solid ${selected ? "#7aa2f7" : color}`,
        borderRadius: 6,
        boxShadow: selected
          ? "0 0 0 1px #7aa2f7, 0 2px 8px rgba(0,0,0,0.4)"
          : "0 2px 8px rgba(0,0,0,0.3)",
        overflow: "visible",
        position: "relative",
      }}
    >
      <div style={{ height: 4, background: color, borderRadius: "6px 6px 0 0" }} />

      <div
        style={{
          padding: "8px 12px",
          background: bgDark,
          borderBottom: "1px solid rgba(255,255,255,0.05)",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <span
          style={{
            color: "#e1e4ff",
            fontSize: 12,
            fontWeight: 700,
            fontFamily: "'Segoe UI', system-ui, sans-serif",
            whiteSpace: "nowrap",
            overflow: "hidden",
            textOverflow: "ellipsis",
            flex: 1,
            marginRight: 8,
          }}
        >
          {displayName}
        </span>
        {isFileOutput && (
          <button
            onClick={async () => {
              setSaving(true);
              try {
                await handleSave();
              } finally {
                setSaving(false);
              }
            }}
            style={{
              background: saving ? "#e0af68" : "#9ece6a",
              border: "none",
              borderRadius: 3,
              color: "#1e2030",
              fontSize: 9,
              fontWeight: 700,
              cursor: "pointer",
              padding: "2px 8px",
              lineHeight: 1,
              minWidth: 48,
            }}
          >
            {saving ? "..." : "SAVE"}
          </button>
        )}
      </div>

      <div style={{ padding: "4px 12px" }}>
        {inputNames.length > 0 &&
          inputNames.map((input: { name: string; type?: string; port_type?: string }) => (
            <div
              key={`in-${input.name}`}
              style={{ display: "flex", alignItems: "center", height: 24, position: "relative" }}
            >
              <Handle
                 type="target"
                 position={Position.Left}
                 id={input.name}
                 style={{
                   left: -5,
                   top: 12,
                   width: 12,
                   height: 12,
                   background: getPortColor(inputNames, input.name),
                   border: `2px solid ${getPortColor(inputNames, input.name)}`,
                   borderRadius: "50%",
                   boxShadow: `0 0 6px ${getPortColor(inputNames, input.name)}66`,
                 }}
               />
              <span
                style={{
                  color: "#565a7e",
                  fontSize: 11,
                  fontFamily: "'Segoe UI', system-ui, sans-serif",
                  paddingLeft: 12,
                }}
              >
                {input.name}
              </span>
            </div>
          ))}

        {outputNames.length > 0 &&
          outputNames.map((output: { name: string; type?: string; port_type?: string }) => (
            <div
              key={`out-${output.name}`}
              style={{ display: "flex", alignItems: "center", height: 24, position: "relative" }}
            >
              <span
                style={{
                  color: "#565a7e",
                  fontSize: 11,
                  fontFamily: "'Segoe UI', system-ui, sans-serif",
                  flex: 1,
                  textAlign: "right",
                  paddingRight: 28,
                }}
              >
                {output.name}
              </span>
             <Handle
                 type="source"
                 position={Position.Right}
                 id={output.name}
                 style={{
                   right: -5,
                   top: 12,
                   width: 12,
                   height: 12,
                   background: getPortColor(outputNames, output.name),
                   border: `2px solid ${getPortColor(outputNames, output.name)}`,
                   borderRadius: "50%",
                   boxShadow: `0 0 6px ${getPortColor(outputNames, output.name)}66`,
                 }}
               />
            </div>
          ))}
      </div>

     {paramEntries.length > 0 && (
        <div
          ref={(el) => {
            if (!el) return;
            const handler = (e: Event) => e.stopPropagation();
            el.addEventListener("pointerdown", handler);
            el.addEventListener("mousedown", handler);
            el.addEventListener("pointermove", handler);
            el.addEventListener("mousemove", handler);
          }}
          style={{ padding: "4px 12px 8px", borderTop: "1px solid rgba(255,255,255,0.05)" }}
        >
          {paramEntries.map(([name, value]) => (
            <ParamRowInline
              key={name}
              nodeId={data.id}
              name={name}
              value={value}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export default memo(CustomNode);
