import React from "react";
import ReactDOM from "react-dom/client";
import NoteMapperEditor from "./components/NoteMapperEditor";

const nodeId = (window as any).__NOTE_MAPPER_NODE_ID__ || "";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <NoteMapperEditor
      nodeId={nodeId}
      onClose={async () => {
        const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
        getCurrentWebviewWindow().close();
      }}
    />
  </React.StrictMode>
);
