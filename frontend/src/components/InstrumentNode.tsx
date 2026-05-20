import { memo, useState, useCallback, useRef, useEffect } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import { setParam, getGraphState } from "../tauri/useTauriGraph";
import { getPushUndoCallback } from "../undoBridge";

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

function getPortColor(portType: string | undefined): string {
  return portType ? (PORT_COLORS[portType] ?? PORT_COLORS["AudioPort"]) : PORT_COLORS["AudioPort"];
}

interface InstrumentData {
  id: string;
  type: string;
  category: string;
  params: Record<string, number | string | boolean>;
  inputs: Array<{ name: string; type?: string; port_type?: string }>;
  outputs: Array<{ name: string; type?: string; port_type?: string }>;
}

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

const waveOptions = ["square", "sawtooth", "triangle", "sine", "pulse", "pulse_half", "noise", "sample_hold"];

const PARAM_RANGES: Record<string, { min: number; max: number; step: number }> = {
  attack: { min: 0.001, max: 5, step: 0.001 },
  decay: { min: 0.001, max: 5, step: 0.001 },
  sustain: { min: 0, max: 1, step: 0.01 },
  release: { min: 0.001, max: 5, step: 0.001 },
  amplitude: { min: 0, max: 1, step: 0.01 },
};

function InlineParam({
  nodeId,
  name,
  value,
  isSelect,
  options,
  onParamChange,
}: {
  nodeId: string;
  name: string;
  value: number | string | boolean;
  isSelect?: boolean;
  options?: string[];
  onParamChange?: (nodeId: string, name: string, prevValue: any, newValue: any) => void;
}) {
  const [local, setLocal] = useState<number | string | boolean>(value);
  const debounceRef = useRef<number | null>(null);
  const prevValueRef = useRef<number | string | boolean>(value);

  useEffect(() => {
    setLocal(value);
    prevValueRef.current = value;
  }, [value]);

  useEffect(() => {
    return () => {
      if (debounceRef.current) window.clearTimeout(debounceRef.current);
    };
  }, []);

  const commitParam = useCallback(
    async (v: number | string | boolean) => {
      try {
        await setParam(nodeId, name, v);
        onParamChange?.(nodeId, name, prevValueRef.current, v);
        const pushUndo = getPushUndoCallback();
        if (pushUndo) {
          pushUndo({ type: "SET_PARAM", nodeId, paramName: name, prevValue: prevValueRef.current, newValue: v });
        }
      } catch (err) {
        console.error("Failed to set param:", err);
        setLocal(prevValueRef.current);
      }
    },
    [nodeId, name, onParamChange]
  );

  const handleChange = useCallback(
    (v: number | string | boolean) => {
      setLocal(v);
      if (debounceRef.current) window.clearTimeout(debounceRef.current);
      debounceRef.current = window.setTimeout(() => {
        commitParam(v);
      }, 150);
    },
    [commitParam]
  );

  const isNum = typeof value === "number";
  const range = PARAM_RANGES[name] || { min: 0, max: 100, step: 0.01 };
  const label = name.replace(/_/g, " ");

  if (isSelect && options) {
    return (
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "2px 0", gap: 6 }}>
        <span style={{ color: "#9999a5", fontSize: 10, textTransform: "capitalize", flexShrink: 0 }}>{label}</span>
        <select
          value={String(local)}
          onChange={(e) => {
            const v = e.target.value;
            handleChange(v);
          }}
          onBlur={() => {
            if (debounceRef.current) {
              window.clearTimeout(debounceRef.current);
              commitParam(local);
            }
          }}
          style={{
            background: "#13141e",
            border: "1px solid #3d4060",
            borderRadius: 3,
            color: "#c0caf5",
            fontSize: 10,
            padding: "2px 4px",
            outline: "none",
            fontFamily: "'Segoe UI', system-ui, sans-serif",
            cursor: "pointer",
            flex: 1,
          }}
        >
          {options.map((opt) => (
            <option key={opt} value={opt}>{opt}</option>
          ))}
        </select>
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
          onMouseUp={() => {
            if (debounceRef.current) {
              window.clearTimeout(debounceRef.current);
              commitParam(local);
            }
          }}
          onTouchEnd={() => {
            if (debounceRef.current) {
              window.clearTimeout(debounceRef.current);
              commitParam(local);
            }
          }}
          style={{ width: "100%", accentColor: "#7aa2f7" }}
        />
      </div>
    );
  }

  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "2px 0" }}>
      <span style={{ color: "#9999a5", fontSize: 10, textTransform: "capitalize" }}>{label}</span>
      <span style={{ color: "#c0caf5", fontSize: 10, fontFamily: "Consolas, monospace" }}>
        {String(local)}
      </span>
    </div>
  );
}

function InstrumentNode({ data: raw, selected }: NodeProps) {
  const data = raw as unknown as InstrumentData;
  const waveform = (data.params.waveform as string) || "square";
  const color = waveColors[waveform] || "#7aa2f7";
  const [localWaveform, setLocalWaveform] = useState(waveform);

  useEffect(() => {
    setLocalWaveform(waveform);
  }, [waveform]);

  const editableParams = [
    { name: "waveform", isSelect: true, options: waveOptions },
    { name: "attack", isSelect: false },
    { name: "decay", isSelect: false },
    { name: "sustain", isSelect: false },
    { name: "release", isSelect: false },
    { name: "amplitude", isSelect: false },
  ];

  const handleParamChange = useCallback(async (nodeId: string, paramName: string, prevValue: any, newValue: any) => {
    try {
      await getGraphState();
    } catch (err) {
      console.error("Failed to refresh graph after param change:", err);
    }
  }, []);

  return (
    <div
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
        fontFamily: "'Segoe UI', system-ui, sans-serif",
      }}
    >
      <div style={{ height: 4, background: color, borderRadius: "6px 6px 0 0" }} />

      <div
        style={{
          padding: "8px 12px",
          background: `rgba(${waveform === "square" ? "122,162,247" : waveform === "sawtooth" ? "224,175,104" : waveform === "triangle" ? "158,206,106" : "125,207,255"}, 0.15)`,
          borderBottom: "1px solid rgba(255,255,255,0.05)",
          display: "flex",
          alignItems: "center",
          gap: 8,
        }}
      >
        <div
          style={{
            width: 12,
            height: 12,
            borderRadius: "50%",
            background: color,
            boxShadow: `0 0 6px ${color}60`,
            flexShrink: 0,
          }}
        />
        <span style={{ color: "#e1e4ff", fontSize: 12, fontWeight: 700 }}>
          Instrument
        </span>
      </div>

      <div
        style={{ padding: "4px 12px" }}
      >
        {data.inputs?.length > 0 &&
          data.inputs.map((input: { name: string; type?: string; port_type?: string }) => (
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
                  background: getPortColor(input.port_type ?? input.type),
                  border: `2px solid ${getPortColor(input.port_type ?? input.type)}`,
                  borderRadius: "50%",
                  boxShadow: `0 0 6px ${getPortColor(input.port_type ?? input.type)}66`,
                }}
              />
              <span style={{ color: "#565a7e", fontSize: 11, paddingLeft: 12 }}>
                {input.name}
              </span>
            </div>
          ))}

        {data.outputs?.length > 0 &&
          data.outputs.map((output: { name: string; type?: string; port_type?: string }) => (
            <div
              key={`out-${output.name}`}
              style={{ display: "flex", alignItems: "center", height: 24, position: "relative" }}
            >
              <span style={{ color: "#565a7e", fontSize: 11, flex: 1, textAlign: "right", paddingRight: 28 }}>
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
                  background: getPortColor(output.port_type ?? output.type),
                  border: `2px solid ${getPortColor(output.port_type ?? output.type)}`,
                  borderRadius: "50%",
                  boxShadow: `0 0 6px ${getPortColor(output.port_type ?? output.type)}66`,
                }}
              />
            </div>
          ))}
      </div>

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
        {editableParams.map(({ name, isSelect, options }) => (
          <InlineParam
            key={name}
            nodeId={data.id}
            name={name}
            value={name === "waveform" ? localWaveform : data.params[name]}
            isSelect={isSelect}
            options={options}
            onParamChange={handleParamChange}
          />
        ))}
      </div>
    </div>
  );
}

export default memo(InstrumentNode);
