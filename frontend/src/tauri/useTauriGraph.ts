import { invoke } from '@tauri-apps/api/core';
import { useState, useEffect, useCallback, useRef } from 'react';

export interface NodeInfo {
  id: string;
  name: string;
  category: string;
  inputs: Array<{ name: string; port_type: string }>;
  outputs: Array<{ name: string; port_type: string }>;
  params: Record<string, any>;
  position: [number, number];
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

// Tauri IPC commands
export async function addNode(nodeType: string, position: [number, number] = [0, 0]): Promise<string> {
  const result: any = await invoke('add_node', {
    request: { node_type: nodeType, position },
  });
  return result.node_id;
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

export async function setNodePosition(nodeId: string, position: [number, number]): Promise<void> {
  await invoke('set_node_position', { request: { node_id: nodeId, position } });
}

export async function setParam(nodeId: string, paramName: string, value: any): Promise<void> {
  await invoke('set_param', { request: { node_id: nodeId, param_name: paramName, value } });
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

export async function saveWavFileDialog(): Promise<string | null> {
  return invoke<string | null>('save_wav_file_dialog');
}

export async function clearGraph(): Promise<void> {
  await invoke('clear_graph');
}

// Hook to replace useWebSocket
export function useTauriGraph() {
  const [graphState, setGraphState] = useState<GraphState>({ nodes: [], edges: [] });
  const [nodeTypes, setNodeTypes] = useState<string[]>([]);
  const [isPlaying, setIsPlaying] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const refreshTimer = useRef<number | null>(null);

  const refreshGraph = useCallback(async () => {
    try {
      const state = await getGraphState();
      setGraphState(state);
    } catch (err) {
      console.error('Failed to refresh graph:', err);
    }
  }, []);

  useEffect(() => {
    const init = async () => {
      try {
        await refreshGraph();
        const status = await engineStatus();
        setIsPlaying(status.running);
      } catch (err) {
        console.error('Failed to initialize:', err);
      } finally {
        setIsLoading(false);
      }
    };
    init();

    refreshTimer.current = window.setInterval(refreshGraph, 500);
    return () => {
      if (refreshTimer.current) {
        clearInterval(refreshTimer.current);
      }
    };
  }, [refreshGraph]);

  const handleNodeAdd = async (nodeType: string, position: [number, number] = [0, 0]) => {
    try {
      await addNode(nodeType, position);
      await refreshGraph();
    } catch (err) {
      console.error('Failed to add node:', err);
    }
  };

  const handleNodePositionChange = async (nodeId: string, position: [number, number]) => {
    try {
      await setNodePosition(nodeId, position);
    } catch (err) {
      console.error('Failed to set node position:', err);
    }
  };

  const handleNodeRemove = async (nodeId: string) => {
    try {
      await removeNode(nodeId);
      await refreshGraph();
    } catch (err) {
      console.error('Failed to remove node:', err);
    }
  };

  const handleConnect = async (source: string, sourceHandle: string, target: string, targetHandle: string) => {
    try {
      await connectNodes(source, sourceHandle, target, targetHandle);
      await refreshGraph();
    } catch (err) {
      console.error('Failed to connect nodes:', err);
    }
  };

  const handleDisconnect = async (source: string, target: string) => {
    try {
      await disconnectNodes(source, target);
      await refreshGraph();
    } catch (err) {
      console.error('Failed to disconnect nodes:', err);
    }
  };

  const handleParamChange = async (nodeId: string, paramName: string, value: any) => {
    try {
      await setParam(nodeId, paramName, value);
      await refreshGraph();
    } catch (err) {
      console.error('Failed to set param:', err);
    }
  };

  const handlePlay = async () => {
    try {
      await startAudio();
      setIsPlaying(true);
    } catch (err) {
      console.error('Failed to start audio:', err);
    }
  };

  const handleStop = async () => {
    try {
      await stopAudio();
      setIsPlaying(false);
    } catch (err) {
      console.error('Failed to stop audio:', err);
    }
  };

  const handleExport = async (path: string, duration: number, sampleRate?: number) => {
    try {
      await exportWav(path, duration, sampleRate);
    } catch (err) {
      console.error('Failed to export:', err);
    }
  };

  const handleClear = async () => {
    try {
      await clearGraph();
      await refreshGraph();
    } catch (err) {
      console.error('Failed to clear graph:', err);
    }
  };

  const handleSave = async (path: string) => {
    try {
      await saveGraph(path);
    } catch (err) {
      console.error('Failed to save graph:', err);
    }
  };

  const handleLoadPreset = async (presetData: { nodes: any[]; edges: any[] }) => {
    try {
      await clearGraph();
      const idMap: Record<string, string> = {};
      for (const n of presetData.nodes) {
        const nodeType = n.name || n.type;
        if (nodeType) {
          const backendId = await addNode(nodeType, n.position || [0, 0]);
          idMap[n.id] = backendId;
        }
      }
      // Apply params
      for (const n of presetData.nodes) {
        const backendId = idMap[n.id];
        if (backendId && n.params) {
          for (const [key, value] of Object.entries(n.params)) {
            await setParam(backendId, key, value);
          }
        }
      }
      await refreshGraph();
      for (const e of presetData.edges) {
        const sourceId = idMap[e.source] || e.source;
        const targetId = idMap[e.target] || e.target;
        await connectNodes(
          sourceId,
          e.source_handle || e.sourceHandle || 'Audio',
          targetId,
          e.target_handle || e.targetHandle || 'Audio'
        );
      }
      await refreshGraph();
    } catch (err) {
      console.error('Failed to load preset:', err);
    }
  };

  return {
    graphState,
    nodeTypes,
    isPlaying,
    isLoading,
    refreshGraph,
    handleNodeAdd,
    handleNodePositionChange,
    handleNodeRemove,
    handleConnect,
    handleDisconnect,
    handleParamChange,
    handlePlay,
    handleStop,
    handleExport,
    handleClear,
    handleSave,
    handleLoadPreset,
  };
}
