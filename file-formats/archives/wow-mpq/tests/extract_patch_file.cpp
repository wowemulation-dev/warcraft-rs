/**
 * Extract raw patch file data from update MPQ
 *
 * This extracts the actual patch file (PTCH format) without applying it,
 * so we can test our patch parsing and application code.
 */

#include <StormLib.h>
#include <iostream>
#include <fstream>
#include <vector>

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
    // Update MPQ containing patch files
    const char* update_mpq = "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data/wow-update-base-15211.MPQ";
    const char* test_file = "Item/ObjectComponents/Head/Helm_Robe_RaidWarlock_F_01_WoF.M2";

    std::cout << "=== Extract Raw Patch File ===\n\n";

    // Open update MPQ WITHOUT base MPQ (no patch chain)
    HANDLE hMpq = nullptr;
    if (!SFileOpenArchive(update_mpq, 0, 0, &hMpq)) {
        std::cerr << "Failed to open MPQ: " << update_mpq << "\n";
        return 1;
    }
    std::cout << "Opened MPQ: " << update_mpq << "\n";

    // Try to open the file
    HANDLE hFile = nullptr;
    if (!SFileOpenFileEx(hMpq, test_file, 0, &hFile)) {
        std::cerr << "Failed to open file: " << test_file << "\n";
        SFileCloseArchive(hMpq);
        return 1;
    }
    std::cout << "Opened file: " << test_file << "\n";

    // Get file size
    DWORD fileSize = SFileGetFileSize(hFile, nullptr);
    std::cout << "File size: " << fileSize << " bytes\n";

    // Read the patch file
    std::vector<unsigned char> buffer(fileSize);
    DWORD bytesRead = 0;
    if (!SFileReadFile(hFile, buffer.data(), fileSize, &bytesRead, nullptr)) {
        std::cerr << "Failed to read file\n";
        SFileCloseFile(hFile);
        SFileCloseArchive(hMpq);
        return 1;
    }
    std::cout << "Read " << bytesRead << " bytes\n";

    // Write raw patch data
    writeFile("/tmp/raw_patch_file.bin", buffer.data(), bytesRead);

    // Dump first 128 bytes
    std::cout << "\nFirst 128 bytes (should be PTCH format):\n";
    for (size_t i = 0; i < std::min(size_t(128), size_t(bytesRead)); i++) {
        printf("%02X ", buffer[i]);
        if ((i + 1) % 16 == 0) printf("\n");
    }
    std::cout << "\n";

    // Check for PTCH signature
    if (bytesRead >= 4) {
        uint32_t sig = *reinterpret_cast<uint32_t*>(buffer.data());
        if (sig == 0x48435450) { // 'PTCH'
            std::cout << "✓ PTCH signature detected\n";
        } else {
            std::cout << "✗ Not a PTCH file (signature: 0x" << std::hex << sig << ")\n";
        }
    }

    SFileCloseFile(hFile);
    SFileCloseArchive(hMpq);

    std::cout << "\n=== Complete ===\n";
    return 0;
}
