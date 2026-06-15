import React, { useState } from "react";
import {
  BlockMapping, BlockEntry, PaintedFamily,
  EDEN_BLOCK_NAMES, MC_BLOCK_OPTIONS, PAINTED_FAMILIES,
} from "../types";

interface Props {
  mapping: BlockMapping;
  onChange: (m: BlockMapping) => void;
}

const GROUP_LABELS: [string, number[]][] = [
  ["Core Blocks", [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23]],
  ["Ramps", [24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39]],
  ["Sides / Fences", [40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55]],
  ["Special Blocks", [56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81]],
  ["Expansion Pack", [82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111]],
];

function BlockRow({ edenId, entry, onUpdate }: {
  edenId: number;
  entry: BlockEntry;
  onUpdate: (e: BlockEntry) => void;
}) {
  const setUnpainted = (id: number, meta: number) =>
    onUpdate({ ...entry, unpainted: { id, meta } });
  const setPaintedFamily = (f: PaintedFamily) =>
    onUpdate({ ...entry, painted_family: f });

  const sel = `${entry.unpainted.id}:${entry.unpainted.meta}`;

  return (
    <tr style={{ borderBottom: "1px solid #1e293b" }}>
      <td style={{ padding: "6px 8px", color: "#94a3b8", fontSize: 12, whiteSpace: "nowrap" }}>
        <span style={{ color: "#475569", marginRight: 6 }}>{edenId}</span>
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
        <select
          value={entry.painted_family}
          onChange={e => setPaintedFamily(e.target.value as PaintedFamily)}
          style={{ ...selectStyle, color: entry.painted_family !== "none" ? "#34d399" : "#64748b" }}
        >
          {PAINTED_FAMILIES.map(f => (
            <option key={f.value} value={f.value}>{f.label}</option>
          ))}
        </select>
      </td>
    </tr>
  );
}

const selectStyle: React.CSSProperties = {
  background: "#0f172a",
  color: "#e2e8f0",
  border: "1px solid #334155",
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
        <span style={{ color: "#94a3b8", fontSize: 13, flex: 1 }}>
          Customise how each Eden block maps to Minecraft.
        </span>
        <button onClick={exportJson} style={btnStyle("#1d4ed8")}>Export JSON</button>
        <label style={{ ...btnStyle("#374151"), cursor: "pointer" }}>
          Import JSON
          <input type="file" accept=".json" style={{ display: "none" }} onChange={importJson} />
        </label>
      </div>

      {GROUP_LABELS.map(([label, ids]) => {
        const open = openGroups[label] ?? false;
        return (
          <div key={label} style={{ marginBottom: 8, border: "1px solid #1e293b", borderRadius: 8, overflow: "hidden" }}>
            <button
              onClick={() => setOpenGroups(g => ({ ...g, [label]: !open }))}
              style={{ width: "100%", textAlign: "left", padding: "10px 14px", background: "#1e293b",
                border: "none", color: "#e2e8f0", fontSize: 13, fontWeight: 600, cursor: "pointer",
                display: "flex", justifyContent: "space-between" }}
            >
              {label} <span style={{ color: "#64748b" }}>{open ? "▲" : "▼"}</span>
            </button>
            {open && (
              <table style={{ width: "100%", borderCollapse: "collapse" }}>
                <thead>
                  <tr style={{ background: "#0f172a" }}>
                    <th style={thStyle}>Eden Block</th>
                    <th style={thStyle}>Unpainted → MC Block</th>
                    <th style={thStyle}>When Painted →</th>
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
  color: "#64748b", fontWeight: 600, textTransform: "uppercase", letterSpacing: "0.05em",
};

function btnStyle(bg: string): React.CSSProperties {
  return {
    background: bg, color: "#f1f5f9", border: "none", borderRadius: 6,
    padding: "6px 12px", fontSize: 12, cursor: "pointer", fontWeight: 600,
  };
}
