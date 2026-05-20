/** WebSocket hook for connecting to the Retro Music Maker backend.
 *  Uses REST for graph operations and WebSocket for live updates. */
import { useRef, useCallback, useEffect, useState } from "react";
import type {
  EngineState,
  ConsoleMessage,
  NodeData,
  AudioStatus,
  NodeTypeInfo,
} from "../types/nodes";

interface UseWebSocketReturn {
  ws: WebSocket | null;
  isConnected: boolean;
  state: EngineState;
  consoleMessages: ConsoleMessage[];
  nodeTypes: Record<string, NodeTypeInfo>;
  addNode: (nodeType: string) => void;
  removeNode: (nodeId: string) => void;
  connectPorts: (
    srcId: string,
    srcPort: string,
    dstId: string,
    dstPort: string
  ) => void;
  disconnectPorts: (srcId: string, dstId: string) => void;
  setParam: (nodeId: string, paramName: string, value: number | string | boolean) => void;
  startAudio: () => void;
  stopAudio: () => void;
  exportWav: () => Promise<Blob | null>;
  clearConsole: () => void;
  onAudioChunk: ((chunk: Float32Array) => void) | null;
  setOnAudioChunk: (cb: ((chunk: Float32Array) => void) | null) => void;
}

export function useWebSocket(): UseWebSocketReturn {
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [state, setState] = useState<EngineState>({ nodes: [], edges: [] });
  const [consoleMessages, setConsoleMessages] = useState<ConsoleMessage[]>([]);
  const [nodeTypes, setNodeTypes] = useState<
    Record<string, NodeTypeInfo>
  >({});
  const audioChunkCallbackRef = useRef<((chunk: Float32Array) => void) | null>(null);

  const addMessage = useCallback(
    (text: string, type: ConsoleMessage["type"] = "info") => {
      setConsoleMessages((prev) => [
        ...prev.slice(-99),
        {
          id: `${Date.now()}-${Math.random().toString(36).slice(2)}`,
          text,
          type,
          timestamp: Date.now(),
        },
      ]);
    },
    []
  );

  // Fetch initial state
  useEffect(() => {
    let cancelled = false;

    const loadState = async () => {
      try {
        const [typesResp, graphResp] = await Promise.all([
          fetch("/api/node-types"),
          fetch("/api/graph"),
        ]);

        if (!cancelled) {
          if (typesResp.ok) {
            const data = await typesResp.json();
            setNodeTypes(data);
            addMessage(`${Object.keys(data).length} node types loaded`, "success");
          }

          if (graphResp.ok) {
            const data = await graphResp.json();
            setState(data);
          }
        }
      } catch (err) {
        if (!cancelled) {
          addMessage("Failed to load initial state", "error");
        }
      }
    };

    loadState();

    return () => {
      cancelled = true;
    };
  }, []);

  // WebSocket connection with reconnection
  useEffect(() => {
    const ws = new WebSocket("ws://localhost:8000/ws");
    wsRef.current = ws;

    ws.onopen = () => {
      setIsConnected(true);
      addMessage("Connected to backend", "success");
    };

    ws.onclose = () => {
      setIsConnected(false);
      addMessage("Backend disconnected - retrying in 3s...", "error");
      reconnectTimeoutRef.current = setTimeout(() => {
        if (wsRef.current?.readyState === WebSocket.CLOSED) {
          const newWs = new WebSocket("ws://localhost:8000/ws");
          wsRef.current = newWs;
        }
      }, 3000);
    };

    ws.onerror = () => {
      setIsConnected(false);
    };

    ws.onmessage = (event) => {
      // Handle binary audio data
      if (event.data instanceof ArrayBuffer) {
        if (audioChunkCallbackRef.current) {
          const float32 = new Float32Array(event.data);
          audioChunkCallbackRef.current(float32);
        }
        return;
      }

      try {
        const msg = JSON.parse(typeof event.data === "string" ? event.data : new TextDecoder().decode(event.data));
        if (msg.type === "init") {
          setState(msg.data);
        }
      } catch {
        // Ignore parse errors
      }
    };

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      try {
        if (ws.readyState === WebSocket.OPEN) {
          ws.close();
        }
      } catch {
        // Already closed
      }
    };
  }, []);

  const setOnAudioChunk = useCallback((cb: ((chunk: Float32Array) => void) | null) => {
    audioChunkCallbackRef.current = cb;
  }, []);

  const addNode = useCallback(
    (_: string) => {
      // No-op - drag-and-drop uses REST directly
    },
    []
  );

  const removeNode = useCallback(
    (nodeId: string) => {
      fetch(`/api/node/${encodeURIComponent(nodeId)}`, { method: "DELETE" })
        .then(() => {
          // Node removed server-side
        })
        .catch(() => {});
    },
    []
  );

  const connectPorts = useCallback(
    (srcId: string, srcPort: string, dstId: string, dstPort: string) => {
      fetch(
        `/api/connect?src_id=${encodeURIComponent(srcId)}&src_port=${encodeURIComponent(srcPort)}&dst_id=${encodeURIComponent(dstId)}&dst_port=${encodeURIComponent(dstPort)}`,
        { method: "POST" }
      ).then(async (resp) => {
        if (!resp.ok) {
          const body = await resp.json().catch(() => ({}));
          addMessage(`Connect failed: ${body.detail || resp.statusText}`, "error");
        }
      }).catch(() => {});
    },
    [addMessage]
  );

  const disconnectPorts = useCallback(
    (srcId: string, dstId: string) => {
      fetch(
        `/api/connect?src_id=${encodeURIComponent(srcId)}&dst_id=${encodeURIComponent(dstId)}`,
        { method: "DELETE" }
      ).catch(() => {});
    },
    []
  );

  const setParam = useCallback(
    (nodeId: string, paramName: string, value: number | string | boolean) => {
      fetch(`/api/node/${encodeURIComponent(nodeId)}/param`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name: paramName, value }),
      }).catch(() => {});
    },
    []
  );

  const startAudio = useCallback(() => {
    fetch("/audio/start", { method: "POST" }).catch(() => {});
  }, []);

  const stopAudio = useCallback(() => {
    fetch("/audio/stop", { method: "POST" }).catch(() => {});
  }, []);

  const exportWav = useCallback(async (): Promise<Blob | null> => {
    try {
      const response = await fetch("/api/audio/export", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ duration: 2.0 }),
      });
      const data = await response.json();
      if (data.data) {
        const byteString = atob(data.data);
        const ab = new ArrayBuffer(byteString.length);
        const ia = new Uint8Array(ab);
        for (let i = 0; i < byteString.length; i++) {
          ia[i] = byteString.charCodeAt(i);
        }
        return new Blob([ab], { type: "audio/wav" });
      }
      return null;
    } catch {
      return null;
    }
  }, []);

  const clearConsole = useCallback(() => {
    setConsoleMessages([]);
  }, []);

  return {
    ws: wsRef.current,
    isConnected,
    state,
    consoleMessages,
    nodeTypes,
    addNode,
    removeNode,
    connectPorts,
    disconnectPorts,
    setParam,
    startAudio,
    stopAudio,
    exportWav,
    clearConsole,
    onAudioChunk: audioChunkCallbackRef.current,
    setOnAudioChunk,
  };
}
