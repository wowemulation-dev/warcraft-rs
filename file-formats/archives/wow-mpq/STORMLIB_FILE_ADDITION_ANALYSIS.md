# StormLib File Addition Analysis

Based on the C++ test program output, here's how StormLib handles file addition:

## Key Observations

1. **Archive Size Behavior**:
   - Initial empty archive: 2092 bytes
   - After adding 5 files: Still shows 2092 bytes (!)
   - After compacting: 3974 bytes
   - This suggests StormLib doesn't update the archive size until compact/close

2. **File Addition Process**:
   - Files are added but archive size remains unchanged
   - Hash table and block table counts don't update immediately
   - StormLib appears to use a lazy update approach

3. **Compression**:
   - Files are compressed during addition
   - Example: "test1.txt" (23 bytes) → 31 bytes compressed
   - Large file: 100,000 bytes → 1,567 bytes compressed

4. **Special Files Created**:
   - `(listfile)` is automatically created after compacting
   - Contains 58 bytes (list of filenames)

5. **File Flags**:
   - All files have flags: 0x84000200
   - Flags include: Compressed, Sector CRC, Exists

## Implementation Strategy for wow-mpq

### Phase 1: Basic Addition (Current Task)

1. **Find Free Space**:

   ```rust
   fn find_free_space(&self, required_size: u64) -> Result<u64> {
       // Option 1: Append to end of archive
       // Option 2: Find deleted file space (later optimization)
   }
   ```

2. **Update Hash Table**:

   ```rust
   fn add_to_hash_table(&mut self, filename: &str, block_index: u32) -> Result<usize> {
       // Calculate hash values
       // Find free or deleted entry
       // Update entry
   }
   ```

3. **Add Block Entry**:

   ```rust
   fn add_block_entry(&mut self, file_offset: u64, size: u32, flags: u32) -> Result<u32> {
       // Add new block table entry
       // Return block index
   }
   ```

4. **Write File Data**:

   ```rust
   fn write_file_data(&mut self, data: &[u8], offset: u64, options: &AddFileOptions) -> Result<u32> {
       // Compress if requested
       // Encrypt if requested
       // Write to archive
       // Return compressed size
   }
   ```

### Phase 2: Table Management

1. **Grow Tables if Needed**:
   - Hash table must remain power of 2
   - Block table can grow freely

2. **Update Archive Header**:
   - Update archive size
   - Update table positions if moved

3. **Maintain Listfile**:
   - Add new filename to (listfile)
   - Update if it exists

### Critical Details

1. **Path Normalization**:
   - Convert `/` to `\` before hashing
   - Store with backslashes in listfile

2. **Encryption Key**:
   - If FIX_KEY: Adjust key by block position
   - Use normalized filename for key generation

3. **Sector Size**:
   - Default: 4096 bytes (sector_shift = 3)
   - Multi-sector files need sector offset table

4. **Flags**:
   - MPQ_FILE_COMPRESS (0x00000200)
   - MPQ_FILE_ENCRYPTED (0x00010000)
   - MPQ_FILE_FIX_KEY (0x00020000)
   - MPQ_FILE_SINGLE_UNIT (0x01000000)
   - MPQ_FILE_EXISTS (0x80000000)

## Test-Driven Implementation

The test shows:

1. Add 4 files with SFileAddFileEx
2. Add 1 file from memory with SFileCreateFile
3. Files are accessible immediately after addition
4. Archive remains valid after close/reopen
