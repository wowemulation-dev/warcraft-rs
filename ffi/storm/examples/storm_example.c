/*
 * Comprehensive example of using the Storm FFI library
 *
 * Compile with:
 * gcc -o storm_example storm_example.c -L../../target/debug -lstorm -Wl,-rpath,../../target/debug
 *
 * Or on Windows:
 * cl storm_example.c /link ../../target/debug/storm.lib
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include "../include/StormLib.h"

// Helper function to print error messages
void PrintError(const char *operation)
{
    uint32_t error = SFileGetLastError();
    printf("Failed to %s (error: %u)\n", operation, error);
}

// Callback function for file enumeration
bool EnumFilesCallback(const char *filename, void *user_data)
{
    int *count = (int *)user_data;
    printf("  [%d] %s\n", *count, filename);
    (*count)++;
    return true; // Continue enumeration
}

// Example: Read a specific file
void ReadFileExample(HANDLE hMpq, const char *filename)
{
    printf("\n--- Reading file: %s ---\n", filename);

    // Check if file exists
    if (!SFileHasFile(hMpq, filename))
    {
        printf("File '%s' not found in archive\n", filename);
        return;
    }

    // Open the file
    HANDLE hFile = NULL;
    if (!SFileOpenFileEx(hMpq, filename, 0, &hFile))
    {
        PrintError("open file");
        return;
    }

    // Get file size
    uint32_t sizeHigh = 0;
    uint32_t sizeLow = SFileGetFileSize(hFile, &sizeHigh);
    uint64_t fileSize = ((uint64_t)sizeHigh << 32) | sizeLow;

    printf("File size: %llu bytes\n", (unsigned long long)fileSize);

    // Allocate buffer
    char *buffer = (char *)malloc((size_t)fileSize + 1);
    if (!buffer)
    {
        printf("Failed to allocate memory\n");
        SFileCloseFile(hFile);
        return;
    }

    // Read the file
    uint32_t bytesRead = 0;
    if (SFileReadFile(hFile, buffer, (uint32_t)fileSize, &bytesRead, NULL))
    {
        buffer[bytesRead] = '\0'; // Null terminate for text files

        // Print first 100 bytes or full content if smaller
        printf("Content (first %u bytes):\n", bytesRead > 100 ? 100 : bytesRead);
        if (bytesRead > 100)
        {
            char temp = buffer[100];
            buffer[100] = '\0';
            printf("%s...\n", buffer);
            buffer[100] = temp;
        }
        else
        {
            printf("%s\n", buffer);
        }
    }
    else
    {
        PrintError("read file");
    }

    // Cleanup
    free(buffer);
    SFileCloseFile(hFile);
}

// Example: Get archive information
void GetArchiveInfo(HANDLE hMpq)
{
    printf("\n--- Archive Information ---\n");

    uint64_t archiveSize = 0;
    uint32_t sizeNeeded = 0;

    // Get archive size
    if (SFileGetFileInfo(hMpq, 1, &archiveSize, sizeof(archiveSize), &sizeNeeded))
    {
        printf("Archive size: %llu bytes\n", (unsigned long long)archiveSize);
    }

    // Get hash table size
    uint32_t hashTableSize = 0;
    if (SFileGetFileInfo(hMpq, 2, &hashTableSize, sizeof(hashTableSize), &sizeNeeded))
    {
        printf("Hash table size: %u entries\n", hashTableSize);
    }

    // Get block table size
    uint32_t blockTableSize = 0;
    if (SFileGetFileInfo(hMpq, 3, &blockTableSize, sizeof(blockTableSize), &sizeNeeded))
    {
        printf("Block table size: %u entries\n", blockTableSize);
    }

    // Get sector size
    uint32_t sectorSize = 0;
    if (SFileGetFileInfo(hMpq, 4, &sectorSize, sizeof(sectorSize), &sizeNeeded))
    {
        printf("Sector size: %u bytes\n", sectorSize);
    }
}

// Example: Enumerate all files
void EnumerateFiles(HANDLE hMpq)
{
    printf("\n--- File Enumeration ---\n");

    int fileCount = 0;
    if (SFileEnumFiles(hMpq, "*", NULL, EnumFilesCallback, &fileCount))
    {
        printf("\nTotal files enumerated: %d\n", fileCount);
    }
    else
    {
        printf("Note: Archive may not have a (listfile)\n");
    }
}

// Example: Test file operations
void TestFileOperations(HANDLE hMpq, const char *filename)
{
    printf("\n--- File Operations Test: %s ---\n", filename);

    HANDLE hFile = NULL;
    if (!SFileOpenFileEx(hMpq, filename, 0, &hFile))
    {
        PrintError("open file for operations test");
        return;
    }

    // Get initial position
    uint64_t position = 0;
    uint32_t sizeNeeded = 0;
    if (SFileGetFileInfo(hFile, 10, &position, sizeof(position), &sizeNeeded))
    {
        printf("Initial position: %llu\n", (unsigned long long)position);
    }

    // Read 10 bytes
    char buffer[11] = {0};
    uint32_t bytesRead = 0;
    if (SFileReadFile(hFile, buffer, 10, &bytesRead, NULL))
    {
        printf("Read %u bytes: '%.10s'\n", bytesRead, buffer);
    }

    // Check position after read
    if (SFileGetFileInfo(hFile, 10, &position, sizeof(position), &sizeNeeded))
    {
        printf("Position after read: %llu\n", (unsigned long long)position);
    }

    // Seek to beginning
    uint32_t newPos = SFileSetFilePointer(hFile, 0, NULL, 0); // FILE_BEGIN
    printf("Position after seek to start: %u\n", newPos);

    // Seek to offset 5
    newPos = SFileSetFilePointer(hFile, 5, NULL, 0);
    printf("Position after seek to 5: %u\n", newPos);

    // Read again
    if (SFileReadFile(hFile, buffer, 10, &bytesRead, NULL))
    {
        buffer[bytesRead] = '\0';
        printf("Read %u bytes from offset 5: '%.10s'\n", bytesRead, buffer);
    }

    SFileCloseFile(hFile);
}

// Example: Work with locale
void TestLocale()
{
    printf("\n--- Locale Test ---\n");

    uint32_t currentLocale = SFileGetLocale();
    printf("Current locale: 0x%04X\n", currentLocale);

    // Set to US English
    uint32_t oldLocale = SFileSetLocale(0x0409);
    printf("Previous locale was: 0x%04X\n", oldLocale);
    printf("New locale: 0x%04X\n", SFileGetLocale());

    // Restore
    SFileSetLocale(oldLocale);
}

int main(int argc, char *argv[])
{
    if (argc < 2)
    {
        printf("Storm FFI Example - StormLib Compatible MPQ Reader\n");
        printf("Usage: %s <mpq_file> [file_to_read]\n", argv[0]);
        printf("\nExample:\n");
        printf("  %s test.mpq                    # List files and info\n", argv[0]);
        printf("  %s test.mpq \"(listfile)\"       # Read specific file\n", argv[0]);
        printf("  %s test.mpq \"war3map.j\"        # Read map script\n", argv[0]);
        return 1;
    }

    HANDLE hMpq = NULL;
    const char *archivePath = argv[1];
    const char *fileToRead = argc >= 3 ? argv[2] : NULL;

    printf("Opening archive: %s\n", archivePath);

    // Try to open the MPQ archive
    if (!SFileOpenArchive(archivePath, 0, 0, &hMpq))
    {
        PrintError("open archive");
        return 1;
    }

    printf("Successfully opened archive!\n");

    // Get archive name (test our function)
    char archiveName[260] = {0};
    if (SFileGetArchiveName(hMpq, archiveName, sizeof(archiveName)))
    {
        printf("Archive path: %s\n", archiveName);
    }

    // Show archive information
    GetArchiveInfo(hMpq);

    // Test locale functions
    TestLocale();

    // If a specific file was requested, read it
    if (fileToRead)
    {
        ReadFileExample(hMpq, fileToRead);

        // Also test file operations if it exists
        if (SFileHasFile(hMpq, fileToRead))
        {
            TestFileOperations(hMpq, fileToRead);
        }
    }
    else
    {
        // Otherwise, enumerate files and try to read some common ones
        EnumerateFiles(hMpq);

        // Try to read common files
        const char *commonFiles[] = {
            "(listfile)",
            "(attributes)",
            "(signature)",
            "war3map.j",
            "war3map.w3i",
            NULL};

        printf("\n--- Checking common files ---\n");
        for (int i = 0; commonFiles[i] != NULL; i++)
        {
            if (SFileHasFile(hMpq, commonFiles[i]))
            {
                printf("Found: %s\n", commonFiles[i]);

                // Read (listfile) to show content
                if (strcmp(commonFiles[i], "(listfile)") == 0)
                {
                    ReadFileExample(hMpq, commonFiles[i]);
                }
            }
        }
    }

    // Test error handling
    printf("\n--- Error Handling Test ---\n");
    SFileSetLastError(12345);
    printf("Set error to: %u\n", SFileGetLastError());

    // Try to open non-existent file
    HANDLE hBadFile = NULL;
    if (!SFileOpenFileEx(hMpq, "this_file_does_not_exist.txt", 0, &hBadFile))
    {
        printf("Expected error for non-existent file: %u\n", SFileGetLastError());
    }

    // Cleanup
    printf("\nClosing archive...\n");
    if (SFileCloseArchive(hMpq))
    {
        printf("Archive closed successfully.\n");
    }
    else
    {
        PrintError("close archive");
    }

    return 0;
}
