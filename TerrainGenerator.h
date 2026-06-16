#pragma once

#include "MCReverseTypes.h"
#include <cstdint>
#include <vector>

struct TerrainParams {
    int width;
    int depth;
    uint32_t seed;
    int baseHeight = 30;
    int waterAmnt = 3; // 1=deep ocean .. 5=full land
};

struct TerrainMetadata {
    int spawnX = 0;
    int spawnY = 32;
    int spawnZ = 0;
    int treesPlaced = 0;
    int flowersPlaced = 0;
    int caveBlocksCarved = 0;
    int minHeight = 0;
    int maxHeight = 0;
    int expectedColumns = 0;
    int generatedColumns = 0;
};

class TerrainGenerator {
public:
    bool generate(const TerrainParams& params,
                  std::vector<EdenColumn>& outColumns,
                  TerrainMetadata* outMeta = nullptr) const;
};
