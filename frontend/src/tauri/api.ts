import { invoke } from '@tauri-apps/api/core';

export interface NodeInfo {
  id: string;
  name: string;
  category: string;
  inputs: Array<{ name: string; port_type: string }>;
  outputs: Array<{ name: string; port_type: string }>;
  params: Record<string, any>;
  position: [number, number];
  node_kind?: "Builtin" | "Template" | "Script";
  definition?: any;
}

export interface Edge {
  source: string;
  source_handle: string;
  target: string;
  target_handle: string;
}

export interface GraphState {
  nodes: NodeInfo[];
  edges: Edge[];
}

export interface EngineStatus {
  running: boolean;
  sample_rate: number;
  block_size: number;
}

export async function addNode(nodeType: string, position: [number, number]): Promise<string> {
  const result = await invoke<NodeInfo>('add_node', {
    request: { node_type: nodeType, position },
  });
  return result.id;
}

export async function removeNode(nodeId: string): Promise<void> {
  await invoke('remove_node', { request: { node_id: nodeId } });
}

export async function connectNodes(source: string, sourceHandle: string, target: string, targetHandle: string): Promise<void> {
  await invoke('connect_nodes', {
    request: { source, source_handle: sourceHandle, target, target_handle: targetHandle },
  });
}

export async function disconnectNodes(source: string, target: string): Promise<void> {
  await invoke('disconnect_nodes', {
    request: { source, target },
  });
}

export async function setParam(nodeId: string, paramName: string, value: any): Promise<void> {
  await invoke('set_param', {
    request: { node_id: nodeId, param_name: paramName, value },
  });
}

export async function getNode(nodeId: string): Promise<NodeInfo | null> {
  return invoke<NodeInfo | null>('get_node', { request: { node_id: nodeId } });
}

export async function getGraphState(): Promise<GraphState> {
  return invoke<GraphState>('get_graph_state');
}

export async function startAudio(): Promise<void> {
  await invoke('start_audio');
}

export async function stopAudio(): Promise<void> {
  await invoke('stop_audio');
}

export async function engineStatus(): Promise<EngineStatus> {
  return invoke<EngineStatus>('engine_status');
}

export async function saveGraph(path: string): Promise<void> {
  await invoke('save_graph', { request: { path } });
}

export async function loadGraph(path: string): Promise<void> {
  await invoke('load_graph', { request: { path } });
}

export async function exportWav(path: string, duration: number, sampleRate?: number): Promise<void> {
  await invoke('export_wav', { request: { path, duration, sample_rate: sampleRate || null } });
}

export async function clearGraph(): Promise<void> {
  await invoke('clear_graph');
}

export async function saveFileOutput(nodeId: string): Promise<void> {
  await invoke('save_file_output', {
    request: { node_id: nodeId },
  });
}

export async function openWavFileDialog(): Promise<string | null> {
  return invoke<string | null>('open_wav_file_dialog');
}

// --- Template API ---

export interface TemplateExposedParam {
  param: string;
  label: string;
}

export interface TemplateInternalNode {
  id: string;
  node_type: string;
  position: [number, number];
  params: Record<string, any>;
}

export interface TemplateEdge {
  source: string;
  source_handle: string;
  target: string;
  target_handle: string;
}

export interface TemplateInputRouting {
  external: string;
  internal_node: string;
  internal_port: string;
}

export interface TemplateOutputRouting {
  internal_node: string;
  internal_port: string;
  external: string;
}

export interface TemplateDefinition {
  name: string;
  category: string;
  description: string;
  inputs: Array<{ name: string; port_type: string }>;
  outputs: Array<{ name: string; port_type: string }>;
  exposed_params: TemplateExposedParam[];
  internal_nodes: TemplateInternalNode[];
  internal_edges: TemplateEdge[];
  input_routing: TemplateInputRouting[];
  output_routing: TemplateOutputRouting[];
}

export interface TemplateInfo {
  name: string;
  category: string;
  description: string;
  inputs: Array<{ name: string; port_type: string }>;
  outputs: Array<{ name: string; port_type: string }>;
  exposed_params: string[];
}

export async function createTemplate(definition: TemplateDefinition): Promise<TemplateInfo> {
  return invoke<TemplateInfo>('create_template', { request: { definition } });
}

export async function listTemplates(): Promise<TemplateInfo[]> {
  return invoke<TemplateInfo[]>('list_templates');
}

export async function deleteTemplate(name: string): Promise<boolean> {
  return invoke<boolean>('delete_template', { params: { name } });
}

export async function loadTemplate(name: string): Promise<TemplateDefinition> {
  return invoke<TemplateDefinition>('load_template', { params: { name } });
}

// --- Script Node API ---

export interface ScriptParamDef {
  name: string;
  type: string;
  default: any;
  min?: any;
  max?: any;
}

export interface ScriptPortDef {
  name: string;
  type: string;
}

export interface ScriptDefinition {
  name: string;
  category: string;
  description: string;
  params: ScriptParamDef[];
  inputs: ScriptPortDef[];
  outputs: ScriptPortDef[];
  script: string;
}

export interface ScriptInfo {
  name: string;
  category: string;
  description: string;
  inputs: Array<{ name: string; port_type: string }>;
  outputs: Array<{ name: string; port_type: string }>;
  params: string[];
}

export async function createScript(definition: ScriptDefinition): Promise<ScriptInfo> {
  return invoke<ScriptInfo>('create_script', { request: { definition } });
}

export async function listScripts(): Promise<ScriptInfo[]> {
  return invoke<ScriptInfo[]>('list_scripts');
}

export async function deleteScript(name: string): Promise<boolean> {
  return invoke<boolean>('delete_script', { name });
}
