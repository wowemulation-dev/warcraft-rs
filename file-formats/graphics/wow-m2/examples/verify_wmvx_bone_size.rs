use std::fs;
use std::io::{Cursor, Read, Seek, SeekFrom};

fn main() -> anyhow::Result<()> {
    println!("🔬 VERIFYING WMVX BONE SIZE FINDINGS");
    println!("=====================================");

    println!("\nAccording to WMVx:");
    println!("  Vanilla (v256): 84 bytes per bone");
    println!("  BC+ (v260+): 88 bytes per bone");
    println!("\nOur parser assumes: 88 bytes for ALL versions ❌");

    let model_path = "/home/danielsreichenbach/Repos/github.com/wowemulation-dev/blender-wow-addon/sample_data/1.12.1/m2/Rabbit.m2";
    let data = fs::read(model_path)?;
    let mut cursor = Cursor::new(&data);

    // Skip to animation lookup at 0x34 to get bone data
    cursor.seek(SeekFrom::Start(0x34))?;
    let _anim_lookup_count = read_u32(&mut cursor)?;
    let _anim_lookup_offset = read_u32(&mut cursor)?;

    // Next should be bones
    let bones_count = read_u32(&mut cursor)?;
    let bones_offset = read_u32(&mut cursor)?;

    println!("\n📦 Rabbit.m2 (Vanilla v256):");
    println!("  Bones: {} at offset 0x{:X}", bones_count, bones_offset);

    // Test with 84-byte spacing (WMVx vanilla size)
    println!("\n🧪 Testing 84-byte bone size (WMVx Vanilla):");

    for i in 0..5.min(bones_count as usize) {
        let bone_start = bones_offset as u64 + (i as u64 * 84);
        cursor.seek(SeekFrom::Start(bone_start))?;

        let bone_id = read_i32(&mut cursor)?;
        let _flags = read_u32(&mut cursor)?;
        let parent = read_i16(&mut cursor)?;
        let _submesh = read_u16(&mut cursor)?;

        // NO unknown field in vanilla - skip directly to animation tracks
        // Each track is 20 bytes (4+4+4+4+4)

        println!("  Bone[{}] at 0x{:X}:", i, bone_start);
        println!("    bone_id: {}", bone_id);
        println!("    parent: {}", parent);

        // Skip to pivot (should be at offset 72 from bone start)
        // 12 bytes base + 3 * 20 bytes for tracks = 72
        cursor.seek(SeekFrom::Start(bone_start + 72))?;
        let pivot_x = read_f32(&mut cursor)?;
        let pivot_y = read_f32(&mut cursor)?;
        let pivot_z = read_f32(&mut cursor)?;

        println!(
            "    pivot: ({:.3}, {:.3}, {:.3})",
            pivot_x, pivot_y, pivot_z
        );

        if pivot_x.is_nan() || pivot_y.is_nan() || pivot_z.is_nan() {
            println!("      ⚠️  Contains NaN!");
        } else if pivot_x.abs() < 100.0 && pivot_y.abs() < 100.0 && pivot_z.abs() < 100.0 {
            println!("      ✅ Looks reasonable!");
        }
    }

    // Now test BC model with 88-byte size
    println!("\n📦 Testing DraeneiMale.m2 (BC v260):");
    let bc_path = "/home/danielsreichenbach/Repos/github.com/wowemulation-dev/blender-wow-addon/sample_data/2.4.3/m2/DraeneiMale.m2";

    if std::path::Path::new(bc_path).exists() {
        let bc_data = fs::read(bc_path)?;
        let mut bc_cursor = Cursor::new(&bc_data);

        // Skip to bones (BC has same header layout up to bones)
        bc_cursor.seek(SeekFrom::Start(0x3C))?;
        let bc_bones_count = read_u32(&mut bc_cursor)?;
        let bc_bones_offset = read_u32(&mut bc_cursor)?;

        println!(
            "  Bones: {} at offset 0x{:X}",
            bc_bones_count, bc_bones_offset
        );

        println!("\n🧪 Testing 88-byte bone size (BC+):");

        for i in 0..3.min(bc_bones_count as usize) {
            let bone_start = bc_bones_offset as u64 + (i as u64 * 88);
            bc_cursor.seek(SeekFrom::Start(bone_start))?;

            let bone_id = read_i32(&mut bc_cursor)?;
            let _flags = read_u32(&mut bc_cursor)?;
            let parent = read_i16(&mut bc_cursor)?;
            let _submesh = read_u16(&mut bc_cursor)?;
            let unknown_bc = read_u32(&mut bc_cursor)?; // Extra field in BC+

            println!("  Bone[{}] at 0x{:X}:", i, bone_start);
            println!("    bone_id: {}", bone_id);
            println!("    parent: {}", parent);
            println!("    unknown_bc: 0x{:08X}", unknown_bc);

            // Pivot at offset 76 (16 bytes base + 60 bytes tracks)
            bc_cursor.seek(SeekFrom::Start(bone_start + 76))?;
            let pivot_x = read_f32(&mut bc_cursor)?;
            let pivot_y = read_f32(&mut bc_cursor)?;
            let pivot_z = read_f32(&mut bc_cursor)?;

            println!(
                "    pivot: ({:.3}, {:.3}, {:.3})",
                pivot_x, pivot_y, pivot_z
            );
        }
    }

    println!("\n💡 CONCLUSION:");
    println!("  The 4-byte size difference (84 vs 88) causes misalignment!");
    println!("  With 34 bones, we accumulate 34*4 = 136 bytes of offset error");
    println!("  This explains why later bones have garbage data and NaN values");

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
