/** Simple tooltip component that appears on hover. */
import { useState, useRef, useEffect } from "react";

interface TooltipProps {
  content: string;
  children: React.ReactNode;
  position?: "top" | "bottom" | "left" | "right";
}

function Tooltip({ content, children, position = "top" }: TooltipProps) {
  const [visible, setVisible] = useState(false);
  const [pos, setPos] = useState({ x: 0, y: 0 });
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!visible || !ref.current) return;
    const rect = ref.current.getBoundingClientRect();
    setPos({
      x: rect.left + rect.width / 2,
      y: rect.top + rect.height / 2,
    });
  }, [visible]);

  const tooltipStyle: React.CSSProperties = {
    position: "fixed",
    pointerEvents: "none",
    zIndex: 9999,
    background: "#181926",
    border: "1px solid #7aa2f7",
    borderRadius: 4,
    padding: "5px 10px",
    fontSize: 11,
    color: "#c0caf5",
    fontFamily: "'Segoe UI', system-ui, sans-serif",
    boxShadow: "0 4px 12px rgba(0,0,0,0.5)",
    whiteSpace: "normal",
    maxWidth: 220,
    lineHeight: 1.4,
  };

  const positionStyle: React.CSSProperties = {};
  if (position === "top") {
    positionStyle.top = pos.y - 40;
    positionStyle.left = pos.x;
    positionStyle.transform = "translate(-50%, -100%)";
  } else if (position === "bottom") {
    positionStyle.top = pos.y + 20;
    positionStyle.left = pos.x;
    positionStyle.transform = "translate(-50%, 0)";
  } else if (position === "left") {
    positionStyle.top = pos.y;
    positionStyle.left = pos.x - 10;
    positionStyle.transform = "translate(-100%, -50%)";
  } else {
    positionStyle.top = pos.y;
    positionStyle.left = pos.x + 10;
    positionStyle.transform = "translate(0, -50%)";
  }

  return (
    <div
      ref={ref}
      onMouseEnter={() => setVisible(true)}
      onMouseLeave={() => setVisible(false)}
    >
      {children}
      {visible && content && (
        <div style={{ ...tooltipStyle, ...positionStyle }}>
          {content}
        </div>
      )}
    </div>
  );
}

export default Tooltip;
