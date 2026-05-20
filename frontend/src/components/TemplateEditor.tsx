import { useState } from "react";
import type { TemplateDefinition, TemplateInternalNode, TemplateEdge, TemplateInputRouting, TemplateOutputRouting } from "../tauri/api";
import { createTemplate, listTemplates, deleteTemplate } from "../tauri/api";

interface TemplateEditorProps {
  onClose: () => void;
  onSave: () => void;
}

const NODE_CATEGORIES: Record<string, string[]> = {
  Oscillator: ["SineOscillator", "SquareOscillator", "SawtoothOscillator", "TriangleOscillator", "NoiseOscillator", "ChipSoundOscillator", "VCO"],
  Filter: ["LowpassFilter", "HighpassFilter", "BandpassFilter", "Bitcrush"],
  Envelope: ["ADSREnvelope"],
  Effect: ["DelayEffect", "Distortion", "Reverb"],
  Mixer: ["Mixer", "Gain"],
  Input: ["KeyboardInput", "MidiInput"],
  Output: ["AudioOutput", "FileOutput"],
  Sequencer: ["StepSequencer", "Arpeggiator", "NoteSequencer", "NoteMapper"],
  Controller: ["Instrument"],
};

export default function TemplateEditor({ onClose, onSave }: TemplateEditorProps) {
  const [name, setName] = useState("");
  const [category, setCategory] = useState("Custom");
  const [description, setDescription] = useState("");
  const [internalNodes, setInternalNodes] = useState<TemplateInternalNode[]>([]);
  const [internalEdges, setInternalEdges] = useState<TemplateEdge[]>([]);
  const [inputRouting, setInputRouting] = useState<TemplateInputRouting[]>([]);
  const [outputRouting, setOutputRouting] = useState<TemplateOutputRouting[]>([]);
  const [error, setError] = useState("");
  const [savedTemplates, setSavedTemplates] = useState<string[]>([]);

  const loadTemplates = async () => {
    try {
      const templates = await listTemplates();
      setSavedTemplates(templates.map((t) => t.name));
    } catch (e) {
      console.error("Failed to load templates:", e);
    }
  };

  const addInternalNode = (nodeType: string) => {
    const newNode: TemplateInternalNode = {
      id: `${nodeType}_${internalNodes.length}`,
      node_type: nodeType,
      position: [internalNodes.length * 200, 0],
      params: {},
    };
    setInternalNodes([...internalNodes, newNode]);
  };

  const removeInternalNode = (index: number) => {
    const nodeId = internalNodes[index].id;
    setInternalNodes(internalNodes.filter((_, i) => i !== index));
    setInternalEdges(internalEdges.filter((e) => e.source !== nodeId && e.target !== nodeId));
    setInputRouting(inputRouting.filter((r) => r.internal_node !== nodeId));
    setOutputRouting(outputRouting.filter((r) => r.internal_node !== nodeId));
  };

  const addEdge = () => {
    const newEdge: TemplateEdge = {
      source: internalNodes[0]?.id || "",
      source_handle: "audio",
      target: internalNodes[1]?.id || "",
      target_handle: "audio",
    };
    setInternalEdges([...internalEdges, newEdge]);
  };

  const updateEdge = (index: number, field: keyof TemplateEdge, value: string) => {
    const updated = [...internalEdges];
    (updated[index] as any)[field] = value;
    setInternalEdges(updated);
  };

  const removeEdge = (index: number) => {
    setInternalEdges(internalEdges.filter((_, i) => i !== index));
  };

  const addInputRouting = () => {
    const newRouting: TemplateInputRouting = {
      external: "audio",
      internal_node: internalNodes[0]?.id || "",
      internal_port: "audio",
    };
    setInputRouting([...inputRouting, newRouting]);
  };

  const updateInputRouting = (index: number, field: keyof TemplateInputRouting, value: string) => {
    const updated = [...inputRouting];
    (updated[index] as any)[field] = value;
    setInputRouting(updated);
  };

  const removeInputRouting = (index: number) => {
    setInputRouting(inputRouting.filter((_, i) => i !== index));
  };

  const addOutputRouting = () => {
    const newRouting: TemplateOutputRouting = {
      internal_node: internalNodes[internalNodes.length - 1]?.id || "",
      internal_port: "audio",
      external: "audio",
    };
    setOutputRouting([...outputRouting, newRouting]);
  };

  const updateOutputRouting = (index: number, field: keyof TemplateOutputRouting, value: string) => {
    const updated = [...outputRouting];
    (updated[index] as any)[field] = value;
    setOutputRouting(updated);
  };

  const removeOutputRouting = (index: number) => {
    setOutputRouting(outputRouting.filter((_, i) => i !== index));
  };

  const handleSave = async () => {
    if (!name.trim()) {
      setError("Template name is required");
      return;
    }
    if (internalNodes.length === 0) {
      setError("Template must have at least one internal node");
      return;
    }

    const definition: TemplateDefinition = {
      name: name.trim(),
      category,
      description,
      inputs: inputRouting.map((r) => ({ name: r.external, port_type: "audio" })),
      outputs: outputRouting.map((r) => ({ name: r.external, port_type: "audio" })),
      exposed_params: [],
      internal_nodes: internalNodes,
      internal_edges: internalEdges,
      input_routing: inputRouting,
      output_routing: outputRouting,
    };

    try {
      await createTemplate(definition);
      await loadTemplates();
      onSave();
      onClose();
    } catch (e) {
      setError(`Failed to save template: ${e}`);
    }
  };

  const handleDelete = async (templateName: string) => {
    try {
      await deleteTemplate(templateName);
      await loadTemplates();
    } catch (e) {
      setError(`Failed to delete template: ${e}`);
    }
  };

  return (
    <div className="template-editor-overlay">
      <div className="template-editor">
        <div className="template-editor-header">
          <h2>Create Template Node</h2>
          <button onClick={onClose} className="close-btn">X</button>
        </div>

        {error && <div className="error-message">{error}</div>}

        <div className="template-form">
          <div className="form-group">
            <label>Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="MyCustomNode"
            />
          </div>

          <div className="form-group">
            <label>Category</label>
            <input
              type="text"
              value={category}
              onChange={(e) => setCategory(e.target.value)}
              placeholder="Custom"
            />
          </div>

          <div className="form-group">
            <label>Description</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="What does this template do?"
              rows={2}
            />
          </div>

          <div className="section">
            <h3>Internal Nodes</h3>
            <div className="add-node-row">
              <select onChange={(e) => { addInternalNode(e.target.value); e.target.value = ""; }}>
                <option value="">Select node type...</option>
                {Object.entries(NODE_CATEGORIES).map(([cat, types]) => (
                  <optgroup key={cat} label={cat}>
                    {types.map((t) => (
                      <option key={t} value={t}>{t}</option>
                    ))}
                  </optgroup>
                ))}
              </select>
            </div>

            {internalNodes.map((node, i) => (
              <div key={i} className="internal-node-item">
                <span>{node.id} ({node.node_type})</span>
                <button onClick={() => removeInternalNode(i)}>Remove</button>
              </div>
            ))}
          </div>

          <div className="section">
            <h3>Internal Edges</h3>
            <button onClick={addEdge} disabled={internalNodes.length < 2}>+ Add Edge</button>
            {internalEdges.map((edge, i) => (
              <div key={i} className="edge-item">
                <select value={edge.source} onChange={(e) => updateEdge(i, "source", e.target.value)}>
                  {internalNodes.map((n) => <option key={n.id} value={n.id}>{n.id}</option>)}
                </select>
                <span>→</span>
                <select value={edge.target} onChange={(e) => updateEdge(i, "target", e.target.value)}>
                  {internalNodes.map((n) => <option key={n.id} value={n.id}>{n.id}</option>)}
                </select>
                <button onClick={() => removeEdge(i)}>×</button>
              </div>
            ))}
          </div>

          <div className="section">
            <h3>Input Routing</h3>
            <button onClick={addInputRouting} disabled={internalNodes.length === 0}>+ Add Input</button>
            {inputRouting.map((routing, i) => (
              <div key={i} className="routing-item">
                <input
                  type="text"
                  value={routing.external}
                  onChange={(e) => updateInputRouting(i, "external", e.target.value)}
                  placeholder="external port"
                  style={{ width: "100px" }}
                />
                <span>→</span>
                <select value={routing.internal_node} onChange={(e) => updateInputRouting(i, "internal_node", e.target.value)}>
                  {internalNodes.map((n) => <option key={n.id} value={n.id}>{n.id}</option>)}
                </select>
                <button onClick={() => removeInputRouting(i)}>×</button>
              </div>
            ))}
          </div>

          <div className="section">
            <h3>Output Routing</h3>
            <button onClick={addOutputRouting} disabled={internalNodes.length === 0}>+ Add Output</button>
            {outputRouting.map((routing, i) => (
              <div key={i} className="routing-item">
                <select value={routing.internal_node} onChange={(e) => updateOutputRouting(i, "internal_node", e.target.value)}>
                  {internalNodes.map((n) => <option key={n.id} value={n.id}>{n.id}</option>)}
                </select>
                <span>→</span>
                <input
                  type="text"
                  value={routing.external}
                  onChange={(e) => updateOutputRouting(i, "external", e.target.value)}
                  placeholder="external port"
                  style={{ width: "100px" }}
                />
                <button onClick={() => removeOutputRouting(i)}>×</button>
              </div>
            ))}
          </div>

          <div className="section">
            <h3>Saved Templates</h3>
            {savedTemplates.length === 0 ? (
              <p style={{ color: "#888" }}>No templates saved yet</p>
            ) : (
              savedTemplates.map((t) => (
                <div key={t} className="template-item">
                  <span>{t}</span>
                  <button onClick={() => handleDelete(t)}>Delete</button>
                </div>
              ))
            )}
          </div>

          <div className="template-actions">
            <button onClick={handleSave} className="save-btn">Save Template</button>
            <button onClick={onClose} className="cancel-btn">Cancel</button>
          </div>
        </div>
      </div>

      <style>{`
        .template-editor-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.7);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }

        .template-editor {
          background: #1a1a2e;
          border: 1px solid #333;
          border-radius: 8px;
          padding: 24px;
          width: 600px;
          max-height: 80vh;
          overflow-y: auto;
          color: #e0e0e0;
        }

        .template-editor-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 20px;
        }

        .template-editor-header h2 {
          margin: 0;
          color: #00d4ff;
        }

        .close-btn {
          background: #ff4444;
          color: white;
          border: none;
          border-radius: 4px;
          padding: 6px 12px;
          cursor: pointer;
        }

        .error-message {
          background: #ff4444;
          color: white;
          padding: 10px;
          border-radius: 4px;
          margin-bottom: 16px;
        }

        .form-group {
          margin-bottom: 16px;
        }

        .form-group label {
          display: block;
          margin-bottom: 6px;
          color: #aaa;
          font-size: 14px;
        }

        .form-group input,
        .form-group textarea,
        .form-group select {
          width: 100%;
          padding: 8px;
          background: #0d0d1a;
          border: 1px solid #333;
          border-radius: 4px;
          color: #e0e0e0;
          font-size: 14px;
        }

        .section {
          margin-top: 20px;
          padding-top: 16px;
          border-top: 1px solid #333;
        }

        .section h3 {
          margin: 0 0 12px 0;
          color: #00d4ff;
          font-size: 16px;
        }

        .internal-node-item,
        .edge-item,
        .routing-item,
        .template-item {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 8px;
          padding: 8px;
          background: #0d0d1a;
          border-radius: 4px;
        }

        .internal-node-item button,
        .edge-item button,
        .routing-item button,
        .template-item button {
          background: #ff4444;
          color: white;
          border: none;
          border-radius: 4px;
          padding: 4px 8px;
          cursor: pointer;
          font-size: 12px;
        }

        .edge-item select,
        .routing-item select {
          background: #0d0d1a;
          border: 1px solid #333;
          color: #e0e0e0;
          padding: 4px;
          border-radius: 4px;
        }

        .edge-item span,
        .routing-item span {
          color: #00d4ff;
        }

        .template-actions {
          display: flex;
          gap: 12px;
          margin-top: 24px;
          padding-top: 16px;
          border-top: 1px solid #333;
        }

        .save-btn {
          flex: 1;
          padding: 12px;
          background: #00d4ff;
          color: #000;
          border: none;
          border-radius: 4px;
          font-weight: bold;
          cursor: pointer;
        }

        .cancel-btn {
          flex: 1;
          padding: 12px;
          background: #333;
          color: #e0e0e0;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }

        .add-node-row select {
          width: 100%;
          padding: 8px;
          background: #0d0d1a;
          border: 1px solid #333;
          border-radius: 4px;
          color: #e0e0e0;
        }

        button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
      `}</style>
    </div>
  );
}
