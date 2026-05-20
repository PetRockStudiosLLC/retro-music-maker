/** Properties panel for editing node parameters.
 *  Shows parameters for the selected node with sliders and inputs. */
import { memo, useCallback, useState, useEffect } from "react";
import type { NodeData } from "../types/nodes";

interface PropertiesPanelProps {
  selectedNode: NodeData | null;
  onParamChange: (nodeId: string, paramName: string, value: number | string | boolean) => void;
}

function PropertiesPanel({ selectedNode, onParamChange }: PropertiesPanelProps) {
  const [localValues, setLocalValues] = useState<Record<string, number | string | boolean>>({});

  useEffect(() => {
    if (selectedNode) {
      setLocalValues({ ...selectedNode.params });
    } else {
      setLocalValues({});
    }
  }, [selectedNode]);

  const handleParamChange = useCallback(
    (nodeId: string, paramName: string, value: number | string | boolean) => {
      setLocalValues((prev) => ({ ...prev, [paramName]: value }));
      onParamChange(nodeId, paramName, value);
    },
    [onParamChange]
  );

  if (!selectedNode) {
    return (
      <div
        style={{
          width: 280,
          height: "100%",
          background: "#1e2030",
          borderLeft: "1px solid #3d4060",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "#565a7e",
          fontSize: 13,
          fontFamily: "'Segoe UI', system-ui, sans-serif",
        }}
      >
        <div style={{ textAlign: "center" }}>
          <div style={{ marginBottom: 8, fontSize: 32 }}>◆</div>
          No node selected
        </div>
      </div>
    );
  }

  const categoryColor = getCategoryColor(selectedNode.category);

  return (
    <div
      style={{
        width: 280,
        height: "100%",
        background: "#1e2030",
        borderLeft: "1px solid #3d4060",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      {/* Node header */}
      <div
        style={{
          padding: "16px 20px 12px",
          borderBottom: "1px solid #3d4060",
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 8,
            marginBottom: 4,
          }}
        >
          <div
            style={{
              width: 8,
              height: 8,
              borderRadius: "50%",
              background: categoryColor,
              flexShrink: 0,
            }}
          />
          <span
            style={{
              color: "#e1e4ff",
              fontSize: 14,
              fontWeight: 700,
              fontFamily: "'Segoe UI', system-ui, sans-serif",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
            }}
          >
            {selectedNode.id}
          </span>
        </div>
        <div
          style={{
            color: "#565a7e",
            fontSize: 11,
            fontFamily: "'Segoe UI', system-ui, sans-serif",
          }}
        >
          {selectedNode.category} • {selectedNode.type}
        </div>
      </div>

      {/* Parameters */}
      <div
        style={{
          flex: 1,
          overflowY: "auto",
          padding: 16,
        }}
      >
        {Object.keys(selectedNode.params).length > 0 ? (
          Object.entries(selectedNode.params)
            .filter(([k]) => !k.startsWith("_"))
            .map(([name, value]) => (
              <ParamRow
                key={name}
                name={name}
                value={value}
                localValue={localValues[name] ?? value}
                onChange={(val) => handleParamChange(selectedNode.id, name, val)}
              />
            ))
        ) : (
          <div style={{ color: "#565a7e", fontSize: 12, textAlign: "center", marginTop: 40 }}>
            No parameters for this node
          </div>
        )}
      </div>
    </div>
  );
}

function getCategoryColor(category: string): string {
  const colors: Record<string, string> = {
    Oscillator: "#7aa2f7",
    Synthesizer: "#e0af68",
    Filter: "#73daca",
    Envelope: "#9ece6a",
    Effects: "#e0af68",
    Mixer: "#bb9af7",
    Input: "#bb9af7",
    Output: "#c0caf5",
    Sequencer: "#7dcfff",
  };
  return colors[category] ?? "#c0caf5";
}

interface ParamRowProps {
  name: string;
  value: number | string | boolean;
  localValue: number | string | boolean;
  onChange: (value: number | string | boolean) => void;
}

const PARAM_RANGES: Record<string, { min: number; max: number; step: number }> = {
  frequency: { min: 20, max: 20000, step: 0.5 },
  amplitude: { min: 0, max: 1, step: 0.01 },
  pulse_width: { min: 0, max: 1, step: 0.01 },
  duty_cycle: { min: 0, max: 1, step: 0.01 },
  attack: { min: 0.001, max: 5, step: 0.001 },
  decay: { min: 0.001, max: 5, step: 0.001 },
  sustain: { min: 0, max: 1, step: 0.01 },
  release: { min: 0.001, max: 5, step: 0.001 },
  time: { min: 0, max: 2, step: 0.001 },
  feedback: { min: 0, max: 0.95, step: 0.01 },
  mix: { min: 0, max: 1, step: 0.01 },
  cutoff: { min: 20, max: 22050, step: 1 },
  resonance: { min: 0.1, max: 10, step: 0.1 },
  volume: { min: 0, max: 2, step: 0.01 },
  amount: { min: 0, max: 1, step: 0.01 },
  bit_depth: { min: 1, max: 16, step: 1 },
  sample_rate: { min: 100, max: 44100, step: 100 },
  bpm: { min: 20, max: 300, step: 1 },
};

function getParamRange(name: string) {
  return PARAM_RANGES[name] ?? { min: 0, max: 100, step: 0.01 };
}

function ParamRow({ name, localValue, onChange }: ParamRowProps) {
  const numValue = typeof localValue === "number" ? localValue : parseFloat(localValue as string);
  const isNumber = typeof localValue === "number" || !isNaN(parseFloat(localValue as string));
  const range = getParamRange(name);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const v = e.target.value;
      if (typeof localValue === "boolean") {
        onChange(v === "true");
      } else {
        onChange(v);
      }
    },
    [localValue, onChange]
  );

  const handleSliderChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onChange(parseFloat(e.target.value));
    },
    [onChange]
  );

  return (
    <div style={{ marginBottom: 12 }}>
      <label
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          marginBottom: 4,
        }}
      >
        <span
          style={{
            color: "#9999a5",
            fontSize: 11,
            fontFamily: "'Segoe UI', system-ui, sans-serif",
            textTransform: "capitalize",
          }}
        >
          {name.replace(/_/g, " ")}
        </span>
        {isNumber ? (
          <span
            style={{
              color: "#c0caf5",
              fontSize: 11,
              fontFamily: "Consolas, monospace",
            }}
          >
            {numValue % 1 !== 0 ? numValue.toFixed(3) : numValue}
          </span>
        ) : (
          <span
            style={{
              color: "#c0caf5",
              fontSize: 11,
              fontFamily: "Consolas, monospace",
            }}
          >
            {String(localValue)}
          </span>
        )}
      </label>

      {isNumber ? (
        <>
          <input
            type="range"
            min={range.min}
            max={range.max}
            step={range.step}
            value={numValue}
            onChange={handleSliderChange}
            style={{
              width: "100%",
              accentColor: "#7aa2f7",
            }}
          />
          <input
            type="text"
            value={numValue}
            onChange={handleChange}
            style={{
              width: "100%",
              marginTop: 4,
              padding: "4px 8px",
              background: "#13141e",
              border: "1px solid #3d4060",
              borderRadius: 3,
              color: "#c0caf5",
              fontSize: 11,
              fontFamily: "Consolas, monospace",
              outline: "none",
              boxSizing: "border-box",
            }}
          />
        </>
      ) : (
        <input
          type="text"
          value={String(localValue)}
          onChange={handleChange}
          style={{
            width: "100%",
            padding: "4px 8px",
            background: "#13141e",
            border: "1px solid #3d4060",
            borderRadius: 3,
            color: "#c0caf5",
            fontSize: 11,
            fontFamily: "'Segoe UI', system-ui, sans-serif",
            outline: "none",
            boxSizing: "border-box",
          }}
        />
      )}
    </div>
  );
}

export default memo(PropertiesPanel);
