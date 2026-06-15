use crate::nbt::{NbtBuf, zlib_compress};
use std::collections::HashMap;

const SECTOR: usize = 4096;

struct RegionBuf {
    /// header: 8 KiB (2 × 4 KiB tables)
    header: [u8; 8192],
    /// chunk payload sectors (after header)
    body: Vec<u8>,
    /// next free sector index (0 and 1 are the header)
    next_sector: usize,
}

impl RegionBuf {
    fn new() -> Self {
        Self { header: [0u8; 8192], body: Vec::new(), next_sector: 2 }
    }

    fn add_chunk(&mut self, local_x: usize, local_z: usize, nbt_payload: &[u8]) {
        let compressed = zlib_compress(nbt_payload);
        // 4-byte length + 1-byte compression type + compressed data
        let length = 1u32 + compressed.len() as u32;
        let mut payload = Vec::with_capacity(5 + compressed.len());
        payload.extend_from_slice(&length.to_be_bytes());
        payload.push(2u8); // zlib
        payload.extend_from_slice(&compressed);

        let sectors_needed = (payload.len() + SECTOR - 1) / SECTOR;
        let offset_sector = self.next_sector;
        self.next_sector += sectors_needed;

        // Pad to full sector multiple
        let padded_len = sectors_needed * SECTOR;
        let start = self.body.len();
        self.body.extend_from_slice(&payload);
        self.body.resize(start + padded_len, 0);

        // Location entry (3 bytes offset, 1 byte count)
        let loc_idx = local_x + local_z * 32;
        let loc_val = ((offset_sector as u32) << 8) | (sectors_needed as u32 & 0xFF);
        self.header[loc_idx * 4..loc_idx * 4 + 4].copy_from_slice(&loc_val.to_be_bytes());
        // Timestamp (second 4 KiB table) — set to 1 so Minecraft doesn't treat it as unwritten
        let ts_idx = SECTOR + loc_idx * 4;
        self.header[ts_idx..ts_idx + 4].copy_from_slice(&1u32.to_be_bytes());
    }

    fn finalise(self) -> Vec<u8> {
        let mut out = Vec::with_capacity(8192 + self.body.len());
        out.extend_from_slice(&self.header);
        out.extend_from_slice(&self.body);
        out
    }
}

/// Build a chunk NBT payload (uncompressed).
fn build_chunk_nbt(chunk_x: i32, chunk_z: i32, sections: &[(u8, Vec<u8>, Vec<u8>)]) -> Vec<u8> {
    let mut buf = NbtBuf::new();
    buf.begin_compound(""); // root unnamed
    buf.begin_compound("Level");
    buf.int("xPos", chunk_x);
    buf.int("zPos", chunk_z);
    buf.long("LastUpdate", 0);
    buf.long("InhabitedTime", 0);
    buf.byte("TerrainPopulated", 1);
    buf.byte("LightPopulated", 1);
    buf.byte_array("Biomes", &[1u8; 256]); // plains everywhere

    // HeightMap: topmost non-air Y+1 per (x,z)
    let mut hm = [0i32; 256];
    for &(sec_y, ref blks, _) in sections {
        for z in 0..16usize {
            for x in 0..16usize {
                for y in (0..16usize).rev() {
                    let vi = (y * 16 + z) * 16 + x;
                    if blks[vi] != 0 {
                        let world_y = sec_y as i32 * 16 + y as i32 + 1;
                        let hm_idx = z * 16 + x;
                        if world_y > hm[hm_idx] { hm[hm_idx] = world_y; }
                        break;
                    }
                }
            }
        }
    }
    buf.int_array("HeightMap", &hm);
    buf.begin_list("Entities", 10, 0);
    buf.begin_list("TileEntities", 10, 0);

    buf.begin_list("Sections", 10, sections.len() as i32);
    for (sec_y, blks, data) in sections {
        buf.begin_list_compound_element();
        buf.byte("Y", *sec_y as i8);
        buf.byte_array("Blocks", blks);
        buf.byte_array("Data", data);
        buf.byte_array("SkyLight", &[0xFFu8; 2048]);
        buf.byte_array("BlockLight", &[0x00u8; 2048]);
        buf.end_list_compound_element();
    }

    buf.end_compound(); // Level
    buf.end_compound(); // root
    buf.0
}

/// Map chunk coords to region coords with proper floor-division.
fn chunk_to_region(c: i32) -> i32 {
    if c >= 0 { c / 32 } else { (c - 31) / 32 }
}
fn chunk_local(c: i32) -> usize {
    c.rem_euclid(32) as usize
}

pub struct AnvilArchive {
    /// region (rx, rz) → RegionBuf
    regions: HashMap<(i32, i32), RegionBuf>,
}

impl AnvilArchive {
    pub fn new() -> Self { Self { regions: HashMap::new() } }

    pub fn write_chunk(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        sections: Vec<(u8, Vec<u8>, Vec<u8>)>, // (section_y, blocks_4096, data_nibbles_2048)
    ) {
        if sections.is_empty() { return; }
        let rx = chunk_to_region(chunk_x);
        let rz = chunk_to_region(chunk_z);
        let lx = chunk_local(chunk_x);
        let lz = chunk_local(chunk_z);
        let nbt = build_chunk_nbt(chunk_x, chunk_z, &sections);
        self.regions
            .entry((rx, rz))
            .or_insert_with(RegionBuf::new)
            .add_chunk(lx, lz, &nbt);
    }

    /// Return all region files as (filename, bytes).
    pub fn into_files(self) -> Vec<(String, Vec<u8>)> {
        self.regions
            .into_iter()
            .map(|((rx, rz), rb)| {
                let name = format!("region/r.{}.{}.mca", rx, rz);
                (name, rb.finalise())
            })
            .collect()
    }
}
