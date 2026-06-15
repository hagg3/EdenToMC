export interface McBlock { id: number; meta: number; }

export type PaintedFamily = "none" | "concrete" | "wool" | "stained_glass" | "terracotta";

export interface BlockEntry {
  unpainted: McBlock;
  painted_family: PaintedFamily;
}

export interface BlockMapping {
  blocks: Record<string, BlockEntry>;
}

export const EDEN_BLOCK_NAMES: Record<number, string> = {
  0: "Air", 1: "Bedrock", 2: "Stone", 3: "Dirt", 4: "Sand",
  5: "Leaves", 6: "Tree Trunk", 7: "Wood Planks", 8: "Grass",
  9: "TNT", 10: "Dark Stone", 11: "Grass 2", 12: "Grass 3",
  13: "Brick", 14: "Cobblestone", 15: "Ice", 16: "Crystal",
  17: "Trampoline", 18: "Ladder", 19: "Cloud", 20: "Water",
  21: "Weave", 22: "Vine", 23: "Lava",
  24: "Stone Ramp S", 25: "Stone Ramp W", 26: "Stone Ramp N", 27: "Stone Ramp E",
  28: "Wood Ramp S", 29: "Wood Ramp W", 30: "Wood Ramp N", 31: "Wood Ramp E",
  32: "Shingle Ramp S", 33: "Shingle Ramp W", 34: "Shingle Ramp N", 35: "Shingle Ramp E",
  36: "Ice Ramp S", 37: "Ice Ramp W", 38: "Ice Ramp N", 39: "Ice Ramp E",
  40: "Stone Side S", 41: "Stone Side W", 42: "Stone Side N", 43: "Stone Side E",
  44: "Wood Side S", 45: "Wood Side W", 46: "Wood Side N", 47: "Wood Side E",
  48: "Shingle Side S", 49: "Shingle Side W", 50: "Shingle Side N", 51: "Shingle Side E",
  52: "Ice Side S", 53: "Ice Side W", 54: "Ice Side N", 55: "Ice Side E",
  56: "Shingle", 57: "Gradient / Neon", 58: "Glass",
  59: "Water (3/4)", 60: "Water (1/2)", 61: "Water (1/4)",
  62: "Lava (3/4)", 63: "Lava (1/2)", 64: "Lava (1/4)",
  65: "Firework", 66: "Door S", 67: "Door W", 68: "Door N", 69: "Door E",
  70: "Door Top", 71: "Golden Cube", 72: "Lightbox", 73: "Flower",
  74: "Steel", 75: "Portal S", 76: "Portal W", 77: "Portal N", 78: "Portal E",
  79: "Portal Top", 80: "Custom", 81: "Block TNT",
  82: "BT Grass", 83: "BT Dark Stone", 84: "BT Stone", 85: "BT Dirt",
  86: "BT Sand", 87: "BT TNT", 88: "BT Wood", 89: "BT Shingle",
  90: "BT Glass", 91: "BT Gradient", 92: "BT Tree", 93: "BT Leaves",
  94: "BT Brick", 95: "BT Cobblestone", 96: "BT Vines", 97: "BT Ladder",
  98: "BT Ice", 99: "BT Crystal", 100: "BT Trampoline", 101: "BT Cloud",
  102: "BT Stone Side", 103: "BT Wood Side", 104: "BT Ice Side",
  105: "BT Shingle Side", 106: "BT Fence", 107: "BT Water",
  108: "BT Lava", 109: "BT Firework", 110: "BT Lightbox", 111: "BT Steel",
};

export const MC_BLOCK_OPTIONS: { label: string; id: number; meta: number }[] = [
  { label: "Air", id: 0, meta: 0 },
  { label: "Stone", id: 1, meta: 0 },
  { label: "Grass Block", id: 2, meta: 0 },
  { label: "Dirt", id: 3, meta: 0 },
  { label: "Cobblestone", id: 4, meta: 0 },
  { label: "Oak Planks", id: 5, meta: 0 },
  { label: "Bedrock", id: 7, meta: 0 },
  { label: "Water", id: 9, meta: 0 },
  { label: "Lava", id: 11, meta: 0 },
  { label: "Sand", id: 12, meta: 0 },
  { label: "Gravel", id: 13, meta: 0 },
  { label: "Oak Log", id: 17, meta: 0 },
  { label: "Oak Leaves", id: 18, meta: 0 },
  { label: "Smooth Sandstone", id: 24, meta: 2 },
  { label: "White Wool", id: 35, meta: 0 },
  { label: "Gold Block", id: 41, meta: 0 },
  { label: "Iron Block", id: 42, meta: 0 },
  { label: "Bricks", id: 45, meta: 0 },
  { label: "TNT", id: 46, meta: 0 },
  { label: "Bookshelf", id: 47, meta: 0 },
  { label: "Mossy Cobblestone", id: 48, meta: 0 },
  { label: "Obsidian", id: 49, meta: 0 },
  { label: "Diamond Block", id: 57, meta: 0 },
  { label: "Ice", id: 79, meta: 0 },
  { label: "Oak Fence", id: 85, meta: 0 },
  { label: "Glowstone", id: 89, meta: 0 },
  { label: "Stone Bricks", id: 98, meta: 0 },
  { label: "Vines", id: 106, meta: 0 },
  { label: "Nether Brick", id: 112, meta: 0 },
  { label: "End Portal Frame", id: 120, meta: 0 },
  { label: "Cobblestone Wall", id: 139, meta: 0 },
  { label: "Poppy", id: 38, meta: 0 },
  { label: "Glass", id: 20, meta: 0 },
  { label: "Sea Lantern", id: 169, meta: 0 },
  { label: "Block of Quartz", id: 155, meta: 0 },
  { label: "Chiseled Quartz", id: 155, meta: 1 },
  { label: "Stained Clay (white)", id: 159, meta: 0 },
  { label: "Oak Stairs", id: 53, meta: 0 },
  { label: "Cobblestone Stairs", id: 67, meta: 0 },
  { label: "Nether Brick Stairs", id: 114, meta: 0 },
  { label: "Quartz Stairs", id: 156, meta: 0 },
  { label: "White Concrete", id: 251, meta: 0 },
  { label: "Oak Door", id: 64, meta: 0 },
  { label: "Carpet (white)", id: 171, meta: 0 },
  { label: "Jungle Planks", id: 5, meta: 3 },
];

export const PAINTED_FAMILIES: { value: PaintedFamily; label: string }[] = [
  { value: "none",          label: "None (ignore paint)" },
  { value: "concrete",      label: "Concrete (16 MC colors)" },
  { value: "wool",          label: "Wool (16 MC colors)" },
  { value: "stained_glass", label: "Stained Glass (16 MC colors)" },
  { value: "terracotta",    label: "Terracotta (16 MC colors)" },
];
