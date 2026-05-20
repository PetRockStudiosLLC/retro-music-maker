/** Console panel showing engine log messages.
 *  Color-coded by type: info (white), success (green), error (red), warning (yellow). */
import { memo, useRef, useEffect } from "react";
import type { ConsoleMessage } from "../types/nodes";

interface ConsoleProps {
  messages: ConsoleMessage[];
  onClear: () => void;
  visible: boolean;
  onToggle: () => void;
}

function Console({ messages, onClear, visible, onToggle }: ConsoleProps) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (ref.current) {
      ref.current.scrollTop = ref.current.scrollHeight;
    }
  }, [messages]);

  if (!visible) {
    return (
      <div
        onClick={onToggle}
        style={{
          height: 32,
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "0 16px",
          background: "#1e2030",
          borderTop: "1px solid #3d4060",
          cursor: "pointer",
          userSelect: "none",
        }}
      >
        <span
          style={{
            color: "#565a7e",
            fontSize: 11,
            fontWeight: 700,
            textTransform: "uppercase",
            letterSpacing: 1.5,
            fontFamily: "'Segoe UI', system-ui, sans-serif",
          }}
        >
          Console
        </span>
        <span style={{ color: "#565a7e", fontSize: 10 }}>▼</span>
      </div>
    );
  }

  return (
    <div
      style={{
        height: 160,
        background: "#0d0e14",
        borderTop: "1px solid #3d4060",
        display: "flex",
        flexDirection: "column",
      }}
    >
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "6px 16px",
          borderBottom: "1px solid #3d4060",
        }}
      >
        <span
          style={{
            color: "#565a7e",
            fontSize: 11,
            fontWeight: 700,
            textTransform: "uppercase",
            letterSpacing: 1.5,
            fontFamily: "'Segoe UI', system-ui, sans-serif",
          }}
        >
          Console
        </span>
        <button
          onClick={onClear}
          style={{
            background: "transparent",
            border: "1px solid #3d4060",
            borderRadius: 3,
            color: "#565a7e",
            fontSize: 10,
            padding: "2px 10px",
            cursor: "pointer",
            fontFamily: "'Segoe UI', system-ui, sans-serif",
          }}
        >
          Clear
        </button>
      </div>

      {/* Messages */}
      <div
        ref={ref}
        style={{
          flex: 1,
          overflowY: "auto",
          padding: "8px 16px",
          fontFamily: "Consolas, 'Courier New', monospace",
          fontSize: 12,
          lineHeight: 1.6,
        }}
      >
        {messages.map((msg) => (
          <div
            key={msg.id}
            style={{
              color:
                msg.type === "success"
                  ? "#9ece6a"
                  : msg.type === "error"
                  ? "#f7768e"
                  : msg.type === "warning"
                  ? "#e0af68"
                  : "#565a7e",
              whiteSpace: "pre-wrap",
              wordBreak: "break-word",
            }}
          >
            {msg.text}
          </div>
        ))}
        {messages.length === 0 && (
          <div style={{ color: "#565a7e", fontStyle: "italic" }}>
            No messages
          </div>
        )}
      </div>
    </div>
  );
}

export default memo(Console);
