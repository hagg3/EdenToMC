use crate::nbt::{NbtBuf, zlib_compress};

/// Generate a minimal level.dat for Minecraft 1.12.2 (DataVersion 1343).
pub fn build_level_dat(world_name: &str, seed: i32, spawn_x: i32, spawn_y: i32, spawn_z: i32) -> Vec<u8> {
    let mut buf = NbtBuf::new();
    buf.begin_compound(""); // root unnamed compound
    buf.begin_compound("Data");

    buf.int("DataVersion", 1343); // 1.12.2
    buf.int("version", 19133);    // Anvil
    buf.byte("initialized", 1);
    buf.string("LevelName", world_name);
    buf.string("generatorName", "default");
    buf.string("generatorOptions", "");
    buf.int("generatorVersion", 1);
    buf.long("RandomSeed", seed as i64);
    buf.byte("MapFeatures", 0);
    buf.long("LastPlayed", 0);
    buf.long("SizeOnDisk", 0);
    buf.byte("allowCommands", 1);
    buf.byte("hardcore", 0);
    buf.int("GameType", 1); // Creative
    buf.byte("Difficulty", 1);
    buf.byte("DifficultyLocked", 0);
    buf.long("Time", 6000); // midday
    buf.long("DayTime", 6000);
    buf.int("SpawnX", spawn_x);
    buf.int("SpawnY", spawn_y);
    buf.int("SpawnZ", spawn_z);
    buf.byte("raining", 0);
    buf.int("rainTime", 100000);
    buf.byte("thundering", 0);
    buf.int("thunderTime", 100000);
    // GameRules compound
    buf.begin_compound("GameRules");
    buf.string("doMobSpawning", "false");
    buf.string("keepInventory", "true");
    buf.end_compound();
    // Version compound
    buf.begin_compound("Version");
    buf.int("Id", 1343);
    buf.string("Name", "1.12.2");
    buf.byte("Snapshot", 0);
    buf.end_compound();

    buf.end_compound(); // Data
    buf.end_compound(); // root

    // level.dat uses gzip (not zlib), but flate2's GzEncoder works the same way.
    // Using zlib here; Minecraft also accepts gzip. We use gzip for compatibility.
    use flate2::{write::GzEncoder, Compression};
    use std::io::Write;
    let mut gz = GzEncoder::new(Vec::new(), Compression::best());
    gz.write_all(&buf.0).unwrap();
    gz.finish().unwrap()
}
