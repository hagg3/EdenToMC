import React, { useEffect, useState, useRef } from "react";
import { DropZone } from "./components/DropZone";
import { BlockMappingEditor } from "./components/BlockMappingEditor";
import { BlockMapping } from "./types";

type Phase = "idle" | "converting" | "done" | "error";

type WasmModule = {
  convert: (bytes: Uint8Array, mapping?: string | null) => Uint8Array<ArrayBufferLike>;
  generate_world: (params: string) => string;
};

export default function App() {
  const wasmRef = useRef<WasmModule | null>(null);
  const [wasmReady, setWasmReady] = useState(false);

  const [file, setFile] = useState<File | null>(null);
  const [fileBytes, setFileBytes] = useState<Uint8Array | null>(null);
  const [phase, setPhase] = useState<Phase>("idle");
  const [status, setStatus] = useState("");
  const [zipBlob, setZipBlob] = useState<Blob | null>(null);
  const [mapping, setMapping] = useState<BlockMapping | null>(null);
  const [showMapping, setShowMapping] = useState(false);

  useEffect(() => {
    import("./wasm/eden_converter.js").then(async (mod) => {
      await mod.default();
      const jsonStr: string = mod.default_mapping_json();
      setMapping(JSON.parse(jsonStr));
      wasmRef.current = { convert: mod.convert, generate_world: mod.generate_world };
      setWasmReady(true);
    }).catch(e => {
      setStatus("Failed to load converter: " + e);
      setPhase("error");
    });
  }, []);

  const handleFile = async (f: File) => {
    setFile(f); setFileBytes(null); setZipBlob(null); setPhase("idle"); setStatus("Reading file…");

    // Raw .eden files larger than ~2 GB cannot fit in a browser ArrayBuffer, and
    // even if they could, WASM would OOM duplicating them in linear memory.
    // ZIP-compressed worlds are fine — their compressed payload is tiny.
    const TWO_GB = 2 * 1024 * 1024 * 1024;
    if (f.size > TWO_GB) {
      try {
        const peek = new Uint8Array(await f.slice(0, 4).arrayBuffer());
        const isZip = peek[0] === 0x50 && peek[1] === 0x4B; // PK magic
        if (!isZip) {
          setPhase("error");
          setStatus(
            `File is ${(f.size / 1e9).toFixed(2)} GB — too large to load uncompressed in-browser. ` +
            `Please use the ZIP-compressed version of this world (the .eden file exported from the Eden app is usually already compressed).`
          );
          return;
        }
      } catch { /* fall through; arrayBuffer() will surface the real error */ }
    }

    try {
      const bytes = new Uint8Array(await f.arrayBuffer());
      setFileBytes(bytes);
      setStatus("");
    } catch (e) {
      setPhase("error");
      const msg = String(e);
      setStatus("Error reading file: " + msg +
        (msg.includes("NotReadable") || msg.includes("permission")
          ? " — ensure the file is not being synced by cloud storage or open in another app."
          : ""));
    }
  };

  const handleConvert = async () => {
    if (!fileBytes || !wasmRef.current || !mapping) return;
    setPhase("converting"); setStatus("Converting — this may take a moment for large worlds…");
    try {
      const bytes = fileBytes;
      await new Promise(r => setTimeout(r, 0));
      const result = wasmRef.current.convert(bytes, JSON.stringify(mapping));
      const blob = new Blob([new Uint8Array(result)], { type: "application/zip" });
      setZipBlob(blob); setPhase("done");
      setStatus(`Done! ${(blob.size / 1024).toFixed(1)} KB zip ready.`);
    } catch (e) {
      setPhase("error"); setStatus("Error: " + String(e));
    }
  };

  const handleDownload = () => {
    if (!zipBlob || !file) return;
    triggerDownload(zipBlob, file.name.replace(/\.eden$/i, "") + "-minecraft.zip");
  };

  return (
    <div style={{ minHeight: "100vh", padding: "32px 16px", color: "#e2e8f0" }}>
      <div style={{ maxWidth: 820, margin: "0 auto" }}>

        {/* Header */}
        <div style={{ marginBottom: 28, textAlign: "center" }}>
          <h1 style={{ fontSize: 32, fontWeight: 800, color: "#f1f5f9", letterSpacing: "-0.02em", margin: 0 }}>
            Eden to Minecraft Converter
          </h1>
          <p style={{ color: "#bbf7d0", marginTop: 8, fontSize: 15 }}>
            Convert <code style={{ color: "#86efac" }}>.eden</code> worlds to the Java Minecraft 1.12+ format.
          </p>
        </div>

        {/* WASM loading */}
        {!wasmReady && (
          <div style={bannerStyle("#14532d", "#86efac")}>⏳ Loading converter…</div>
        )}

        <DropZone onFile={handleFile} disabled={!wasmReady || phase === "converting"} />

        {file && (
          <div style={{ marginTop: 20, display: "flex", gap: 12, alignItems: "center", flexWrap: "wrap" }}>
            <div style={{ flex: 1, color: "#94a3b8", fontSize: 14 }}>
              📄 <strong style={{ color: "#e2e8f0" }}>{file.name}</strong>{" "}
              ({(file.size / 1024).toFixed(1)} KB)
            </div>
            <button onClick={handleConvert} disabled={!wasmReady || !fileBytes || phase === "converting"}
              style={primaryBtn(!wasmReady || !fileBytes || phase === "converting")}>
              {phase === "converting" ? "Converting…" : "Convert"}
            </button>
            {phase === "done" && (
              <button onClick={handleDownload} style={primaryBtn(false, "#15803d")}>
                ⬇ Download ZIP
              </button>
            )}
          </div>
        )}

        {status && (
          <div style={bannerStyle(
            phase === "error" ? "#3f1e1e" : phase === "done" ? "#14532d" : "#0d3d26",
            phase === "error" ? "#f87171" : phase === "done" ? "#4ade80" : "#94a3b8",
          )}>{status}</div>
        )}

        {phase === "done" && <InstallInstructions />}

        <div style={{ marginTop: 28 }}>
          <button onClick={() => setShowMapping(v => !v)} style={ghostBtn}>
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

function InstallInstructions() {
  return (
    <div style={{ marginTop: 16, background: "#14532d", borderRadius: 10, padding: 16 }}>
      <div style={{ fontWeight: 700, marginBottom: 8, color: "#e2e8f0" }}>How to install in Minecraft</div>
      <ol style={{ color: "#94a3b8", fontSize: 13, paddingLeft: 20, lineHeight: 1.9, margin: 0 }}>
        <li>Extract the ZIP — you'll get <code>level.dat</code> and a <code>region/</code> folder.</li>
        <li>Open your Minecraft saves folder: <code>~/.minecraft/saves/</code></li>
        <li>Create a new folder (e.g. <code>MyEdenWorld</code>) and paste everything inside it.</li>
        <li>Launch Minecraft 1.12.2 → Singleplayer — the world will appear in the list.</li>
      </ol>
    </div>
  );
}

function triggerDownload(blob: Blob, filename: string) {
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = filename;
  a.click();
}

function bannerStyle(bg: string, color: string): React.CSSProperties {
  return { marginTop: 16, padding: "10px 14px", borderRadius: 8, background: bg, color, fontSize: 13 };
}

function primaryBtn(disabled: boolean, bg = "#15803d"): React.CSSProperties {
  return {
    background: disabled ? "#166534" : bg,
    color: "#f1f5f9", border: "none", borderRadius: 8,
    padding: "10px 20px", fontWeight: 700, fontSize: 14,
    cursor: disabled ? "not-allowed" : "pointer", transition: "background 0.15s",
  };
}

const ghostBtn: React.CSSProperties = {
  background: "none", border: "1px solid #1a6b3a", color: "#86efac",
  borderRadius: 8, padding: "8px 16px", cursor: "pointer", fontSize: 13,
};
