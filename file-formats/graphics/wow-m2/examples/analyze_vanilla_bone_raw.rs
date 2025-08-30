use std::fs;
use std::io::{Cursor, Read, Seek, SeekFrom};

fn main() -> anyhow::Result<()> {
    println!("🔬 ANALYZING VANILLA BONE RAW STRUCTURE");
    println!("=======================================");

    let model_path = "/home/danielsreichenbach/Repos/github.com/wowemulation-dev/blender-wow-addon/sample_data/1.12.1/m2/Rabbit.m2";
    let data = fs::read(model_path)?;
    let mut cursor = Cursor::new(&data);

    // Skip to bones at 0x3C
    cursor.seek(SeekFrom::Start(0x3C))?;
    let bones_count = read_u32(&mut cursor)?;
    let bones_offset = read_u32(&mut cursor)?;

    println!("📦 Rabbit.m2 (Vanilla v256):");
    println!("  Bones: {} at offset 0x{:X}", bones_count, bones_offset);
    println!();

    // Analyze first bone's raw bytes
    cursor.seek(SeekFrom::Start(bones_offset as u64))?;

    println!("🔍 First bone raw data (hex dump):");
    let mut bone_bytes = vec![0u8; 88]; // Read 88 bytes to see the pattern
    cursor.read_exact(&mut bone_bytes)?;

    for (i, chunk) in bone_bytes.chunks(16).enumerate() {
        print!("  0x{:04X}: ", i * 16);
        for byte in chunk {
            print!("{:02X} ", byte);
        }
        println!();
    }

    println!("\n📊 Interpreting bone fields:");
    cursor.seek(SeekFrom::Start(bones_offset as u64))?;

    // Standard fields
    let bone_id = read_i32(&mut cursor)?;
    let flags = read_u32(&mut cursor)?;
    let parent = read_i16(&mut cursor)?;
    let submesh = read_u16(&mut cursor)?;

    println!("  bone_id: {} (0x{:08X})", bone_id, bone_id);
    println!("  flags: 0x{:08X}", flags);
    println!("  parent: {}", parent);
    println!("  submesh: {}", submesh);

    // Now read the next bytes to understand the structure
    println!("\n  After standard fields (offset 0x0C):");
    let pos = cursor.position();

    // Try reading as direct values (not M2Track)
    let trans_x = read_f32(&mut cursor)?;
    let trans_y = read_f32(&mut cursor)?;
    let trans_z = read_f32(&mut cursor)?;
    println!(
        "    As Vec3 translation: ({:.3}, {:.3}, {:.3})",
        trans_x, trans_y, trans_z
    );

    // Reset and try reading as M2Track header
    cursor.seek(SeekFrom::Start(pos))?;
    let interp_type = read_u16(&mut cursor)?;
    let global_seq = read_u16(&mut cursor)?;
    let ts_count = read_u32(&mut cursor)?;
    let ts_offset = read_u32(&mut cursor)?;
    println!(
        "    As M2Track: interp={}, global_seq={}, timestamps={}@0x{:X}",
        interp_type, global_seq, ts_count, ts_offset
    );

    // Check what makes more sense
    if trans_x.abs() < 100.0
        && trans_y.abs() < 100.0
        && trans_z.abs() < 100.0
        && !trans_x.is_nan()
        && !trans_y.is_nan()
        && !trans_z.is_nan()
    {
        println!("    ✅ Looks like direct Vec3 values!");
    } else if ts_offset > 0x1000 && ts_offset < data.len() as u32 {
        println!("    ✅ Looks like M2Track structure!");
    }

    // Analyze bone 1 to confirm pattern
    println!("\n🔍 Second bone analysis:");
    cursor.seek(SeekFrom::Start(bones_offset as u64 + 84))?; // Try 84-byte offset

    let bone2_id = read_i32(&mut cursor)?;
    let bone2_flags = read_u32(&mut cursor)?;
    let bone2_parent = read_i16(&mut cursor)?;

    println!("  At 84-byte offset:");
    println!("    bone_id: {} (0x{:08X})", bone2_id, bone2_id);
    println!("    flags: 0x{:08X}", bone2_flags);
    println!("    parent: {}", bone2_parent);

    // Try 88-byte offset
    cursor.seek(SeekFrom::Start(bones_offset as u64 + 88))?;
    let bone2b_id = read_i32(&mut cursor)?;
    let bone2b_flags = read_u32(&mut cursor)?;
    let bone2b_parent = read_i16(&mut cursor)?;

    println!("\n  At 88-byte offset:");
    println!("    bone_id: {} (0x{:08X})", bone2b_id, bone2b_id);
    println!("    flags: 0x{:08X}", bone2b_flags);
    println!("    parent: {}", bone2b_parent);

    // Determine which offset looks correct
    if bone2_parent >= -1 && bone2_parent < bones_count as i16 {
        println!("\n  ✅ 84-byte spacing looks correct!");
    } else if bone2b_parent >= -1 && bone2b_parent < bones_count as i16 {
        println!("\n  ✅ 88-byte spacing looks correct!");
    }

    Ok(())
}

fn read_u32<R: Read>(r: &mut R) -> std::io::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_i32<R: Read>(r: &mut R) -> std::io::Result<i32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

fn read_i16<R: Read>(r: &mut R) -> std::io::Result<i16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf)?;
    Ok(i16::from_le_bytes(buf))
}

fn read_u16<R: Read>(r: &mut R) -> std::io::Result<u16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_f32<R: Read>(r: &mut R) -> std::io::Result<f32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}
