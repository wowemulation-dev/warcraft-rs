#ifndef STORMLIB_H
#define STORMLIB_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* Type definitions */
typedef void* HANDLE;
typedef uint32_t DWORD;
typedef int32_t LONG;
typedef uint32_t LCID;

/* Archive operations */
bool SFileOpenArchive(const char* archive_name, DWORD priority, DWORD flags, HANDLE* archive);
bool SFileCreateArchive(const char* archive_name, DWORD flags, DWORD max_file_count, HANDLE* archive);
bool SFileCloseArchive(HANDLE archive);

/* File operations */
bool SFileOpenFileEx(HANDLE archive, const char* filename, DWORD search_scope, HANDLE* file);
bool SFileCloseFile(HANDLE file);
bool SFileReadFile(HANDLE file, void* buffer, DWORD to_read, DWORD* read, void* overlapped);
DWORD SFileGetFileSize(HANDLE file, DWORD* file_size_high);
DWORD SFileSetFilePointer(HANDLE file, LONG distance, LONG* distance_high, DWORD method);
DWORD SFileGetFilePointer(HANDLE file, LONG* file_pos_high);
bool SFileHasFile(HANDLE archive, const char* filename);
bool SFileExtractFile(HANDLE archive, const char* filename, const char* local_filename, DWORD search_scope);

/* Information functions */
bool SFileGetArchiveName(HANDLE archive, char* buffer, DWORD buffer_size);
bool SFileGetFileName(HANDLE file, char* buffer);
DWORD SFileGetFileInfo(HANDLE file_or_archive, DWORD info_class, void* buffer, DWORD buffer_size, DWORD* length_needed);

/* Enumeration */
typedef bool (*SFILE_ENUM_CALLBACK)(const char* filename, void* user_data);
bool SFileEnumFiles(HANDLE archive, const char* search_mask, const char* list_file, SFILE_ENUM_CALLBACK callback, void* user_data);

/* Utility functions */
LCID SFileSetLocale(LCID locale);
LCID SFileGetLocale(void);
DWORD SFileGetLastError(void);
void SFileSetLastError(DWORD error);

/* Verification functions */
bool SFileVerifyFile(HANDLE archive, const char* filename, DWORD flags);
DWORD SFileVerifyArchive(HANDLE archive);
bool SFileSignArchive(HANDLE archive, DWORD signature_type);
bool SFileGetAttributes(HANDLE archive);

#ifdef __cplusplus
}
#endif

#endif /* STORMLIB_H */
