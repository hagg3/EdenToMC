import React, { useRef, useState } from "react";

interface Props {
  onFile: (file: File) => void;
  disabled?: boolean;
}

export function DropZone({ onFile, disabled }: Props) {
  const [dragging, setDragging] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragging(false);
    if (disabled) return;
    const file = e.dataTransfer.files[0];
    if (file) onFile(file);
  };

  return (
    <div
      onClick={() => !disabled && inputRef.current?.click()}
      onDragOver={e => { e.preventDefault(); if (!disabled) setDragging(true); }}
      onDragLeave={() => setDragging(false)}
      onDrop={handleDrop}
      style={{
        border: `2px dashed ${dragging ? "#4ade80" : "#1a6b3a"}`,
        borderRadius: 12,
        padding: "48px 32px",
        textAlign: "center",
        cursor: disabled ? "not-allowed" : "pointer",
        background: dragging ? "#166534" : "#14532d",
        transition: "all 0.15s",
        opacity: disabled ? 0.5 : 1,
      }}
    >
      <div style={{ fontSize: 48, marginBottom: 12 }}>📂</div>
      <div style={{ fontSize: 18, fontWeight: 600, color: "#f1f5f9" }}>
        Drop your <code>.eden</code> file here
      </div>
      <div style={{ fontSize: 14, color: "#94a3b8", marginTop: 8 }}>
        or click to browse
      </div>
      <input
        ref={inputRef}
        type="file"
        accept=".eden"
        style={{ display: "none" }}
        onChange={e => { const f = e.target.files?.[0]; if (f) onFile(f); }}
      />
    </div>
  );
}
