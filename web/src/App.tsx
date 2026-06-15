import React, { useEffect, useState, useRef } from "react";
import { DropZone } from "./components/DropZone";
import { BlockMappingEditor } from "./components/BlockMappingEditor";
import { BlockMapping } from "./types";

type Phase = "idle" | "converting" | "done" | "error";

export default function App() {
  const wasmRef = useRef<{ convert: (bytes: Uint8Array, mapping: string | undefined) => Uint8Array } | null>(null);
  const [wasmReady, setWasmReady] = useState(false);
  const [file, setFile] = useState<File | null>(null);
  const [phase, setPhase] = useState<Phase>("idle");
  const [status, setStatus] = useState("");
  const [zipBlob, setZipBlob] = useState<Blob | null>(null);
  const [mapping, setMapping] = useState<BlockMapping | null>(null);
  const [showMapping, setShowMapping] = useState(false);

  useEffect(() => {
    import("./wasm/eden_converter.js").then(async (mod) => {
      await mod.default(); // init WASM
      // Load default mapping from WASM
      const jsonStr: string = mod.default_mapping_json();
      setMapping(JSON.parse(jsonStr));
      wasmRef.current = { convert: mod.convert };
      setWasmReady(true);
    }).catch(e => {
      setStatus("Failed to load converter: " + e);
      setPhase("error");
    });
  }, []);

  const handleFile = (f: File) => {
    setFile(f);
    setZipBlob(null);
    setPhase("idle");
    setStatus("");
  };

  const handleConvert = async () => {
    if (!file || !wasmRef.current || !mapping) return;
    setPhase("converting");
    setStatus("Reading file…");
    try {
      const arrayBuf = await file.arrayBuffer();
      const bytes = new Uint8Array(arrayBuf);
      setStatus("Converting — this may take a moment for large worlds…");
      // Run in a microtask so the UI can update
      await new Promise(r => setTimeout(r, 0));
      const mappingJson = JSON.stringify(mapping);
      const result: Uint8Array = wasmRef.current.convert(bytes, mappingJson);
      const blob = new Blob([result.buffer as ArrayBuffer], { type: "application/zip" });
      setZipBlob(blob);
      setPhase("done");
      setStatus(`Done! ${(blob.size / 1024).toFixed(1)} KB zip ready.`);
    } catch (e: unknown) {
      setPhase("error");
      setStatus("Error: " + String(e));
    }
  };

  const handleDownload = () => {
    if (!zipBlob || !file) return;
    const a = document.createElement("a");
    a.href = URL.createObjectURL(zipBlob);
    const worldName = file.name.replace(/\.eden$/i, "");
    a.download = `${worldName}-minecraft.zip`;
    a.click();
  };

  return (
    <div style={{ minHeight: "100vh", padding: "32px 16px" }}>
      <div style={{ maxWidth: 780, margin: "0 auto" }}>
        {/* Header */}
        <div style={{ marginBottom: 32, textAlign: "center" }}>
          <h1 style={{ fontSize: 32, fontWeight: 800, color: "#f1f5f9", letterSpacing: "-0.02em" }}>
            Eden → Minecraft
          </h1>
          <p style={{ color: "#64748b", marginTop: 8, fontSize: 15 }}>
            Converts <code style={{ color: "#94a3b8" }}>.eden</code> worlds to a Minecraft 1.12 world folder — 100% in your browser, nothing uploaded.
          </p>
        </div>

        {/* Status banner */}
        {!wasmReady && (
          <div style={bannerStyle("#1e3a5f", "#60a5fa")}>⏳ Loading converter…</div>
        )}

        {/* Drop zone */}
        <DropZone onFile={handleFile} disabled={!wasmReady || phase === "converting"} />

        {/* File info + controls */}
        {file && (
          <div style={{ marginTop: 20, display: "flex", gap: 12, alignItems: "center", flexWrap: "wrap" }}>
            <div style={{ flex: 1, color: "#94a3b8", fontSize: 14 }}>
              📄 <strong style={{ color: "#e2e8f0" }}>{file.name}</strong>{" "}
              ({(file.size / 1024).toFixed(1)} KB)
            </div>
            <button
              onClick={handleConvert}
              disabled={!wasmReady || phase === "converting"}
              style={primaryBtn(phase === "converting")}
            >
              {phase === "converting" ? "Converting…" : "Convert"}
            </button>
            {phase === "done" && (
              <button onClick={handleDownload} style={primaryBtn(false, "#059669")}>
                ⬇ Download ZIP
              </button>
            )}
          </div>
        )}

        {/* Status message */}
        {status && (
          <div style={bannerStyle(
            phase === "error" ? "#3f1e1e" : phase === "done" ? "#14532d" : "#1e293b",
            phase === "error" ? "#f87171" : phase === "done" ? "#4ade80" : "#94a3b8",
          )}>
            {status}
          </div>
        )}

        {/* Install instructions */}
        {phase === "done" && (
          <div style={{ marginTop: 16, background: "#1e293b", borderRadius: 10, padding: 16 }}>
            <div style={{ fontWeight: 700, marginBottom: 8, color: "#e2e8f0" }}>How to install</div>
            <ol style={{ color: "#94a3b8", fontSize: 13, paddingLeft: 20, lineHeight: 1.9 }}>
              <li>Extract the ZIP — you'll get <code>level.dat</code> and a <code>region/</code> folder.</li>
              <li>Open your Minecraft saves folder: <code>~/.minecraft/saves/</code></li>
              <li>Create a new folder (e.g. <code>MyEdenWorld</code>) and paste everything inside it.</li>
              <li>Launch Minecraft 1.12.2 → Singleplayer — the world will appear in the list.</li>
            </ol>
          </div>
        )}

        {/* Block mapping toggle */}
        <div style={{ marginTop: 28 }}>
          <button
            onClick={() => setShowMapping(v => !v)}
            style={{ background: "none", border: "1px solid #334155", color: "#94a3b8",
              borderRadius: 8, padding: "8px 16px", cursor: "pointer", fontSize: 13 }}
          >
            {showMapping ? "▲ Hide" : "▼ Show"} block mapping editor
          </button>
        </div>

        {showMapping && mapping && (
          <div style={{ marginTop: 16 }}>
            <BlockMappingEditor mapping={mapping} onChange={setMapping} />
          </div>
        )}
      </div>
    </div>
  );
}

function bannerStyle(bg: string, color: string): React.CSSProperties {
  return {
    marginTop: 16, padding: "10px 14px", borderRadius: 8,
    background: bg, color, fontSize: 13,
  };
}

function primaryBtn(disabled: boolean, bg = "#2563eb"): React.CSSProperties {
  return {
    background: disabled ? "#334155" : bg,
    color: "#f1f5f9", border: "none", borderRadius: 8,
    padding: "10px 20px", fontWeight: 700, fontSize: 14,
    cursor: disabled ? "not-allowed" : "pointer",
    transition: "background 0.15s",
  };
}
