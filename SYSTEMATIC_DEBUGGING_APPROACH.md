# Systematic Debugging and Development Approach

This document captures the **exact methodology** that successfully resolved complex compression issues in warcraft-rs. This approach should be followed for all future feature implementations and issue resolutions.

## Core Principle: **Reference Implementation First**

Never debug our implementation in isolation. Always establish ground truth through StormLib comparison.

## The 6-Step Systematic Process

### Step 1: **Issue Identification and Evidence Collection**

- Capture exact error messages, warnings, and failure patterns
- Identify specific files, archives, or scenarios causing issues
- Document the scope: single file vs systemic issue
- Never assume root cause without evidence

**Example:**

```
❌ Sparse compression buffer underruns
❌ ZLIB decompression failures
❌ Specific files: (attributes), (listfile)
❌ Specific archives: wow-update-enUS-15595.MPQ
```

### Step 2: **StormLib Reference Test Creation**

Create comprehensive C++ tests using the existing StormLib environment:

- **Location**: `file-formats/archives/wow-mpq/tests/stormlib_comparison/`
- **Purpose**: Establish "ground truth" behavior
- **Scope**: Test the exact same scenarios that fail in our implementation

**Template Structure:**

```cpp
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>
#include <StormLib.h>

void test_specific_scenario(const char* archive_path, const char* filename) {
    printf("=== StormLib Reference Test: [Scenario Name] ===\n");

    // 1. Open and validate
    // 2. Extract detailed information
    // 3. Perform exact operations that fail in Rust
    // 4. Show raw data, flags, compression methods, etc.
    // 5. Document expected behavior
}

int main(int argc, char* argv[]) {
    // Test provided files
    // Test known problematic cases
    // Show usage examples
}
```

**Compilation:**

```bash
cd tests/stormlib_comparison/
g++ -I/path/to/StormLib/src test_name.cpp /path/to/StormLib/build/libstorm.so -o test_name
LD_LIBRARY_PATH=/path/to/StormLib/build ./test_name
```

### Step 3: **Behavioral Analysis and Comparison**

- Run StormLib tests to establish correct behavior
- Run our implementation on same test cases
- Document exact differences in:
  - Data interpretation
  - Flag handling
  - Algorithm application
  - Expected vs actual results

**Key Analysis Points:**

- What does StormLib do that we don't?
- What assumptions are we making that are wrong?
- What data are we processing incorrectly?

### Step 4: **Root Cause Identification**

- Use detailed debugging to trace exact data flow
- Add extensive logging to our implementation
- Compare byte-by-byte what StormLib sees vs what we see
- Never guess - always validate with concrete evidence

**Debug Logging Example:**

```rust
log::debug!("Raw compressed data (first 32 bytes): {:02X?}", &data[..32.min(data.len())]);
log::debug!("Compression method: flags=0x{:08X}, mask=0x{:02X}", flags, mask);
log::debug!("Expected result: {} bytes, actual: {} bytes", expected, actual);
```

### Step 5: **Systematic Fix Implementation**

- Fix the **specific root cause** identified, not symptoms
- Use predefined constants and existing architecture
- Maintain API compatibility
- Add comprehensive error handling
- Document why the fix works

**Implementation Principles:**

- Use existing compression flags/constants
- Follow established patterns in codebase
- Add methods to existing structures rather than creating new ones
- Preserve backward compatibility

### Step 6: **Comprehensive Verification**

- Test fix against original failing cases
- Test against multiple MPQ versions and formats
- Verify no regressions in existing functionality
- Run full test suite
- Test edge cases discovered during investigation

**Verification Checklist:**

```bash
# Original failing cases
cargo run -- mpq info /path/to/problematic.mpq

# Multiple versions
cargo run -- mpq info /path/to/v2.mpq
cargo run -- mpq info /path/to/v3.mpq
cargo run -- mpq info /path/to/v4.mpq

# Regression testing
cargo test
cargo clippy
cargo fmt --check
```

## Critical Success Factors

### ✅ **DO:**

- Create multiple StormLib tests for different scenarios
- Test with real-world files, not just synthetic data
- Use detailed logging to trace data flow
- Compare exact byte sequences between implementations
- Fix root causes, not symptoms
- Document findings and approach
- Verify across multiple file versions/formats

### ❌ **DON'T:**

- Debug our implementation in isolation
- Make assumptions about file formats without verification
- Guess at root causes
- Fix symptoms instead of underlying issues
- Skip verification with real-world files
- Forget to test regressions

## Specific Tools and Locations

### StormLib Environment

- **StormLib Source**: `/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib`
- **Built Library**: `/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/build/libstorm.so`
- **Test Location**: `file-formats/archives/wow-mpq/tests/stormlib_comparison/`

### Test Data

- **WoW 1.12.1**: `/home/danielsreichenbach/Downloads/wow/1.12.1/Data/`
- **WoW 2.4.3**: `/home/danielsreichenbach/Downloads/wow/2.4.3/Data/`
- **WoW 3.3.5a**: `/home/danielsreichenbach/Downloads/wow/3.3.5a/Data/`
- **WoW 4.3.4**: `/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data/`
- **WoW 5.4.8**: `/home/danielsreichenbach/Downloads/wow/5.4.8/5.4.8/Data/`

### Quality Assurance

```bash
# Before committing any fix:
cargo fmt --all
cargo check --all-features --all-targets
cargo clippy --all-targets --all-features
cargo test --all-features
cargo test --no-default-features
```

## Example Success Case: Compression Method Detection

**Issue**: Sparse compression buffer underruns, ZLIB failures
**StormLib Tests**:

- `test_compression_detection_systematic.cpp`
- `test_raw_data_analysis.cpp`
- `test_compression_issues_base_osx.cpp`

**Root Cause Found**: SINGLE_UNIT files DO have compression method byte prefixes (contrary to documentation)

**Fix**: Modified single-unit file reading to skip first byte like sectored files

**Result**: All compression issues resolved across all MPQ versions

## Enforcement

This approach is **mandatory** for:

- All new feature implementations
- All bug fixes and issue resolutions
- All format compatibility improvements
- Any changes to core parsing logic

**No exceptions.** Always create StormLib reference tests first, then implement fixes based on evidence.
