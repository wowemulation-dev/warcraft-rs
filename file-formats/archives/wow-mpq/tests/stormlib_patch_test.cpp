/**
 * StormLib Patch File Test
 *
 * This program uses StormLib to:
 * 1. Open a patch chain (base MPQ + update MPQs)
 * 2. Extract a file that exists in multiple versions (base + patches)
 * 3. Save the final patched result
 *
 * This provides reference data for testing wow-mpq patch implementation.
 */

#include <StormLib.h>
#include <iostream>
#include <fstream>
#include <vector>
#include <cstring>

void writeFile(const char* path, const void* data, size_t size) {
    std::ofstream out(path, std::ios::binary);
    if (!out) {
        std::cerr << "Failed to open output file: " << path << "\n";
        return;
    }
    out.write(static_cast<const char*>(data), size);
    std::cout << "Wrote " << size << " bytes to " << path << "\n";
}

int main() {
    // Cataclysm 4.3.4 archives
    const char* base_mpq = "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data/art.MPQ";
    const char* update1 = "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data/wow-update-base-15211.MPQ";
    const char* update2 = "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data/wow-update-base-15354.MPQ";
    const char* update3 = "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data/wow-update-base-15595.MPQ";

    // Test file that appears in updates (has patch files)
    const char* test_file = "Item/ObjectComponents/Head/Helm_Robe_RaidWarlock_F_01_WoF.M2";

    std::cout << "=== StormLib Patch File Test ===\n\n";

    // Step 1: Extract base file (before any patches)
    std::cout << "Step 1: Extracting base file (no patches)\n";
    std::cout << "-------------------------------------------\n";
    HANDLE hBaseOnly = nullptr;
    if (!SFileOpenArchive(base_mpq, 0, 0, &hBaseOnly)) {
        std::cerr << "Failed to open base MPQ: " << base_mpq << "\n";
        return 1;
    }
    std::cout << "Opened base MPQ: " << base_mpq << "\n";

    HANDLE hBaseFile = nullptr;
    if (SFileOpenFileEx(hBaseOnly, test_file, 0, &hBaseFile)) {
        DWORD baseSize = SFileGetFileSize(hBaseFile, nullptr);
        std::vector<unsigned char> baseBuffer(baseSize);
        DWORD baseBytesRead = 0;
        if (SFileReadFile(hBaseFile, baseBuffer.data(), baseSize, &baseBytesRead, nullptr)) {
            writeFile("/tmp/stormlib_base_file.bin", baseBuffer.data(), baseBytesRead);
            std::cout << "Base file size: " << baseBytesRead << " bytes\n";
        }
        SFileCloseFile(hBaseFile);
    } else {
        std::cout << "File not found in base MPQ (will be added by patches)\n";
    }
    SFileCloseArchive(hBaseOnly);
    std::cout << "\n";

    // Step 2: Open with patch chain
    std::cout << "Step 2: Opening with patch chain\n";
    std::cout << "-------------------------------------------\n";
    HANDLE hMpq = nullptr;
    if (!SFileOpenArchive(base_mpq, 0, 0, &hMpq)) {
        std::cerr << "Failed to open base MPQ: " << base_mpq << "\n";
        return 1;
    }
    std::cout << "Opened base MPQ: " << base_mpq << "\n";

    // Add patch archives in order
    if (!SFileOpenPatchArchive(hMpq, update1, nullptr, 0)) {
        std::cerr << "Failed to add patch: " << update1 << "\n";
    } else {
        std::cout << "Added patch: " << update1 << "\n";
    }

    if (!SFileOpenPatchArchive(hMpq, update2, nullptr, 0)) {
        std::cerr << "Failed to add patch: " << update2 << "\n";
    } else {
        std::cout << "Added patch: " << update2 << "\n";
    }

    if (!SFileOpenPatchArchive(hMpq, update3, nullptr, 0)) {
        std::cerr << "Failed to add patch: " << update3 << "\n";
    } else {
        std::cout << "Added patch: " << update3 << "\n";
    }

    std::cout << "\n";

    // Step 3: Extract final patched file
    std::cout << "Step 3: Extracting final patched file\n";
    std::cout << "-------------------------------------------\n";
    HANDLE hFile = nullptr;
    if (!SFileOpenFileEx(hMpq, test_file, 0, &hFile)) {
        std::cerr << "Failed to open file: " << test_file << "\n";
        SFileCloseArchive(hMpq);
        return 1;
    }
    std::cout << "Opened file: " << test_file << "\n";

    // Get file size
    DWORD fileSize = SFileGetFileSize(hFile, nullptr);
    std::cout << "Patched file size: " << fileSize << " bytes\n";

    // Read the file
    std::vector<unsigned char> buffer(fileSize);
    DWORD bytesRead = 0;
    if (!SFileReadFile(hFile, buffer.data(), fileSize, &bytesRead, nullptr)) {
        std::cerr << "Failed to read file\n";
        SFileCloseFile(hFile);
        SFileCloseArchive(hMpq);
        return 1;
    }
    std::cout << "Read " << bytesRead << " bytes\n";

    // Write the final patched result
    writeFile("/tmp/stormlib_patched_result.bin", buffer.data(), bytesRead);

    // Also dump first 128 bytes as hex for inspection
    std::cout << "\nFirst 128 bytes of patched file (hex):\n";
    for (size_t i = 0; i < std::min(size_t(128), size_t(bytesRead)); i++) {
        printf("%02X ", buffer[i]);
        if ((i + 1) % 16 == 0) printf("\n");
    }
    std::cout << "\n";

    SFileCloseFile(hFile);
    SFileCloseArchive(hMpq);

    std::cout << "\n=== Test Complete ===\n";
    std::cout << "Files saved:\n";
    std::cout << "  Base:    /tmp/stormlib_base_file.bin\n";
    std::cout << "  Patched: /tmp/stormlib_patched_result.bin\n";

    return 0;
}
