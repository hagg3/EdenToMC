use crate::noise::Noise2D;
use serde::Deserialize;

// Eden block types from Constants.h
pub const TYPE_NONE: u8 = 0;
pub const TYPE_STONE: u8 = 2;
pub const TYPE_DIRT: u8 = 3;
pub const TYPE_SAND: u8 = 4;
pub const TYPE_LEAVES: u8 = 5;
pub const TYPE_TREE: u8 = 6;
pub const TYPE_GRASS: u8 = 8;
pub const TYPE_WATER: u8 = 20;
pub const TYPE_FLOWER: u8 = 73;

#[derive(Deserialize, Clone)]
pub struct TerrainParams {
    pub width: u32,
    pub depth: u32,
    pub seed: u32,
    #[serde(default = "default_base_height")]
    pub base_height: i32,
    #[serde(default = "default_water_amnt")]
    pub water_amnt: u32,
}

fn default_base_height() -> i32 { 36 }
fn default_water_amnt() -> u32 { 3 }

pub struct TerrainMeta {
    pub spawn_x: i32,
    pub spawn_y: i32,
    pub spawn_z: i32,
    pub trees_placed: u32,
    pub flowers_placed: u32,
    pub caves_carved: u32,
    pub min_height: i32,
    pub max_height: i32,
    pub cols_x: usize,
    pub cols_z: usize,
}

#[derive(Clone, Default)]
pub struct TerrainBlock {
    pub block_type: u8,
    pub color: u8,
}

pub struct TerrainWorld {
    pub width: usize,
    pub depth: usize,
    /// Flat: blocks[wx * depth * 64 + wz * 64 + wy]
    pub blocks: Vec<TerrainBlock>,
    pub meta: TerrainMeta,
}

pub fn world_idx(wx: usize, wy: usize, wz: usize, depth: usize) -> usize {
    wx * depth * 64 + wz * 64 + wy
}

fn clamp(v: i32, lo: i32, hi: i32) -> i32 {
    if v < lo { lo } else if v > hi { hi } else { v }
}

fn hash3(seed: u32, x: i32, y: i32, z: i32) -> u32 {
    let mut h = seed ^ 0x9e3779b9u32;
    h ^= (x as u32).wrapping_mul(374761393);
    h ^= (y as u32).wrapping_mul(668265263);
    h ^= (z as u32).wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^= h >> 16;
    h
}

fn get_top_solid_y(blocks: &[TerrainBlock], wx: i32, wz: i32, width: usize, depth: usize) -> i32 {
    if wx < 0 || wz < 0 || wx >= width as i32 || wz >= depth as i32 {
        return 0;
    }
    for y in (0..64i32).rev() {
        let t = blocks[world_idx(wx as usize, y as usize, wz as usize, depth)].block_type;
        if t != TYPE_NONE && t != TYPE_WATER && t != TYPE_FLOWER && t != TYPE_LEAVES {
            return y;
        }
    }
    0
}

fn should_place_leaf(seed: u32, x: i32, y: i32, z: i32, normalized_dist: f64) -> bool {
    if normalized_dist <= 0.0 { return true; }
    if normalized_dist >= 1.0 { return false; }
    let base_keep = 1.0 - normalized_dist * normalized_dist;
    let h = hash3(seed, x, y, z);
    let jitter = (h & 1023) as f64 / 1023.0;
    jitter < base_keep
}

pub fn generate(params: &TerrainParams) -> Result<TerrainWorld, String> {
    let width = params.width as usize;
    let depth = params.depth as usize;
    if width == 0 || depth == 0 || width % 16 != 0 || depth % 16 != 0 {
        return Err("width and depth must be positive multiples of 16".into());
    }
    if width > 1024 || depth > 1024 {
        return Err("maximum world size is 1024x1024 blocks".into());
    }

    let water_amnt = clamp(params.water_amnt as i32, 1, 5);
    let water_level: i32 = match water_amnt {
        1 => 40,
        2 => 35,
        3 => 32,
        4 => 27,
        _ => -1,
    };
    let snow_height = 48i32;
    let cols_x = width / 16;
    let cols_z = depth / 16;
    let noise = Noise2D::new(params.seed);

    let total_blocks = width * 64 * depth;
    let mut blocks: Vec<TerrainBlock> = vec![TerrainBlock::default(); total_blocks];
    let mut heightmap = vec![0i32; width * depth];
    let mut water_mask = vec![false; width * depth];

    let mut min_h = i32::MAX;
    let mut max_h = i32::MIN;

    // Stage 1+2: heightmap + base terrain
    for wx in 0..width {
        for wz in 0..depth {
            let n = noise.fractal(wx as f64, wz as f64, 4, 0.02, 0.5);
            let h = clamp((params.base_height as f64 + n * 18.0) as i32, 1, 62);
            heightmap[wx * depth + wz] = h;
            min_h = min_h.min(h);
            max_h = max_h.max(h);

            let stone_top = (h - 3).max(0);
            let dirt_top = (h - 1).max(0);
            for y in 0..=stone_top {
                blocks[world_idx(wx, y as usize, wz, depth)].block_type = TYPE_STONE;
            }
            for y in (stone_top + 1)..=dirt_top {
                blocks[world_idx(wx, y as usize, wz, depth)].block_type = TYPE_DIRT;
            }
            blocks[world_idx(wx, h as usize, wz, depth)].block_type = TYPE_GRASS;
        }
    }

    // Stage 3: caves - threshold 0.62 (less aggressive than original 0.52)
    let mut caves_carved = 0u32;
    for wx in 0..width {
        for wz in 0..depth {
            let h = heightmap[wx * depth + wz];
            // Leave at least 3 blocks of solid crust; floor at y=1 (never carve y=0)
            let cave_top = (h - 4).max(1);
            for y in 1..=cave_top {
                let c = noise.fractal3d(wx as f64, y as f64, wz as f64, 3, 0.08, 0.5);
                if c > 0.62 {
                    let idx = world_idx(wx, y as usize, wz, depth);
                    blocks[idx].block_type = TYPE_NONE;
                    blocks[idx].color = 0;
                    caves_carved += 1;
                }
            }
        }
    }

    // Stage 4: water
    let mut water_blocks = 0u32;
    for wx in 0..width {
        for wz in 0..depth {
            if water_level < 0 { continue; }
            let h = heightmap[wx * depth + wz];
            let pond_n = noise.fractal(wx as f64 + 1000.0, wz as f64 + 1000.0, 3, 0.03, 0.5);
            let river_n = noise.fractal(wx as f64 + 4000.0, wz as f64 - 4000.0, 2, 0.01, 0.6);
            let pond = pond_n > 0.62 && h < water_level + 3;
            let river = river_n > -0.03 && river_n < 0.03 && h < 45;
            let local_wl = water_level + if pond { 1 } else { 0 };

            if h < water_level || pond || river {
                blocks[world_idx(wx, h as usize, wz, depth)].block_type = TYPE_SAND;
                for y in (h + 1)..=local_wl.min(62) {
                    let idx = world_idx(wx, y as usize, wz, depth);
                    if blocks[idx].block_type != TYPE_WATER {
                        blocks[idx].block_type = TYPE_WATER;
                        water_blocks += 1;
                    }
                }
                water_mask[wx * depth + wz] = true;
            }
        }
    }
    let _ = water_blocks;

    // Stage 5: beaches near water edges
    for wx in 1..(width - 1) {
        for wz in 1..(depth - 1) {
            let top_y = get_top_solid_y(&blocks, wx as i32, wz as i32, width, depth);
            let idx = world_idx(wx, top_y as usize, wz, depth);
            if blocks[idx].block_type != TYPE_GRASS { continue; }
            let mut water_neighbors = 0i32;
            for dz in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dz == 0 { continue; }
                    let nx = wx as i32 + dx;
                    let nz = wz as i32 + dz;
                    if nx >= 0 && nz >= 0 && nx < width as i32 && nz < depth as i32
                        && water_mask[nx as usize * depth + nz as usize]
                    {
                        water_neighbors += 1;
                    }
                }
            }
            if water_neighbors >= 3 && water_level >= 0 && top_y <= water_level + 2 {
                blocks[idx].block_type = TYPE_SAND;
            }
        }
    }

    // Stage 6: vegetation - keep 4-block margin so tree canopy stays in-bounds
    let edge = 4usize;
    let mut trees_placed = 0u32;
    let mut flowers_placed = 0u32;
    for wx in edge..(width.saturating_sub(edge)) {
        for wz in edge..(depth.saturating_sub(edge)) {
            let top_y = get_top_solid_y(&blocks, wx as i32, wz as i32, width, depth);
            let idx = world_idx(wx, top_y as usize, wz, depth);
            if blocks[idx].block_type != TYPE_GRASS { continue; }
            if water_level >= 0 && top_y <= water_level { continue; }
            if top_y + 1 >= 63 { continue; }
            if blocks[world_idx(wx, (top_y + 1) as usize, wz, depth)].block_type != TYPE_NONE {
                continue;
            }

            let h = hash3(params.seed, wx as i32, top_y, wz as i32);
            if h % 1000 < 6 {
                // Minimum trunk height 4 (less stumpy than original minimum of 3)
                let trunk = 4 + (h % 4) as i32;
                for i in 1..=trunk {
                    let ty = top_y + i;
                    if ty >= 63 { break; }
                    let tidx = world_idx(wx, ty as usize, wz, depth);
                    blocks[tidx].block_type = TYPE_TREE;
                }
                let leaf_top = (top_y + trunk).min(62);
                let leaf_pattern = h % 3;
                for dy in 1i32..=5 {
                    for dx in -4i32..=4 {
                        for dz in -4i32..=4 {
                            let ax = wx as i32 + dx;
                            let az = wz as i32 + dz;
                            let ay = leaf_top + dy;
                            if ax < 0 || az < 0 || ax >= width as i32 || az >= depth as i32
                                || ay < 1 || ay >= 63 {
                                continue;
                            }
                            let (inside, norm) = match leaf_pattern {
                                0 => {
                                    let ex = dx as f64 / 3.2;
                                    let ey = (dy - 2) as f64 / 2.2;
                                    let ez = dz as f64 / 3.2;
                                    let d = ex * ex + ey * ey + ez * ez;
                                    (d <= 1.0, d.min(1.0).sqrt())
                                }
                                1 => {
                                    let radial = dx * dx + dz * dz;
                                    let inside = radial <= 12 && dy >= 1 && dy <= 3;
                                    let plane = (radial as f64).sqrt() / 3.6;
                                    let vert = (dy - 2).unsigned_abs() as f64 / 1.8;
                                    (inside, ((plane + vert) * 0.65).min(1.0))
                                }
                                _ => {
                                    let radial = dx * dx + dz * dz;
                                    let max_r: i32 = match dy {
                                        5 | 4 => if dy == 5 { 0 } else { 1 },
                                        3 => 2,
                                        2 => 3,
                                        _ => 4,
                                    };
                                    let inside = radial <= max_r * max_r;
                                    let norm = if max_r <= 0 {
                                        0.0
                                    } else {
                                        ((radial as f64).sqrt() / max_r as f64).min(1.0)
                                    };
                                    (inside, norm)
                                }
                            };
                            if !inside { continue; }
                            if !should_place_leaf(params.seed + 97, ax, ay, az, norm) { continue; }
                            let lidx = world_idx(ax as usize, ay as usize, az as usize, depth);
                            if blocks[lidx].block_type == TYPE_NONE {
                                blocks[lidx].block_type = TYPE_LEAVES;
                            }
                        }
                    }
                }
                trees_placed += 1;
            } else if h % 1000 < 12 {
                let fidx = world_idx(wx, (top_y + 1) as usize, wz, depth);
                blocks[fidx].block_type = TYPE_FLOWER;
                flowers_placed += 1;
            }
        }
    }

    // Stage 7: snow at high altitudes (white-tinted sand block, color index 1)
    for wx in 0..width {
        for wz in 0..depth {
            let top_y = get_top_solid_y(&blocks, wx as i32, wz as i32, width, depth);
            if top_y < snow_height { continue; }
            let idx = world_idx(wx, top_y as usize, wz, depth);
            let t = blocks[idx].block_type;
            if t == TYPE_GRASS || t == TYPE_DIRT || t == TYPE_STONE || t == TYPE_SAND {
                blocks[idx].block_type = TYPE_SAND;
                blocks[idx].color = 1;
            }
        }
    }

    let spawn_x = (width / 2) as i32;
    let spawn_z = (depth / 2) as i32;
    let spawn_y = clamp(
        get_top_solid_y(&blocks, spawn_x, spawn_z, width, depth) + 1,
        1, 62,
    );

    let meta = TerrainMeta {
        spawn_x, spawn_y, spawn_z,
        trees_placed, flowers_placed, caves_carved,
        min_height: min_h, max_height: max_h,
        cols_x, cols_z,
    };

    Ok(TerrainWorld { width, depth, blocks, meta })
}
