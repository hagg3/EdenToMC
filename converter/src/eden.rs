use std::collections::HashMap;

pub struct EdenHeader {
    pub seed: i32,
    pub player_x: f32,
    pub player_y: f32,
    pub player_z: f32,
    pub name: String,
    pub sky_colors: [u8; 16],
    pub directory_offset: u64,
}

pub struct ChunkColumn {
    pub cx: i32,
    pub cz: i32,
    /// 4 vertical chunks of 16×16×16, block types
    pub blocks: Vec<u8>, // len = 4 * 4096
    /// 4 vertical chunks of 16×16×16, paint bytes
    pub paints: Vec<u8>, // len = 4 * 4096
}

pub struct EdenWorld {
    pub header: EdenHeader,
    pub columns: Vec<ChunkColumn>,
    pub player_chunk_x: i32,
    pub player_chunk_z: i32,
}

fn read_u32_le(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(data[off..off + 4].try_into().unwrap())
}
fn read_u64_le(data: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(data[off..off + 8].try_into().unwrap())
}
fn read_f32_le(data: &[u8], off: usize) -> f32 {
    f32::from_le_bytes(data[off..off + 4].try_into().unwrap())
}
fn read_i32_le(data: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(data[off..off + 4].try_into().unwrap())
}

fn read_cstr(data: &[u8], off: usize, max_len: usize) -> String {
    let slice = &data[off..off + max_len];
    let end = slice.iter().position(|&b| b == 0).unwrap_or(max_len);
    String::from_utf8_lossy(&slice[..end]).into_owned()
}

pub fn parse_world(data: &[u8]) -> Result<EdenWorld, String> {
    if data.len() < 228 {
        return Err("File too small to be a valid Eden world".into());
    }

    // Header layout (from EdenFileLoader.h WorldFileHeader):
    // 0:  int level_seed          (4)
    // 4:  Vector pos (x,y,z)      (12)
    // 16: Vector home (x,y,z)     (12)
    // 28: float yaw               (4)
    // 32: u64 directory_offset    (8)
    // 40: char name[50]           (50)
    // 90: int version             (4)
    // 94: char hash[36]           (36)
    // 130: u8 skycolors[16]       (16)
    // 146: int goldencubes        (4)
    // 150: char reserved[44]      (44)
    // Total = 194 bytes — but the struct includes padding; the C++ says 228 bytes total
    // Actually: 4+12+12+4+8+50+4+36+16+4+44 = 194. With alignment the C struct pads to 228.
    // The directory_offset is at byte 32 per MROB.txt (bytes 32-35 = LE u32) and the
    // CLAUDE.md says "Bytes 32–35: LE u32 — chunk pointer table offset".
    // But EdenFileLoader.h places it at offset 32 as a u64 (8 bytes).
    // Let's read as u32 (MROB says 4 bytes) and also try u64 if needed.

    let seed = read_i32_le(data, 0);
    let px = read_f32_le(data, 4);
    let py = read_f32_le(data, 8);
    let pz = read_f32_le(data, 12);
    // home at 16, yaw at 28
    // directory_offset: MROB says bytes 32-35 (u32 LE), CLAUDE.md says same
    let dir_off_u32 = read_u32_le(data, 32) as u64;
    // Also try reading as u64 — if the high 4 bytes look like part of the name, use u32
    let dir_off_u64 = read_u64_le(data, 32);
    // The name starts at byte 40 per the struct. If the u64 interpretation would make
    // directory_offset > file size, fall back to u32.
    let directory_offset = if dir_off_u64 < data.len() as u64 && dir_off_u64 > 228 {
        dir_off_u64
    } else {
        dir_off_u32
    };

    let name = read_cstr(data, 40, 50);
    let mut sky_colors = [0u8; 16];
    // sky colors at byte 130 (after hash at 94..130)
    if data.len() >= 146 {
        sky_colors.copy_from_slice(&data[130..146]);
    }

    let header = EdenHeader { seed, player_x: px, player_y: py, player_z: pz, name, sky_colors, directory_offset };

    let player_chunk_x = (px / 16.0).floor() as i32;
    let player_chunk_z = (pz / 16.0).floor() as i32;

    // Read chunk directory: each entry is 16 bytes: i32 x, i32 (pad?), i32 z, u64 offset
    // From EdenFileLoader.h: struct ColumnIndex { int x, z; unsigned long long chunk_offset; }
    // That's 4+4+8 = 16 bytes. But with struct alignment, int x at 0, int z at 4, u64 at 8.
    let dir_off = directory_offset as usize;
    if dir_off >= data.len() {
        return Err(format!("Directory offset {} beyond file size {}", dir_off, data.len()));
    }

    let mut col_map: HashMap<(i32, i32), u64> = HashMap::new();
    let mut pos = dir_off;
    while pos + 16 <= data.len() {
        let cx = read_i32_le(data, pos);
        let cz = read_i32_le(data, pos + 4);
        let offset = read_u64_le(data, pos + 8);
        // Sanity check: offset should be within file and after header
        if offset > 0 && offset < data.len() as u64 {
            col_map.insert((cx, cz), offset);
        }
        pos += 16;
    }

    if col_map.is_empty() {
        return Err("No valid chunk columns found in directory".into());
    }

    let mut columns = Vec::new();
    for ((cx, cz), offset) in &col_map {
        let off = *offset as usize;
        // Each column: 4 chunks, each chunk = 4096 blocks + 4096 paints = 8192 bytes
        let col_size = 4 * 8192;
        if off + col_size > data.len() {
            continue; // truncated column, skip
        }
        let raw = &data[off..off + col_size];
        let mut blocks = Vec::with_capacity(4 * 4096);
        let mut paints = Vec::with_capacity(4 * 4096);
        for cy in 0..4 {
            let base = cy * 8192;
            blocks.extend_from_slice(&raw[base..base + 4096]);
            paints.extend_from_slice(&raw[base + 4096..base + 8192]);
        }
        columns.push(ChunkColumn { cx: *cx, cz: *cz, blocks, paints });
    }

    Ok(EdenWorld { header, columns, player_chunk_x, player_chunk_z })
}

/// Voxel index within a chunk's flat array.
/// Eden layout: x * 16 * 16 + z * 16 + y  (from C++ code)
pub fn eden_voxel_idx(x: usize, z: usize, y: usize) -> usize {
    x * 256 + z * 16 + y
}

/// Minecraft Anvil voxel index: (y * 16 + z) * 16 + x
pub fn anvil_voxel_idx(x: usize, z: usize, y: usize) -> usize {
    (y * 16 + z) * 16 + x
}
