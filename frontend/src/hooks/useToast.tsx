import { createContext, useContext, useCallback, useState, useMemo } from "react";

export type ToastType = "success" | "error" | "info" | "warning";

interface Toast {
  id: string;
  message: string;
  type: ToastType;
}

interface ToastContextType {
  toast: (message: string, type?: ToastType) => void;
}

const ToastContext = createContext<ToastContextType>({ toast: () => {} });

export function useToast() {
  return useContext(ToastContext);
}

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const toast = useCallback((message: string, type: ToastType = "info") => {
    const id = Date.now().toString() + Math.random().toString(36).slice(2);
    setToasts((prev) => [...prev, { id, message, type }]);
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 4000);
  }, []);

  const remove = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const value = useMemo(() => ({ toast }), [toast]);

  return (
    <ToastContext.Provider value={value}>
      {children}
      <div
        style={{
          position: "fixed",
          bottom: 60,
          right: 16,
          display: "flex",
          flexDirection: "column",
          gap: 6,
          zIndex: 10000,
          pointerEvents: "none",
        }}
      >
        {toasts.map((t) => (
          <div
            key={t.id}
            onClick={() => remove(t.id)}
            style={{
              background: "#1e2030",
              borderLeft: `3px solid ${
                t.type === "success" ? "#9ece6a"
                  : t.type === "error" ? "#f7768e"
                    : t.type === "warning" ? "#e0af68"
                      : "#7aa2f7"
              }`,
              borderRadius: 4,
              padding: "8px 14px",
              fontSize: 12,
              color: "#c0caf5",
              fontFamily: "'Segoe UI', system-ui, sans-serif",
              boxShadow: "0 4px 12px rgba(0,0,0,0.4)",
              maxWidth: 320,
              cursor: "pointer",
              pointerEvents: "auto",
              opacity: 0.95,
              animation: "toastIn 0.2s ease-out",
            }}
          >
            {t.message}
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}
