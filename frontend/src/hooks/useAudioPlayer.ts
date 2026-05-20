/** Web Audio API hook for playing streamed audio chunks from the backend. */
import { useRef, useEffect, useCallback, useState } from "react";

interface UseAudioPlayerReturn {
  isPlaying: boolean;
  startStreaming: (ws: WebSocket | null, sampleRate?: number, blockSize?: number) => void;
  stopStreaming: (ws: WebSocket | null) => void;
  onAudioChunk: (chunk: Float32Array) => void;
}

export function useAudioPlayer(): UseAudioPlayerReturn {
  const audioContextRef = useRef<AudioContext | null>(null);
  const workletNodeRef = useRef<AudioWorkletNode | null>(null);
  const isPlayingRef = useRef(false);
  const [isPlaying, setIsPlaying] = useState(false);

  const initAudioContext = useCallback(async () => {
    if (!audioContextRef.current) {
      const ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
      await ctx.audioWorklet.addModule("/audio-stream-processor.js");
      audioContextRef.current = ctx;
    }
    if (audioContextRef.current.state === "suspended") {
      await audioContextRef.current.resume();
    }
    return audioContextRef.current;
  }, []);

  const startStreaming = useCallback(
    async (ws: WebSocket | null, sampleRate: number = 44100, blockSize: number = 512) => {
      if (!ws) return;

      const ctx = await initAudioContext();

      const workletNode = new AudioWorkletNode(ctx, "audio-stream-processor", {
        channelCount: 1,
        outputChannelCount: [1],
      });
      workletNodeRef.current = workletNode;
      workletNode.connect(ctx.destination);

      isPlayingRef.current = true;
      setIsPlaying(true);

      ws.send(JSON.stringify({
        type: "start_streaming",
        sample_rate: sampleRate,
        block_size: blockSize,
      }));
    },
    [initAudioContext]
  );

  const stopStreaming = useCallback((ws: WebSocket | null) => {
    isPlayingRef.current = false;
    setIsPlaying(false);

    if (workletNodeRef.current) {
      workletNodeRef.current.disconnect();
      workletNodeRef.current = null;
    }

    if (ws) {
      ws.send(JSON.stringify({ type: "stop_audio" }));
    }
  }, []);

  const onAudioChunk = useCallback((chunk: Float32Array) => {
    if (isPlayingRef.current && workletNodeRef.current) {
      workletNodeRef.current.port.postMessage({
        type: "chunk",
        data: chunk,
      });
    }
  }, []);

  useEffect(() => {
    return () => {
      if (workletNodeRef.current) {
        workletNodeRef.current.disconnect();
      }
      if (audioContextRef.current) {
        audioContextRef.current.close();
      }
    };
  }, []);

  return {
    isPlaying,
    startStreaming,
    stopStreaming,
    onAudioChunk,
  };
}
