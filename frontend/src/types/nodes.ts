/** TypeScript types for Retro Music Maker nodes and engine state. */

export interface PortInfo {
  name: string;
  type: "AudioPort" | "ControlPort" | "TriggerPort";
}

export interface NodeTypeInfo {
  name: string;
  category: string;
  inputs?: Array<{name: string; type: string}>;
  outputs?: Array<{name: string; type: string}>;
  params?: Record<string, number | string | boolean>;
}

export interface GraphEdge {
  source: string;
  sourceHandle: string;
  target: string;
  targetHandle: string;
}

export interface EngineState {
  nodes: Array<{
    id: string;
    type: string;
    category: string;
    params: Record<string, number | string | boolean>;
    inputs: Array<{ name: string; type: string }>;
    outputs: Array<{ name: string; type: string }>;
    position?: [number, number];
    node_kind?: "Builtin" | "Template" | "Script";
    definition?: any;
  }>;
  edges: GraphEdge[];
}

export interface ConsoleMessage {
  id: string;
  text: string;
  type: "info" | "success" | "error" | "warning";
  timestamp: number;
}

export type AudioStatus = "stopped" | "running" | "exporting";

export interface NodeData {
  id: string;
  type: string;
  category: string;
  params: Record<string, number | string | boolean>;
  inputs: Array<{ name: string; type: string }>;
  outputs: Array<{ name: string; type: string }>;
}
