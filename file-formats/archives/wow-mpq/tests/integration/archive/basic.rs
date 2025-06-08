//! Basic tests for the wow_mpq library

#[test]
fn test_format_version_values() {
    assert_eq!(wow_mpq::FormatVersion::V1 as u16, 0);
    assert_eq!(wow_mpq::FormatVersion::V2 as u16, 1);
    assert_eq!(wow_mpq::FormatVersion::V3 as u16, 2);
    assert_eq!(wow_mpq::FormatVersion::V4 as u16, 3);
}

#[test]
fn test_sector_size_calculation() {
    assert_eq!(wow_mpq::calculate_sector_size(0), 512);
    assert_eq!(wow_mpq::calculate_sector_size(3), 4096);
    assert_eq!(wow_mpq::calculate_sector_size(8), 131072);
}

#[test]
fn test_signatures() {
    assert_eq!(wow_mpq::signatures::MPQ_ARCHIVE, 0x1A51504D);
    assert_eq!(wow_mpq::signatures::MPQ_USERDATA, 0x1B51504D);
    assert_eq!(wow_mpq::signatures::HET_TABLE, 0x1A544548);
    assert_eq!(wow_mpq::signatures::BET_TABLE, 0x1A544542);
    assert_eq!(wow_mpq::signatures::STRONG_SIGNATURE, *b"NGIS");
}
