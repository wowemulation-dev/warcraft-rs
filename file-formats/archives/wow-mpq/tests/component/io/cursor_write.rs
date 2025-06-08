use std::io::{Cursor, Seek, SeekFrom, Write};

#[test]
fn test_cursor_overwrite() {
    let mut cursor = Cursor::new(Vec::new());

    // Write 208 zeros (v4 header size)
    cursor.write_all(&vec![0u8; 208]).unwrap();

    // Write some data after
    cursor.write_all(b"DATA").unwrap();

    // Now seek back and write at position 0
    cursor.seek(SeekFrom::Start(0)).unwrap();
    cursor.write_all(&[0x4D, 0x50, 0x51, 0x1A]).unwrap(); // MPQ signature

    // Check the result
    let data = cursor.into_inner();
    println!("Total size: {}", data.len());
    println!("First 16 bytes: {:02X?}", &data[..16]);

    // Verify MPQ signature is at start
    assert_eq!(&data[..4], &[0x4D, 0x50, 0x51, 0x1A]);
}
