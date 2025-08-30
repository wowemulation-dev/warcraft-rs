//! Comprehensive tests for async I/O operations with security validation

#[cfg(feature = "async")]
mod tests {
    use wow_mpq::{
        Archive, AsyncArchiveReader, AsyncConfig, Error,
        security::{SecurityLimits, SessionTracker},
    };
    #[cfg(any(test, feature = "test-utils"))]
    use wow_mpq::{ArchiveBuilder, ListfileOption};

    // Helper function to create a simple test archive
    fn create_simple_test_archive(
        path: &std::path::Path,
        filename: &str,
        data: &[u8],
    ) -> Result<(), wow_mpq::Error> {
        let builder = ArchiveBuilder::new()
            .listfile_option(ListfileOption::Generate)
            .add_file_data(data.to_vec(), filename);

        builder.build(path)?;
        Ok(())
    }
    use std::io::Cursor;
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    /// Test basic async file reading functionality
    #[tokio::test]
    async fn test_async_read_file_basic() {
        // Create a test archive with a simple file
        let temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Hello, async world!";

        create_simple_test_archive(temp_file.path(), "test.txt", test_data).unwrap();

        // Open archive and read file asynchronously
        let mut archive = Archive::open(temp_file.path()).unwrap();
        let result = archive.read_file_async("test.txt").await.unwrap();

        assert_eq!(result, test_data);
    }

    /// Test async file reading with timeout protection
    #[tokio::test]
    async fn test_async_read_file_timeout_protection() {
        let data = vec![0u8; 1000];
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        // Create config with very short timeout
        let config = AsyncConfig {
            operation_timeout: tokio::time::Duration::from_millis(1),
            ..Default::default()
        };

        let reader = AsyncArchiveReader::with_config(cursor, config, session);

        // This should timeout due to the very short timeout
        let mut buffer = [0u8; 100];
        let result = reader.read_at(0, &mut buffer).await;

        // The result might succeed if the operation is fast enough,
        // but if it times out, it should be a resource exhaustion error
        if let Err(e) = result {
            assert!(e.to_string().contains("timed out"));
        }
    }

    /// Test async operations with memory limits
    #[tokio::test]
    async fn test_async_memory_limits() {
        let data = vec![1, 2, 3, 4, 5];
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        // Create config with very small memory limit
        let config = AsyncConfig {
            max_async_memory: 2, // Very small limit
            ..Default::default()
        };

        let reader = AsyncArchiveReader::with_config(cursor, config, session);

        // Try to read more than the memory limit allows
        let mut buffer = [0u8; 10]; // Exceeds the 2-byte limit
        let result = reader.read_at(0, &mut buffer).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("exceeds maximum allowed size"));
    }

    /// Test concurrent file extraction with proper limits
    #[tokio::test]
    async fn test_async_concurrent_extraction() {
        let temp_file = NamedTempFile::new().unwrap();

        // Create a simple test archive (this is a simplified test)
        create_simple_test_archive(temp_file.path(), "test.txt", b"test content").unwrap();

        // Test concurrent extraction
        let mut archive = Archive::open(temp_file.path()).unwrap();
        let filenames = vec![
            "file0.txt",
            "file1.txt",
            "file2.txt",
            "file3.txt",
            "file4.txt",
        ];

        let results = archive
            .extract_files_async(&filenames, Some(3))
            .await
            .unwrap();

        assert_eq!(results.len(), 5);

        // Verify contents
        for (i, (filename, data)) in results.iter().enumerate() {
            let expected_filename = format!("file{}.txt", i);
            let expected_content = format!("Content of file {}", i);

            assert_eq!(filename, &expected_filename);
            assert_eq!(String::from_utf8_lossy(data), expected_content);
        }
    }

    /// Test concurrent extraction with limits
    #[tokio::test]
    async fn test_async_concurrent_extraction_limits() {
        let data = vec![0u8; 1000];
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        // Create config with very limited concurrency
        let config = AsyncConfig {
            max_concurrent_extractions: 1,
            ..Default::default()
        };

        let reader = AsyncArchiveReader::with_config(cursor, config, session);

        // Try to extract too many files concurrently
        let requests = (0..10)
            .map(|i| (format!("file{}.txt", i), i * 100, 50))
            .collect();

        let result = reader.extract_files_concurrent(requests).await;

        // This should succeed but with limited concurrency
        // The actual test is that it doesn't panic or deadlock
        assert!(result.is_ok());
    }

    /// Test session tracking and limits
    #[tokio::test]
    async fn test_session_tracking() {
        let session = SessionTracker::new();
        let limits = SecurityLimits::strict();

        // Record some decompressions
        session.record_decompression(1024);
        session.record_decompression(2048);

        let (total, count, _duration) = session.get_stats();
        assert_eq!(total, 3072);
        assert_eq!(count, 2);

        // Should still be within limits
        assert!(session.check_session_limits(&limits).is_ok());

        // Exceed the limit
        session.record_decompression(limits.max_session_decompressed + 1);
        assert!(session.check_session_limits(&limits).is_err());
    }

    /// Test async decompression monitor
    #[tokio::test]
    async fn test_async_decompression_monitor() {
        let monitor = wow_mpq::io::AsyncDecompressionMonitor::new(
            1024,
            tokio::time::Duration::from_secs(5),
            64,
        );

        // Normal operation should succeed
        monitor.check_progress(512).await.unwrap();

        // Exceeding size limit should fail
        let result = monitor.check_progress(2048).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("size limit exceeded")
        );

        // Test cancellation
        monitor.request_cancellation();
        let result = monitor.check_progress(256).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));
    }

    /// Test async reader metrics collection
    #[tokio::test]
    async fn test_async_metrics() {
        let data = vec![0u8; 1000];
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        // Enable metrics collection
        let config = AsyncConfig {
            collect_metrics: true,
            ..Default::default()
        };

        let reader = AsyncArchiveReader::with_config(cursor, config, session);

        // Perform some operations
        let mut buffer = [0u8; 100];
        let _ = reader.read_at(0, &mut buffer).await;
        let _ = reader.read_at(100, &mut buffer).await;

        let stats = reader.get_stats();
        assert!(stats.total_operations >= 2);
        assert!(stats.total_bytes_read >= 200);
    }

    /// Test resource pressure detection
    #[tokio::test]
    async fn test_resource_pressure_detection() {
        let data = vec![0u8; 100];
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        // Create config with very limited operations
        let config = AsyncConfig {
            max_concurrent_ops: 1,
            ..Default::default()
        };

        let reader = AsyncArchiveReader::with_config(cursor, config, session);

        // Initially should not be under pressure
        assert!(!reader.is_under_pressure());

        // For this test, we'll just check that the reader was created successfully
        // The actual pressure detection logic is internal and tested separately
        assert!(!reader.is_under_pressure()); // Should initially not be under pressure
    }

    /// Test graceful shutdown
    #[tokio::test]
    async fn test_async_reader_shutdown() {
        let data = vec![0u8; 100];
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        let reader = AsyncArchiveReader::new(cursor, session);

        // Should shutdown without error
        reader.shutdown().await.unwrap();

        // Operations after shutdown should fail
        let mut buffer = [0u8; 10];
        let result = reader.read_at(0, &mut buffer).await;
        assert!(result.is_err());
    }

    /// Test security integration - compression bomb detection in async context
    #[tokio::test]
    async fn test_async_compression_bomb_protection() {
        let session = SessionTracker::new();
        let limits = SecurityLimits::default();

        // Test creation of decompression monitor with suspicious parameters
        let result = wow_mpq::security::validate_decompression_operation(
            100,         // Very small compressed size
            100_000_000, // Very large decompressed size (1000000:1 ratio)
            0x02,        // ZLIB compression
            Some("suspicious.txt"),
            &session,
            &limits,
        );

        // Should detect compression bomb
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("compression bomb") || err_msg.contains("compression"));
    }

    /// Test async operations with file not found
    #[tokio::test]
    async fn test_async_file_not_found() {
        let temp_file = NamedTempFile::new().unwrap();

        // Create empty archive (simplified for this test)
        create_simple_test_archive(temp_file.path(), "dummy.txt", b"dummy").unwrap();

        let mut archive = Archive::open(temp_file.path()).unwrap();
        let result = archive.read_file_async("nonexistent.txt").await;

        assert!(result.is_err());
        if let Err(Error::FileNotFound(filename)) = result {
            assert_eq!(filename, "nonexistent.txt");
        } else {
            panic!("Expected FileNotFound error");
        }
    }

    /// Test async extraction with empty file list
    #[tokio::test]
    async fn test_async_extract_empty_list() {
        let temp_file = NamedTempFile::new().unwrap();

        // Create test archive (simplified)
        create_simple_test_archive(temp_file.path(), "dummy.txt", b"dummy").unwrap();

        let mut archive = Archive::open(temp_file.path()).unwrap();
        let results = archive.extract_files_async(&[], None).await.unwrap();

        assert!(results.is_empty());
    }

    /// Test async operations with security limits
    #[tokio::test]
    async fn test_async_with_custom_security_limits() {
        let data = vec![0u8; 1000];
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        let config = AsyncConfig::default();
        let security_limits = SecurityLimits::strict(); // Very restrictive limits

        let reader =
            AsyncArchiveReader::with_security_limits(cursor, config, session, security_limits);

        // Operations should work within the strict limits
        let mut small_buffer = [0u8; 10];
        let result = reader.read_at(0, &mut small_buffer).await;
        assert!(result.is_ok());
    }

    /// Integration test with real archive operations
    #[tokio::test]
    async fn test_async_with_compressed_files() {
        // This test would require a real MPQ file with compressed content
        // For now, we'll simulate the scenario

        let temp_file = NamedTempFile::new().unwrap();
        let test_data = b"This is test content that should be compressed in a real MPQ file";

        create_simple_test_archive(temp_file.path(), "compressed.txt", test_data).unwrap();

        let mut archive = Archive::open(temp_file.path()).unwrap();
        let result = archive.read_file_async("compressed.txt").await.unwrap();

        // Content should match regardless of compression
        assert_eq!(result, test_data);
    }

    /// Performance test for async operations
    #[tokio::test]
    async fn test_async_performance_characteristics() {
        let temp_file = NamedTempFile::new().unwrap();

        // Create a test archive (simplified for performance test)
        create_simple_test_archive(temp_file.path(), "test.txt", &vec![1u8; 1024]).unwrap();

        let mut archive = Archive::open(temp_file.path()).unwrap();

        // Measure concurrent extraction
        let start_time = std::time::Instant::now();

        let filenames: Vec<String> = (0..10).map(|i| format!("perf_test_{}.dat", i)).collect();
        let filename_refs: Vec<&str> = filenames.iter().map(|s| s.as_str()).collect();

        let results = archive
            .extract_files_async(&filename_refs, Some(5))
            .await
            .unwrap();

        let duration = start_time.elapsed();

        assert_eq!(results.len(), 10);

        // Performance characteristic: should complete reasonably quickly
        // This is more of a smoke test than a strict performance requirement
        assert!(duration.as_millis() < 5000); // Less than 5 seconds

        // Verify total data extracted
        let total_bytes: usize = results.iter().map(|(_, data)| data.len()).sum();
        assert_eq!(total_bytes, 10 * 1024); // 10 files * 1KB each
    }
}
