/*
 * Basic example of using the Storm FFI library
 *
 * Compile with:
 * gcc -o basic basic.c -L../../target/debug -lstorm
 *
 * Note: The CLI tool is available as 'storm-cli' after installation
 */

#include <stdio.h>
#include <stdbool.h>
#include "../include/StormLib.h"

int main(int argc, char *argv[])
{
    if (argc < 2)
    {
        printf("Usage: %s <mpq_file>\n", argv[0]);
        return 1;
    }

    HANDLE hMpq = NULL;

    // Try to open the MPQ archive
    if (SFileOpenArchive(argv[1], 0, 0, &hMpq))
    {
        printf("Successfully opened archive: %s\n", argv[1]);

        // TODO: Add file operations once implemented

        // Close the archive
        SFileCloseArchive(hMpq);
    }
    else
    {
        printf("Failed to open archive: %s (error: %u)\n", argv[1], SFileGetLastError());
        return 1;
    }

    return 0;
}
