# Eden → Minecraft Converter

Convert [Eden World Builder](https://www.eden-game.com/) worlds into playable Minecraft Java Edition worlds — entirely in your browser. No installs, no uploads, nothing leaves your device.

[![Live App](https://img.shields.io/badge/Open_Web_App-2563eb?style=for-the-badge)](https://hagg3.github.io/EdenToMC/)

---

## Using the web app

The easiest way — no building required.

**1. Open the app**  
Go to **[hagg3.github.io/EdenToMC](https://hagg3.github.io/EdenToMC/)** in any modern browser (Chrome, Firefox, Safari, Edge).

**2. Drop your Eden world file**  
Drag your `.eden` file onto the drop zone, or click it to browse.  
Eden world files are typically found on your device at:

| Platform | Location |
|---|---|
| iOS (via Files app) | `On My iPhone → Eden` |
| Shared worlds (downloaded) | Usually a `.zip` — extract first, the `.eden` file is inside |

**3. Click Convert, then Download**  
The converter runs locally in your browser. When it finishes, click **Download ZIP** — you'll get a file named `<WorldName>-minecraft.zip`.

---

## Installing the Minecraft world

1. Extract the ZIP. You'll find `level.dat` and a `region/` folder inside.
2. Open your Minecraft saves folder:
   - **Windows:** `%APPDATA%\.minecraft\saves\`
   - **macOS:** `~/Library/Application Support/minecraft/saves/`
   - **Linux:** `~/.minecraft/saves/`
3. Create a new folder inside `saves/` — name it whatever you like (e.g. `MyEdenWorld`).
4. Move `level.dat` and the `region/` folder into that new folder.
5. Launch **Minecraft Java Edition 1.12.2** → Singleplayer. Your world will appear in the list.

> **Version note:** The converter produces Minecraft 1.12 Anvil format. It will also open in many later versions via world conversion, but 1.12.2 gives the most faithful result.

---

## Customising block mapping

Eden has 111 block types and 54 paint colours. The converter ships with a default mapping, but you can change how every block translates:

1. Click **"Show block mapping editor"** below the drop zone.
2. Blocks are grouped into sections (Core, Ramps, Sides, Special, Expansion Pack). Expand any group to edit it.
3. For each Eden block you can set two things:
   - **Unpainted → MC Block** — which Minecraft block to use when the Eden block has no paint applied.
   - **When Painted →** — which coloured Minecraft block family to use when the Eden block *is* painted. Options:
     - `None` — always use the unpainted block, ignore paint.
     - `Concrete` — maps paint colour to one of Minecraft's 16 concrete colours.
     - `Wool` — same for wool.
     - `Stained Glass` — same for stained glass.
     - `Terracotta` — same for stained hardened clay.
4. Click **Export JSON** to save your mapping. Click **Import JSON** to reload it on a future visit.

---

## Building from source

### Prerequisites

| Tool | Install |
|---|---|
| Rust (stable) | [rustup.rs](https://rustup.rs) |
| wasm32 target | `rustup target add wasm32-unknown-unknown` |
| wasm-pack | `cargo install wasm-pack` |
| Node.js 18+ | [nodejs.org](https://nodejs.org) |

### Steps

```bash
# 1. Clone the repo
git clone https://github.com/hagg3/EdenToMC.git
cd EdenToMC

# 2. Build the Rust WASM module (outputs to web/src/wasm/)
cd converter
wasm-pack build --target web --out-dir ../web/src/wasm

# 3. Install frontend dependencies
cd ../web
npm install

# 4. Start the dev server (hot-reload)
npm run dev
# → opens at http://localhost:5173

# 5. Or build a production bundle
npx tsc && npx vite build
# → output in web/dist/
```

To rebuild the WASM after changing Rust code, re-run step 2.

---

## How it works

```
.eden file (binary)
      │
      ▼
  Rust / WASM  (converter/src/)
  ├─ eden.rs      Parse binary header, chunk directory, and 4×16³ chunk columns
  ├─ block_map.rs Map each Eden block type + paint byte → Minecraft block ID + metadata
  ├─ nbt.rs       Encode Minecraft NBT (big-endian binary)
  ├─ anvil.rs     Pack NBT chunks into Anvil region files (.mca)
  └─ level_dat.rs Generate level.dat (world name, seed, spawn, game rules)
      │
      ▼
  lib.rs  ──►  convert(bytes, mappingJson?) → Uint8Array
                                              (a ZIP archive)
      │
      ▼
  React frontend  (web/src/)
  ├─ App.tsx                  File drop, WASM call, download trigger
  ├─ components/DropZone.tsx  Drag-and-drop file input
  └─ components/BlockMappingEditor.tsx  Per-block mapping UI, JSON import/export
      │
      ▼
  Browser downloads  WorldName-minecraft.zip
  containing:  level.dat  +  region/r.X.Z.mca  (one per 512×512 block region)
```

**Eden file format** (reverse-engineered — see `MROB.txt` for the full notes):
- Fixed 228-byte header: seed, player position, world name, sky colours, directory offset.
- Chunk directory: one 16-byte entry per chunk column — `(cx, cz, file_offset)`.
- Each chunk column: 4 vertical 16×16×16 chunks, each chunk = 4096 block-type bytes + 4096 paint bytes.

**Coordinate mapping**: the converter recenters the world so the player's position becomes Minecraft chunk (0, 0).

**Colour support**: Eden's 54 paint colours are mapped to the nearest of Minecraft's 16 dye colours. The mapping table is in `converter/src/block_map.rs` (`MC_COLORS`).

---

## Project structure

```
EdenToMC/
├── converter/              Rust crate → compiled to WebAssembly
│   └── src/
│       ├── lib.rs          WASM entry points (convert, default_mapping_json)
│       ├── eden.rs         Eden binary parser
│       ├── nbt.rs          NBT encoder + zlib/gzip helpers
│       ├── anvil.rs        Anvil .mca region builder
│       ├── level_dat.rs    level.dat generator
│       └── block_map.rs    Eden→MC block + colour mapping tables
├── web/                    React + TypeScript + Vite web app
│   └── src/
│       ├── App.tsx         Main app — file drop, conversion, download
│       ├── types.ts        Shared types and Eden/MC block name tables
│       └── components/
│           ├── DropZone.tsx
│           └── BlockMappingEditor.tsx
├── .github/workflows/
│   ├── pages.yml           CI: build + deploy to GitHub Pages on push to main
│   └── release.yml         CI: build + attach zip to GitHub Release on tag push
├── MROB.txt                Eden file format reverse-engineering notes (by mrob27)
└── *.cpp / *.h             Legacy C++ CLI tool (see below)
```

---

## Legacy C++ tool

The original converter is a command-line C++ program targeting Windows (Visual Studio). It outputs raw Minecraft region files to a `ConvertedWorld/` folder. Those files then have to be manually placed into a Minecraft world.

```bash
./eden_tool_v2 <path/to/world.eden>
# → writes ConvertedWorld/region/*.mca
```

The C++ tool does not generate `level.dat` and does not support Eden paint colours. The web app supersedes it, but the source is kept for reference.
