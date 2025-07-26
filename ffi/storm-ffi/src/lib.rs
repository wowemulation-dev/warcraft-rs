//! StormLib-compatible C API for the storm MPQ archive library

use libc::{c_char, c_void};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs;
use std::path::Path;
use std::ptr;
use std::sync::{LazyLock, Mutex};

use wow_mpq::{
    AddFileOptions, Archive, ArchiveBuilder, AttributesOption, FileEntry, FormatVersion,
    ListfileOption, MutableArchive,
};

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
static FIND_HANDLES: LazyLock<Mutex<HashMap<usize, FindHandle>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// Thread-local error storage
thread_local! {
    static LAST_ERROR: RefCell<u32> = const { RefCell::new(ERROR_SUCCESS) };
    static LOCALE: RefCell<u32> = const { RefCell::new(LOCALE_NEUTRAL) };
}

// Internal handle structures
#[allow(clippy::large_enum_variant)]
enum ArchiveHandle {
    ReadOnly {
        archive: Archive,
        path: String,
    },
    Mutable {
        archive: MutableArchive,
        path: String,
    },
}

struct FileHandle {
    archive_handle: usize,
    filename: String,
    data: Vec<u8>,
    position: usize,
    size: u64,
}

impl ArchiveHandle {
    /// Get read-only access to the archive
    fn archive(&self) -> &Archive {
        match self {
            ArchiveHandle::ReadOnly { archive, .. } => archive,
            ArchiveHandle::Mutable { archive, .. } => archive.archive(),
        }
    }

    /// Get mutable access to the archive (if it's mutable)
    fn mutable_archive(&mut self) -> Option<&mut MutableArchive> {
        match self {
            ArchiveHandle::ReadOnly { .. } => None,
            ArchiveHandle::Mutable { archive, .. } => Some(archive),
        }
    }

    /// Get the path
    fn path(&self) -> &str {
        match self {
            ArchiveHandle::ReadOnly { path, .. } => path,
            ArchiveHandle::Mutable { path, .. } => path,
        }
    }
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
            let archive_handle = ArchiveHandle::ReadOnly {
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
    let (file_info_opt, read_result) = match archive_handle {
        ArchiveHandle::ReadOnly { archive, .. } => match archive.find_file(filename_str) {
            Ok(Some(file_info)) => {
                let read_result = archive.read_file(filename_str);
                (Some(file_info), read_result)
            }
            Ok(None) => (
                None,
                Err(wow_mpq::Error::FileNotFound(filename_str.to_string())),
            ),
            Err(e) => (None, Err(e)),
        },
        ArchiveHandle::Mutable { archive, .. } => match archive.find_file(filename_str) {
            Ok(Some(file_info)) => {
                let read_result = archive.read_file(filename_str);
                (Some(file_info), read_result)
            }
            Ok(None) => (
                None,
                Err(wow_mpq::Error::FileNotFound(filename_str.to_string())),
            ),
            Err(e) => (None, Err(e)),
        },
    };

    if let Some(file_info) = file_info_opt {
        match read_result {
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
    } else {
        set_last_error(ERROR_FILE_NOT_FOUND);
        false
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
        matches!(
            archive_handle.archive().find_file(filename_str),
            Ok(Some(_))
        )
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
    let header = archive_handle.archive().header();

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
    let c_path = match CString::new(archive_handle.path()) {
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
    let file_list = match archive_handle {
        ArchiveHandle::ReadOnly { archive, .. } => archive.list(),
        ArchiveHandle::Mutable { archive, .. } => archive.list(),
    };

    match file_list {
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
    let file_data = match archive_handle {
        ArchiveHandle::ReadOnly { archive, .. } => archive.read_file(source_filename),
        ArchiveHandle::Mutable { archive, .. } => archive.read_file(source_filename),
    };

    match file_data {
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
    let file_info = match archive_handle.archive().find_file(filename_str) {
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
        let read_result = match archive_handle {
            ArchiveHandle::ReadOnly { archive, .. } => archive.read_file(filename_str),
            ArchiveHandle::Mutable { archive, .. } => archive.read_file(filename_str),
        };
        match read_result {
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
        // Load attributes if possible
        match archive_handle {
            ArchiveHandle::ReadOnly { archive, .. } => {
                let _ = archive.load_attributes();
            }
            ArchiveHandle::Mutable { archive, .. } => {
                let _ = archive.load_attributes();
            }
        }

        if let Some(attrs) = archive_handle
            .archive()
            .get_file_attributes(file_info.block_index)
        {
            if let Some(expected_crc) = attrs.crc32 {
                // Read file data to calculate CRC
                let read_result = match archive_handle {
                    ArchiveHandle::ReadOnly { archive, .. } => archive.read_file(filename_str),
                    ArchiveHandle::Mutable { archive, .. } => archive.read_file(filename_str),
                };
                match read_result {
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
        // Load attributes if possible
        match archive_handle {
            ArchiveHandle::ReadOnly { archive, .. } => {
                let _ = archive.load_attributes();
            }
            ArchiveHandle::Mutable { archive, .. } => {
                let _ = archive.load_attributes();
            }
        }

        if let Some(attrs) = archive_handle
            .archive()
            .get_file_attributes(file_info.block_index)
        {
            if let Some(expected_md5) = attrs.md5 {
                // Read file data to calculate MD5
                let read_result = match archive_handle {
                    ArchiveHandle::ReadOnly { archive, .. } => archive.read_file(filename_str),
                    ArchiveHandle::Mutable { archive, .. } => archive.read_file(filename_str),
                };
                match read_result {
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
        // Verify signature
        let signature_result = match archive_handle {
            ArchiveHandle::ReadOnly { archive, .. } => archive.verify_signature(),
            ArchiveHandle::Mutable { archive, .. } => archive.verify_signature(),
        };

        match signature_result {
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
        let file_list = match archive_handle {
            ArchiveHandle::ReadOnly { archive, .. } => archive.list(),
            ArchiveHandle::Mutable { archive, .. } => archive.list(),
        };
        let file_list = file_list.unwrap_or_default();

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

                if !SFileVerifyFile(
                    id_to_handle(archive_id),
                    filename_cstr.as_ptr(),
                    all_verify_flags,
                ) {
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

/// Add a file to an archive from disk with extended options
///
/// # Safety
///
/// - `archive` must be a valid archive handle
/// - `filename` and `archived_name` must be valid null-terminated C strings
#[no_mangle]
pub unsafe extern "C" fn SFileAddFileEx(
    archive: HANDLE,
    filename: *const c_char,
    archived_name: *const c_char,
    flags: u32,
    compression: u32,
    _compression_next: u32,
) -> bool {
    // Validate parameters
    if archive.is_null() || filename.is_null() || archived_name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    // Convert handle
    let archive_id = match handle_to_id(archive) {
        Some(id) => id,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return false;
        }
    };

    // Convert strings
    let filename_str = match CStr::from_ptr(filename).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    };

    let archived_name_str = match CStr::from_ptr(archived_name).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    };

    // Get mutable archive handle
    let mut archives = ARCHIVES.lock().unwrap();
    let archive_handle = match archives.get_mut(&archive_id) {
        Some(handle) => handle,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return false;
        }
    };

    let mutable_archive = match archive_handle.mutable_archive() {
        Some(archive) => archive,
        None => {
            set_last_error(ERROR_ACCESS_DENIED);
            return false;
        }
    };

    // Configure options based on flags and compression
    let mut options = AddFileOptions::new();

    // Set compression method
    use wow_mpq::compression::CompressionMethod;
    let compression_method = match compression {
        0 => CompressionMethod::None,
        0x02 => CompressionMethod::Zlib,
        0x08 => CompressionMethod::PKWare,
        0x10 => CompressionMethod::BZip2,
        0x20 => CompressionMethod::Sparse,
        0x40 => CompressionMethod::AdpcmMono,
        0x80 => CompressionMethod::AdpcmStereo,
        0x12 => CompressionMethod::Lzma,
        _ => CompressionMethod::Zlib, // Default fallback
    };
    options = options.compression(compression_method);

    // Handle flags
    const MPQ_FILE_ENCRYPTED: u32 = 0x00010000;
    const MPQ_FILE_FIX_KEY: u32 = 0x00020000;
    const MPQ_FILE_REPLACEEXISTING: u32 = 0x80000000;

    if (flags & MPQ_FILE_ENCRYPTED) != 0 {
        options = options.encrypt();
    }
    if (flags & MPQ_FILE_FIX_KEY) != 0 {
        options = options.fix_key();
    }
    if (flags & MPQ_FILE_REPLACEEXISTING) != 0 {
        options = options.replace_existing(true);
    }

    // Add the file
    match mutable_archive.add_file(filename_str, archived_name_str, options) {
        Ok(()) => {
            set_last_error(ERROR_SUCCESS);
            true
        }
        Err(e) => {
            let error_code = match e {
                wow_mpq::Error::FileNotFound(_) => ERROR_FILE_NOT_FOUND,
                wow_mpq::Error::FileExists(_) => ERROR_ALREADY_EXISTS,
                wow_mpq::Error::Io(_) => ERROR_ACCESS_DENIED,
                _ => ERROR_ACCESS_DENIED,
            };
            set_last_error(error_code);
            false
        }
    }
}

/// Add a file to an archive from disk
///
/// # Safety
///
/// - `archive` must be a valid archive handle
/// - `filename` and `archived_name` must be valid null-terminated C strings
#[no_mangle]
pub unsafe extern "C" fn SFileAddFile(
    archive: HANDLE,
    filename: *const c_char,
    archived_name: *const c_char,
    flags: u32,
) -> bool {
    SFileAddFileEx(archive, filename, archived_name, flags, 0x02, 0) // Use ZLIB compression by default
}

/// Remove a file from an archive
///
/// # Safety
///
/// - `archive` must be a valid archive handle
/// - `filename` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn SFileRemoveFile(
    archive: HANDLE,
    filename: *const c_char,
    _search_scope: u32,
) -> bool {
    // Validate parameters
    if archive.is_null() || filename.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    // Convert handle
    let archive_id = match handle_to_id(archive) {
        Some(id) => id,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return false;
        }
    };

    // Convert filename
    let filename_str = match CStr::from_ptr(filename).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    };

    // Get mutable archive handle
    let mut archives = ARCHIVES.lock().unwrap();
    let archive_handle = match archives.get_mut(&archive_id) {
        Some(handle) => handle,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return false;
        }
    };

    let mutable_archive = match archive_handle.mutable_archive() {
        Some(archive) => archive,
        None => {
            set_last_error(ERROR_ACCESS_DENIED);
            return false;
        }
    };

    // Remove the file
    match mutable_archive.remove_file(filename_str) {
        Ok(()) => {
            set_last_error(ERROR_SUCCESS);
            true
        }
        Err(e) => {
            let error_code = match e {
                wow_mpq::Error::FileNotFound(_) => ERROR_FILE_NOT_FOUND,
                _ => ERROR_ACCESS_DENIED,
            };
            set_last_error(error_code);
            false
        }
    }
}

/// Rename a file in an archive
///
/// # Safety
///
/// - `archive` must be a valid archive handle
/// - `old_filename` and `new_filename` must be valid null-terminated C strings
#[no_mangle]
pub unsafe extern "C" fn SFileRenameFile(
    archive: HANDLE,
    old_filename: *const c_char,
    new_filename: *const c_char,
) -> bool {
    // Validate parameters
    if archive.is_null() || old_filename.is_null() || new_filename.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    // Convert handle
    let archive_id = match handle_to_id(archive) {
        Some(id) => id,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return false;
        }
    };

    // Convert filenames
    let old_filename_str = match CStr::from_ptr(old_filename).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    };

    let new_filename_str = match CStr::from_ptr(new_filename).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    };

    // Get mutable archive handle
    let mut archives = ARCHIVES.lock().unwrap();
    let archive_handle = match archives.get_mut(&archive_id) {
        Some(handle) => handle,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return false;
        }
    };

    let mutable_archive = match archive_handle.mutable_archive() {
        Some(archive) => archive,
        None => {
            set_last_error(ERROR_ACCESS_DENIED);
            return false;
        }
    };

    // Rename the file
    match mutable_archive.rename_file(old_filename_str, new_filename_str) {
        Ok(()) => {
            set_last_error(ERROR_SUCCESS);
            true
        }
        Err(e) => {
            let error_code = match e {
                wow_mpq::Error::FileNotFound(_) => ERROR_FILE_NOT_FOUND,
                wow_mpq::Error::FileExists(_) => ERROR_ALREADY_EXISTS,
                _ => ERROR_ACCESS_DENIED,
            };
            set_last_error(error_code);
            false
        }
    }
}

/// Flush pending changes to an archive
///
/// # Safety
///
/// - `archive` must be a valid archive handle
#[no_mangle]
pub unsafe extern "C" fn SFileFlushArchive(archive: HANDLE) -> bool {
    // Validate parameters
    if archive.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    // Convert handle
    let archive_id = match handle_to_id(archive) {
        Some(id) => id,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return false;
        }
    };

    // Get mutable archive handle
    let mut archives = ARCHIVES.lock().unwrap();
    let archive_handle = match archives.get_mut(&archive_id) {
        Some(handle) => handle,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return false;
        }
    };

    let mutable_archive = match archive_handle.mutable_archive() {
        Some(archive) => archive,
        None => {
            set_last_error(ERROR_SUCCESS); // Read-only archives don't need flushing
            return true;
        }
    };

    // Flush the archive
    match mutable_archive.flush() {
        Ok(()) => {
            set_last_error(ERROR_SUCCESS);
            true
        }
        Err(_) => {
            set_last_error(ERROR_ACCESS_DENIED);
            false
        }
    }
}

/// Compact an archive to remove deleted files
///
/// # Safety
///
/// - `archive` must be a valid archive handle  
/// - `listfile` can be null
#[no_mangle]
pub unsafe extern "C" fn SFileCompactArchive(
    archive: HANDLE,
    _listfile: *const c_char,
    _reserved: bool,
) -> bool {
    // Validate parameters
    if archive.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    // Convert handle
    let archive_id = match handle_to_id(archive) {
        Some(id) => id,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return false;
        }
    };

    // Get mutable archive handle
    let mut archives = ARCHIVES.lock().unwrap();
    let archive_handle = match archives.get_mut(&archive_id) {
        Some(handle) => handle,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return false;
        }
    };

    let mutable_archive = match archive_handle.mutable_archive() {
        Some(archive) => archive,
        None => {
            set_last_error(ERROR_ACCESS_DENIED);
            return false;
        }
    };

    // Compact the archive
    match mutable_archive.compact() {
        Ok(()) => {
            set_last_error(ERROR_SUCCESS);
            true
        }
        Err(_) => {
            set_last_error(ERROR_NOT_SUPPORTED); // compact() currently returns NotImplemented
            false
        }
    }
}

/// Extended MPQ creation info structure for SFileCreateArchive2
#[repr(C)]
pub struct SFILE_CREATE_MPQ {
    /// Size of this structure, in bytes
    pub cb_size: u32,
    /// Version of the MPQ to be created (1-4)
    pub mpq_version: u32,
    /// Reserved, must be NULL
    pub user_data: *mut c_void,
    /// Reserved, must be 0
    pub cb_user_data: u32,
    /// Stream flags for creating the MPQ
    pub stream_flags: u32,
    /// File flags for (listfile). Use MPQ_FILE_DEFAULT_INTERNAL to set default flags
    pub file_flags_1: u32,
    /// File flags for (attributes). Use MPQ_FILE_DEFAULT_INTERNAL to set default flags
    pub file_flags_2: u32,
    /// File flags for (signature). Use MPQ_FILE_DEFAULT_INTERNAL to set default flags
    pub file_flags_3: u32,
    /// Flags for the (attributes) file. If 0, no attributes will be created
    pub attr_flags: u32,
    /// Sector size (expressed as power of two). If 0, default sector size is 4096 bytes
    pub sector_size: u32,
    /// Size of raw data chunk for MD5
    pub raw_chunk_size: u32,
    /// File limit for the MPQ
    pub max_file_count: u32,
}

// Default internal file flags
const _MPQ_FILE_DEFAULT_INTERNAL: u32 = 0xFFFFFFFF;

// Attribute flags for (attributes) file
const MPQ_ATTRIBUTE_CRC32: u32 = 0x00000001;
const _MPQ_ATTRIBUTE_FILETIME: u32 = 0x00000002;
const MPQ_ATTRIBUTE_MD5: u32 = 0x00000004;
const _MPQ_ATTRIBUTE_PATCH_BIT: u32 = 0x00000008;
const _MPQ_ATTRIBUTE_ALL: u32 = 0x0000000F;

// File find data structure
#[repr(C)]
pub struct SFILE_FIND_DATA {
    /// Full name of the found file
    pub c_file_name: [c_char; 260], // MAX_PATH = 260
    /// Plain name of the found file (pointer into cFileName)
    pub sz_plain_name: *mut c_char,
    /// Hash index of the file
    pub hash_index: u32,
    /// Block table index of the file
    pub block_index: u32,
    /// File size in bytes
    pub file_size: u32,
    /// File flags from block table
    pub file_flags: u32,
    /// Compressed file size
    pub comp_size: u32,
    /// Low 32-bits of the file time
    pub file_time_lo: u32,
    /// High 32-bits of the file time (0 if not present)
    pub file_time_hi: u32,
    /// Compound of file locale (16 bits) and platform (8 bits)
    pub lc_locale: u32, // LCID type
}

// Find handle structure
struct FindHandle {
    archive_handle: usize,
    file_list: Vec<FileEntry>,
    current_index: usize,
    search_mask: String,
}

impl FindHandle {
    fn matches_mask(&self, filename: &str) -> bool {
        // Simple wildcard matching supporting * and ?
        let mask = &self.search_mask;

        // Handle simple cases first
        if mask == "*" || mask == "*.*" {
            return true;
        }

        // Simple wildcard matching without regex
        wildcard_match(&mask.to_lowercase(), &filename.to_lowercase())
    }
}

// Simple wildcard pattern matching
fn wildcard_match(pattern: &str, text: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();
    let mut text_chars = text.chars().peekable();

    while let Some(p) = pattern_chars.peek().copied() {
        match p {
            '*' => {
                pattern_chars.next();
                // If * is at the end, match everything
                if pattern_chars.peek().is_none() {
                    return true;
                }
                // Try matching rest of pattern at each position
                let remaining_pattern: String = pattern_chars.collect();
                let mut remaining_text = text_chars.clone();
                loop {
                    let remaining_text_str: String = remaining_text.clone().collect();
                    if wildcard_match(&remaining_pattern, &remaining_text_str) {
                        return true;
                    }
                    if remaining_text.next().is_none() {
                        return false;
                    }
                }
            }
            '?' => {
                pattern_chars.next();
                if text_chars.next().is_none() {
                    return false;
                }
            }
            _ => {
                pattern_chars.next();
                match text_chars.next() {
                    Some(t) if t == p => continue,
                    _ => return false,
                }
            }
        }
    }

    // Pattern consumed, text should be too
    text_chars.next().is_none()
}

/// File finding functions with wildcard support
///
/// # Safety
///
/// - `h_mpq` must be a valid MPQ archive handle
/// - `sz_mask` must be a valid null-terminated C string (can be NULL for "*")
/// - `lp_find_file_data` must be a valid pointer to SFILE_FIND_DATA
/// - `sz_list_file` can be NULL
#[no_mangle]
pub unsafe extern "C" fn SFileFindFirstFile(
    h_mpq: HANDLE,
    sz_mask: *const c_char,
    lp_find_file_data: *mut SFILE_FIND_DATA,
    _sz_list_file: *const c_char, // Ignored - we use internal listfile
) -> HANDLE {
    // Validate parameters
    if lp_find_file_data.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE;
    }

    let archive_id = match handle_to_id(h_mpq) {
        Some(id) => id,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return INVALID_HANDLE_VALUE;
        }
    };

    // Get search mask
    let search_mask = if sz_mask.is_null() {
        "*".to_string()
    } else {
        match CStr::from_ptr(sz_mask).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                set_last_error(ERROR_INVALID_PARAMETER);
                return INVALID_HANDLE_VALUE;
            }
        }
    };

    // Get file list from archive
    let file_list = {
        let mut archives = ARCHIVES.lock().unwrap();
        match archives.get_mut(&archive_id) {
            Some(archive_handle) => {
                match archive_handle {
                    ArchiveHandle::ReadOnly { archive, .. } => {
                        match archive.list() {
                            Ok(list) => list,
                            Err(_) => {
                                // Try list_all if no listfile
                                match archive.list_all() {
                                    Ok(list) => list,
                                    Err(_) => {
                                        set_last_error(ERROR_FILE_NOT_FOUND);
                                        return INVALID_HANDLE_VALUE;
                                    }
                                }
                            }
                        }
                    }
                    ArchiveHandle::Mutable { archive, .. } => match archive.list() {
                        Ok(list) => list,
                        Err(_) => {
                            set_last_error(ERROR_FILE_NOT_FOUND);
                            return INVALID_HANDLE_VALUE;
                        }
                    },
                }
            }
            None => {
                set_last_error(ERROR_INVALID_HANDLE);
                return INVALID_HANDLE_VALUE;
            }
        }
    };

    // Create find handle
    let mut find_handle = FindHandle {
        archive_handle: archive_id,
        file_list,
        current_index: 0,
        search_mask,
    };

    // Find first matching file
    let found_file = loop {
        if find_handle.current_index >= find_handle.file_list.len() {
            set_last_error(ERROR_FILE_NOT_FOUND);
            return INVALID_HANDLE_VALUE;
        }

        let file = &find_handle.file_list[find_handle.current_index];
        if find_handle.matches_mask(&file.name) {
            break Some(file);
        }

        find_handle.current_index += 1;
    };

    if let Some(file) = found_file {
        // Fill find data
        fill_find_data(lp_find_file_data, file, archive_id);

        // Store find handle
        let mut next_id = NEXT_HANDLE.lock().unwrap();
        let handle_id = *next_id;
        *next_id += 1;
        drop(next_id);

        find_handle.current_index += 1; // Move to next for SFileFindNextFile
        FIND_HANDLES.lock().unwrap().insert(handle_id, find_handle);

        set_last_error(ERROR_SUCCESS);
        id_to_handle(handle_id)
    } else {
        set_last_error(ERROR_FILE_NOT_FOUND);
        INVALID_HANDLE_VALUE
    }
}

/// Find the next file matching the search criteria
///
/// # Safety
///
/// - `h_find` must be a valid find handle from SFileFindFirstFile
/// - `lp_find_file_data` must be a valid pointer to SFILE_FIND_DATA
#[no_mangle]
pub unsafe extern "C" fn SFileFindNextFile(
    h_find: HANDLE,
    lp_find_file_data: *mut SFILE_FIND_DATA,
) -> bool {
    if lp_find_file_data.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    let handle_id = match handle_to_id(h_find) {
        Some(id) => id,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return false;
        }
    };

    let mut find_handles = FIND_HANDLES.lock().unwrap();
    let find_handle = match find_handles.get_mut(&handle_id) {
        Some(handle) => handle,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return false;
        }
    };

    // Find next matching file
    let found_file = loop {
        if find_handle.current_index >= find_handle.file_list.len() {
            set_last_error(ERROR_NO_MORE_FILES);
            return false;
        }

        let file = &find_handle.file_list[find_handle.current_index];
        find_handle.current_index += 1;

        if find_handle.matches_mask(&file.name) {
            break Some(file);
        }
    };

    if let Some(file) = found_file {
        fill_find_data(lp_find_file_data, file, find_handle.archive_handle);
        set_last_error(ERROR_SUCCESS);
        true
    } else {
        set_last_error(ERROR_NO_MORE_FILES);
        false
    }
}

/// Close a find handle
///
/// # Safety
///
/// - `h_find` must be a valid find handle from SFileFindFirstFile
#[no_mangle]
pub unsafe extern "C" fn SFileFindClose(h_find: HANDLE) -> bool {
    let handle_id = match handle_to_id(h_find) {
        Some(id) => id,
        None => {
            set_last_error(ERROR_INVALID_HANDLE);
            return false;
        }
    };

    if FIND_HANDLES.lock().unwrap().remove(&handle_id).is_some() {
        set_last_error(ERROR_SUCCESS);
        true
    } else {
        set_last_error(ERROR_INVALID_HANDLE);
        false
    }
}

// Helper function to fill SFILE_FIND_DATA
unsafe fn fill_find_data(
    find_data: *mut SFILE_FIND_DATA,
    file_entry: &FileEntry,
    archive_id: usize,
) {
    let find_data = &mut *find_data;

    // Copy filename
    let name_bytes = file_entry.name.as_bytes();
    let copy_len = name_bytes.len().min(259); // Leave room for null terminator

    // Zero the entire buffer first
    find_data.c_file_name.fill(0);

    // Copy the filename
    for (i, &byte) in name_bytes.iter().take(copy_len).enumerate() {
        find_data.c_file_name[i] = byte as c_char;
    }

    // Set plain name pointer (points to last component after backslash)
    let plain_name_offset = file_entry.name.rfind('\\').map(|pos| pos + 1).unwrap_or(0);
    find_data.sz_plain_name = find_data.c_file_name.as_mut_ptr().add(plain_name_offset);

    // Set file information
    find_data.hash_index = if let Some((hash_idx, _)) = file_entry.table_indices {
        hash_idx as u32
    } else {
        0xFFFFFFFF
    };
    find_data.block_index = if let Some((_, Some(block_idx))) = file_entry.table_indices {
        block_idx as u32
    } else {
        0xFFFFFFFF
    };
    find_data.file_size = file_entry.size as u32;
    find_data.file_flags = file_entry.flags;
    find_data.comp_size = file_entry.compressed_size as u32;
    find_data.file_time_lo = 0; // Timestamp not available in basic FileEntry
    find_data.file_time_hi = 0;
    find_data.lc_locale = 0; // Locale not directly available in FileEntry

    // Try to get additional info from archive if available
    let mut archives = ARCHIVES.lock().unwrap();
    if let Some(archive_handle) = archives.get_mut(&archive_id) {
        match archive_handle {
            ArchiveHandle::ReadOnly { archive, .. } => {
                // Try to get file attributes for timestamp
                if let Some((_, Some(block_idx))) = file_entry.table_indices {
                    if let Some(attrs) = archive.get_file_attributes(block_idx) {
                        if let Some(filetime) = attrs.filetime {
                            find_data.file_time_lo = (filetime & 0xFFFFFFFF) as u32;
                            find_data.file_time_hi = (filetime >> 32) as u32;
                        }
                    }
                }
            }
            ArchiveHandle::Mutable { archive, .. } => {
                // Try to get file attributes for timestamp
                if let Some((_, Some(block_idx))) = file_entry.table_indices {
                    if let Some(attrs) = archive.archive().get_file_attributes(block_idx) {
                        if let Some(filetime) = attrs.filetime {
                            find_data.file_time_lo = (filetime & 0xFFFFFFFF) as u32;
                            find_data.file_time_hi = (filetime >> 32) as u32;
                        }
                    }
                }
            }
        }
    }
}

// Error code for no more files
const ERROR_NO_MORE_FILES: u32 = 18;

/// Create an MPQ archive with extended options
///
/// # Safety
///
/// - `filename` must be a valid null-terminated C string
/// - `create_info` must point to a valid SFILE_CREATE_MPQ structure
/// - `handle` must be a valid pointer to write the output handle
#[no_mangle]
pub unsafe extern "C" fn SFileCreateArchive2(
    filename: *const c_char,
    create_info: *const SFILE_CREATE_MPQ,
    handle: *mut HANDLE,
) -> bool {
    // Validate parameters
    if filename.is_null() || create_info.is_null() || handle.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return false;
    }

    // Validate structure size
    let info = &*create_info;
    if info.cb_size != std::mem::size_of::<SFILE_CREATE_MPQ>() as u32 {
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

    // Determine MPQ format version
    let version = match info.mpq_version {
        1 => FormatVersion::V1,
        2 => FormatVersion::V2,
        3 => FormatVersion::V3,
        4 => FormatVersion::V4,
        _ => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return false;
        }
    };

    // Calculate hash table size from max file count
    let _hash_table_size = if info.max_file_count > 0 {
        // Round up to next power of 2, minimum 4
        info.max_file_count.max(4).next_power_of_two()
    } else {
        // Default size
        16
    };

    // Calculate sector size
    let sector_size = if info.sector_size > 0 && info.sector_size <= 23 {
        info.sector_size as u16
    } else {
        3 // Default to 4KB sectors (512 << 3)
    };

    // Create the archive using ArchiveBuilder
    let mut builder = ArchiveBuilder::new()
        .version(version)
        .block_size(sector_size);

    // Configure listfile based on file_flags_1
    if info.file_flags_1 != 0 {
        builder = builder.listfile_option(ListfileOption::Generate);
    } else {
        builder = builder.listfile_option(ListfileOption::None);
    }

    // Configure attributes based on attr_flags
    if info.attr_flags != 0 {
        if (info.attr_flags & MPQ_ATTRIBUTE_MD5) != 0 {
            builder = builder.attributes_option(AttributesOption::GenerateFull);
        } else if (info.attr_flags & MPQ_ATTRIBUTE_CRC32) != 0 {
            builder = builder.attributes_option(AttributesOption::GenerateCrc32);
        }
    }

    // Build the archive
    match builder.build(filename_str) {
        Ok(_) => {
            // Open the created archive for use
            // Try as mutable first
            match MutableArchive::open(filename_str) {
                Ok(mut_archive) => {
                    // Generate new handle ID
                    let mut next_id = NEXT_HANDLE.lock().unwrap();
                    let handle_id = *next_id;
                    *next_id += 1;
                    drop(next_id);

                    // Store archive as mutable
                    let archive_handle = ArchiveHandle::Mutable {
                        archive: mut_archive,
                        path: filename_str.to_string(),
                    };
                    ARCHIVES.lock().unwrap().insert(handle_id, archive_handle);

                    // Return handle
                    *handle = id_to_handle(handle_id);
                    set_last_error(ERROR_SUCCESS);
                    true
                }
                Err(_) => {
                    // Fall back to read-only open
                    SFileOpenArchive(filename, 0, 0, handle)
                }
            }
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
