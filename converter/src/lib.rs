mod nbt;
mod eden;
mod block_map;
mod anvil;
mod level_dat;

use wasm_bindgen::prelude::*;
use std::io::Write;

#[wasm_bindgen]
pub fn convert(eden_bytes: &[u8], mapping_json: Option<String>) -> Result<Vec<u8>, JsValue> {
    let mapping = if let Some(json) = mapping_json {
        block_map::mapping_from_json(&json).map_err(|e| JsValue::from_str(&e))?
    } else {
        block_map::default_mapping()
    };

    let world = eden::parse_world(eden_bytes)
        .map_err(|e| JsValue::from_str(&e))?;

    let mut archive = anvil::AnvilArchive::new();

    for col in &world.columns {
        let out_cx = col.cx - world.player_chunk_x;
        let out_cz = col.cz - world.player_chunk_z;

        let mut sections: Vec<(u8, Vec<u8>, Vec<u8>)> = Vec::new();

        for cy in 0..4usize {
            let mut blks = vec![0u8; 4096];
            let mut data = vec![0u8; 2048];
            let mut has_blocks = false;

            for ex in 0..16usize {
                for ez in 0..16usize {
                    for ey in 0..16usize {
                        let eden_idx = eden::eden_voxel_idx(ex, ez, ey) + cy * 4096;
                        let block_type = col.blocks[eden_idx];
                        let paint_byte = col.paints[eden_idx];
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
