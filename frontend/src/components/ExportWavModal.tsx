import { useState } from "react";

interface ExportWavModalProps {
  onClose: () => void;
  onExport: (duration: number, sampleRate: number) => Promise<void>;
}

export default function ExportWavModal({ onClose, onExport }: ExportWavModalProps) {
  const [duration, setDuration] = useState(30);
  const [sampleRate, setSampleRate] = useState(44100);
  const [exporting, setExporting] = useState(false);
  const [error, setError] = useState("");

  const handleExport = async () => {
    if (duration <= 0 || duration > 300) {
      setError("Duration must be between 1 and 300 seconds.");
      return;
    }
    setExporting(true);
    setError("");
    try {
      await onExport(duration, sampleRate);
      onClose();
    } catch (err: any) {
      setError(err.message || "Export failed");
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className="export-modal-overlay" onClick={onClose}>
      <div className="export-modal" onClick={(e) => e.stopPropagation()}>
        <div className="export-modal-header">
          <h2>Export WAV</h2>
          <button className="close-btn" onClick={onClose}>✕</button>
        </div>

        {error && <div className="error-message">{error}</div>}

        <div className="form-group">
          <label>Duration (seconds)</label>
          <input
            type="number"
            min={1}
            max={300}
            step={1}
            value={duration}
            onChange={(e) => setDuration(Number(e.target.value))}
            disabled={exporting}
          />
          <span className="form-hint">1 - 300 seconds</span>
        </div>

        <div className="form-group">
          <label>Sample Rate (Hz)</label>
          <select
            value={sampleRate}
            onChange={(e) => setSampleRate(Number(e.target.value))}
            disabled={exporting}
          >
            <option value={22050}>22050 Hz (CD quality)</option>
            <option value={44100}>44100 Hz (CD quality)</option>
            <option value={48000}>48000 Hz (Broadcast)</option>
            <option value={88200}>88200 Hz (Hi-Res)</option>
            <option value={96000}>96000 Hz (Hi-Res)</option>
          </select>
        </div>

        <div className="export-info">
          <span>Estimated file size: ~{((duration * sampleRate * 2) / (1024 * 1024)).toFixed(1)} MB</span>
        </div>

        <div className="export-modal-actions">
          <button
            className="cancel-btn"
            onClick={onClose}
            disabled={exporting}
          >
            Cancel
          </button>
          <button
            className="export-btn"
            onClick={handleExport}
            disabled={exporting}
          >
            {exporting ? "Exporting..." : "Choose File & Export"}
          </button>
        </div>

        <style>{`
          .export-modal-overlay {
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

          .export-modal {
            background: #1a1a2e;
            border: 1px solid #333;
            border-radius: 8px;
            padding: 24px;
            width: 380px;
            color: #e0e0e0;
          }

          .export-modal-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 20px;
          }

          .export-modal-header h2 {
            margin: 0;
            color: #00d4ff;
            font-size: 18px;
          }

          .close-btn {
            background: transparent;
            color: #999;
            border: none;
            font-size: 18px;
            cursor: pointer;
            padding: 4px 8px;
          }

          .close-btn:hover {
            color: #fff;
          }

          .error-message {
            background: rgba(255, 68, 68, 0.15);
            color: #ff6b6b;
            padding: 10px;
            border-radius: 4px;
            margin-bottom: 16px;
            font-size: 13px;
          }

          .form-group {
            margin-bottom: 16px;
          }

          .form-group label {
            display: block;
            margin-bottom: 6px;
            font-size: 13px;
            color: #aaa;
          }

          .form-group input[type="number"],
          .form-group select {
            width: 100%;
            padding: 8px 10px;
            background: #0d0d1a;
            border: 1px solid #333;
            border-radius: 4px;
            color: #e0e0e0;
            font-size: 14px;
            outline: none;
          }

          .form-group input:focus,
          .form-group select:focus {
            border-color: #00d4ff;
          }

          .form-hint {
            display: block;
            margin-top: 4px;
            font-size: 11px;
            color: #666;
          }

          .export-info {
            margin: 16px 0;
            padding: 10px;
            background: rgba(0, 212, 255, 0.08);
            border-radius: 4px;
            font-size: 12px;
            color: #787cb8;
            text-align: center;
          }

          .export-modal-actions {
            display: flex;
            gap: 10px;
            justify-content: flex-end;
            margin-top: 20px;
          }

          .cancel-btn,
          .export-btn {
            padding: 8px 16px;
            border-radius: 4px;
            font-size: 13px;
            cursor: pointer;
            border: none;
          }

          .cancel-btn {
            background: #333;
            color: #ccc;
          }

          .cancel-btn:hover {
            background: #444;
          }

          .export-btn {
            background: #00d4ff;
            color: #000;
            font-weight: 600;
          }

          .export-btn:hover:not(:disabled) {
            background: #33ddff;
          }

          .export-btn:disabled {
            opacity: 0.5;
            cursor: not-allowed;
          }

          .cancel-btn:disabled {
            opacity: 0.5;
            cursor: not-allowed;
          }
        `}</style>
      </div>
    </div>
  );
}
