import React, { useEffect, useState, useRef } from "react";
import { DropZone } from "./components/DropZone";
import { BlockMappingEditor } from "./components/BlockMappingEditor";
import { BlockMapping } from "./types";

type Phase = "idle" | "converting" | "done" | "error";
type Tab = "convert" | "generate";

type WasmModule = {
  convert: (bytes: Uint8Array, mapping?: string | null) => Uint8Array<ArrayBufferLike>;
  generate_world: (params: string) => string;
};

interface TerrainStats {
  spawn_x: number; spawn_y: number; spawn_z: number;
  trees_placed: number; flowers_placed: number; caves_carved: number;
  min_height: number; max_height: number;
  cols_x: number; cols_z: number;
}

const WATER_LABELS: Record<number, string> = {
  1: "Deep Ocean (sea level 40)",
  2: "Ocean (sea level 35)",
  3: "Mixed — default (sea level 32)",
  4: "Mostly Land (sea level 27)",
  5: "Full Land (no water)",
};

export default function App() {
  const wasmRef = useRef<WasmModule | null>(null);
  const [wasmReady, setWasmReady] = useState(false);
  const [tab, setTab] = useState<Tab>("convert");

  // ── Convert tab state ──────────────────────────────────────────────
  const [file, setFile] = useState<File | null>(null);
  const [phase, setPhase] = useState<Phase>("idle");
  const [status, setStatus] = useState("");
  const [zipBlob, setZipBlob] = useState<Blob | null>(null);
  const [mapping, setMapping] = useState<BlockMapping | null>(null);
  const [showMapping, setShowMapping] = useState(false);

  // ── Generate tab state ─────────────────────────────────────────────
  const [genWidth, setGenWidth] = useState(128);
  const [genDepth, setGenDepth] = useState(128);
  const [genSeed, setGenSeed] = useState(() => Math.floor(Math.random() * 1000000));
  const [genBaseHeight, setGenBaseHeight] = useState(30);
  const [genWaterAmnt, setGenWaterAmnt] = useState(3);
  const [genPhase, setGenPhase] = useState<Phase>("idle");
  const [genStatus, setGenStatus] = useState("");
  const [edenBlob, setEdenBlob] = useState<Blob | null>(null);
  const [genZipBlob, setGenZipBlob] = useState<Blob | null>(null);
  const [terrainStats, setTerrainStats] = useState<TerrainStats | null>(null);

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

  // ── Convert handlers ─────────────────────────────────────────────────────
  const handleFile = (f: File) => {
    setFile(f); setZipBlob(null); setPhase("idle"); setStatus("");
  };

  const handleConvert = async () => {
    if (!file || !wasmRef.current || !mapping) return;
    setPhase("converting"); setStatus("Reading file…");
    try {
      const bytes = new Uint8Array(await file.arrayBuffer());
      setStatus("Converting — this may take a moment for large worlds…");
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

  // ── Generate handlers ──────────────────────────────────────────────────
  const handleGenerate = async () => {
    if (!wasmRef.current) return;
    setGenPhase("converting"); setGenStatus("Generating terrain…");
    setEdenBlob(null); setGenZipBlob(null); setTerrainStats(null);
    try {
      const params = JSON.stringify({
        width: genWidth, depth: genDepth, seed: genSeed,
        base_height: genBaseHeight, water_amnt: genWaterAmnt,
      });
      await new Promise(r => setTimeout(r, 0));
      const resultJson: string = wasmRef.current.generate_world(params);
      const result = JSON.parse(resultJson) as { eden: string; stats: TerrainStats };
      const edenBytes = base64ToBytes(result.eden);
      const blob = new Blob([edenBytes], { type: "application/octet-stream" });
      setEdenBlob(blob);
      setTerrainStats(result.stats);
      setGenPhase("done");
      const sizeMB = (blob.size / 1024 / 1024).toFixed(2);
      setGenStatus(`Generated ${genWidth}×${genDepth} world — ${sizeMB} MB Eden file ready.`);
    } catch (e) {
      setGenPhase("error"); setGenStatus("Error: " + String(e));
    }
  };

  const handleDownloadEden = () => {
    if (!edenBlob) return;
    triggerDownload(edenBlob, `terrain-${genSeed}.eden`);
  };

  const handleConvertGenerated = async () => {
    if (!edenBlob || !wasmRef.current || !mapping) return;
    setGenStatus("Converting to Minecraft…");
    try {
      const bytes = new Uint8Array(await edenBlob.arrayBuffer());
      await new Promise(r => setTimeout(r, 0));
      const result = wasmRef.current.convert(bytes, JSON.stringify(mapping));
      const zip = new Blob([new Uint8Array(result)], { type: "application/zip" });
      setGenZipBlob(zip);
      setGenStatus(`Minecraft world ready — ${(zip.size / 1024).toFixed(1)} KB zip.`);
    } catch (e) {
      setGenPhase("error"); setGenStatus("Error converting: " + String(e));
    }
  };

  const handleDownloadMC = () => {
    if (!genZipBlob) return;
    triggerDownload(genZipBlob, `terrain-${genSeed}-minecraft.zip`);
  };

  // ── Render ─────────────────────────────────────────────────────────────────
  return (
    <div style={{ minHeight: "100vh", padding: "32px 16px", color: "#e2e8f0" }}>
      <div style={{ maxWidth: 820, margin: "0 auto" }}>

        {/* Header */}
        <div style={{ marginBottom: 28, textAlign: "center" }}>
          <h1 style={{ fontSize: 32, fontWeight: 800, color: "#f1f5f9", letterSpacing: "-0.02em", margin: 0 }}>
            Eden Tools
          </h1>
          <p style={{ color: "#64748b", marginTop: 8, fontSize: 15 }}>
            Convert <code style={{ color: "#94a3b8" }}>.eden</code> worlds to Minecraft 1.12
            {" "}or generate new procedural worlds — 100% in your browser.
          </p>
        </div>

        {/* WASM loading */}
        {!wasmReady && (
          <div style={bannerStyle("#1e3a5f", "#60a5fa")}>⏳ Loading converter…</div>
        )}

        {/* Tab switcher */}
        <div style={{ display: "flex", gap: 8, marginBottom: 24 }}>
          {(["convert", "generate"] as Tab[]).map(t => (
            <button key={t} onClick={() => setTab(t)} style={tabBtn(tab === t)}>
              {t === "convert" ? "Convert .eden → Minecraft" : "Generate World"}
            </button>
          ))}
        </div>

        {/* ── Convert tab ──────────────────────────────────────────────────────── */}
        {tab === "convert" && (
          <>
            <DropZone onFile={handleFile} disabled={!wasmReady || phase === "converting"} />

            {file && (
              <div style={{ marginTop: 20, display: "flex", gap: 12, alignItems: "center", flexWrap: "wrap" }}>
                <div style={{ flex: 1, color: "#94a3b8", fontSize: 14 }}>
                  📄 <strong style={{ color: "#e2e8f0" }}>{file.name}</strong>{" "}
                  ({(file.size / 1024).toFixed(1)} KB)
                </div>
                <button onClick={handleConvert} disabled={!wasmReady || phase === "converting"}
                  style={primaryBtn(phase === "converting")}>
                  {phase === "converting" ? "Converting…" : "Convert"}
                </button>
                {phase === "done" && (
                  <button onClick={handleDownload} style={primaryBtn(false, "#059669")}>
                    ⬇ Download ZIP
                  </button>
                )}
              </div>
            )}

            {status && (
              <div style={bannerStyle(
                phase === "error" ? "#3f1e1e" : phase === "done" ? "#14532d" : "#1e293b",
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
          </>
        )}

        {/* ── Generate tab ────────────────────────────────────────────────────── */}
        {tab === "generate" && (
          <>
            <div style={panelStyle}>
              <div style={{ fontWeight: 700, fontSize: 16, marginBottom: 20, color: "#f1f5f9" }}>
                Terrain Parameters
              </div>

              <div style={formGrid}>
                {/* World size */}
                <label style={labelStyle}>World Width</label>
                <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                  {[64, 128, 256, 512].map(v => (
                    <button key={v} onClick={() => setGenWidth(v)} style={chipBtn(genWidth === v)}>
                      {v} blocks
                    </button>
                  ))}
                </div>

                <label style={labelStyle}>World Depth</label>
                <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                  {[64, 128, 256, 512].map(v => (
                    <button key={v} onClick={() => setGenDepth(v)} style={chipBtn(genDepth === v)}>
                      {v} blocks
                    </button>
                  ))}
                </div>

                {/* Seed */}
                <label style={labelStyle}>Seed</label>
                <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                  <input
                    type="number"
                    value={genSeed}
                    onChange={e => setGenSeed(parseInt(e.target.value) || 0)}
                    style={inputStyle}
                  />
                  <button
                    onClick={() => setGenSeed(Math.floor(Math.random() * 1000000))}
                    style={ghostBtn}
                    title="Random seed"
                  >
                    🎲
                  </button>
                </div>

                {/* Base height */}
                <label style={labelStyle}>
                  Base Height&nbsp;
                  <span style={{ color: "#64748b", fontWeight: 400 }}>({genBaseHeight})</span>
                </label>
                <div>
                  <input
                    type="range" min={10} max={52} step={1}
                    value={genBaseHeight}
                    onChange={e => setGenBaseHeight(parseInt(e.target.value))}
                    style={{ width: "100%", accentColor: "#3b82f6" }}
                  />
                  <div style={{ display: "flex", justifyContent: "space-between", fontSize: 11, color: "#64748b" }}>
                    <span>10 (low, flat)</span><span>52 (high, mountainous)</span>
                  </div>
                </div>

                {/* Water amount */}
                <label style={labelStyle}>Water Coverage</label>
                <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                  {[1, 2, 3, 4, 5].map(v => (
                    <label key={v} style={{ display: "flex", alignItems: "center", gap: 8,
                      cursor: "pointer", fontSize: 13, color: v === genWaterAmnt ? "#93c5fd" : "#94a3b8" }}>
                      <input type="radio" name="waterAmnt" value={v} checked={genWaterAmnt === v}
                        onChange={() => setGenWaterAmnt(v)}
                        style={{ accentColor: "#3b82f6" }} />
                      {WATER_LABELS[v]}
                    </label>
                  ))}
                </div>
              </div>

              {/* Summary */}
              <div style={{ marginTop: 16, padding: "10px 14px", background: "#0f172a",
                borderRadius: 8, fontSize: 13, color: "#64748b" }}>
                World: <strong style={{ color: "#94a3b8" }}>{genWidth}×{genDepth}</strong> blocks
                ({(genWidth / 16).toFixed(0)}×{(genDepth / 16).toFixed(0)} chunks) ·{" "}
                ~{((genWidth * genDepth * 64 * 2) / 1024 / 1024).toFixed(1)} MB Eden file
              </div>

              {/* Generate button */}
              <div style={{ marginTop: 20 }}>
                <button
                  onClick={handleGenerate}
                  disabled={!wasmReady || genPhase === "converting"}
                  style={primaryBtn(genPhase === "converting", "#7c3aed")}
                >
                  {genPhase === "converting" ? "Generating…" : "Generate Terrain"}
                </button>
              </div>
            </div>

            {/* Status */}
            {genStatus && (
              <div style={bannerStyle(
                genPhase === "error" ? "#3f1e1e" : genPhase === "done" ? "#14532d" : "#1e293b",
                genPhase === "error" ? "#f87171" : genPhase === "done" ? "#4ade80" : "#94a3b8",
              )}>{genStatus}</div>
            )}

            {/* Stats + download panel */}
            {edenBlob && terrainStats && (
              <div style={{ ...panelStyle, marginTop: 16 }}>
                <div style={{ fontWeight: 700, fontSize: 15, marginBottom: 14, color: "#f1f5f9" }}>
                  Generation Results
                </div>
                <div style={statsGrid}>
                  <StatItem label="Height range" value={`${terrainStats.min_height} – ${terrainStats.max_height}`} />
                  <StatItem label="Trees placed" value={terrainStats.trees_placed.toLocaleString()} />
                  <StatItem label="Flowers placed" value={terrainStats.flowers_placed.toLocaleString()} />
                  <StatItem label="Cave blocks carved" value={terrainStats.caves_carved.toLocaleString()} />
                  <StatItem label="Spawn point" value={`(${terrainStats.spawn_x}, ${terrainStats.spawn_y}, ${terrainStats.spawn_z})`} />
                  <StatItem label="Columns" value={`${terrainStats.cols_x}×${terrainStats.cols_z}`} />
                </div>

                <div style={{ marginTop: 16, display: "flex", gap: 10, flexWrap: "wrap" }}>
                  <button onClick={handleDownloadEden} style={primaryBtn(false, "#0284c7")}>
                    ⬇ Download .eden
                  </button>
                  <button
                    onClick={handleConvertGenerated}
                    disabled={!wasmReady}
                    style={primaryBtn(!wasmReady, "#059669")}
                  >
                    Convert → Minecraft ZIP
                  </button>
                  {genZipBlob && (
                    <button onClick={handleDownloadMC} style={primaryBtn(false, "#16a34a")}>
                      ⬇ Download MC ZIP
                    </button>
                  )}
                </div>

                {genZipBlob && <InstallInstructions />}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}

// ── Small components ───────────────────────────────────────────────────────────────────

function StatItem({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ padding: "8px 12px", background: "#0f172a", borderRadius: 6 }}>
      <div style={{ fontSize: 11, color: "#64748b", marginBottom: 2 }}>{label}</div>
      <div style={{ fontSize: 14, fontWeight: 600, color: "#e2e8f0" }}>{value}</div>
    </div>
  );
}

function InstallInstructions() {
  return (
    <div style={{ marginTop: 16, background: "#1e293b", borderRadius: 10, padding: 16 }}>
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

// ── Utilities ───────────────────────────────────────────────────────────────────────────

function base64ToBytes(b64: string): Uint8Array<ArrayBuffer> {
  const bin = atob(b64);
  const arr = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) arr[i] = bin.charCodeAt(i);
  return arr;
}

function triggerDownload(blob: Blob, filename: string) {
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = filename;
  a.click();
}

// ── Styles ──────────────────────────────────────────────────────────────────────────────

function bannerStyle(bg: string, color: string): React.CSSProperties {
  return { marginTop: 16, padding: "10px 14px", borderRadius: 8, background: bg, color, fontSize: 13 };
}

function primaryBtn(disabled: boolean, bg = "#2563eb"): React.CSSProperties {
  return {
    background: disabled ? "#334155" : bg,
    color: "#f1f5f9", border: "none", borderRadius: 8,
    padding: "10px 20px", fontWeight: 700, fontSize: 14,
    cursor: disabled ? "not-allowed" : "pointer", transition: "background 0.15s",
  };
}

function tabBtn(active: boolean): React.CSSProperties {
  return {
    background: active ? "#1e40af" : "#1e293b",
    color: active ? "#bfdbfe" : "#94a3b8",
    border: `1px solid ${active ? "#3b82f6" : "#334155"}`,
    borderRadius: 8, padding: "9px 18px", fontWeight: 600, fontSize: 13,
    cursor: "pointer", transition: "all 0.15s",
  };
}

function chipBtn(active: boolean): React.CSSProperties {
  return {
    background: active ? "#1e3a5f" : "#1e293b",
    color: active ? "#93c5fd" : "#64748b",
    border: `1px solid ${active ? "#3b82f6" : "#334155"}`,
    borderRadius: 6, padding: "5px 12px", fontSize: 13,
    cursor: "pointer", transition: "all 0.15s",
  };
}

const ghostBtn: React.CSSProperties = {
  background: "none", border: "1px solid #334155", color: "#94a3b8",
  borderRadius: 8, padding: "8px 16px", cursor: "pointer", fontSize: 13,
};

const panelStyle: React.CSSProperties = {
  background: "#1e293b", borderRadius: 12, padding: 24, border: "1px solid #334155",
};

const formGrid: React.CSSProperties = {
  display: "grid", gridTemplateColumns: "160px 1fr",
  gap: "14px 20px", alignItems: "start",
};

const labelStyle: React.CSSProperties = {
  fontSize: 13, fontWeight: 600, color: "#94a3b8",
  paddingTop: 4, alignSelf: "start",
};

const inputStyle: React.CSSProperties = {
  background: "#0f172a", border: "1px solid #334155", borderRadius: 6,
  color: "#e2e8f0", padding: "6px 10px", fontSize: 14, width: 140,
};

const statsGrid: React.CSSProperties = {
  display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))", gap: 8,
};
