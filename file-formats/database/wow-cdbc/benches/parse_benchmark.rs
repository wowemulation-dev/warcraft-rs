use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::io::Cursor;
use wow_cdbc::{DbcParser, FieldType, Schema, SchemaField};

fn create_test_dbc() -> Vec<u8> {
    let mut data = Vec::new();

    // Calculate string block size first
    let mut string_block = Vec::new();
    string_block.push(0); // Empty string at offset 0

    let mut offsets = Vec::new();
    for i in 0..1000 {
        offsets.push(string_block.len() as u32);
        let name = format!("Item_{i}");
        string_block.extend_from_slice(name.as_bytes());
        string_block.push(0); // Null terminator
    }

    // Header
    data.extend_from_slice(b"WDBC"); // Magic
    data.extend_from_slice(&1000u32.to_le_bytes()); // Record count
    data.extend_from_slice(&3u32.to_le_bytes()); // Field count
    data.extend_from_slice(&12u32.to_le_bytes()); // Record size
    data.extend_from_slice(&(string_block.len() as u32).to_le_bytes()); // String block size

    // Records
    for (i, &offset) in offsets.iter().enumerate() {
        data.extend_from_slice(&(i as u32).to_le_bytes()); // ID
        data.extend_from_slice(&offset.to_le_bytes()); // Name offset
        data.extend_from_slice(&((i * 100) as u32).to_le_bytes()); // Value
    }

    // String block
    data.extend_from_slice(&string_block);

    data
}

fn parse_benchmark(c: &mut Criterion) {
    let data = create_test_dbc();

    c.bench_function("parse_header", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(black_box(&data));
            let parser = DbcParser::parse(&mut cursor).unwrap();
            black_box(parser.header());
        })
    });

    c.bench_function("parse_records_raw", |b| {
        b.iter(|| {
            let parser = DbcParser::parse_bytes(black_box(&data)).unwrap();
            black_box(parser.parse_records()).unwrap();
        })
    });

    c.bench_function("parse_records_with_schema", |b| {
        b.iter(|| {
            let parser = DbcParser::parse_bytes(black_box(&data)).unwrap();

            let mut schema = Schema::new("Test");
            schema.add_field(SchemaField::new("ID", FieldType::UInt32));
            schema.add_field(SchemaField::new("Name", FieldType::String));
            schema.add_field(SchemaField::new("Value", FieldType::UInt32));
            schema.set_key_field("ID");

            let parser = parser.with_schema(schema).unwrap();
            black_box(parser.parse_records()).unwrap();
        })
    });
}

criterion_group!(benches, parse_benchmark);
criterion_main!(benches);
