import React, { useState, useEffect } from 'react';
import { createScript, listScripts, deleteScript, ScriptDefinition, ScriptInfo, ScriptParamDef, ScriptPortDef } from '../tauri/api';

const defaultScript = `-- Script Node: Simple Gain
-- The process function is called for each audio block
-- params: table of parameter values
-- input: table of input samples (1-indexed)
-- output: table to write output samples (1-indexed)

function process(node, input, output, params)
  local gain = params.gain or 1.0
  for i = 1, output.size do
    output[i] = (input[i] or 0) * gain
  end
end`;

const defaultCategory = "Custom";

export default function ScriptEditor({ onClose, onSave }: { onClose: () => void; onSave: () => void }) {
  const [name, setName] = useState("MyScript");
  const [category, setCategory] = useState(defaultCategory);
  const [description, setDescription] = useState("");
  const [script, setScript] = useState(defaultScript);
  const [inputs, setInputs] = useState<ScriptPortDef[]>([{ name: "Audio", type: "audio" }]);
  const [outputs, setOutputs] = useState<ScriptPortDef[]>([{ name: "Audio", type: "audio" }]);
  const [params, setParams] = useState<ScriptParamDef[]>([
    { name: "gain", type: "float", default: 1.0, min: 0, max: 2 },
  ]);
  const [savedScripts, setSavedScripts] = useState<ScriptInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadScripts();
  }, []);

  const loadScripts = async () => {
    try {
      const scripts = await listScripts();
      setSavedScripts(scripts);
    } catch (e) {
      // ignore
    }
  };

  const handleSave = async () => {
    if (!name.trim()) {
      setError("Name is required");
      return;
    }
    setSaving(true);
    setError(null);
    try {
      const definition: ScriptDefinition = {
        name: name.trim(),
        category: category.trim() || defaultCategory,
        description: description.trim(),
        params,
        inputs,
        outputs,
        script,
      };
      await createScript(definition);
      await loadScripts();
      onSave();
    } catch (e: any) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (scriptName: string) => {
    try {
      await deleteScript(scriptName);
      await loadScripts();
    } catch (e) {
      // ignore
    }
  };

  const addInput = () => setInputs([...inputs, { name: "Input", type: "audio" }]);
  const removeInput = (idx: number) => setInputs(inputs.filter((_, i) => i !== idx));
  const updateInput = (idx: number, field: keyof ScriptPortDef, value: string) => {
    const updated = [...inputs];
    (updated[idx] as any)[field] = value;
    setInputs(updated);
  };

  const addOutput = () => setOutputs([...outputs, { name: "Output", type: "audio" }]);
  const removeOutput = (idx: number) => setOutputs(outputs.filter((_, i) => i !== idx));
  const updateOutput = (idx: number, field: keyof ScriptPortDef, value: string) => {
    const updated = [...outputs];
    (updated[idx] as any)[field] = value;
    setOutputs(updated);
  };

  const addParam = () => setParams([...params, { name: "param", type: "float", default: 0, min: 0, max: 1 }]);
  const removeParam = (idx: number) => setParams(params.filter((_, i) => i !== idx));
  const updateParam = (idx: number, field: keyof ScriptParamDef, value: any) => {
    const updated = [...params];
    (updated[idx] as any)[field] = value;
    setParams(updated);
  };

  return (
    <div style={{
      position: "fixed", inset: 0, zIndex: 10000,
      background: "rgba(0,0,0,0.7)", display: "flex", alignItems: "center", justifyContent: "center",
    }} onClick={onClose}>
      <div
        style={{
          background: "#1a1b26", borderRadius: 8, width: 900, maxHeight: "90vh",
          display: "flex", flexDirection: "column", overflow: "hidden",
          boxShadow: "0 16px 48px rgba(0,0,0,0.6)", border: "1px solid #3d4060",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div style={{ padding: "16px 20px", borderBottom: "1px solid #3d4060", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <span style={{ fontSize: 14, fontWeight: 700, color: "#e1e4ff" }}>📜 Script Node Editor</span>
          <button onClick={onClose} style={{ background: "none", border: "none", color: "#787cb8", fontSize: 20, cursor: "pointer" }}>✕</button>
        </div>

        <div style={{ flex: 1, overflow: "auto", padding: 20 }}>
          {/* Basic Info */}
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12, marginBottom: 16 }}>
            <div>
              <label style={labelStyle}>Name</label>
              <input style={inputStyle} value={name} onChange={(e) => setName(e.target.value)} placeholder="MyScript" />
            </div>
            <div>
              <label style={labelStyle}>Category</label>
              <input style={inputStyle} value={category} onChange={(e) => setCategory(e.target.value)} placeholder="Custom" />
            </div>
          </div>
          <div style={{ marginBottom: 16 }}>
            <label style={labelStyle}>Description</label>
            <input style={inputStyle} value={description} onChange={(e) => setDescription(e.target.value)} placeholder="What this script does..." />
          </div>

          {/* Script Editor */}
          <div style={{ marginBottom: 16 }}>
            <label style={labelStyle}>Lua Script</label>
            <textarea
              style={{
                ...inputStyle, minHeight: 200, fontFamily: "'Cascadia Code', 'Fira Code', monospace",
                fontSize: 12, background: "#13141e", color: "#c0caf5", lineHeight: 1.6,
              }}
              value={script}
              onChange={(e) => setScript(e.target.value)}
              spellCheck={false}
            />
            <div style={{ fontSize: 10, color: "#565a7e", marginTop: 4 }}>
              Must define a <code style={{ color: "#7aa2f7" }}>process(node, input, output, params)</code> function.
            </div>
          </div>

          {/* Ports */}
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 16 }}>
            <div>
              <div style={{ ...sectionHeaderStyle, borderColor: "#7aa2f7" }}>
                Inputs ({inputs.length})
                <button onClick={addInput} style={addBtnStyle}>+</button>
              </div>
              {inputs.map((inp, i) => (
                <div key={i} style={{ display: "flex", gap: 6, marginBottom: 6 }}>
                  <input style={{ ...inputStyle, flex: 1, minHeight: 28, padding: "4px 8px" }} value={inp.name} onChange={(e) => updateInput(i, "name", e.target.value)} />
                  <select style={{ ...inputStyle, width: 80, minHeight: 28, padding: "4px 4px" }} value={inp.type} onChange={(e) => updateInput(i, "type" as any, e.target.value)}>
                    <option value="audio">audio</option>
                    <option value="control">control</option>
                    <option value="trigger">trigger</option>
                  </select>
                  <button onClick={() => removeInput(i)} style={removeBtnStyle}>✕</button>
                </div>
              ))}
            </div>
            <div>
              <div style={{ ...sectionHeaderStyle, borderColor: "#9ece6a" }}>
                Outputs ({outputs.length})
                <button onClick={addOutput} style={addBtnStyle}>+</button>
              </div>
              {outputs.map((out, i) => (
                <div key={i} style={{ display: "flex", gap: 6, marginBottom: 6 }}>
                  <input style={{ ...inputStyle, flex: 1, minHeight: 28, padding: "4px 8px" }} value={out.name} onChange={(e) => updateOutput(i, "name", e.target.value)} />
                  <select style={{ ...inputStyle, width: 80, minHeight: 28, padding: "4px 4px" }} value={out.type} onChange={(e) => updateOutput(i, "type" as any, e.target.value)}>
                    <option value="audio">audio</option>
                    <option value="control">control</option>
                    <option value="trigger">trigger</option>
                  </select>
                  <button onClick={() => removeOutput(i)} style={removeBtnStyle}>✕</button>
                </div>
              ))}
            </div>
          </div>

          {/* Parameters */}
          <div style={{ marginBottom: 16 }}>
            <div style={{ ...sectionHeaderStyle, borderColor: "#e0af68" }}>
              Parameters ({params.length})
              <button onClick={addParam} style={addBtnStyle}>+</button>
            </div>
            {params.map((p, i) => (
              <div key={i} style={{ display: "flex", gap: 6, marginBottom: 6, alignItems: "center" }}>
                <input style={{ ...inputStyle, width: 100, minHeight: 28, padding: "4px 8px" }} value={p.name} onChange={(e) => updateParam(i, "name", e.target.value)} placeholder="name" />
                <select style={{ ...inputStyle, width: 70, minHeight: 28, padding: "4px 4px" }} value={p.type} onChange={(e) => updateParam(i, "type" as any, e.target.value)}>
                  <option value="float">float</option>
                  <option value="int">int</option>
                  <option value="bool">bool</option>
                  <option value="string">string</option>
                </select>
                <input style={{ ...inputStyle, width: 60, minHeight: 28, padding: "4px 8px" }} type="number" value={p.default} onChange={(e) => updateParam(i, "default", parseFloat(e.target.value))} placeholder="default" />
                <input style={{ ...inputStyle, width: 50, minHeight: 28, padding: "4px 8px" }} type="number" value={p.min ?? ""} onChange={(e) => updateParam(i, "min", e.target.value ? parseFloat(e.target.value) : undefined)} placeholder="min" />
                <input style={{ ...inputStyle, width: 50, minHeight: 28, padding: "4px 8px" }} type="number" value={p.max ?? ""} onChange={(e) => updateParam(i, "max", e.target.value ? parseFloat(e.target.value) : undefined)} placeholder="max" />
                <button onClick={() => removeParam(i)} style={removeBtnStyle}>✕</button>
              </div>
            ))}
          </div>

          {/* Saved Scripts */}
          {savedScripts.length > 0 && (
            <div>
              <div style={sectionHeaderStyle}>Saved Scripts</div>
              {savedScripts.map((s) => (
                <div key={s.name} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "6px 10px", background: "#13141e", borderRadius: 4, marginBottom: 4 }}>
                  <div>
                    <span style={{ color: "#c0caf5", fontSize: 12, fontWeight: 600 }}>{s.name}</span>
                    <span style={{ color: "#565a7e", fontSize: 10, marginLeft: 8 }}>{s.category}</span>
                    {s.description && <span style={{ color: "#787cb8", fontSize: 10, marginLeft: 8 }}>- {s.description}</span>}
                  </div>
                  <button onClick={() => handleDelete(s.name)} style={{ ...removeBtnStyle, fontSize: 10 }}>Delete</button>
                </div>
              ))}
            </div>
          )}

          {error && (
            <div style={{ marginTop: 12, padding: "8px 12px", background: "#2c1f1f", border: "1px solid #f7768e", borderRadius: 4, color: "#f7768e", fontSize: 12 }}>
              {error}
            </div>
          )}
        </div>

        {/* Footer */}
        <div style={{ padding: "12px 20px", borderTop: "1px solid #3d4060", display: "flex", justifyContent: "flex-end", gap: 8 }}>
          <button onClick={onClose} style={{ ...btnStyle, background: "#1e2030", color: "#787cb8" }}>Cancel</button>
          <button onClick={handleSave} disabled={saving} style={{ ...btnStyle, background: "#7aa2f7", color: "#1a1b26", fontWeight: 700, opacity: saving ? 0.5 : 1 }}>
            {saving ? "Saving..." : "Save Script"}
          </button>
        </div>
      </div>
    </div>
  );
}

const labelStyle: React.CSSProperties = {
  display: "block", fontSize: 10, fontWeight: 700, color: "#787cb8",
  textTransform: "uppercase", letterSpacing: 1, marginBottom: 4,
};

const inputStyle: React.CSSProperties = {
  width: "100%", padding: "6px 10px", background: "#13141e", border: "1px solid #3d4060",
  borderRadius: 4, color: "#c0caf5", fontSize: 12, fontFamily: "'Segoe UI', system-ui, sans-serif",
  boxSizing: "border-box", outline: "none",
};

const sectionHeaderStyle: React.CSSProperties = {
  fontSize: 11, fontWeight: 700, color: "#a9b1d6", marginBottom: 8,
  display: "flex", alignItems: "center", justifyContent: "space-between",
  paddingBottom: 4, borderBottom: "1px solid #3d4060",
};

const addBtnStyle: React.CSSProperties = {
  background: "#7aa2f7", color: "#1a1b26", border: "none", borderRadius: 3,
  width: 22, height: 22, fontSize: 14, cursor: "pointer", fontWeight: 700,
};

const removeBtnStyle: React.CSSProperties = {
  background: "none", border: "none", color: "#f7768e", cursor: "pointer",
  fontSize: 12, padding: "2px 4px",
};

const btnStyle: React.CSSProperties = {
  padding: "8px 16px", border: "none", borderRadius: 4, fontSize: 12,
  cursor: "pointer", fontFamily: "'Segoe UI', system-ui, sans-serif",
};
