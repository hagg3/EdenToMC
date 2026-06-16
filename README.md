# Eden Tools — Eden ↔ Minecraft Converter & World Generator

Convert [Eden World Builder](https://www.eden-game.com/) worlds into playable Minecraft Java Edition worlds, or generate brand-new procedural worlds — entirely in your browser.

[![Live App](https://img.shields.io/badge/Open_Web_App-2563eb?style=for-the-badge)](https://hagg3.github.io/EdenToMC/)

---

> **⚠️ Experimental software — expect bugs**
>
> All features in this project — including the original Eden→Minecraft converter, the new procedural terrain generator, and the Minecraft→Eden importer — are **experimental and known to be buggy**. Converted worlds may have missing blocks, incorrect terrain, broken spawns, or other artefacts. Terrain-generated worlds are particularly rough and should be considered a proof of concept. Use at your own risk and always keep a backup of your original `.eden` files.

---

## Features

### Convert `.eden` → Minecraft
Drop an Eden World Builder `.eden` file and download a ready-to-play Minecraft 1.12 world ZIP. Runs 100% in your browser — nothing is uploaded.

### Generate a procedural world *(new, experimental)*
Create a fresh world from noise-based terrain generation directly in the browser. Tune the size, seed, base height, and water coverage, then download the result as a `.eden` file or convert it straight to a Minecraft ZIP.

### Minecraft regions → Eden *(new, CLI-only, experimental)*
Import Minecraft `.mca` region files back into the Eden format using the C++ command-line tool. Useful for round-tripping or inspecting Minecraft terrain in Eden.

---

## Using the web app

The easiest way — no building required.

**1. Open the app**
Go to **[hagg3.github.io/EdenToMC](https://hagg3.github.io/EdenToMC/)** in any modern browser (Chrome, Firefox, Safari, Edge).

### Convert tab

**2. Drop your Eden world file**
Drag your `.eden` file onto the drop zone, or click it to browse.
Eden world files are typically found on your device at:

| Platform | Location |
|---|---|
| iOS (via Files app) | `On My iPhone → Eden` |
| Shared worlds (downloaded) | Usually a `.zip` — extract first, the `.eden` file is inside |

**3. Click Convert, then Download**
The converter runs locally in your browser. When it finishes, click **Download ZIP** — you'll get a file named `<WorldName>-minecraft.zip`.

### Generate World tab *(experimental)*

1. Choose a world size (64–512 blocks per side).
2. Enter a seed number or hit 🎲 to randomise.
3. Adjust **Base Height** (how high the terrain sits) and **Water Coverage** (sea level).
4. Click **Generate Terrain** — generation runs in the browser and may take 5–30 seconds for larger worlds.
5. Once done, you can:
   - **Download .eden** — load the world directly in the Eden app.
   - **Convert → Minecraft ZIP** — convert the generated world to Minecraft format and download.

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

### Eden → Minecraft conversion

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
  lib.rs  ──►  convert(bytes, mappingJson?) → Uint8Array  (a ZIP archive)
      │
      ▼
  Browser downloads  WorldName-minecraft.zip
  containing:  level.dat  +  region/r.X.Z.mca
```

### Procedural terrain generation *(new)*

```
TerrainParams (width, depth, seed, baseHeight, waterAmnt)
      │
      ▼
  Rust / WASM  (converter/src/)
  ├─ noise.rs      Fractal Brownian Motion value noise (2D + 3D)
  └─ terrain.rs    7-stage generator:
                   1. Heightmap  (fractal noise, baseHeight ± 18)
                   2. Fill       (stone / dirt / grass layers)
                   3. Caves      (3D noise threshold 0.62)
                   4. Water      (sea level by waterAmnt; ponds + rivers)
                   5. Beaches    (grass→sand near water)
                   6. Vegetation (3 tree canopy shapes + flowers)
                   7. Snow       (sand with color=1 above y=48)
      │
      ▼
  eden_writer.rs  ──►  generate_world() → JSON { eden: <base64>, stats: {...} }
      │
      ▼
  Browser: download .eden  and/or  convert → Minecraft ZIP
```

**Eden file format** (reverse-engineered — see `MROB.txt` for full notes):
- Fixed 192-byte header: seed, player position, world name, sky colours, directory offset.
- Chunk directory: one 16-byte entry per chunk column — `(cx, cz, file_offset)`.
- Each chunk column: 4 vertical 16×16×16 sub-chunks; each sub-chunk = 4096 block-type bytes + 4096 paint bytes.
- Voxel index within a sub-chunk: `x * 256 + z * 16 + y`.

**Coordinate mapping**: the converter recenters the world so the player's position becomes Minecraft chunk (0, 0).

**Colour support**: Eden's 54 paint colours are mapped to the nearest of Minecraft's 16 dye colours. The mapping table is in `converter/src/block_map.rs` (`MC_COLORS`).

---

## Project structure

```
EdenToMC/
├── converter/              Rust crate → compiled to WebAssembly
│   └── src/
│       ├── lib.rs          WASM entry points (convert, default_mapping_json, generate_world)
│       ├── eden.rs         Eden binary parser
│       ├── nbt.rs          NBT encoder + zlib/gzip helpers
│       ├── anvil.rs        Anvil .mca region builder
│       ├── level_dat.rs    level.dat generator
│       ├── block_map.rs    Eden→MC block + colour mapping tables
│       ├── noise.rs        Fractal Brownian Motion value noise  [new]
│       ├── terrain.rs      7-stage procedural terrain generator  [new]
│       └── eden_writer.rs  Serialize TerrainWorld → .eden binary  [new]
├── web/                    React + TypeScript + Vite web app
│   └── src/
│       ├── App.tsx         Two-tab UI: convert + generate  [updated]
│       ├── types.ts        Shared types and block name tables
│       └── components/
│           ├── DropZone.tsx
│           └── BlockMappingEditor.tsx
├── .github/workflows/
│   ├── pages.yml           CI: build + deploy to GitHub Pages on push to main
│   └── release.yml         CI: build + attach zip to GitHub Release on tag push
├── CLAUDE.md               Codebase notes for AI assistants
├── MROB.txt                Eden file format reverse-engineering notes
└── *.cpp / *.h             C++ CLI tool (see below)
```

---

## C++ CLI tool

The command-line tool targets Windows (Visual Studio 2019+) and supports three modes:

```bash
# Convert FILE.eden → ConvertedWorld/ (Minecraft region files)
./eden_tool FILE.eden

# Convert Minecraft region folder → Eden world file  [new, experimental]
./eden_tool mc2eden <region_folder> <out.eden>

# Generate a procedural Eden world from scratch  [new, experimental]
./eden_tool generate <width> <depth> <seed> <out.eden> [baseHeight] [waterAmnt(1-5)]
# e.g.: ./eden_tool generate 256 256 42 world.eden 30 3
```

The `mc2eden` importer reads `.mca` region files, downsamples Minecraft's 256-block height to Eden's 64-block limit using mode-voting (4:1), and maps block names to Eden types. Coverage is basic — most non-standard blocks become air.

The C++ tool does not generate `level.dat` and does not support Eden paint colours. The web app supersedes it for most use cases, but the source is kept for reference and offline use.

> **Note:** The C++ tool requires zlib on the compiler's include/library path. Build with Visual Studio using `EdenFileReader.vcxproj`.
