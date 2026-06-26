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

    // Transparently handle ZIP-wrapped .eden files (magic bytes: 50 4B 03 04).
    // The decompressed buffer must outlive `world` and the conversion loop below.
    let _decompressed;
    let eden_bytes: &[u8] = {
        let cow = decompress_if_zip(eden_bytes).map_err(|e| JsValue::from_str(&e))?;
        _decompressed = cow;
        &*_decompressed
    };

    let world = eden::parse_world(eden_bytes)
        .map_err(|e| JsValue::from_str(&e))?;

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

/// Detect and decompress a ZIP-wrapped .eden file.
/// Checks for the PK magic (50 4B 03 04); if present, decompresses entry 0 and
/// returns it as an owned buffer. Otherwise returns a borrowed view of the input.
fn decompress_if_zip(bytes: &[u8]) -> Result<std::borrow::Cow<[u8]>, String> {
    if !bytes.starts_with(b"PK\x03\x04") {
        return Ok(std::borrow::Cow::Borrowed(bytes));
    }
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Invalid ZIP archive: {}", e))?;
    if archive.len() == 0 {
        return Err("ZIP archive contains no files".into());
    }
    let mut entry = archive.by_index(0)
        .map_err(|e| format!("Cannot read ZIP entry: {}", e))?;
    let mut buf = Vec::new();
    std::io::Read::read_to_end(&mut entry, &mut buf)
        .map_err(|e| format!("ZIP decompression failed: {}", e))?;
    Ok(std::borrow::Cow::Owned(buf))
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
