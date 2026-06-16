#include "TerrainGenerator.h"
#include "Constants.h"
#include "Noise.h"

#include <algorithm>
#include <cstdio>
#include <limits>
#include <string>
#include <vector>

namespace {

static inline int clampInt(int v, int lo, int hi) {
    if (v < lo) return lo;
    if (v > hi) return hi;
    return v;
}

static inline size_t idx2D(int x, int z, int width) {
    return (size_t)z * (size_t)width + (size_t)x;
}

static inline uint32_t hash3(uint32_t seed, int x, int y, int z) {
    uint32_t h = seed ^ 0x9e3779b9u;
    h ^= (uint32_t)x * 374761393u;
    h ^= (uint32_t)y * 668265263u;
    h ^= (uint32_t)z * 2246822519u;
    h = (h ^ (h >> 13)) * 1274126177u;
    h ^= (h >> 16);
    return h;
}

static bool shouldPlaceLeaf(uint32_t seed, int x, int y, int z, double normalizedDist) {
    if (normalizedDist <= 0.0) return true;
    if (normalizedDist >= 1.0) return false;
    double baseKeep = 1.0 - normalizedDist * normalizedDist;
    uint32_t h = hash3(seed, x, y, z);
    double jitter = (double)(h & 1023u) / 1023.0;
    return jitter < baseKeep;
}

static int getTopSolidY(const std::vector<EdenColumn>& cols, int width, int depth, int wx, int wz) {
    if (wx < 0 || wz < 0 || wx >= width || wz >= depth) return 0;
    int colsX = width / 16;
    int cx = wx / 16, cz = wz / 16;
    int lx = wx % 16, lz = wz % 16;
    const EdenColumn& col = cols[(size_t)cz * (size_t)colsX + (size_t)cx];
    for (int y = 63; y >= 0; --y) {
        uint8_t t = col.blocks[lx][y][lz].type;
        if (t != TYPE_NONE && t != TYPE_WATER && t != TYPE_FLOWER && t != TYPE_LEAVES) return y;
    }
    return 0;
}

} // namespace

bool TerrainGenerator::generate(const TerrainParams& params,
                                 std::vector<EdenColumn>& outColumns,
                                 TerrainMetadata* outMeta) const {
    if (params.width <= 0 || params.depth <= 0
        || (params.width % 16) != 0 || (params.depth % 16) != 0) {
        std::printf("Invalid terrain size: width/depth must be positive multiples of 16.\n");
        return false;
    }

    int waterAmnt = clampInt(params.waterAmnt, 1, 5);
    int waterLevel;
    switch (waterAmnt) {
        case 1: waterLevel = 40; break;
        case 2: waterLevel = 35; break;
        case 3: waterLevel = 32; break;
        case 4: waterLevel = 27; break;
        default: waterLevel = -1; break; // fully dry land
    }
    const int snowHeight = 48;
    const int colsX = params.width / 16;
    const int colsZ = params.depth / 16;
    const int totalColumns = colsX * colsZ;
    Noise2D noise(params.seed);

    outColumns.clear();
    outColumns.reserve((size_t)totalColumns);

    std::vector<int> heightmap((size_t)params.width * (size_t)params.depth, 0);
    std::vector<uint8_t> waterMask((size_t)params.width * (size_t)params.depth, 0);

    int minH = std::numeric_limits<int>::max();
    int maxH = std::numeric_limits<int>::min();
    int generated = 0;

    // Stage 1+2: heightmap + terrain fill
    for (int cz = 0; cz < colsZ; ++cz) {
        for (int cx = 0; cx < colsX; ++cx) {
            EdenColumn col;
            col.x = cx;
            col.z = cz;
            for (int x = 0; x < 16; ++x)
                for (int z = 0; z < 16; ++z)
                    for (int y = 0; y < 64; ++y) {
                        col.blocks[x][y][z].type = TYPE_NONE;
                        col.blocks[x][y][z].color = 0;
                    }

            for (int lx = 0; lx < 16; ++lx) {
                for (int lz = 0; lz < 16; ++lz) {
                    int wx = cx * 16 + lx;
                    int wz = cz * 16 + lz;
                    double n = noise.fractal((double)wx, (double)wz, 4, 0.02, 0.5);
                    int H = clampInt((int)((double)params.baseHeight + n * 18.0), 1, 62);
                    minH = std::min(minH, H);
                    maxH = std::max(maxH, H);
                    heightmap[idx2D(wx, wz, params.width)] = H;

                    int stoneTop = std::max(0, H - 3);
                    int dirtTop  = std::max(0, H - 1);
                    for (int y = 0; y <= stoneTop; ++y) col.blocks[lx][y][lz].type = TYPE_STONE;
                    for (int y = stoneTop + 1; y <= dirtTop; ++y) col.blocks[lx][y][lz].type = TYPE_DIRT;
                    col.blocks[lx][H][lz].type = TYPE_GRASS;
                }
            }
            outColumns.push_back(col);
            generated++;
            if (generated % 64 == 0 || generated == totalColumns)
                std::printf("Generation: %d / %d columns\n", generated, totalColumns);
        }
    }

    // Stage 3: caves — threshold 0.62 (less aggressive than original 0.52)
    int cavesCarved = 0;
    for (int wz = 0; wz < params.depth; ++wz) {
        for (int wx = 0; wx < params.width; ++wx) {
            int H = heightmap[idx2D(wx, wz, params.width)];
            int cx = wx / 16, cz = wz / 16, lx = wx % 16, lz = wz % 16;
            EdenColumn& col = outColumns[(size_t)cz * (size_t)colsX + (size_t)cx];
            // Start at y=1 (keep bedrock-equivalent floor), leave 4-block surface crust
            for (int y = 1; y <= H - 4; ++y) {
                double c = noise.fractal3D((double)wx, (double)y, (double)wz, 3, 0.08, 0.5);
                if (c > 0.62) {
                    col.blocks[lx][y][lz].type = TYPE_NONE;
                    col.blocks[lx][y][lz].color = 0;
                    cavesCarved++;
                }
            }
        }
    }

    // Stage 4: water
    int waterBlocksPlaced = 0;
    for (int wz = 0; wz < params.depth; ++wz) {
        for (int wx = 0; wx < params.width; ++wx) {
            int H = heightmap[idx2D(wx, wz, params.width)];
            int cx = wx / 16, cz = wz / 16, lx = wx % 16, lz = wz % 16;
            EdenColumn& col = outColumns[(size_t)cz * (size_t)colsX + (size_t)cx];

            if (waterLevel < 0) continue;
            double pondN  = noise.fractal((double)wx + 1000.0, (double)wz + 1000.0, 3, 0.03, 0.5);
            double riverN = noise.fractal((double)wx + 4000.0, (double)wz - 4000.0, 2, 0.01, 0.6);
            bool pond  = (pondN  > 0.62  && H < waterLevel + 3);
            bool river = (riverN > -0.03 && riverN < 0.03 && H < 45);
            int localWL = waterLevel + (pond ? 1 : 0);

            if (H < waterLevel || pond || river) {
                col.blocks[lx][H][lz].type = TYPE_SAND;
                for (int y = H + 1; y <= localWL && y < 63; ++y) {
                    if (col.blocks[lx][y][lz].type != TYPE_WATER) {
                        col.blocks[lx][y][lz].type = TYPE_WATER;
                        waterBlocksPlaced++;
                    }
                }
                waterMask[idx2D(wx, wz, params.width)] = 1;
            }
        }
    }

    // Stage 5: beaches
    for (int wz = 1; wz < params.depth - 1; ++wz) {
        for (int wx = 1; wx < params.width - 1; ++wx) {
            int topY = getTopSolidY(outColumns, params.width, params.depth, wx, wz);
            int cx = wx / 16, cz = wz / 16, lx = wx % 16, lz = wz % 16;
            EdenColumn& col = outColumns[(size_t)cz * (size_t)colsX + (size_t)cx];
            if (col.blocks[lx][topY][lz].type != TYPE_GRASS) continue;

            int waterNeighbors = 0;
            for (int dz = -1; dz <= 1; ++dz)
                for (int dx = -1; dx <= 1; ++dx) {
                    if (dx == 0 && dz == 0) continue;
                    waterNeighbors += (int)waterMask[idx2D(wx + dx, wz + dz, params.width)];
                }
            if (waterNeighbors >= 3 && waterLevel >= 0 && topY <= waterLevel + 2)
                col.blocks[lx][topY][lz].type = TYPE_SAND;
        }
    }

    // Stage 6: vegetation — 4-block edge guard keeps canopy in-bounds
    int treesPlaced = 0, flowersPlaced = 0;
    const int EDGE = 4;
    for (int wz = EDGE; wz < params.depth - EDGE; ++wz) {
        for (int wx = EDGE; wx < params.width - EDGE; ++wx) {
            int topY = getTopSolidY(outColumns, params.width, params.depth, wx, wz);
            int cx = wx / 16, cz = wz / 16, lx = wx % 16, lz = wz % 16;
            EdenColumn& col = outColumns[(size_t)cz * (size_t)colsX + (size_t)cx];
            if (col.blocks[lx][topY][lz].type != TYPE_GRASS) continue;
            if (waterLevel >= 0 && topY <= waterLevel) continue;
            if (topY + 1 >= 63) continue;
            if (col.blocks[lx][topY + 1][lz].type != TYPE_NONE) continue;

            uint32_t h = hash3(params.seed, wx, topY, wz);
            if ((h % 1000u) < 6u) {
                // Minimum trunk 4 (avoids stumpy look)
                int trunk = 4 + (int)(h % 4u);
                for (int i = 1; i <= trunk && topY + i < 63; ++i) {
                    col.blocks[lx][topY + i][lz].type = TYPE_TREE;
                }
                int leafTopY = std::min(62, topY + trunk);
                int leafPattern = (int)(h % 3u);
                for (int dz = -4; dz <= 4; ++dz)
                for (int dx = -4; dx <= 4; ++dx)
                for (int dy = 1; dy <= 5; ++dy) {
                    int ax = wx + dx, az = wz + dz, ay = leafTopY + dy;
                    if (ax < 0 || az < 0 || ax >= params.width || az >= params.depth
                        || ay < 1 || ay >= 63) continue;

                    bool patternInside = false;
                    double norm = 1.0;
                    if (leafPattern == 0) {
                        double ex = (double)dx / 3.2;
                        double ey = (double)(dy - 2) / 2.2;
                        double ez = (double)dz / 3.2;
                        double d = ex*ex + ey*ey + ez*ez;
                        patternInside = (d <= 1.0);
                        norm = std::sqrt(std::min(1.0, d));
                    } else if (leafPattern == 1) {
                        int radial = dx*dx + dz*dz;
                        patternInside = (radial <= 12 && dy >= 1 && dy <= 3);
                        double plane = std::sqrt((double)radial) / 3.6;
                        double vert  = std::abs((double)(dy - 2)) / 1.8;
                        norm = std::min(1.0, (plane + vert) * 0.65);
                    } else {
                        int radial = dx*dx + dz*dz;
                        int maxR = (dy >= 5) ? 0 : (dy == 4) ? 1 : (dy == 3) ? 2 : (dy == 2) ? 3 : 4;
                        patternInside = (radial <= maxR * maxR);
                        norm = (maxR <= 0) ? 0.0 : std::min(1.0, std::sqrt((double)radial) / (double)maxR);
                    }
                    if (!patternInside) continue;
                    if (!shouldPlaceLeaf(params.seed + 97u, ax, ay, az, norm)) continue;

                    int acx = ax / 16, acz = az / 16, alx = ax % 16, alz = az % 16;
                    EdenColumn& acol = outColumns[(size_t)acz * (size_t)colsX + (size_t)acx];
                    if (acol.blocks[alx][ay][alz].type == TYPE_NONE)
                        acol.blocks[alx][ay][alz].type = TYPE_LEAVES;
                }
                treesPlaced++;
            } else if ((h % 1000u) < 12u) {
                col.blocks[lx][topY + 1][lz].type = TYPE_FLOWER;
                flowersPlaced++;
            }
        }
    }

    // Stage 7: snow on high ground (white-tinted sand)
    for (int wz = 0; wz < params.depth; ++wz) {
        for (int wx = 0; wx < params.width; ++wx) {
            int topY = getTopSolidY(outColumns, params.width, params.depth, wx, wz);
            if (topY < snowHeight) continue;
            int cx = wx / 16, cz = wz / 16, lx = wx % 16, lz = wz % 16;
            EdenColumn& col = outColumns[(size_t)cz * (size_t)colsX + (size_t)cx];
            uint8_t t = col.blocks[lx][topY][lz].type;
            if (t == TYPE_GRASS || t == TYPE_DIRT || t == TYPE_STONE || t == TYPE_SAND) {
                col.blocks[lx][topY][lz].type = TYPE_SAND;
                col.blocks[lx][topY][lz].color = 1; // light color index
            }
        }
    }

    int spawnX = params.width / 2;
    int spawnZ = params.depth / 2;
    int spawnY = clampInt(getTopSolidY(outColumns, params.width, params.depth, spawnX, spawnZ) + 1, 1, 62);

    if (outMeta) {
        outMeta->spawnX = spawnX;
        outMeta->spawnY = spawnY;
        outMeta->spawnZ = spawnZ;
        outMeta->treesPlaced = treesPlaced;
        outMeta->flowersPlaced = flowersPlaced;
        outMeta->caveBlocksCarved = cavesCarved;
        outMeta->minHeight = minH;
        outMeta->maxHeight = maxH;
        outMeta->expectedColumns = totalColumns;
        outMeta->generatedColumns = generated;
    }

    std::printf("Terrain done. Heights: %d..%d | trees: %d | flowers: %d | caves: %d blocks | water blocks: %d\n",
                minH, maxH, treesPlaced, flowersPlaced, cavesCarved, waterBlocksPlaced);
    std::printf("Spawn: (%d, %d, %d)\n", spawnX, spawnY, spawnZ);
    return true;
}
