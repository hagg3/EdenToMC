use crate::terrain::{TerrainWorld, TerrainParams, world_idx};

// Eden voxel index within a 16x16x16 sub-chunk: x * 256 + z * 16 + y
fn eden_voxel_idx(lx: usize, lz: usize, ly: usize) -> usize {
    lx * 256 + lz * 16 + ly
}

fn write_i32_le(buf: &mut [u8], off: usize, v: i32) {
    buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
}
fn write_u64_le(buf: &mut [u8], off: usize, v: u64) {
    buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
}
fn write_f32_le(buf: &mut [u8], off: usize, v: f32) {
    buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// Serialize a generated terrain world to Eden .eden file bytes.
/// Header layout (192 bytes, matching WorldFileHeader in EdenFileLoader.h):
///   0:  i32 level_seed
///   4:  f32 pos.x, pos.y, pos.z   (12)
///  16:  f32 home.x, home.y, home.z (12)
///  28:  f32 yaw
///  32:  u64 directory_offset
///  40:  char name[50]
///  90:  i32 version  (no alignment padding - ARM unaligned access)
///  94:  char hash[36]
/// 130:  u8 skycolors[16]
/// 146:  i32 goldencubes
/// 150:  char reserved[40]
/// 190:  2 bytes struct trailing padding
/// = 192 bytes total
pub fn write_eden_file(world: &TerrainWorld, params: &TerrainParams) -> Vec<u8> {
    const HEADER_SIZE: usize = 192;
    const SUB_CHUNK_SIZE: usize = 8192; // 4096 blocks + 4096 colors
    const COL_DATA_SIZE: usize = 4 * SUB_CHUNK_SIZE;
    const DIR_ENTRY_SIZE: usize = 16; // i32 cx + i32 cz + u64 offset

    let num_cols = world.meta.cols_x * world.meta.cols_z;
    let dir_offset = HEADER_SIZE + num_cols * COL_DATA_SIZE;
    let total = dir_offset + num_cols * DIR_ENTRY_SIZE;

    let mut buf = vec![0u8; total];

    // --- Header ---
    let seed = params.seed as i32;
    let spawn_x = world.meta.spawn_x as f32;
    let spawn_y = world.meta.spawn_y as f32;
    let spawn_z = world.meta.spawn_z as f32;

    write_i32_le(&mut buf, 0, seed);
    write_f32_le(&mut buf, 4, spawn_x);
    write_f32_le(&mut buf, 8, spawn_y);
    write_f32_le(&mut buf, 12, spawn_z);
    write_f32_le(&mut buf, 16, spawn_x); // home = spawn
    write_f32_le(&mut buf, 20, spawn_y);
    write_f32_le(&mut buf, 24, spawn_z);
    // yaw at 28 stays 0.0
    write_u64_le(&mut buf, 32, dir_offset as u64);
    let name = b"TerrainGen";
    buf[40..40 + name.len()].copy_from_slice(name);
    write_i32_le(&mut buf, 90, 4); // FILE_VERSION 4

    // Sky color 6 = light blue sky → game renders grass as green.
    // Without this, all bytes are 0 → majority-vote returns 0 → pink/magenta grass.
    for i in 130..146 {
        buf[i] = 6;
    }

    // --- Column data ---
    let mut col_idx = 0usize;
    let mut dir_entries: Vec<(i32, i32, u64)> = Vec::with_capacity(num_cols);

    // Write in z-outer x-inner order (matching Eden column grid convention)
    for cz in 0..world.meta.cols_z {
        for cx in 0..world.meta.cols_x {
            let col_offset = HEADER_SIZE + col_idx * COL_DATA_SIZE;
            dir_entries.push((cx as i32, cz as i32, col_offset as u64));

            for cy in 0..4usize {
                let blocks_base = col_offset + cy * SUB_CHUNK_SIZE;
                let colors_base = blocks_base + 4096;

                for lx in 0..16usize {
                    for lz in 0..16usize {
                        for ly in 0..16usize {
                            let wx = cx * 16 + lx;
                            let wy = cy * 16 + ly;
                            let wz = cz * 16 + lz;
                            let w_idx = world_idx(wx, wy, wz, world.depth);
                            let e_idx = eden_voxel_idx(lx, lz, ly);
                            buf[blocks_base + e_idx] = world.blocks[w_idx].block_type;
                            buf[colors_base + e_idx] = world.blocks[w_idx].color;
                        }
                    }
                }
            }
            col_idx += 1;
        }
    }

    // --- Directory ---
    let mut dir_pos = dir_offset;
    for (cx, cz, offset) in dir_entries {
        write_i32_le(&mut buf, dir_pos, cx);
        write_i32_le(&mut buf, dir_pos + 4, cz);
        write_u64_le(&mut buf, dir_pos + 8, offset);
        dir_pos += DIR_ENTRY_SIZE;
    }

    buf
}
