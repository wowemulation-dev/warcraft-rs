//! Integration tests for the ADT parser

use pretty_assertions::assert_eq;
use std::io::Cursor;
use wow_adt::{AdtVersion, ChunkHeader, MverChunk};

#[test]
fn test_chunk_header_parsing() {
    use binrw::BinRead;

    // Create a simple chunk header
    let mut data = Vec::new();
    data.extend_from_slice(b"REVM"); // Magic (stored reversed in file)
    data.extend_from_slice(&[4, 0, 0, 0]); // Size (4 bytes, little endian)

    let mut cursor = Cursor::new(data);

    // Parse the header
    let header = ChunkHeader::read(&mut cursor).unwrap();

    // Validate the result
    assert_eq!(header.magic, *b"MVER"); // After reversing during read
    assert_eq!(header.size, 4);
    assert_eq!(header.magic_as_string(), "MVER");
}

#[test]
fn test_mver_parsing() {
    use binrw::BinRead;

    // Create a simple MVER chunk
    let mut data = Vec::new();
    data.extend_from_slice(b"REVM"); // Magic (stored reversed in file)
    data.extend_from_slice(&[4, 0, 0, 0]); // Size (4 bytes, little endian)
    data.extend_from_slice(&[18, 0, 0, 0]); // Version (18, standard for ADT)

    let mut cursor = Cursor::new(data);

    // Parse the MVER chunk
    let mver = MverChunk::read(&mut cursor).unwrap();

    // Validate the result
    assert_eq!(mver.version, 18);
}

#[test]
fn test_version_comparison() {
    // Test version ordering for supported versions
    assert!(AdtVersion::VanillaEarly < AdtVersion::VanillaLate);
    assert!(AdtVersion::VanillaLate < AdtVersion::TBC);
    assert!(AdtVersion::TBC < AdtVersion::WotLK);
    assert!(AdtVersion::WotLK < AdtVersion::Cataclysm);
    assert!(AdtVersion::Cataclysm < AdtVersion::MoP);
}

#[test]
fn test_version_to_string() {
    assert_eq!(AdtVersion::VanillaEarly.to_string(), "Vanilla 1.x (Early)");
    assert_eq!(AdtVersion::VanillaLate.to_string(), "Vanilla 1.9+");
    assert_eq!(AdtVersion::TBC.to_string(), "The Burning Crusade 2.x");
    assert_eq!(AdtVersion::WotLK.to_string(), "Wrath of the Lich King 3.x");
    assert_eq!(AdtVersion::Cataclysm.to_string(), "Cataclysm 4.x");
    assert_eq!(AdtVersion::MoP.to_string(), "Mists of Pandaria 5.x");
}
