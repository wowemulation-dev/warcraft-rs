//! StormLib-compatible C API for the storm MPQ archive library

use libc::{c_char, c_void};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs;
use std::path::Path;
use std::ptr;
use std::sync::{LazyLock, Mutex};

use wow_mpq::{Archive, ArchiveBuilder, FormatVersion, ListfileOption};

/// Archive handle type
pub type HANDLE = *mut c_void;

/// Invalid handle value
pub const INVALID_HANDLE_VALUE: HANDLE = ptr::null_mut();

// Thread-safe handle management with lazy initialization
static NEXT_HANDLE: LazyLock<Mutex<usize>> = LazyLock::new(|| Mutex::new(1));
static ARCHIVES: LazyLock<Mutex<HashMap<usize, ArchiveHandle>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static FILES: LazyLock<Mutex<HashMap<usize, FileHandle>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// Thread-local error storage
thread_local! {
    static LAST_ERROR: RefCell<u32> = const { RefCell::new(ERROR_SUCCESS) };
    static LOCALE: RefCell<u32> = const { RefCell::new(LOCALE_NEUTRAL) };
}

// Internal handle structures
struct ArchiveHandle {
    archive: Archive,
    path: String,
}

struct FileHandle {
    archive_handle: usize,
    filename: String,
    data: Vec<u8>,
    position: usize,
    size: u64,
}

// Error codes (matching Windows/StormLib error codes)
const ERROR_SUCCESS: u32 = 0;
const ERROR_FILE_NOT_FOUND: u32 = 2;
const ERROR_ACCESS_DENIED: u32 = 5;
const ERROR_INVALID_HANDLE: u32 = 6;
const _ERROR_NOT_ENOUGH_MEMORY: u32 = 8;
const ERROR_INVALID_PARAMETER: u32 = 87;
const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
const ERROR_ALREADY_EXISTS: u32 = 183;
const ERROR_FILE_CORRUPT: u32 = 1392;
const ERROR_NOT_SUPPORTED: u32 = 50;

// Locale constants
const LOCALE_NEUTRAL: u32 = 0;

// Search scope flags (for SFileOpenFileEx)
const _SFILE_OPEN_FROM_MPQ: u32 = 0x00000000;

// Verification flags (for SFileVerifyFile and SFileVerifyArchive)
const SFILE_VERIFY_SECTOR_CRC: u32 = 0x01;
const SFILE_VERIFY_FILE_CRC: u32 = 0x02;
const SFILE_VERIFY_FILE_MD5: u32 = 0x04;
const _SFILE_VERIFY_RAW_MD5: u32 = 0x08;
const SFILE_VERIFY_SIGNATURE: u32 = 0x10;
const SFILE_VERIFY_ALL_FILES: u32 = 0x20;
const SFILE_VERIFY_ALL: u32 = 0xFF;

// Info classes for SFileGetFileInfo
const SFILE_INFO_ARCHIVE_SIZE: u32 = 1;
const SFILE_INFO_HASH_TABLE_SIZE: u32 = 2;
const SFILE_INFO_BLOCK_TABLE_SIZE: u32 = 3;
const SFILE_INFO_SECTOR_SIZE: u32 = 4;
const _SFILE_INFO_NUM_FILES: u32 = 5;
const _SFILE_INFO_STREAM_FLAGS: u32 = 6;
const SFILE_INFO_FILE_SIZE: u32 = 7;
const _SFILE_INFO_COMPRESSED_SIZE: u32 = 8;
const _SFILE_INFO_FLAGS: u32 = 9;
const SFILE_INFO_POSITION: u32 = 10;
const _SFILE_INFO_KEY: u32 = 11;
const _SFILE_INFO_KEY_UNFIXED: u32 = 12;

// Archive open flags
const _MPQ_OPEN_NO_LISTFILE: u32 = 0x0001;
const _MPQ_OPEN_NO_ATTRIBUTES: u32 = 0x0002;
const _MPQ_OPEN_FORCE_MPQ_V1: u32 = 0x0004;
const _MPQ_OPEN_CHECK_SECTOR_CRC: u32 = 0x0008;

// Archive creation flags (for SFileCreateArchive)
const CREATE_NEW: u32 = 1;
const CREATE_ALWAYS: u32 = 2;
const OPEN_EXISTING: u32 = 3;
const OPEN_ALWAYS: u32 = 4;
const TRUNCATE_EXISTING: u32 = 5;

// MPQ format version flags
const _MPQ_CREATE_ARCHIVE_V1: u32 = 0x00000000;
const _MPQ_CREATE_ARCHIVE_V2: u32 = 0x01000000;
const _MPQ_CREATE_ARCHIVE_V3: u32 = 0x02000000;
const _MPQ_CREATE_ARCHIVE_V4: u32 = 0x03000000;

// Helper functions
fn set_last_error(error: u32) {
    LAST_ERROR.with(|e| *e.borrow_mut() = error);
}

fn handle_to_id(handle: HANDLE) -> Option<usize> {
    if handle.is_null() {
        None
    } else {
        Some(handle as usize)
    }
}

fn id_to_handle(id: usize) -> HANDLE {
    id as HANDLE
}

/// Open an MPQ archive
///
/// # Safety
///
/// - `filename` must be a valid null-terminated C string
/// - `handle` must be a valid pointer to write the output handle
#[no_mangle]
pub unsafe extern "C" fn SFileOpenArchive(
    filename: *const c_char,
    _priority: u32, // Ignored - StormLib legacy parameter
    _flags: u32,    // Archive open flags
    handle: *mut HANDLE,
) -> bool {
    // Validate parameters
    if filename.is_null() || handle.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    // Convert filename from C string
    let filename_str = match CStr::from_ptr(filename).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    };

    // Open the archive
    match Archive::open(filename_str) {
        Ok(archive) => {
            // Generate new handle ID
            let mut next_id = NEXT_HANDLE.lock().unwrap();
            let handle_id = *next_id;
            *next_id += 1;
            drop(next_id);

            // Store archive
            let archive_handle = ArchiveHandle {
                archive,
                path: filename_str.to_string(),
            };
            ARCHIVES.lock().unwrap().insert(handle_id, archive_handle);

            // Return handle
            *handle = id_to_handle(handle_id);
            set_last_error(ERROR_SUCCESS);
            true
        }
        Err(e) => {
            // Map wow_mpq errors to Windows error codes
            let error_code = match e {
                wow_mpq::Error::FileNotFound(_) => ERROR_FILE_NOT_FOUND,
                wow_mpq::Error::InvalidFormat(_) => ERROR_FILE_CORRUPT,
                wow_mpq::Error::Io(_) => ERROR_ACCESS_DENIED,
                _ => ERROR_FILE_CORRUPT,
            };
            set_last_error(error_code);
            false
        }
    }
}

/// Create a new MPQ archive
///
/// # Safety
///
/// - `filename` must be a valid null-terminated C string
/// - `handle` must be a valid pointer to write the output handle
#[no_mangle]
pub unsafe extern "C" fn SFileCreateArchive(
    filename: *const c_char,
    creation_disposition: u32,
    hash_table_size: u32,
    handle: *mut HANDLE,
) -> bool {
    // Validate parameters
    if filename.is_null() || handle.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    // Validate hash table size (must be power of 2)
    if !wow_mpq::is_power_of_two(hash_table_size) || hash_table_size < 4 {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    // Convert filename from C string
    let filename_str = match CStr::from_ptr(filename).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    };

    // Check if file exists for creation disposition handling
    let file_exists = Path::new(filename_str).exists();
    match creation_disposition {
        CREATE_NEW => {
            if file_exists {
                set_last_error(ERROR_ALREADY_EXISTS);
                return false;
            }
        }
        CREATE_ALWAYS => {
            // Always create, overwrite if exists
        }
        OPEN_EXISTING => {
            // This should use SFileOpenArchive instead
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
        OPEN_ALWAYS => {
            if file_exists {
                // Open existing archive
                return SFileOpenArchive(filename, 0, 0, handle);
            }
            // Create new if doesn't exist
        }
        TRUNCATE_EXISTING => {
            if !file_exists {
                set_last_error(ERROR_FILE_NOT_FOUND);
                return false;
            }
            // Truncate and recreate
        }
        _ => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    }

    // Determine MPQ format version (default to V2 for compatibility)
    // TODO: Allow V4 format selection via flags
    let version = FormatVersion::V2;

    // Create the archive using ArchiveBuilder
    // Note: hash_table_size is automatically calculated by ArchiveBuilder
    // based on the number of files added. For now we create a minimal archive.
    let result = ArchiveBuilder::new()
        .version(version)
        .listfile_option(ListfileOption::Generate)
        .build(filename_str);

    match result {
        Ok(_) => {
            // Archive created successfully, now open it for use
            SFileOpenArchive(filename, 0, 0, handle)
        }
        Err(e) => {
            // Map creation errors to Windows error codes
            let error_code = match e {
                wow_mpq::Error::FileNotFound(_) => ERROR_FILE_NOT_FOUND,
                wow_mpq::Error::InvalidFormat(_) => ERROR_INVALID_PARAMETER,
                wow_mpq::Error::Io(_) => ERROR_ACCESS_DENIED,
                _ => ERROR_ACCESS_DENIED,
            };
            set_last_error(error_code);
            false
        }
    }
}

/// Close an MPQ archive
#[no_mangle]
pub extern "C" fn SFileCloseArchive(handle: HANDLE) -> bool {
    if let Some(handle_id) = handle_to_id(handle) {
        // Remove any open files from this archive
        FILES
            .lock()
            .unwrap()
            .retain(|_, file| file.archive_handle != handle_id);

        // Close the archive
        if ARCHIVES.lock().unwrap().remove(&handle_id).is_some() {
            set_last_error(ERROR_SUCCESS);
            true
        } else {
            set_last_error(ERROR_INVALID_HANDLE);
            false
        }
    } else {
        set_last_error(ERROR_INVALID_HANDLE);
        false
    }
}

/// Open a file in the archive
///
/// # Safety
///
/// - `filename` must be a valid null-terminated C string
/// - `file_handle` must be a valid pointer to write the output handle
#[no_mangle]
pub unsafe extern "C" fn SFileOpenFileEx(
    archive: HANDLE,
    filename: *const c_char,
    _search_scope: u32,
    file_handle: *mut HANDLE,
) -> bool {
    // Validate parameters
    if filename.is_null() || file_handle.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    let Some(archive_id) = handle_to_id(archive) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    // Convert filename
    let filename_str = match CStr::from_ptr(filename).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    };

    // Get the archive
    let mut archives = ARCHIVES.lock().unwrap();
    let Some(archive_handle) = archives.get_mut(&archive_id) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    // Try to find and read the file
    match archive_handle.archive.find_file(filename_str) {
        Ok(Some(file_info)) => {
            // Read the file data
            match archive_handle.archive.read_file(filename_str) {
                Ok(data) => {
                    // Generate file handle
                    let mut next_id = NEXT_HANDLE.lock().unwrap();
                    let file_id = *next_id;
                    *next_id += 1;
                    drop(next_id);

                    // Create file handle
                    let file = FileHandle {
                        archive_handle: archive_id,
                        filename: filename_str.to_string(),
                        data,
                        position: 0,
                        size: file_info.file_size,
                    };

                    // Store file handle
                    FILES.lock().unwrap().insert(file_id, file);

                    // Return handle
                    *file_handle = id_to_handle(file_id);
                    set_last_error(ERROR_SUCCESS);
                    true
                }
                Err(_) => {
                    set_last_error(ERROR_FILE_CORRUPT);
                    false
                }
            }
        }
        Ok(None) => {
            set_last_error(ERROR_FILE_NOT_FOUND);
            false
        }
        Err(_) => {
            set_last_error(ERROR_FILE_CORRUPT);
            false
        }
    }
}

/// Close a file
#[no_mangle]
pub extern "C" fn SFileCloseFile(file: HANDLE) -> bool {
    if let Some(file_id) = handle_to_id(file) {
        if FILES.lock().unwrap().remove(&file_id).is_some() {
            set_last_error(ERROR_SUCCESS);
            true
        } else {
            set_last_error(ERROR_INVALID_HANDLE);
            false
        }
    } else {
        set_last_error(ERROR_INVALID_HANDLE);
        false
    }
}

/// Read from a file
///
/// # Safety
///
/// - `buffer` must be a valid pointer with at least `to_read` bytes available
/// - `read` if not null, must be a valid pointer to write the bytes read
#[no_mangle]
pub unsafe extern "C" fn SFileReadFile(
    file: HANDLE,
    buffer: *mut c_void,
    to_read: u32,
    read: *mut u32,
    _overlapped: *mut c_void, // Ignored - no async I/O support
) -> bool {
    // Validate parameters
    if buffer.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    let Some(file_id) = handle_to_id(file) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    // Get file handle
    let mut files = FILES.lock().unwrap();
    let Some(file_handle) = files.get_mut(&file_id) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    // Calculate how much we can read
    let remaining = file_handle.data.len().saturating_sub(file_handle.position);
    let bytes_to_read = (to_read as usize).min(remaining);

    if bytes_to_read == 0 {
        if !read.is_null() {
            *read = 0;
        }
        set_last_error(ERROR_SUCCESS);
        return true;
    }

    // Copy data to buffer
    let src = &file_handle.data[file_handle.position..file_handle.position + bytes_to_read];
    std::ptr::copy_nonoverlapping(src.as_ptr(), buffer as *mut u8, bytes_to_read);

    // Update position
    file_handle.position += bytes_to_read;

    // Set bytes read
    if !read.is_null() {
        *read = bytes_to_read as u32;
    }

    set_last_error(ERROR_SUCCESS);
    true
}

/// Get file size
///
/// # Safety
///
/// - `high` if not null, must be a valid pointer to write the high 32 bits
#[no_mangle]
pub unsafe extern "C" fn SFileGetFileSize(file: HANDLE, high: *mut u32) -> u32 {
    let Some(file_id) = handle_to_id(file) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return 0xFFFFFFFF; // INVALID_FILE_SIZE
    };

    let files = FILES.lock().unwrap();
    let Some(file_handle) = files.get(&file_id) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return 0xFFFFFFFF;
    };

    let size = file_handle.size;

    // Split 64-bit size into high and low parts
    if !high.is_null() {
        *high = (size >> 32) as u32;
    }

    set_last_error(ERROR_SUCCESS);
    (size & 0xFFFFFFFF) as u32
}

/// Set file position
///
/// # Safety
///
/// - `file_pos_high` if not null, must be a valid pointer to read/write the high 32 bits
#[no_mangle]
pub unsafe extern "C" fn SFileSetFilePointer(
    file: HANDLE,
    file_pos: i32,
    file_pos_high: *mut i32,
    move_method: u32, // 0=FILE_BEGIN, 1=FILE_CURRENT, 2=FILE_END
) -> u32 {
    let Some(file_id) = handle_to_id(file) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return 0xFFFFFFFF; // INVALID_SET_FILE_POINTER
    };

    let mut files = FILES.lock().unwrap();
    let Some(file_handle) = files.get_mut(&file_id) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return 0xFFFFFFFF;
    };

    // Combine high and low parts into 64-bit offset
    let mut offset = file_pos as i64;
    if !file_pos_high.is_null() {
        let high = *file_pos_high as i64;
        offset |= high << 32;
    }

    // Calculate new position
    let new_pos = match move_method {
        0 => offset as usize,                                   // FILE_BEGIN
        1 => (file_handle.position as i64 + offset) as usize,   // FILE_CURRENT
        2 => (file_handle.data.len() as i64 + offset) as usize, // FILE_END
        _ => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return 0xFFFFFFFF;
        }
    };

    // Clamp to file size
    file_handle.position = new_pos.min(file_handle.data.len());

    // Return new position
    let pos = file_handle.position as u64;
    if !file_pos_high.is_null() {
        *file_pos_high = (pos >> 32) as i32;
    }

    set_last_error(ERROR_SUCCESS);
    (pos & 0xFFFFFFFF) as u32
}

/// Check if file exists in archive
///
/// # Safety
///
/// - `filename` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn SFileHasFile(archive: HANDLE, filename: *const c_char) -> bool {
    if filename.is_null() {
        return false;
    }

    let Some(archive_id) = handle_to_id(archive) else {
        return false;
    };

    let filename_str = match CStr::from_ptr(filename).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let archives = ARCHIVES.lock().unwrap();
    if let Some(archive_handle) = archives.get(&archive_id) {
        matches!(archive_handle.archive.find_file(filename_str), Ok(Some(_)))
    } else {
        false
    }
}

/// Get file information
///
/// # Safety
///
/// - `buffer` if not null, must be a valid pointer with at least `buffer_size` bytes
/// - `size_needed` if not null, must be a valid pointer to write the required size
#[no_mangle]
pub unsafe extern "C" fn SFileGetFileInfo(
    file_or_archive: HANDLE,
    info_class: u32,
    buffer: *mut c_void,
    buffer_size: u32,
    size_needed: *mut u32,
) -> bool {
    if buffer.is_null() && buffer_size > 0 {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    // Check if this is a file or archive handle
    let handle_id = match handle_to_id(file_or_archive) {
        Some(id) => id,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return false;
        }
    };

    // Try as file first
    if let Some(file_handle) = FILES.lock().unwrap().get(&handle_id) {
        return get_file_info(file_handle, info_class, buffer, buffer_size, size_needed);
    }

    // Try as archive
    if let Some(archive_handle) = ARCHIVES.lock().unwrap().get(&handle_id) {
        return get_archive_info(archive_handle, info_class, buffer, buffer_size, size_needed);
    }

    set_last_error(ERROR_INVALID_HANDLE);
    false
}

// Helper function for file info
unsafe fn get_file_info(
    file_handle: &FileHandle,
    info_class: u32,
    buffer: *mut c_void,
    buffer_size: u32,
    size_needed: *mut u32,
) -> bool {
    match info_class {
        SFILE_INFO_FILE_SIZE => {
            let needed = 8u32;
            if !size_needed.is_null() {
                *size_needed = needed;
            }
            if buffer_size >= needed {
                *(buffer as *mut u64) = file_handle.size;
                set_last_error(ERROR_SUCCESS);
                true
            } else {
                set_last_error(ERROR_INSUFFICIENT_BUFFER);
                false
            }
        }
        SFILE_INFO_POSITION => {
            let needed = 8u32;
            if !size_needed.is_null() {
                *size_needed = needed;
            }
            if buffer_size >= needed {
                *(buffer as *mut u64) = file_handle.position as u64;
                set_last_error(ERROR_SUCCESS);
                true
            } else {
                set_last_error(ERROR_INSUFFICIENT_BUFFER);
                false
            }
        }
        _ => {
            set_last_error(ERROR_NOT_SUPPORTED);
            false
        }
    }
}

// Helper function for archive info
unsafe fn get_archive_info(
    archive_handle: &ArchiveHandle,
    info_class: u32,
    buffer: *mut c_void,
    buffer_size: u32,
    size_needed: *mut u32,
) -> bool {
    let header = archive_handle.archive.header();

    match info_class {
        SFILE_INFO_ARCHIVE_SIZE => {
            let needed = 8u32;
            if !size_needed.is_null() {
                *size_needed = needed;
            }
            if buffer_size >= needed {
                *(buffer as *mut u64) = header.get_archive_size();
                set_last_error(ERROR_SUCCESS);
                true
            } else {
                set_last_error(ERROR_INSUFFICIENT_BUFFER);
                false
            }
        }
        SFILE_INFO_HASH_TABLE_SIZE => {
            let needed = 4u32;
            if !size_needed.is_null() {
                *size_needed = needed;
            }
            if buffer_size >= needed {
                *(buffer as *mut u32) = header.hash_table_size;
                set_last_error(ERROR_SUCCESS);
                true
            } else {
                set_last_error(ERROR_INSUFFICIENT_BUFFER);
                false
            }
        }
        SFILE_INFO_BLOCK_TABLE_SIZE => {
            let needed = 4u32;
            if !size_needed.is_null() {
                *size_needed = needed;
            }
            if buffer_size >= needed {
                *(buffer as *mut u32) = header.block_table_size;
                set_last_error(ERROR_SUCCESS);
                true
            } else {
                set_last_error(ERROR_INSUFFICIENT_BUFFER);
                false
            }
        }
        SFILE_INFO_SECTOR_SIZE => {
            let needed = 4u32;
            if !size_needed.is_null() {
                *size_needed = needed;
            }
            if buffer_size >= needed {
                *(buffer as *mut u32) = header.sector_size() as u32;
                set_last_error(ERROR_SUCCESS);
                true
            } else {
                set_last_error(ERROR_INSUFFICIENT_BUFFER);
                false
            }
        }
        _ => {
            set_last_error(ERROR_NOT_SUPPORTED);
            false
        }
    }
}

/// Get archive name from handle
///
/// # Safety
///
/// - `buffer` must be a valid pointer with at least `buffer_size` bytes available
#[no_mangle]
pub unsafe extern "C" fn SFileGetArchiveName(
    archive: HANDLE,
    buffer: *mut c_char,
    buffer_size: u32,
) -> bool {
    if buffer.is_null() || buffer_size == 0 {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    let Some(archive_id) = handle_to_id(archive) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    let archives = ARCHIVES.lock().unwrap();
    let Some(archive_handle) = archives.get(&archive_id) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    // Convert path to C string
    let c_path = match CString::new(archive_handle.path.as_str()) {
        Ok(s) => s,
        Err(_) => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    };

    let path_bytes = c_path.as_bytes_with_nul();
    if path_bytes.len() > buffer_size as usize {
        set_last_error(ERROR_INSUFFICIENT_BUFFER);
        return false;
    }

    // Copy to buffer
    std::ptr::copy_nonoverlapping(path_bytes.as_ptr(), buffer as *mut u8, path_bytes.len());

    set_last_error(ERROR_SUCCESS);
    true
}

/// Enumerate files in archive
///
/// # Safety
///
/// - `search_mask` if not null, must be a valid null-terminated C string
/// - `_list_file` if not null, must be a valid null-terminated C string (ignored)
/// - `callback` function pointer must be valid for the duration of enumeration
#[no_mangle]
pub unsafe extern "C" fn SFileEnumFiles(
    archive: HANDLE,
    search_mask: *const c_char,
    _list_file: *const c_char, // Path to external listfile (ignored for now)
    callback: Option<extern "C" fn(*const c_char, *mut c_void) -> bool>,
    user_data: *mut c_void,
) -> bool {
    let Some(callback_fn) = callback else {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    };

    let Some(archive_id) = handle_to_id(archive) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    // Get search pattern
    let pattern = if search_mask.is_null() {
        "*".to_string()
    } else {
        match CStr::from_ptr(search_mask).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                set_last_error(ERROR_INVALID_PARAMETER);
                return false;
            }
        }
    };

    // Get archive
    let mut archives = ARCHIVES.lock().unwrap();
    let Some(archive_handle) = archives.get_mut(&archive_id) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    // List files
    match archive_handle.archive.list() {
        Ok(entries) => {
            for entry in entries {
                // Simple pattern matching (just * for now)
                if pattern == "*" || entry.name.contains(&pattern.replace("*", "")) {
                    let c_name = match CString::new(entry.name.as_str()) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };

                    // Call the callback
                    if !callback_fn(c_name.as_ptr(), user_data) {
                        // Callback returned false, stop enumeration
                        break;
                    }
                }
            }
            set_last_error(ERROR_SUCCESS);
            true
        }
        Err(_) => {
            // No listfile, but that's okay
            set_last_error(ERROR_SUCCESS);
            true
        }
    }
}

/// Set locale for file operations
#[no_mangle]
pub extern "C" fn SFileSetLocale(locale: u32) -> u32 {
    let old_locale = LOCALE.with(|l| {
        let old = *l.borrow();
        *l.borrow_mut() = locale;
        old
    });
    old_locale
}

/// Get current locale
#[no_mangle]
pub extern "C" fn SFileGetLocale() -> u32 {
    LOCALE.with(|l| *l.borrow())
}

/// Get last error
#[no_mangle]
pub extern "C" fn SFileGetLastError() -> u32 {
    LAST_ERROR.with(|e| *e.borrow())
}

/// Set last error
#[no_mangle]
pub extern "C" fn SFileSetLastError(error: u32) {
    set_last_error(error);
}

// Additional utility functions

/// Get file name from handle
///
/// # Safety
///
/// - `buffer` must be a valid pointer with sufficient space for the filename
#[no_mangle]
pub unsafe extern "C" fn SFileGetFileName(file: HANDLE, buffer: *mut c_char) -> bool {
    if buffer.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    let Some(file_id) = handle_to_id(file) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    let files = FILES.lock().unwrap();
    let Some(file_handle) = files.get(&file_id) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    let c_name = match CString::new(file_handle.filename.as_str()) {
        Ok(s) => s,
        Err(_) => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    };

    std::ptr::copy_nonoverlapping(c_name.as_ptr(), buffer, c_name.as_bytes_with_nul().len());

    set_last_error(ERROR_SUCCESS);
    true
}

/// Extract a file from archive to disk
///
/// # Safety
///
/// - `filename` must be a valid null-terminated C string
/// - `local_filename` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn SFileExtractFile(
    archive: HANDLE,
    filename: *const c_char,
    local_filename: *const c_char,
    _search_scope: u32, // Ignored - StormLib legacy parameter
) -> bool {
    // Validate parameters
    if filename.is_null() || local_filename.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    let Some(archive_id) = handle_to_id(archive) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    // Convert filenames from C strings
    let source_filename = match CStr::from_ptr(filename).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    };

    let dest_filename = match CStr::from_ptr(local_filename).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    };

    // Get the archive
    let mut archives = ARCHIVES.lock().unwrap();
    let Some(archive_handle) = archives.get_mut(&archive_id) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    // Try to read the file from the archive
    match archive_handle.archive.read_file(source_filename) {
        Ok(data) => {
            // Create parent directories if they don't exist
            let dest_path = Path::new(dest_filename);
            if let Some(parent) = dest_path.parent() {
                if fs::create_dir_all(parent).is_err() {
                    set_last_error(ERROR_ACCESS_DENIED);
                    return false;
                }
            }

            // Write the file data to disk
            match fs::write(dest_path, data) {
                Ok(_) => {
                    set_last_error(ERROR_SUCCESS);
                    true
                }
                Err(_) => {
                    set_last_error(ERROR_ACCESS_DENIED);
                    false
                }
            }
        }
        Err(e) => {
            // Map wow_mpq errors to Windows error codes
            let error_code = match e {
                wow_mpq::Error::FileNotFound(_) => ERROR_FILE_NOT_FOUND,
                wow_mpq::Error::InvalidFormat(_) => ERROR_FILE_CORRUPT,
                wow_mpq::Error::Io(_) => ERROR_ACCESS_DENIED,
                _ => ERROR_FILE_CORRUPT,
            };
            set_last_error(error_code);
            false
        }
    }
}

/// Verify file integrity
///
/// # Safety
///
/// - `filename` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn SFileVerifyFile(
    archive: HANDLE,
    filename: *const c_char,
    flags: u32,
) -> bool {
    // Validate parameters
    if filename.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    let Some(archive_id) = handle_to_id(archive) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    // Convert filename from C string
    let filename_str = match CStr::from_ptr(filename).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    };

    // Get the archive
    let mut archives = ARCHIVES.lock().unwrap();
    let Some(archive_handle) = archives.get_mut(&archive_id) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    // Find the file first to get file info
    let file_info = match archive_handle.archive.find_file(filename_str) {
        Ok(Some(info)) => info,
        Ok(None) => {
            set_last_error(ERROR_FILE_NOT_FOUND);
            return false;
        }
        Err(_) => {
            set_last_error(ERROR_FILE_CORRUPT);
            return false;
        }
    };

    // If no flags specified, verify everything available
    let verify_flags = if flags == 0 { SFILE_VERIFY_ALL } else { flags };

    // Verify sector CRC (if requested and available)
    if (verify_flags & SFILE_VERIFY_SECTOR_CRC) != 0 && file_info.has_sector_crc() {
        // Reading the file automatically validates sector CRCs
        match archive_handle.archive.read_file(filename_str) {
            Ok(_) => {
                // File read successfully, CRCs validated automatically
            }
            Err(wow_mpq::Error::ChecksumMismatch { .. }) => {
                set_last_error(ERROR_FILE_CORRUPT);
                return false;
            }
            Err(_) => {
                set_last_error(ERROR_FILE_CORRUPT);
                return false;
            }
        }
    }

    // Verify file CRC32 from attributes (if requested and available)
    if (verify_flags & SFILE_VERIFY_FILE_CRC) != 0 {
        // Load attributes if not already loaded
        let _ = archive_handle.archive.load_attributes();

        if let Some(attrs) = archive_handle
            .archive
            .get_file_attributes(file_info.block_index)
        {
            if let Some(expected_crc) = attrs.crc32 {
                // Read file data to calculate CRC
                match archive_handle.archive.read_file(filename_str) {
                    Ok(data) => {
                        let actual_crc = crc32fast::hash(&data);
                        if actual_crc != expected_crc {
                            set_last_error(ERROR_FILE_CORRUPT);
                            return false;
                        }
                    }
                    Err(_) => {
                        set_last_error(ERROR_FILE_CORRUPT);
                        return false;
                    }
                }
            }
        }
    }

    // Verify file MD5 from attributes (if requested and available)
    if (verify_flags & SFILE_VERIFY_FILE_MD5) != 0 {
        // Load attributes if not already loaded
        let _ = archive_handle.archive.load_attributes();

        if let Some(attrs) = archive_handle
            .archive
            .get_file_attributes(file_info.block_index)
        {
            if let Some(expected_md5) = attrs.md5 {
                // Read file data to calculate MD5
                match archive_handle.archive.read_file(filename_str) {
                    Ok(data) => {
                        use md5::{Digest, Md5};
                        let mut hasher = Md5::new();
                        hasher.update(&data);
                        let actual_md5: [u8; 16] = hasher.finalize().into();
                        if actual_md5 != expected_md5 {
                            set_last_error(ERROR_FILE_CORRUPT);
                            return false;
                        }
                    }
                    Err(_) => {
                        set_last_error(ERROR_FILE_CORRUPT);
                        return false;
                    }
                }
            }
        }
    }

    set_last_error(ERROR_SUCCESS);
    true
}

/// Verify archive integrity
///
/// # Safety
///
/// This function is unsafe because it:
/// - Dereferences raw pointers passed as HANDLE arguments
/// - Calls other unsafe functions that manipulate raw pointers
/// - Must be called with valid archive handles obtained from `SFileOpenArchive`
#[no_mangle]
pub unsafe extern "C" fn SFileVerifyArchive(archive: HANDLE, flags: u32) -> bool {
    let Some(archive_id) = handle_to_id(archive) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    // Get the archive
    let mut archives = ARCHIVES.lock().unwrap();
    let Some(archive_handle) = archives.get_mut(&archive_id) else {
        set_last_error(ERROR_INVALID_HANDLE);
        return false;
    };

    // If no flags specified, verify signature by default
    let verify_flags = if flags == 0 {
        SFILE_VERIFY_SIGNATURE
    } else {
        flags
    };

    // Verify digital signature (if requested)
    if (verify_flags & SFILE_VERIFY_SIGNATURE) != 0 {
        match archive_handle.archive.verify_signature() {
            Ok(wow_mpq::SignatureStatus::WeakValid) | Ok(wow_mpq::SignatureStatus::StrongValid) => {
                // Valid signature found
            }
            Ok(wow_mpq::SignatureStatus::None) => {
                // No signature is acceptable for some archives
            }
            Ok(wow_mpq::SignatureStatus::WeakInvalid)
            | Ok(wow_mpq::SignatureStatus::StrongInvalid) => {
                set_last_error(ERROR_FILE_CORRUPT);
                return false;
            }
            Ok(wow_mpq::SignatureStatus::StrongNoKey) => {
                // Strong signature present but no key - treat as warning, continue
            }
            Err(_) => {
                set_last_error(ERROR_FILE_CORRUPT);
                return false;
            }
        }
    }

    // Verify all files in the archive (if requested)
    if (verify_flags & SFILE_VERIFY_ALL_FILES) != 0 {
        // Get list of all files in the archive
        let file_list = match archive_handle.archive.list() {
            Ok(list) => list,
            Err(_) => {
                // If we can't get the file list, try to enumerate from tables
                match archive_handle.archive.list_all() {
                    Ok(list) => list,
                    Err(_) => {
                        set_last_error(ERROR_FILE_CORRUPT);
                        return false;
                    }
                }
            }
        };

        // Verify each file individually
        for file_entry in file_list {
            // Skip special files and directories
            if file_entry.name.starts_with('(') && file_entry.name.ends_with(')') {
                continue;
            }
            if file_entry.name.ends_with('/') || file_entry.name.ends_with('\\') {
                continue;
            }

            // Determine what verification to perform based on file flags
            let file_verify_flags = if file_entry.has_sector_crc() {
                SFILE_VERIFY_SECTOR_CRC
            } else {
                0
            };

            // Add CRC/MD5 verification if attributes are available
            let all_verify_flags =
                file_verify_flags | SFILE_VERIFY_FILE_CRC | SFILE_VERIFY_FILE_MD5;

            // Use our own SFileVerifyFile function for consistency
            unsafe {
                let filename_cstr = match std::ffi::CString::new(file_entry.name.as_str()) {
                    Ok(s) => s,
                    Err(_) => continue, // Skip files with invalid names
                };

                if !SFileVerifyFile(archive, filename_cstr.as_ptr(), all_verify_flags) {
                    let last_error = SFileGetLastError();
                    // Only fail on corruption, not missing attributes
                    if last_error == ERROR_FILE_CORRUPT {
                        return false;
                    }
                }
            }
        }
    }

    set_last_error(ERROR_SUCCESS);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_conversion() {
        let id = 42usize;
        let handle = id_to_handle(id);
        assert_eq!(handle_to_id(handle), Some(id));
        assert_eq!(handle_to_id(ptr::null_mut()), None);
    }

    #[test]
    fn test_error_handling() {
        set_last_error(ERROR_FILE_NOT_FOUND);
        assert_eq!(SFileGetLastError(), ERROR_FILE_NOT_FOUND);

        SFileSetLastError(ERROR_SUCCESS);
        assert_eq!(SFileGetLastError(), ERROR_SUCCESS);
    }

    #[test]
    fn test_locale() {
        let old = SFileSetLocale(0x409); // US English
        assert_eq!(old, LOCALE_NEUTRAL);
        assert_eq!(SFileGetLocale(), 0x409);

        SFileSetLocale(old); // Restore
    }

    #[test]
    fn test_extract_file_invalid_params() {
        // Test SFileExtractFile with invalid parameters
        unsafe {
            // Null filename should fail
            assert!(!SFileExtractFile(
                ptr::null_mut(),
                ptr::null(),
                c"output.txt".as_ptr(),
                0
            ));
            assert_eq!(SFileGetLastError(), ERROR_INVALID_PARAMETER);

            // Null local filename should fail
            assert!(!SFileExtractFile(
                ptr::null_mut(),
                c"test.txt".as_ptr(),
                ptr::null(),
                0
            ));
            assert_eq!(SFileGetLastError(), ERROR_INVALID_PARAMETER);

            // Invalid handle should fail
            assert!(!SFileExtractFile(
                ptr::null_mut(),
                c"test.txt".as_ptr(),
                c"output.txt".as_ptr(),
                0
            ));
            assert_eq!(SFileGetLastError(), ERROR_INVALID_HANDLE);
        }
    }

    #[test]
    fn test_verify_file_invalid_params() {
        // Test SFileVerifyFile with invalid parameters
        unsafe {
            // Null filename should fail
            assert!(!SFileVerifyFile(
                ptr::null_mut(),
                ptr::null(),
                SFILE_VERIFY_ALL
            ));
            assert_eq!(SFileGetLastError(), ERROR_INVALID_PARAMETER);

            // Invalid handle should fail
            assert!(!SFileVerifyFile(
                ptr::null_mut(),
                c"test.txt".as_ptr(),
                SFILE_VERIFY_ALL
            ));
            assert_eq!(SFileGetLastError(), ERROR_INVALID_HANDLE);
        }
    }

    #[test]
    fn test_verify_archive_invalid_params() {
        // Test SFileVerifyArchive with invalid parameters
        // Invalid handle should fail
        unsafe {
            assert!(!SFileVerifyArchive(ptr::null_mut(), SFILE_VERIFY_SIGNATURE));
        }
        assert_eq!(SFileGetLastError(), ERROR_INVALID_HANDLE);
    }

    #[test]
    fn test_create_archive_invalid_params() {
        // Test SFileCreateArchive with invalid parameters
        unsafe {
            let mut handle = ptr::null_mut();

            // Null filename should fail
            assert!(!SFileCreateArchive(
                ptr::null(),
                CREATE_ALWAYS,
                16,
                &mut handle
            ));
            assert_eq!(SFileGetLastError(), ERROR_INVALID_PARAMETER);

            // Null handle should fail
            assert!(!SFileCreateArchive(
                c"test.mpq".as_ptr(),
                CREATE_ALWAYS,
                16,
                ptr::null_mut()
            ));
            assert_eq!(SFileGetLastError(), ERROR_INVALID_PARAMETER);

            // Invalid hash table size (not power of 2) should fail
            assert!(!SFileCreateArchive(
                c"test.mpq".as_ptr(),
                CREATE_ALWAYS,
                15, // Not a power of 2
                &mut handle
            ));
            assert_eq!(SFileGetLastError(), ERROR_INVALID_PARAMETER);

            // Hash table size too small should fail
            assert!(!SFileCreateArchive(
                c"test.mpq".as_ptr(),
                CREATE_ALWAYS,
                2, // Less than minimum of 4
                &mut handle
            ));
            assert_eq!(SFileGetLastError(), ERROR_INVALID_PARAMETER);
        }
    }
}
