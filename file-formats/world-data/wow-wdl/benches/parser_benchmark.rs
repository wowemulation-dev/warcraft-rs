//! Benchmarks for the WDL parser

use criterion::{Criterion, criterion_group, criterion_main};
use std::io::Cursor;

use wow_wdl::conversion::convert_wdl_file;
use wow_wdl::parser::WdlParser;
use wow_wdl::types::*;
use wow_wdl::version::WdlVersion;

fn create_test_file() -> Vec<u8> {
    let mut file = WdlFile::with_version(WdlVersion::Wotlk);

    // Create some test data

    // Add a WMO filename and index
    file.wmo_filenames
        .push("World/wmo/Azeroth/Buildings/Human_Farm/Farm.wmo".to_string());
    file.wmo_indices.push(0);

    // Add a WMO placement
    let placement = ModelPlacement {
        id: 1,
        wmo_id: 0,
        position: Vec3d::new(100.0, 200.0, 50.0),
        rotation: Vec3d::new(0.0, 0.0, 0.0),
        bounds: BoundingBox {
            min: Vec3d::new(-10.0, -10.0, -10.0),
            max: Vec3d::new(10.0, 10.0, 10.0),
        },
        flags: 0,
        doodad_set: 0,
        name_set: 0,
        padding: 0,
    };
    file.wmo_placements.push(placement);

    // Add a heightmap tile
    let mut heightmap = HeightMapTile::new();
    for i in 0..HeightMapTile::OUTER_COUNT {
        heightmap.outer_values[i] = (i as i16) % 100;
    }
    for i in 0..HeightMapTile::INNER_COUNT {
        heightmap.inner_values[i] = ((i + 100) as i16) % 100;
    }
    file.heightmap_tiles.insert((10, 20), heightmap);

    // Set the offset for this tile (actual value will be calculated during write)
    file.map_tile_offsets[20 * 64 + 10] = 1;

    // Add holes data
    let mut holes = HolesData::new();
    holes.set_hole(5, 7, true);
    holes.set_hole(8, 9, true);
    file.holes_data.insert((10, 20), holes);

    // Write the file to a buffer
    let parser = WdlParser::with_version(WdlVersion::Wotlk);
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    parser.write(&mut cursor, &file).unwrap();

    buffer
}

fn bench_parse(c: &mut Criterion) {
    let data = create_test_file();

    c.bench_function("parse_wdl", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(&data);
            let parser = WdlParser::new();
            parser.parse(&mut cursor).unwrap()
        })
    });
}

fn bench_write(c: &mut Criterion) {
    let data = create_test_file();
    let mut cursor = Cursor::new(&data);
    let parser = WdlParser::new();
    let file = parser.parse(&mut cursor).unwrap();

    c.bench_function("write_wdl", |b| {
        b.iter(|| {
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            parser.write(&mut cursor, &file).unwrap();
            buffer
        })
    });
}

fn bench_convert(c: &mut Criterion) {
    let data = create_test_file();
    let mut cursor = Cursor::new(&data);
    let parser = WdlParser::with_version(WdlVersion::Wotlk);
    let file = parser.parse(&mut cursor).unwrap();

    c.bench_function("convert_wotlk_to_legion", |b| {
        b.iter(|| convert_wdl_file(&file, WdlVersion::Legion).unwrap())
    });
}

criterion_group!(benches, bench_parse, bench_write, bench_convert);
criterion_main!(benches);
