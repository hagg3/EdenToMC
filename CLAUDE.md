# CLAUDE.md — EdenToMC

## Project overview

EdenToMC converts [Eden World Builder](https://www.eden-game.com/) `.eden` worlds into playable Minecraft Java Edition 1.12 worlds. It has three parts:

- **Rust/WASM converter** (`converter/`) — compiled to WebAssembly, runs in-browser
- **React web UI** (`web/`) — drag-and-drop conversion + procedural terrain generation
- **C++ CLI tool** (repo root `*.cpp/h`) — legacy command-line converter for Windows (Visual Studio)

## Build commands

### WASM (Rust → browser)
```bash
cd converter
wasm-pack build --target web --out-dir ../web/src/wasm
```
Must be re-run any time `converter/src/` changes.

### Web frontend
```bash
cd web
npm install
npm run dev          # dev server at http://localhost:5173
npx tsc && npx vite build  # production build → web/dist/
```

### C++ CLI (Windows / Visual Studio)
Open `EdenFileReader.vcxproj` in Visual Studio 2019+. Requires zlib on the include/lib path.

## Architecture

### Rust converter (`converter/src/`)

| File | Purpose |
|---|---|
| `lib.rs` | WASM entry points: `convert()`, `default_mapping_json()`, `generate_world()` |
| `eden.rs` | Parse `.eden` binary: 192-byte header, column directory, 4×16³ sub-chunks |
| `block_map.rs` | Eden block type + paint byte → Minecraft block ID + metadata |
| `nbt.rs` | Minecraft NBT encoder (big-endian) |
| `anvil.rs` | Pack NBT chunks into Anvil `.mca` region files |
| `level_dat.rs` | Generate `level.dat` (world name, seed, spawn) |
| `noise.rs` | Fractal Brownian Motion value noise (2D and 3D) |
| `terrain.rs` | 7-stage procedural world generator |
| `eden_writer.rs` | Serialize a `TerrainWorld` to `.eden` binary |

### Eden file format (reverse-engineered — see `MROB.txt`)
- **Header**: 192 bytes. Key fields:
  - `0`: `i32 level_seed`
  - `4–15`: `f32 pos.x/y/z` (player position)
  - `16–27`: `f32 home.x/y/z`
  - `28`: `f32 yaw`
  - `32`: `u64 directory_offset`
  - `40–89`: `char name[50]`
  - `90`: `i32 version` (= 4, **no alignment padding** — ARM unaligned read)
  - `130–145`: `u8 skycolors[16]`
- **Column directory**: `(i32 cx, i32 cz, u64 offset)` × N — each 16 bytes
- **Column data**: 4 sub-chunks × (4096 block-type bytes + 4096 paint bytes) = 32 768 bytes/column
- **Voxel index** within a sub-chunk: `x * 256 + z * 16 + y`

### Terrain generator stages (`terrain.rs` / `TerrainGenerator.cpp`)
1. **Heightmap** — fractal noise, `baseHeight ± 18`, clamped 1–62
2. **Terrain fill** — stone (bottom), dirt (middle), grass (surface)
3. **Caves** — 3D fractal noise threshold 0.62; floor at y=1; 4-block surface crust
4. **Water** — sea level by `water_amnt` (1=40, 2=35, 3=32, 4=27, 5=none); ponds + rivers
5. **Beaches** — grass→sand near water edges
6. **Vegetation** — trees (3 canopy patterns: ellipsoid/flat/conifer) + flowers; 4-block world edge guard
7. **Snow** — sand (color=1) above y=48

### `generate_world()` WASM export
- Input: JSON string `{ width, depth, seed, base_height?, water_amnt? }`
- Output: JSON string `{ eden: "<base64>", stats: { spawn_x, spawn_y, spawn_z, trees_placed, flowers_placed, caves_carved, min_height, max_height, cols_x, cols_z } }`

### C++ CLI entry points (`main.cpp`)
```
./eden_tool                                         # convert FILE.eden → ConvertedWorld/
./eden_tool mc2eden <region_folder> <out.eden>      # Minecraft region → Eden
./eden_tool generate <w> <d> <seed> <out.eden> [baseHeight] [waterAmnt]
```

## Key types

### Rust
- `TerrainParams` — `{ width, depth, seed, base_height=30, water_amnt=3 }`
- `TerrainWorld` — `{ width, depth, blocks: Vec<TerrainBlock>, meta: TerrainMeta }`
- `world_idx(wx, wy, wz, depth)` — `wx * depth * 64 + wz * 64 + wy`

### C++
- `EdenColumn` — `{ x, z, blocks[16][64][16] }`
- `EdenBlock` — `{ type: u8, color: u8 }`
- `CHUNK_SIZE` = 16, height limit = 64 (4 sub-chunks × 16)

## Eden block types (key subset)
```
TYPE_NONE=0, TYPE_STONE=2, TYPE_DIRT=3, TYPE_SAND=4,
TYPE_LEAVES=5, TYPE_TREE=6, TYPE_GRASS=8, TYPE_WATER=20, TYPE_FLOWER=73
```
Full list in `Constants.h`.

## Notes / gotchas

- The `.eden` header is 192 bytes, **not** 228. `eden.rs` has a comment claiming 228 — that is incorrect.
- `int version` sits at byte 90 with no padding: the game targets ARM iOS which allows unaligned 32-bit reads.
- The C++ `EdenWriter` relies on `EdenFileLoader.h` for the `WorldFileHeader` and `ColumnIndex` struct definitions.
- `AnvilReader.cpp` links against **zlib** — ensure it is available on the include/library path.
- `MCToEdenMapper` logs unknown block names once via a deduplicating `std::set<std::string> unknownLogged`.
- Terrain generation is computationally heavy — a 512×512 world takes several seconds even in WASM release mode.
- The WASM bundle (`web/src/wasm/`) must be committed/deployed separately; it is not rebuilt automatically by the web CI.
