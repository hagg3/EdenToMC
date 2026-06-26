mod nbt;
mod eden;
mod block_map;
mod anvil;
mod level_dat;
mod noise;
mod terrain;
mod eden_writer;

use wasm_bindgen::prelude::*;
use std::io::Write;

#[wasm_bindgen]
pub fn convert(eden_bytes: &[u8], mapping_json: Option<String>) -> Result<Vec<u8>, JsValue> {
    let mapping = if let Some(json) = mapping_json {
        block_map::mapping_from_json(&json).map_err(|e| JsValue::from_str(&e))?
    } else {
        block_map::default_mapping()
    };

    // ZIP-wrapped .eden: detect by PK\x03\x04 magic OR by scanning for the EOCD
    // signature (PK\x05\x06) near the end of file.  The EOCD scan catches
    // self-extracting archives and other ZIP variants that don't open with PK\x03\x04.
    let is_zip = eden_bytes.starts_with(b"PK\x03\x04")
        || (eden_bytes.len() >= 22 && {
            // ZIP spec allows up to 64 KB of comment after EOCD; scan accordingly.
            let start = eden_bytes.len().saturating_sub(65_558);
            eden_bytes[start..].windows(4).any(|w| w == b"PK\x05\x06")
        });
    if is_zip {
        return convert_zip(eden_bytes, mapping);
    }

    let world = eden::parse_world(eden_bytes)
        .map_err(|e| {
            // Include the first 8 bytes as hex to help diagnose unknown formats
            // (gzip = 1F 8B, zlib = 78 9C/DA/01, raw Eden = 4-byte LE seed).
            let hex: String = eden_bytes.iter().take(8)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            JsValue::from_str(&format!("{} [file header: {}]", e, hex))
        })?;

    let mut archive = anvil::AnvilArchive::new();

    for col in &world.columns {
        let out_cx = col.cx - world.player_chunk_x;
        let out_cz = col.cz - world.player_chunk_z;

        let mut sections: Vec<(u8, Vec<u8>, Vec<u8>)> = Vec::new();

        let num_chunks = col.chunks_per_column;
        for cy in 0..num_chunks {
            let mut blks = vec![0u8; 4096];
            let mut data = vec![0u8; 2048];
            let mut has_blocks = false;

            for ex in 0..16usize {
                for ez in 0..16usize {
                    for ey in 0..16usize {
                        let local_idx = eden::eden_voxel_idx(ex, ez, ey);
                        let block_type = col.get_block(eden_bytes, cy, local_idx);
                        let paint_byte = col.get_paint(eden_bytes, cy, local_idx);
                        let mc = block_map::resolve(&mapping, block_type, paint_byte);
                        if mc.id == 0 { continue; }
                        has_blocks = true;
                        let anvil_idx = eden::anvil_voxel_idx(ex, ez, ey);
                        blks[anvil_idx] = mc.id;
                        // Pack 4-bit metadata
                        let nibble_idx = anvil_idx / 2;
                        if anvil_idx % 2 == 0 {
                            data[nibble_idx] = (data[nibble_idx] & 0xF0) | (mc.meta & 0x0F);
                        } else {
                            data[nibble_idx] = (data[nibble_idx] & 0x0F) | ((mc.meta & 0x0F) << 4);
                        }
                    }
                }
            }

            if has_blocks {
                sections.push((cy as u8, blks, data));
            }
        }

        archive.write_chunk(out_cx, out_cz, sections);
    }

    // Build level.dat
    let level_dat = level_dat::build_level_dat(
        &world.header.name,
        world.header.seed,
        0, // spawn at origin (recentered)
        world.header.player_y as i32,
        0,
    );

    // Pack everything into a zip
    let mut zip_buf = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut zip_buf);
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("level.dat", options)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        zip.write_all(&level_dat)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        for (name, bytes) in archive.into_files() {
            // Ensure the region/ directory entry exists
            let _ = zip.add_directory("region/", options);
            zip.start_file(&name, options)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            zip.write_all(&bytes)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        }

        zip.finish().map_err(|e| JsValue::from_str(&e.to_string()))?;
    }

    Ok(zip_buf.into_inner())
}

/// Return the default block mapping as JSON (for the web UI to display).
#[wasm_bindgen]
pub fn default_mapping_json() -> String {
    let m = block_map::default_mapping();
    serde_json::to_string(&m).unwrap_or_default()
}

/// Generate a procedural Eden world and return raw .eden file bytes.
/// `params_json` must be a JSON object with fields:
///   width (u32), depth (u32), seed (u32),
///   base_height (i32, optional, default 30),
///   water_amnt (u32 1-5, optional, default 3)
/// Returns JSON: { eden: <base64 eden bytes>, stats: { ... } }
#[wasm_bindgen]
pub fn generate_world(params_json: &str) -> Result<String, JsValue> {
    let params: terrain::TerrainParams = serde_json::from_str(params_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid params: {}", e)))?;

    let world = terrain::generate(&params)
        .map_err(|e| JsValue::from_str(&e))?;

    let meta_json = serde_json::json!({
        "spawn_x": world.meta.spawn_x,
        "spawn_y": world.meta.spawn_y,
        "spawn_z": world.meta.spawn_z,
        "trees_placed": world.meta.trees_placed,
        "flowers_placed": world.meta.flowers_placed,
        "caves_carved": world.meta.caves_carved,
        "min_height": world.meta.min_height,
        "max_height": world.meta.max_height,
        "cols_x": world.meta.cols_x,
        "cols_z": world.meta.cols_z,
    });

    let eden_bytes = eden_writer::write_eden_file(&world, &params);

    // Base64-encode the Eden bytes for safe JSON transport
    let eden_b64 = base64_encode(&eden_bytes);

    let result = serde_json::json!({
        "eden": eden_b64,
        "stats": meta_json,
    });

    Ok(result.to_string())
}

// ── ZIP streaming support ────────────────────────────────────────────────────
//
// Eden files sometimes come as ZIP archives (PK magic 50 4B 03 04).  The naive
// approach – decompress the whole entry into a Vec<u8> – fails for 2+ GB worlds
// because the decompressed buffer and the ZIP input sit in WASM memory together,
// exhausting the 32-bit address space.
//
// Instead we use two streaming passes over the ZIP entry:
//   Pass 1 (zip_read_world_info): reads the 228-byte Eden header, skips forward
//           to directory_offset, reads only the directory (~hundreds of KB).
//   Pass 2 (zip_stream_convert): re-opens the entry, streams columns in file
//           order, converts each in-place, writes to AnvilArchive immediately.
//
// Peak WASM memory: ZIP input + one column buffer (131 KB) + AnvilArchive output.

struct ZipWorldInfo {
    world_name: String,
    seed: i32,
    player_y: f32,
    player_chunk_x: i32,
    player_chunk_z: i32,
    /// (cx, cz, byte-offset into decompressed stream)
    columns: Vec<(i32, i32, usize)>,
    chunks_per_column: usize,
}

fn zip_read_world_info(zip_bytes: &[u8]) -> Result<ZipWorldInfo, String> {
    use std::io::Read;
    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Invalid ZIP archive: {}", e))?;
    if archive.len() == 0 {
        return Err("ZIP archive contains no files".into());
    }
    let mut entry = archive.by_index(0)
        .map_err(|e| format!("Cannot open ZIP entry: {}", e))?;

    let total_size = entry.size(); // decompressed size from ZIP central directory

    // Read Eden header (228 bytes).
    let mut hdr = [0u8; 228];
    entry.read_exact(&mut hdr)
        .map_err(|e| format!("Failed to read Eden header from ZIP: {}", e))?;

    // Parse header fields — same offsets as eden::parse_world.
    let seed       = i32::from_le_bytes(hdr[0..4].try_into().unwrap());
    let player_x   = f32::from_le_bytes(hdr[4..8].try_into().unwrap());
    let player_y   = f32::from_le_bytes(hdr[8..12].try_into().unwrap());
    let player_z   = f32::from_le_bytes(hdr[12..16].try_into().unwrap());
    let dir_off_u32 = u32::from_le_bytes(hdr[32..36].try_into().unwrap()) as u64;
    let dir_off_u64 = u64::from_le_bytes(hdr[32..40].try_into().unwrap());
    let version    = i32::from_le_bytes(hdr[90..94].try_into().unwrap());
    let name_bytes = &hdr[40..90];
    let name_end   = name_bytes.iter().position(|&b| b == 0).unwrap_or(50);
    let world_name = String::from_utf8_lossy(&name_bytes[..name_end]).into_owned();

    // Choose directory offset (same u32/u64 fallback as eden::parse_world).
    let directory_offset: usize =
        if total_size > 0 && dir_off_u64 < total_size && dir_off_u64 > 228 {
            dir_off_u64 as usize
        } else {
            dir_off_u32 as usize
        };

    // Skip from end of header to start of directory.  Uses a 64 KB stack
    // buffer so memory is O(1) regardless of how many GB we skip over.
    let to_skip = directory_offset.saturating_sub(228);
    skip_bytes(&mut entry, to_skip)
        .map_err(|e| format!("Failed to skip to column directory: {}", e))?;

    // Read ONLY the directory portion — cap at 4 MiB (enough for ~262 K columns).
    // Using read_to_end here would materialise the remainder of the decompressed
    // stream (up to 3+ GB for a world whose directory sits early in the file),
    // causing an out-of-memory panic in WASM.
    let dir_read_limit: u64 = if total_size > 0 && total_size > directory_offset as u64 {
        (total_size - directory_offset as u64).min(4_194_304) // ≤ 4 MiB
    } else {
        4_194_304 // unknown total size: cap defensively
    };
    let mut dir_buf = Vec::new();
    entry.by_ref().take(dir_read_limit)
        .read_to_end(&mut dir_buf)
        .map_err(|e| format!("Failed to read column directory: {}", e))?;

    // Parse directory entries: [i32 cx, i32 cz, u64 offset] × N.
    let mut col_map: std::collections::HashMap<(i32, i32), u64> =
        std::collections::HashMap::new();
    let mut pos = 0;
    while pos + 16 <= dir_buf.len() {
        let cx  = i32::from_le_bytes(dir_buf[pos..pos+4].try_into().unwrap());
        let cz  = i32::from_le_bytes(dir_buf[pos+4..pos+8].try_into().unwrap());
        let off = u64::from_le_bytes(dir_buf[pos+8..pos+16].try_into().unwrap());
        // Accept any column whose data fits before the directory.
        // (Column data always precedes the directory in valid Eden files.)
        if off > 0 && (off as usize) < directory_offset {
            col_map.insert((cx, cz), off);
        }
        pos += 16;
    }

    if col_map.is_empty() {
        return Err("No valid columns found in ZIP Eden world directory".into());
    }

    // Detect chunks_per_column via min-gap heuristic (same as eden::parse_world).
    let chunks_per_column = {
        let mut offsets: Vec<u64> = col_map.values().copied().collect();
        if offsets.len() >= 2 {
            offsets.sort_unstable();
            let min_gap = offsets.windows(2)
                .filter_map(|w| w[1].checked_sub(w[0]).filter(|&g| g > 0))
                .min()
                .unwrap_or(131_072);
            if min_gap < 131_072 { 4 } else { 16 }
        } else {
            if version >= 5 { 16 } else { 4 }
        }
    };

    let col_size = chunks_per_column * 8192;
    // Upper bound for column data: either the directory start (columns always
    // precede the directory) or, if total_size is known, the full stream length.
    let data_upper = if total_size > 0 { total_size as usize } else { directory_offset };
    let columns: Vec<(i32, i32, usize)> = col_map.into_iter()
        .filter_map(|((cx, cz), off)| {
            let off = off as usize;
            if off + col_size <= data_upper { Some((cx, cz, off)) } else { None }
        })
        .collect();

    let player_chunk_x = (player_x / 16.0).floor() as i32;
    let player_chunk_z = (player_z / 16.0).floor() as i32;

    Ok(ZipWorldInfo { world_name, seed, player_y, player_chunk_x, player_chunk_z,
                      columns, chunks_per_column })
}

fn zip_stream_convert(
    zip_bytes: &[u8],
    info: &ZipWorldInfo,
    mapping: &block_map::BlockMapping,
) -> Result<anvil::AnvilArchive, String> {
    use std::io::Read;
    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("ZIP error on second pass: {}", e))?;
    let mut entry = archive.by_index(0)
        .map_err(|e| format!("ZIP entry error on second pass: {}", e))?;

    let mut anvil_archive = anvil::AnvilArchive::new();
    let col_size = info.chunks_per_column * 8192;

    // Sort by file offset so we only ever move forward through the stream.
    let mut sorted_cols = info.columns.clone();
    sorted_cols.sort_by_key(|&(_, _, off)| off);

    let mut current_pos: usize = 0;
    let mut col_buf = vec![0u8; col_size];

    for &(cx, cz, data_offset) in &sorted_cols {
        if data_offset > current_pos {
            skip_bytes(&mut entry, data_offset - current_pos)
                .map_err(|e| format!("Skip error before column ({}, {}): {}", cx, cz, e))?;
            current_pos = data_offset;
        }

        entry.read_exact(&mut col_buf)
            .map_err(|e| format!("Read error at column ({}, {}): {}", cx, cz, e))?;
        current_pos += col_size;

        let out_cx = cx - info.player_chunk_x;
        let out_cz = cz - info.player_chunk_z;
        let mut sections: Vec<(u8, Vec<u8>, Vec<u8>)> = Vec::new();

        for cy in 0..info.chunks_per_column {
            let mut blks = vec![0u8; 4096];
            let mut data = vec![0u8; 2048];
            let mut has_blocks = false;

            for ex in 0..16usize {
                for ez in 0..16usize {
                    for ey in 0..16usize {
                        let local_idx = eden::eden_voxel_idx(ex, ez, ey);
                        let block_type = col_buf[cy * 8192 + local_idx];
                        let paint_byte = col_buf[cy * 8192 + 4096 + local_idx];
                        let mc = block_map::resolve(mapping, block_type, paint_byte);
                        if mc.id == 0 { continue; }
                        has_blocks = true;
                        let anvil_idx = eden::anvil_voxel_idx(ex, ez, ey);
                        blks[anvil_idx] = mc.id;
                        let nibble_idx = anvil_idx / 2;
                        if anvil_idx % 2 == 0 {
                            data[nibble_idx] = (data[nibble_idx] & 0xF0) | (mc.meta & 0x0F);
                        } else {
                            data[nibble_idx] = (data[nibble_idx] & 0x0F) | ((mc.meta & 0x0F) << 4);
                        }
                    }
                }
            }

            if has_blocks { sections.push((cy as u8, blks, data)); }
        }

        anvil_archive.write_chunk(out_cx, out_cz, sections);
    }

    Ok(anvil_archive)
}

fn convert_zip(zip_bytes: &[u8], mapping: block_map::BlockMapping) -> Result<Vec<u8>, JsValue> {
    let info = zip_read_world_info(zip_bytes)
        .map_err(|e| JsValue::from_str(&e))?;

    let archive = zip_stream_convert(zip_bytes, &info, &mapping)
        .map_err(|e| JsValue::from_str(&e))?;

    let level_dat = level_dat::build_level_dat(
        &info.world_name, info.seed, 0, info.player_y as i32, 0,
    );

    let mut zip_buf = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut zip_buf);
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("level.dat", options)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        zip.write_all(&level_dat)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        for (name, bytes) in archive.into_files() {
            let _ = zip.add_directory("region/", options);
            zip.start_file(&name, options)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            zip.write_all(&bytes)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        }
        zip.finish().map_err(|e| JsValue::from_str(&e.to_string()))?;
    }

    Ok(zip_buf.into_inner())
}

/// Read and discard `n` bytes from `reader` using a fixed 64 KB stack buffer.
/// Memory usage is O(1) regardless of how many bytes are skipped.
fn skip_bytes<R: std::io::Read>(reader: &mut R, mut n: usize) -> std::io::Result<()> {
    let mut buf = [0u8; 65536];
    while n > 0 {
        let to_read = n.min(buf.len());
        let read = reader.read(&mut buf[..to_read])?;
        if read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof, "truncated stream while skipping"));
        }
        n -= read;
    }
    Ok(())
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let combined = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((combined >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((combined >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 { CHARS[((combined >> 6) & 0x3F) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { CHARS[(combined & 0x3F) as usize] as char } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::{TerrainParams, generate};
    use crate::eden_writer::write_eden_file;

    #[test]
    fn test_generate_and_write() {
        let params = TerrainParams {
            width: 32,
            depth: 32,
            seed: 42,
            base_height: 20,
            water_amnt: 5, // no water
        };
        let world = generate(&params).expect("generate failed");
        println!("Spawn: ({}, {}, {})", world.meta.spawn_x, world.meta.spawn_y, world.meta.spawn_z);
        println!("Height range: {}..{}", world.meta.min_height, world.meta.max_height);

        let bytes = write_eden_file(&world, &params);
        println!("File size: {} bytes", bytes.len());

        // Parse the file back
        let parsed = crate::eden::parse_world(&bytes).expect("parse failed");
        println!("Player pos: ({}, {}, {})", parsed.header.player_x, parsed.header.player_y, parsed.header.player_z);
        println!("Version: {}", parsed.header.version);
        println!("Columns: {}", parsed.columns.len());
        println!("Dir offset: {}", parsed.header.directory_offset);

        // Check center column blocks
        let cx_center = world.meta.spawn_x / 16;
        let cz_center = world.meta.spawn_z / 16;
        println!("Center column: ({}, {})", cx_center, cz_center);
        
        let col = parsed.columns.iter().find(|c| c.cx == cx_center && c.cz == cz_center);
        if let Some(col) = col {
            println!("Found center column!");
            use crate::eden::eden_voxel_idx;
            for y in 0..32usize {
                let cy = y / 16;
                let ly = y % 16;
                let bt = col.get_block(&bytes, cy, eden_voxel_idx(8, 8, ly));
                if bt != 0 {
                    println!("  y={}: block_type={}", y, bt);
                }
            }
        } else {
            println!("Center column NOT FOUND!");
        }

        // Verify version at offset 90
        use std::io::Read;
        let v90 = i32::from_le_bytes(bytes[90..94].try_into().unwrap());
        let v92 = i32::from_le_bytes(bytes[92..96].try_into().unwrap());
        println!("Version at offset 90: {} (should be 4)", v90);
        println!("Version at offset 92: {} (should be 0)", v92);
    }
}
