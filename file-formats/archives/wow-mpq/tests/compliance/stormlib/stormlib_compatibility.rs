//! Tests to verify StormLib compatibility

use std::fs;
use tempfile::TempDir;
use wow_mpq::{archive::Archive, builder::ArchiveBuilder, header::FormatVersion};

#[test]
fn test_compression_byte_prefix() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.mpq");

    // Create test data that will compress well
    let test_data = vec![0u8; 1000]; // Zeros compress very well

    // Create an archive with compressed file
    let builder = ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .add_file_data_with_options(
            test_data.clone(),
            "test.dat",
            wow_mpq::compression::flags::ZLIB,
            false, // encrypt
            0,     // locale
        );

    builder.build(&archive_path).unwrap();

    // Open the archive and read the file
    let mut archive = Archive::open(&archive_path).unwrap();
    let file_data = archive.read_file("test.dat").unwrap();
    assert_eq!(file_data, test_data);

    // Clean up
    drop(archive);
    fs::remove_file(&archive_path).unwrap();
}

#[test]
fn test_multi_compression_format() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test_multi.mpq");

    // Create audio-like test data
    let mut test_data = Vec::new();
    for i in 0..1000 {
        let sample = ((i as f32 * 0.1).sin() * 10000.0) as i16;
        test_data.extend_from_slice(&sample.to_le_bytes());
    }

    // Create an archive with multi-compressed file
    let builder = ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .add_file_data_with_options(
            test_data.clone(),
            "audio.wav",
            wow_mpq::compression::flags::ADPCM_MONO | wow_mpq::compression::flags::ZLIB,
            false, // encrypt
            0,     // locale
        );

    builder.build(&archive_path).unwrap();

    // Open the archive and read the file
    let mut archive = Archive::open(&archive_path).unwrap();
    let file_data = archive.read_file("audio.wav").unwrap();

    // ADPCM is lossy, so we just check the size is correct
    assert_eq!(file_data.len(), test_data.len());

    // Clean up
    drop(archive);
    fs::remove_file(&archive_path).unwrap();
}

#[test]
fn test_uncompressed_small_data() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test_small.mpq");

    // Create small test data that won't compress well
    let test_data = b"Small";

    // Create an archive with "compressed" file
    let builder = ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .add_file_data_with_options(
            test_data.to_vec(),
            "small.txt",
            wow_mpq::compression::flags::ZLIB, // Request compression
            false,                             // encrypt
            0,                                 // locale
        );

    builder.build(&archive_path).unwrap();

    // Open the archive and read the file
    let mut archive = Archive::open(&archive_path).unwrap();
    let file_data = archive.read_file("small.txt").unwrap();
    assert_eq!(file_data, test_data);

    // Verify the file is stored uncompressed (compression not beneficial)
    let file_info = archive.find_file("small.txt").unwrap().unwrap();
    assert!(
        !file_info.is_compressed(),
        "Small file should not be compressed"
    );

    // Clean up
    drop(archive);
    fs::remove_file(&archive_path).unwrap();
}
