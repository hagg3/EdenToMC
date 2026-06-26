import React, { useState } from "react";
import {
  BlockMapping, BlockEntry, McBlock, PaintedFamily,
  EDEN_BLOCK_NAMES, MC_BLOCK_OPTIONS, PAINTED_FAMILIES, EDEN_PAINT_COLORS,
} from "../types";

interface Props {
  mapping: BlockMapping;
  onChange: (m: BlockMapping) => void;
}

const GROUP_LABELS: [string, number[]][] = [
  ["Core Blocks", [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23]],
  ["Ramps", [24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39]],
  ["Side Ramps / Wedges", [40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55]],
  ["Special Blocks", [56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81]],
  ["Expansion Blocks", [82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111]],
];

function PaintOverrideRow({ paintByte, block, onUpdate, onRemove }: {
  paintByte: number;
  block: McBlock;
  onUpdate: (b: McBlock) => void;
  onRemove: () => void;
}) {
  const color = EDEN_PAINT_COLORS[paintByte - 1];
  const [r, g, b] = color.rgb;
  const sel = `${block.id}:${block.meta}`;
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 4 }}>
      <span style={{
        display: "inline-block", width: 14, height: 14, borderRadius: 3, flexShrink: 0,
        background: `rgb(${r},${g},${b})`, border: "1px solid #1a6b3a",
      }} />
      <span style={{ color: "#4d9970", fontSize: 11, minWidth: 100, flexShrink: 0 }}>
        {paintByte} – {color.label}
      </span>
      <span style={{ color: "#3d7a5a", fontSize: 11 }}>→</span>
      <select
        value={sel}
        onChange={e => {
          const [id, meta] = e.target.value.split(":").map(Number);
          onUpdate({ id, meta });
        }}
        style={{ ...selectStyle, flex: 1 }}
      >
        {MC_BLOCK_OPTIONS.map(o => (
          <option key={`${o.id}:${o.meta}`} value={`${o.id}:${o.meta}`}>{o.label}</option>
        ))}
      </select>
      <button onClick={onRemove} style={{
        background: "none", border: "none", color: "#ef4444", cursor: "pointer",
        fontSize: 14, padding: "0 2px", lineHeight: 1, flexShrink: 0,
      }}>✕</button>
    </div>
  );
}

function AddOverrideRow({ existingKeys, onAdd }: {
  existingKeys: Set<number>;
  onAdd: (paintByte: number, block: McBlock) => void;
}) {
  const [paintByte, setPaintByte] = useState<number>(() => {
    for (let i = 1; i <= 54; i++) if (!existingKeys.has(i)) return i;
    return 1;
  });
  const [blockSel, setBlockSel] = useState("1:0");

  const availablePaints = Array.from({ length: 54 }, (_, i) => i + 1).filter(p => !existingKeys.has(p));
  if (availablePaints.length === 0) return null;

  const handleAdd = () => {
    const [id, meta] = blockSel.split(":").map(Number);
    onAdd(paintByte, { id, meta });
    const next = availablePaints.find(p => p !== paintByte && !existingKeys.has(p));
    if (next) setPaintByte(next);
  };

  return (
    <div style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 6, paddingTop: 6, borderTop: "1px solid #1a6b3a" }}>
      <select
        value={paintByte}
        onChange={e => setPaintByte(Number(e.target.value))}
        style={{ ...selectStyle, width: 160, flexShrink: 0 }}
      >
        {availablePaints.map(p => {
          const c = EDEN_PAINT_COLORS[p - 1];
          return <option key={p} value={p}>{p} – {c.label}</option>;
        })}
      </select>
      <span style={{ color: "#3d7a5a", fontSize: 11 }}>→</span>
      <select
        value={blockSel}
        onChange={e => setBlockSel(e.target.value)}
        style={{ ...selectStyle, flex: 1 }}
      >
        {MC_BLOCK_OPTIONS.map(o => (
          <option key={`${o.id}:${o.meta}`} value={`${o.id}:${o.meta}`}>{o.label}</option>
        ))}
      </select>
      <button onClick={handleAdd} style={{
        background: "#15803d", color: "#f1f5f9", border: "none", borderRadius: 5,
        padding: "3px 8px", fontSize: 11, cursor: "pointer", fontWeight: 600, flexShrink: 0,
      }}>+ Add</button>
    </div>
  );
}

function BlockRow({ edenId, entry, onUpdate }: {
  edenId: number;
  entry: BlockEntry;
  onUpdate: (e: BlockEntry) => void;
}) {
  const [advancedOpen, setAdvancedOpen] = useState(false);

  const setUnpainted = (id: number, meta: number) =>
    onUpdate({ ...entry, unpainted: { id, meta } });
  const setPaintedFamily = (f: PaintedFamily) =>
    onUpdate({ ...entry, painted_family: f });

  const paintColors = entry.paint_colors ?? {};
  const overrideCount = Object.keys(paintColors).length;

  const setOverride = (paintByte: number, block: McBlock) => {
    onUpdate({ ...entry, paint_colors: { ...paintColors, [String(paintByte)]: block } });
  };
  const removeOverride = (paintByte: number) => {
    const next = { ...paintColors };
    delete next[String(paintByte)];
    onUpdate({ ...entry, paint_colors: Object.keys(next).length > 0 ? next : undefined });
  };

  const sel = `${entry.unpainted.id}:${entry.unpainted.meta}`;

  return (
    <>
      <tr style={{ borderBottom: advancedOpen ? "none" : "1px solid #1a6b3a" }}>
        <td style={{ padding: "6px 8px", color: "#86efac", fontSize: 12, whiteSpace: "nowrap" }}>
          <span style={{ color: "#3d7a5a", marginRight: 6 }}>{edenId}</span>
          {EDEN_BLOCK_NAMES[edenId] ?? `Block ${edenId}`}
        </td>
        <td style={{ padding: "6px 8px" }}>
          <select
            value={sel}
            onChange={e => {
              const [id, meta] = e.target.value.split(":").map(Number);
              setUnpainted(id, meta);
            }}
            style={selectStyle}
          >
            {MC_BLOCK_OPTIONS.map(o => (
              <option key={`${o.id}:${o.meta}`} value={`${o.id}:${o.meta}`}>{o.label}</option>
            ))}
          </select>
        </td>
        <td style={{ padding: "6px 8px" }}>
          <div style={{ display: "flex", gap: 4, alignItems: "center" }}>
            <select
              value={entry.painted_family}
              onChange={e => setPaintedFamily(e.target.value as PaintedFamily)}
              style={{ ...selectStyle, flex: 1, color: entry.painted_family !== "none" ? "#34d399" : "#4d9970" }}
            >
              {PAINTED_FAMILIES.map(f => (
                <option key={f.value} value={f.value}>{f.label}</option>
              ))}
            </select>
            <button
              onClick={() => setAdvancedOpen(o => !o)}
              title={advancedOpen ? "Close overrides" : "Add per-color overrides"}
              style={{
                background: advancedOpen ? "#166534" : (overrideCount > 0 ? "#166534" : "#14532d"),
                border: `1px solid ${overrideCount > 0 ? "#22c55e" : "#1a6b3a"}`,
                color: overrideCount > 0 ? "#4ade80" : "#4d9970",
                borderRadius: 5, padding: "3px 7px", fontSize: 11,
                cursor: "pointer", whiteSpace: "nowrap", flexShrink: 0,
              }}
            >
              {advancedOpen ? "▲" : "⚙"}{overrideCount > 0 ? ` ${overrideCount}` : ""}
            </button>
          </div>
        </td>
      </tr>
      {advancedOpen && (
        <tr style={{ borderBottom: "1px solid #1a6b3a" }}>
          <td colSpan={3} style={{ padding: "6px 12px 10px 24px", background: "#052e16" }}>
            <div style={{ fontSize: 11, color: "#3d7a5a", marginBottom: 4 }}>
              Per-color overrides — these take priority over the family setting above.
            </div>
            {Object.entries(paintColors)
              .map(([k, v]) => ({ paintByte: Number(k), block: v }))
              .sort((a, b) => a.paintByte - b.paintByte)
              .map(({ paintByte, block }) => (
                <PaintOverrideRow
                  key={paintByte}
                  paintByte={paintByte}
                  block={block}
                  onUpdate={b => setOverride(paintByte, b)}
                  onRemove={() => removeOverride(paintByte)}
                />
              ))}
            <AddOverrideRow
              existingKeys={new Set(Object.keys(paintColors).map(Number))}
              onAdd={setOverride}
            />
          </td>
        </tr>
      )}
    </>
  );
}

const selectStyle: React.CSSProperties = {
  background: "#0d3d26",
  color: "#e2e8f0",
  border: "1px solid #1a6b3a",
  borderRadius: 6,
  padding: "4px 6px",
  fontSize: 12,
  width: "100%",
};

export function BlockMappingEditor({ mapping, onChange }: Props) {
  const [openGroups, setOpenGroups] = useState<Record<string, boolean>>({ "Core Blocks": true });

  const update = (id: number, entry: BlockEntry) => {
    onChange({ blocks: { ...mapping.blocks, [String(id)]: entry } });
  };

  const exportJson = () => {
    const blob = new Blob([JSON.stringify(mapping, null, 2)], { type: "application/json" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = "eden-mapping.json";
    a.click();
  };

  const importJson = (e: React.ChangeEvent<HTMLInputElement>) => {
    const f = e.target.files?.[0];
    if (!f) return;
    f.text().then(txt => {
      try { onChange(JSON.parse(txt)); } catch { alert("Invalid mapping JSON"); }
    });
  };

  return (
    <div>
      <div style={{ display: "flex", gap: 8, marginBottom: 16, alignItems: "center" }}>
        <span style={{ color: "#86efac", fontSize: 13, flex: 1 }}>
          Customise how each Eden block maps to Minecraft. Use ⚙ for per-color overrides.
        </span>
        <button onClick={exportJson} style={btnStyle("#15803d")}>Export JSON</button>
        <label style={{ ...btnStyle("#14532d"), cursor: "pointer" }}>
          Import JSON
          <input type="file" accept=".json" style={{ display: "none" }} onChange={importJson} />
        </label>
      </div>

      {GROUP_LABELS.map(([label, ids]) => {
        const open = openGroups[label] ?? false;
        return (
          <div key={label} style={{ marginBottom: 8, border: "1px solid #1a6b3a", borderRadius: 8, overflow: "hidden" }}>
            <button
              onClick={() => setOpenGroups(g => ({ ...g, [label]: !open }))}
              style={{ width: "100%", textAlign: "left", padding: "10px 14px", background: "#14532d",
                border: "none", color: "#e2e8f0", fontSize: 13, fontWeight: 600, cursor: "pointer",
                display: "flex", justifyContent: "space-between" }}
            >
              {label} <span style={{ color: "#4d9970" }}>{open ? "▲" : "▼"}</span>
            </button>
            {open && (
              <table style={{ width: "100%", borderCollapse: "collapse" }}>
                <thead>
                  <tr style={{ background: "#0d3d26" }}>
                    <th style={thStyle}>Eden Block</th>
                    <th style={thStyle}>MC block (unpainted)</th>
                    <th style={thStyle}>MC block (if painted)</th>
                  </tr>
                </thead>
                <tbody>
                  {ids.map(id => {
                    const entry = mapping.blocks[String(id)];
                    if (!entry) return null;
                    return <BlockRow key={id} edenId={id} entry={entry} onUpdate={e => update(id, e)} />;
                  })}
                </tbody>
              </table>
            )}
          </div>
        );
      })}
    </div>
  );
}

const thStyle: React.CSSProperties = {
  padding: "6px 8px", textAlign: "left", fontSize: 11,
  color: "#4d9970", fontWeight: 600, textTransform: "uppercase", letterSpacing: "0.05em",
};

function btnStyle(bg: string): React.CSSProperties {
  return {
    background: bg, color: "#f1f5f9", border: "none", borderRadius: 6,
    padding: "6px 12px", fontSize: 12, cursor: "pointer", fontWeight: 600,
  };
}
