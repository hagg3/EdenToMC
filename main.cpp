
#include "EdenFileLoader.h"
#include "MC2EdenConverter.h"
#include "TerrainGenerator.h"
#include "EdenWriter.h"
#include <cstdint>
#include <cstdlib>
#include <cstdio>
#include <string>
#include <vector>

static void printUsage(const char* prog) {
    printf("Usage:\n");
    printf("  %s                                    -- convert FILE.eden to ConvertedWorld/\n", prog);
    printf("  %s mc2eden <region_folder> <out.eden> -- convert MC region files back to Eden\n", prog);
    printf("  %s generate <w> <d> <seed> <out.eden> [baseHeight] [waterAmnt(1-5)]\n", prog);
}

int main(int argc, char** argv)
{
    if (argc >= 2 && std::string(argv[1]) == "mc2eden") {
        if (argc < 4) {
            printf("Usage: %s mc2eden <region_folder> <out.eden>\n", argv[0]);
            return 1;
        }
        MC2EdenConverter converter;
        return converter.convertRegionFolderToEden(argv[2], argv[3]) ? 0 : 2;
    }

    if (argc >= 2 && std::string(argv[1]) == "generate") {
        if (argc < 6) {
            printf("Usage: %s generate <width> <depth> <seed> <out.eden> [baseHeight] [waterAmnt]\n", argv[0]);
            return 1;
        }
        TerrainParams params;
        params.width      = std::atoi(argv[2]);
        params.depth      = std::atoi(argv[3]);
        params.seed       = (uint32_t)std::strtoul(argv[4], nullptr, 10);
        params.baseHeight = (argc >= 7) ? std::atoi(argv[6]) : 30;
        params.waterAmnt  = (argc >= 8) ? std::atoi(argv[7]) : 3;

        TerrainGenerator generator;
        std::vector<EdenColumn> columns;
        TerrainMetadata meta;
        if (!generator.generate(params, columns, &meta)) return 2;
        printf("Generated %zu columns (expected %d)\n", columns.size(), meta.expectedColumns);
        if ((int)columns.size() != meta.expectedColumns) {
            printf("Column count mismatch — aborting.\n");
            return 2;
        }
        EdenWriter writer;
        if (!writer.writeWorld(argv[5], columns, params.seed, "TerrainGen",
                               meta.spawnX, meta.spawnY, meta.spawnZ, meta.expectedColumns))
            return 3;
        printf("Eden terrain world written: %s\n", argv[5]);
        return 0;
    }

    if (argc >= 2 && (std::string(argv[1]) == "--help" || std::string(argv[1]) == "-h")) {
        printUsage(argv[0]);
        return 0;
    }

    // Default: convert FILE.eden → ConvertedWorld/
    EdenFileLoader efl;
    char worldFile[] = "FILE.eden";
    const char* outputWorld = "ConvertedWorld";
    printf("Converting %s → %s/\n", worldFile, outputWorld);
    efl.convertToMinecraft(worldFile, outputWorld);
    printf("Done.\n");
    return 0;
}
