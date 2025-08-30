//! Memory-mapped file access performance demonstration
//!
//! This example shows how to use the memory-mapped file functionality for high-performance
//! file access with comprehensive security boundaries and cross-platform compatibility.

#[cfg(feature = "mmap")]
use std::sync::Arc;
#[cfg(feature = "mmap")]
use std::time::Instant;
#[cfg(feature = "mmap")]
use wow_mpq::{MemoryMapConfig, MemoryMappedArchive, SecurityLimits, SessionTracker};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("MPQ Memory Mapping Performance Demo");
    println!("==================================");

    #[cfg(feature = "mmap")]
    {
        println!("Memory mapping feature is ENABLED");
        demonstrate_memory_mapping()?;
    }

    #[cfg(not(feature = "mmap"))]
    {
        println!("Memory mapping feature is DISABLED");
        println!(
            "To enable memory mapping, build with: cargo run --features mmap --example memory_mapped_performance"
        );
    }

    Ok(())
}

#[cfg(feature = "mmap")]
fn demonstrate_memory_mapping() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a temporary file with test data for demonstration
    let mut temp_file = NamedTempFile::new().unwrap();
    let test_data = (0u8..255).cycle().take(1024 * 1024).collect::<Vec<u8>>(); // 1MB of test data
    temp_file.write_all(&test_data).unwrap();
    temp_file.flush().unwrap();

    println!("\nTesting different memory mapping configurations:");

    // Test configurations
    let configs = [
        ("Default", MemoryMapConfig::default()),
        ("Strict", MemoryMapConfig::strict()),
        ("Permissive", MemoryMapConfig::permissive()),
    ];

    for (name, config) in &configs {
        println!("\n{} Configuration:", name);
        println!(
            "  Max mapping size: {:.1} MB",
            config.max_map_size as f64 / (1024.0 * 1024.0)
        );
        println!("  Mapping enabled: {}", config.enable_mapping);
        println!("  Read-ahead: {}", config.read_ahead);
        println!("  Advisory locking: {}", config.advisory_locking);

        let security_limits = SecurityLimits::default();
        let session_tracker = Arc::new(SessionTracker::new());

        // Create memory-mapped archive
        let start = Instant::now();
        let mmap_result = MemoryMappedArchive::new(
            temp_file.path(),
            config.clone(),
            security_limits,
            session_tracker,
        );
        let creation_time = start.elapsed();

        match mmap_result {
            Ok(mmap_archive) => {
                println!("  ✓ Memory mapping created in {:?}", creation_time);
                println!("  File size: {} bytes", mmap_archive.file_size());
                println!("  Health status: {}", mmap_archive.is_healthy());

                // Demonstrate reading performance
                let start = Instant::now();
                let mut buffer = vec![0u8; 4096];
                let mut total_read = 0;

                for offset in (0..test_data.len()).step_by(4096) {
                    let read_size = std::cmp::min(4096, test_data.len() - offset);
                    buffer.resize(read_size, 0);

                    match mmap_archive.read_at(offset as u64, &mut buffer[..read_size]) {
                        Ok(()) => total_read += read_size,
                        Err(e) => {
                            println!("  ✗ Read error at offset {}: {}", offset, e);
                            break;
                        }
                    }
                }

                let read_time = start.elapsed();
                println!(
                    "  ✓ Read {} bytes in {} operations: {:?}",
                    total_read,
                    test_data.len() / 4096,
                    read_time
                );

                // Demonstrate slice access
                let start = Instant::now();
                match mmap_archive.get_slice(0, 1024) {
                    Ok(slice) => {
                        let slice_time = start.elapsed();
                        println!(
                            "  ✓ Got 1KB slice in {:?}, first 16 bytes: {:02x?}",
                            slice_time,
                            &slice[..16]
                        );
                    }
                    Err(e) => println!("  ✗ Slice error: {}", e),
                }

                // Show statistics
                let stats = mmap_archive.stats();
                println!("  Statistics:");
                println!("    Bytes mapped: {}", stats.bytes_mapped);
                println!("    Active mappings: {}", stats.active_mappings);
                println!("    Failed mappings: {}", stats.failed_mappings);
            }
            Err(e) => {
                println!("  ✗ Memory mapping failed: {}", e);
            }
        }
    }

    // Test disabled configuration
    println!("\nDisabled Configuration:");
    let disabled_config = MemoryMapConfig::disabled();
    let security_limits = SecurityLimits::default();
    let session_tracker = Arc::new(SessionTracker::new());

    match MemoryMappedArchive::new(
        temp_file.path(),
        disabled_config,
        security_limits,
        session_tracker,
    ) {
        Ok(_) => println!("  ✗ Unexpected success with disabled config"),
        Err(e) => println!("  ✓ Expected error: {}", e),
    }

    println!("\nMemory Mapping Benefits:");
    println!("• Zero-copy access to file data");
    println!("• OS-level caching and optimization");
    println!("• Reduced memory pressure for large files");
    println!("• Cross-platform compatibility (Windows, Linux, macOS)");

    println!("\nSecurity Features:");
    println!("• File size validation against security limits");
    println!("• Bounds checking for all memory access");
    println!("• Resource exhaustion prevention");
    println!("• Graceful fallback when mapping fails");

    Ok(())
}

#[cfg(feature = "mmap")]
#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::NamedTempFile;
    use wow_mpq::{MemoryMapConfig, MemoryMappedArchive, SecurityLimits, SessionTracker};

    #[test]
    fn test_memory_mapped_performance_demo() -> wow_mpq::Result<()> {
        // Create a temporary file with test data
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = vec![0xAAu8; 64 * 1024]; // 64KB of test data for quick test
        temp_file.write_all(&test_data).unwrap();
        temp_file.flush().unwrap();

        // Test default configuration
        let config = MemoryMapConfig::default();
        let security_limits = SecurityLimits::default();
        let session_tracker = Arc::new(SessionTracker::new());

        let mmap_archive =
            MemoryMappedArchive::new(temp_file.path(), config, security_limits, session_tracker)?;

        // Verify basic functionality
        assert_eq!(mmap_archive.file_size(), test_data.len() as u64);
        assert!(mmap_archive.is_healthy());

        // Test reading
        let mut buffer = vec![0u8; 1024];
        mmap_archive.read_at(0, &mut buffer)?;
        assert_eq!(&buffer[..], &test_data[..1024]);

        // Test slice access
        let slice = mmap_archive.get_slice(1024, 1024)?;
        assert_eq!(slice, &test_data[1024..2048]);

        println!("Memory mapping performance test completed successfully!");
        Ok(())
    }

    #[test]
    fn test_all_configurations() -> wow_mpq::Result<()> {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = vec![0x55u8; 32 * 1024]; // 32KB for configuration testing
        temp_file.write_all(&test_data).unwrap();
        temp_file.flush().unwrap();

        let security_limits = SecurityLimits::default();
        let session_tracker = Arc::new(SessionTracker::new());

        // Test all valid configurations
        let configs = [
            MemoryMapConfig::default(),
            MemoryMapConfig::strict(),
            MemoryMapConfig::permissive(),
        ];

        for config in &configs {
            let mmap_archive = MemoryMappedArchive::new(
                temp_file.path(),
                config.clone(),
                security_limits.clone(),
                session_tracker.clone(),
            )?;

            assert_eq!(mmap_archive.file_size(), test_data.len() as u64);
            assert!(mmap_archive.is_healthy());

            // Test basic read
            let mut buffer = vec![0u8; 256];
            mmap_archive.read_at(0, &mut buffer)?;
            assert_eq!(&buffer[..], &test_data[..256]);
        }

        // Test disabled configuration
        let disabled_config = MemoryMapConfig::disabled();
        let result = MemoryMappedArchive::new(
            temp_file.path(),
            disabled_config,
            security_limits,
            session_tracker,
        );
        assert!(result.is_err());

        Ok(())
    }
}
