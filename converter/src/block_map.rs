use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A Minecraft 1.12 block (numeric ID + metadata).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct McBlock {
    pub id: u8,
    pub meta: u8,
}

impl McBlock {
    pub const fn new(id: u8, meta: u8) -> Self { Self { id, meta } }
    pub const AIR: Self = Self::new(0, 0);
}

/// Per-block mapping entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockEntry {
    pub unpainted: McBlock,
    /// Which colored family to use when paint_byte > 0.
    /// "concrete", "wool", "stained_glass", "terracotta", or "none".
    #[serde(default = "default_painted_family")]
    pub painted_family: String,
    /// Per-paint-byte overrides (paint byte 1–54 → specific McBlock).
    /// Takes priority over painted_family for the matching paint byte.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub paint_colors: HashMap<u8, McBlock>,
}

fn default_painted_family() -> String { "none".into() }

/// Top-level mapping config (serialisable for the web UI).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockMapping {
    pub blocks: HashMap<u8, BlockEntry>,
}

/// Minecraft dye color index (0-15) matching MC wool/concrete meta values.
const MC_COLORS: [u8; 55] = [
    0,  // 0  Unpainted → white (unused; unpainted blocks go through `unpainted` field)
    14, // 1  LightRed → red
    1,  // 2  LightOrange → orange
    4,  // 3  LightYellow → yellow
    5,  // 4  LightGreen → lime
    9,  // 5  LightCyan → cyan
    3,  // 6  LightBlue → light_blue
    10, // 7  LightPurple → purple
    6,  // 8  LightPink → pink
    0,  // 9  White → white
    14, // 10 MediumLightRed → red
    1,  // 11 MediumLightOrange → orange
    4,  // 12 MediumLightYellow → yellow
    13, // 13 MediumLightGreen → green
    9,  // 14 MediumLightCyan → cyan
    11, // 15 MediumLightBlue → blue
    10, // 16 MediumLightPurple → purple
    6,  // 17 MediumLightPink → pink
    8,  // 18 MediumLightGray → light_gray
    14, // 19 Red → red
    1,  // 20 Orange → orange
    4,  // 21 Yellow → yellow
    13, // 22 Green → green
    9,  // 23 Cyan → cyan
    11, // 24 Blue → blue
    10, // 25 Purple → purple
    6,  // 26 Pink → pink
    7,  // 27 Gray → gray
    14, // 28 MediumDarkRed → red
    1,  // 29 MediumDarkOrange → orange
    4,  // 30 MediumDarkYellow → yellow
    13, // 31 MediumDarkGreen → green
    9,  // 32 MediumDarkCyan → cyan
    11, // 33 MediumDarkBlue → blue
    10, // 34 MediumDarkPurple → purple
    6,  // 35 MediumDarkPink → pink
    7,  // 36 MediumDarkGray → gray
    14, // 37 DarkRed → red
    12, // 38 DarkOrange → brown
    4,  // 39 DarkYellow → yellow
    13, // 40 DarkGreen → green
    9,  // 41 DarkCyan → cyan
    11, // 42 DarkBlue → blue
    10, // 43 DarkPurple → purple
    6,  // 44 DarkPink → pink
    7,  // 45 DarkGray → gray
    14, // 46 VeryDarkRed → red
    12, // 47 VeryDarkOrange → brown
    4,  // 48 VeryDarkYellow → yellow
    13, // 49 VeryDarkGreen → green
    9,  // 50 VeryDarkCyan → cyan
    11, // 51 VeryDarkBlue → blue
    10, // 52 VeryDarkPurple → purple
    6,  // 53 VeryDarkPink → pink
    15, // 54 Black → black
];

/// Resolve paint_byte (0 = unpainted, 1-54 = paint index) to MC dye color (0-15).
fn paint_to_mc_color(paint_byte: u8) -> u8 {
    if paint_byte == 0 || paint_byte as usize >= MC_COLORS.len() { 0 }
    else { MC_COLORS[paint_byte as usize] }
}

/// Resolve a colored block family + MC color index to a McBlock.
fn colored_block(family: &str, mc_color: u8) -> Option<McBlock> {
    let meta = mc_color & 0x0F;
    match family {
        "concrete"      => Some(McBlock::new(251, meta)), // 1.12 concrete
        "wool"          => Some(McBlock::new(35, meta)),
        "stained_glass" => Some(McBlock::new(95, meta)),
        "terracotta"    => Some(McBlock::new(159, meta)), // stained hardened clay
        _ => None,
    }
}

pub fn resolve(mapping: &BlockMapping, block_type: u8, paint_byte: u8) -> McBlock {
    if block_type == 0 { return McBlock::AIR; }
    let entry = match mapping.blocks.get(&block_type) {
        Some(e) => e,
        None => return McBlock::new(1, 0), // stone fallback
    };
    if paint_byte > 0 {
        // Per-color override takes priority over the family mapping.
        if let Some(mc) = entry.paint_colors.get(&paint_byte) {
            return *mc;
        }
        if entry.painted_family != "none" {
            let mc_color = paint_to_mc_color(paint_byte);
            if let Some(b) = colored_block(&entry.painted_family, mc_color) {
                return b;
            }
        }
    }
    entry.unpainted
}

pub fn default_mapping() -> BlockMapping {
    let mut blocks: HashMap<u8, BlockEntry> = HashMap::new();

    let mut add = |id: u8, mc_id: u8, mc_meta: u8, family: &str| {
        blocks.insert(id, BlockEntry {
            unpainted: McBlock::new(mc_id, mc_meta),
            painted_family: family.into(),
            paint_colors: HashMap::new(),
        });
    };

    // Core blocks
    add(1,  7,   0, "none");          // Bedrock
    add(2,  1,   0, "concrete");      // Stone
    add(3,  3,   0, "terracotta");    // Dirt
    add(4,  12,  0, "none");          // Sand
    add(5,  18,  0, "none");          // Leaves
    add(6,  17,  0, "none");          // Tree trunk
    add(7,  5,   0, "wool");          // Wood planks
    add(8,  2,   0, "none");          // Grass
    add(9,  46,  0, "none");          // TNT
    add(10, 4,   0, "concrete");      // Dark Stone → Cobblestone
    add(11, 2,   0, "none");          // Grass2
    add(12, 2,   0, "none");          // Grass3
    add(13, 45,  0, "concrete");      // Brick
    add(14, 98,  0, "concrete");      // Cobblestone → Stone Bricks
    add(15, 79,  0, "none");          // Ice → Ice block
    add(16, 155, 1, "stained_glass"); // Crystal → Chiseled Quartz (painted → stained glass)
    add(17, 171, 0, "wool");          // Trampoline → Carpet (painted → wool)
    add(18, 5,   3, "none");          // Ladder → Jungle Planks (no MC ladder color)
    add(19, 35,  0, "wool");          // Cloud → White Wool (painted → colored wool)
    add(20, 9,   0, "none");          // Water
    add(21, 85,  0, "none");          // Weave → Oak Fence
    add(22, 48,  0, "none");          // Vine → Mossy Cobblestone
    add(23, 11,  0, "none");          // Lava
    // Ramps → Stairs. Eden ramp direction = high-edge direction (S/W/N/E).
    // MC stair meta: 0=ascending east, 1=west, 2=south, 3=north (high end matches direction).
    // Pattern: S→2, W→1, N→3, E→0
    // Stone ramps (24-27) → Cobblestone Stairs
    add(24, 67, 2, "none"); // Stone Ramp S
    add(25, 67, 1, "none"); // Stone Ramp W
    add(26, 67, 3, "none"); // Stone Ramp N
    add(27, 67, 0, "none"); // Stone Ramp E
    // Wood ramps (28-31) → Oak Stairs
    add(28, 53, 2, "none"); // Wood Ramp S
    add(29, 53, 1, "none"); // Wood Ramp W
    add(30, 53, 3, "none"); // Wood Ramp N
    add(31, 53, 0, "none"); // Wood Ramp E
    // Shingle ramps (32-35) → Nether Brick Stairs
    add(32, 114, 2, "none"); // Shingle Ramp S
    add(33, 114, 1, "none"); // Shingle Ramp W
    add(34, 114, 3, "none"); // Shingle Ramp N
    add(35, 114, 0, "none"); // Shingle Ramp E
    // Ice ramps (36-39) → Quartz Stairs
    add(36, 156, 2, "none"); // Ice Ramp S
    add(37, 156, 1, "none"); // Ice Ramp W
    add(38, 156, 3, "none"); // Ice Ramp N
    add(39, 156, 0, "none"); // Ice Ramp E
    // Stone sides (40-43) → Cobblestone Wall
    for i in 40u8..=43 { add(i, 139, 0, "none"); }
    // Wood sides (44-47) → Oak Fence
    for i in 44u8..=47 { add(i, 85, 0, "none"); }
    // Shingle sides (48-51) → Cobblestone Wall
    for i in 48u8..=51 { add(i, 139, 0, "none"); }
    // Ice sides (52-55) → Cobblestone Wall
    for i in 52u8..=55 { add(i, 139, 0, "none"); }
    add(56, 112, 0, "none");          // Shingle → Nether Brick
    add(57, 251, 0, "concrete");      // Gradient/NeonSquare → White Concrete (painted → concrete)
    add(58, 20,  0, "stained_glass"); // Glass (painted → stained glass)
    add(59, 9,   0, "none");          // Water3
    add(60, 9,   0, "none");          // Water2
    add(61, 9,   0, "none");          // Water1
    add(62, 89,  0, "none");          // Lava3 → Glowstone (decorative)
    add(63, 89,  0, "none");          // Lava2
    add(64, 11,  0, "none");          // Lava1 → actual lava
    add(65, 38,  0, "none");          // Firework → Poppy
    // Doors (66-70) → Oak Door
    for i in 66u8..=70 { add(i, 64, 0, "none"); }
    add(71, 41,  0, "none");          // Golden Cube → Gold Block
    add(72, 89,  0, "none");          // Lightbox → Glowstone
    add(73, 38,  0, "none");          // Flower → Poppy
    add(74, 42,  0, "none");          // Steel → Iron Block
    // Portals (75-79) → End Portal Frame (decorative approximation)
    for i in 75u8..=79 { add(i, 120, 0, "none"); }
    add(80, 1,   0, "concrete");      // Custom
    add(81, 46,  0, "none");          // Block TNT

    // Expansion pack blocks (82-111) mirror their base types
    add(82,  2,   0, "none");         // BT Grass
    add(83,  4,   0, "concrete");     // BT Dark Stone
    add(84,  1,   0, "concrete");     // BT Stone
    add(85,  3,   0, "terracotta");   // BT Dirt
    add(86,  12,  0, "none");         // BT Sand
    add(87,  46,  0, "none");         // BT TNT
    add(88,  5,   0, "wool");         // BT Wood
    add(89,  112, 0, "none");         // BT Shingle
    add(90,  20,  0, "stained_glass");// BT Glass
    add(91,  251, 0, "concrete");     // BT Gradient
    add(92,  17,  0, "none");         // BT Tree
    add(93,  18,  0, "none");         // BT Leaves
    add(94,  45,  0, "concrete");     // BT Brick
    add(95,  98,  0, "concrete");     // BT Cobblestone
    add(96,  106, 0, "none");         // BT Vines
    add(97,  5,   3, "none");         // BT Ladder
    add(98,  79,  0, "none");         // BT Ice
    add(99,  155, 1, "stained_glass");// BT Crystal
    add(100, 171, 0, "wool");         // BT Trampoline
    add(101, 35,  0, "wool");         // BT Cloud
    add(102, 139, 0, "none");         // BT Stone Side
    add(103, 85,  0, "none");         // BT Wood Side
    add(104, 139, 0, "none");         // BT Ice Side
    add(105, 139, 0, "none");         // BT Shingle Side
    add(106, 85,  0, "none");         // BT Fence
    add(107, 9,   0, "none");         // BT Water
    add(108, 11,  0, "none");         // BT Lava
    add(109, 38,  0, "none");         // BT Firework
    add(110, 89,  0, "none");         // BT Lightbox
    add(111, 42,  0, "none");         // BT Steel

    BlockMapping { blocks }
}

pub fn mapping_from_json(json: &str) -> Result<BlockMapping, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}
