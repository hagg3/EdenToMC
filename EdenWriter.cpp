#include "EdenWriter.h"
#include "EdenFileLoader.h"
#include "Constants.h"

#include <cstdio>
#include <cstring>

bool EdenWriter::writeWorld(
    const std::string& outPath,
    const std::vector<EdenColumn>& columns,
    uint32_t levelSeed,
    const std::string& worldName,
    int spawnX,
    int spawnY,
    int spawnZ,
    int expectedColumns) {

    FILE* fp = std::fopen(outPath.c_str(), "wb");
    if (!fp) {
        std::printf("Failed to open output file: %s\n", outPath.c_str());
        return false;
    }

    WorldFileHeader header;
    std::memset(&header, 0, sizeof(header));
    header.level_seed = (int)levelSeed;
    header.pos.x = (float)spawnX;
    header.pos.y = (float)spawnY;
    header.pos.z = (float)spawnZ;
    header.home = header.pos;
    header.yaw = 0.0f;
    header.version = FILE_VERSION;
    std::snprintf(header.name, sizeof(header.name), "%s", worldName.c_str());

    if (std::fwrite(&header, sizeof(header), 1, fp) != 1) {
        std::fclose(fp);
        return false;
    }

    std::vector<ColumnIndex> indexes;
    indexes.reserve(columns.size());
    size_t writtenColumns = 0;

    for (const EdenColumn& col : columns) {
        ColumnIndex idx;
        idx.x = col.x;
        idx.z = col.z;
        idx.chunk_offset = (unsigned long long)std::ftell(fp);
        indexes.push_back(idx);

        // 4 vertical sub-chunks of 16 blocks each
        for (int cy = 0; cy < 4; ++cy) {
            block8 blockChunk[CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
            color8 colorChunk[CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
            for (int x = 0; x < CHUNK_SIZE; ++x) {
                for (int z = 0; z < CHUNK_SIZE; ++z) {
                    for (int y = 0; y < CHUNK_SIZE; ++y) {
                        int worldY = cy * CHUNK_SIZE + y;
                        // Eden voxel index: x * 256 + z * 16 + y
                        int i = x * CHUNK_SIZE * CHUNK_SIZE + z * CHUNK_SIZE + y;
                        blockChunk[i] = (block8)col.blocks[x][worldY][z].type;
                        colorChunk[i] = (color8)col.blocks[x][worldY][z].color;
                    }
                }
            }
            if (std::fwrite(blockChunk, sizeof(blockChunk), 1, fp) != 1) {
                std::fclose(fp);
                return false;
            }
            if (std::fwrite(colorChunk, sizeof(colorChunk), 1, fp) != 1) {
                std::fclose(fp);
                return false;
            }
        }
        writtenColumns++;
    }

    header.directory_offset = (unsigned long long)std::ftell(fp);
    for (const ColumnIndex& idx : indexes) {
        if (std::fwrite(&idx, sizeof(ColumnIndex), 1, fp) != 1) {
            std::fclose(fp);
            return false;
        }
    }

    // Rewrite header with correct directory_offset
    std::fseek(fp, 0, SEEK_SET);
    if (std::fwrite(&header, sizeof(header), 1, fp) != 1) {
        std::fclose(fp);
        return false;
    }

    std::fclose(fp);

    int expected = (expectedColumns >= 0) ? expectedColumns : (int)columns.size();
    std::printf("EdenWriter: wrote %zu columns (expected %d) to %s\n",
                writtenColumns, expected, outPath.c_str());
    if ((int)writtenColumns != expected) {
        std::printf("Column count mismatch — output may be invalid.\n");
        return false;
    }
    return true;
}
