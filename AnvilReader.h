#pragma once
#include "MCReverseTypes.h"
#include <string>
#include <vector>

class AnvilReader {
public:
    std::vector<ChunkColumn> readRegionFolder(const std::string& inputRegionFolder);
    bool debugPrintFirstChunk(const std::string& mcaPath);
};
