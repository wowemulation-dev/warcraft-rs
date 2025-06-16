use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use wow_cdbc::{DbcParser, FieldType, Schema, SchemaField, StringRef};

fn create_large_test_dbc(record_count: u32) -> Vec<u8> {
    let mut data = Vec::new();

    // Calculate string block size first
    let mut string_block = Vec::new();
    string_block.push(0); // Empty string at offset 0

    let mut offsets = Vec::new();
    for i in 0..record_count {
        offsets.push(string_block.len() as u32);
        let name = format!("Item_{}", i);
        string_block.extend_from_slice(name.as_bytes());
        string_block.push(0); // Null terminator
    }

    // Header
    data.extend_from_slice(b"WDBC"); // Magic
    data.extend_from_slice(&record_count.to_le_bytes()); // Record count
    data.extend_from_slice(&3u32.to_le_bytes()); // Field count
    data.extend_from_slice(&12u32.to_le_bytes()); // Record size
    data.extend_from_slice(&(string_block.len() as u32).to_le_bytes()); // String block size

    // Records
    for i in 0..record_count {
        data.extend_from_slice(&i.to_le_bytes()); // ID
        data.extend_from_slice(&offsets[i as usize].to_le_bytes()); // Name offset
        data.extend_from_slice(&(i * 100).to_le_bytes()); // Value
    }

    // String block
    data.extend_from_slice(&string_block);

    data
}

fn benchmark_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("dbc_parsing");

    // Test different record counts
    for record_count in [100, 1000, 10000, 100000].iter() {
        let data = create_large_test_dbc(*record_count);
        let size = data.len() as u64;

        group.throughput(Throughput::Bytes(size));
        group.bench_with_input(
            BenchmarkId::new("standard_parsing", record_count),
            &data,
            |b, data| {
                b.iter(|| {
                    let parser = DbcParser::parse_bytes(black_box(data)).unwrap();
                    black_box(parser.parse_records()).unwrap();
                })
            },
        );

        #[cfg(feature = "parallel")]
        group.bench_with_input(
            BenchmarkId::new("parallel_parsing", record_count),
            &data,
            |b, data| {
                b.iter(|| {
                    let parser = DbcParser::parse_bytes(black_box(data)).unwrap();
                    let header = parser.header();
                    let string_block = parser.parse_records().unwrap().string_block().clone();

                    black_box(wow_cdbc::parse_records_parallel(
                        data,
                        header,
                        None,
                        std::sync::Arc::new(string_block),
                    ))
                    .unwrap();
                })
            },
        );
    }

    group.finish();
}

fn benchmark_string_lookups(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_lookups");

    let data = create_large_test_dbc(10000);
    let parser = DbcParser::parse_bytes(&data).unwrap();
    let record_set = parser.parse_records().unwrap();

    // Generate string references
    let string_refs: Vec<StringRef> = (0..1000).map(|i| StringRef::new((i * 10) as u32)).collect();

    group.bench_function("standard_string_lookup", |b| {
        b.iter(|| {
            for &string_ref in &string_refs {
                black_box(record_set.get_string(string_ref)).unwrap();
            }
        })
    });

    // Clone record set and enable caching
    let mut cached_record_set = record_set.clone();
    cached_record_set.enable_string_caching();

    group.bench_function("cached_string_lookup", |b| {
        b.iter(|| {
            for &string_ref in &string_refs {
                black_box(cached_record_set.get_string(string_ref)).unwrap();
            }
        })
    });

    group.finish();
}

fn benchmark_key_lookups(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_lookups");

    let data = create_large_test_dbc(10000);

    let mut schema = Schema::new("Test");
    schema.add_field(SchemaField::new("ID", FieldType::UInt32));
    schema.add_field(SchemaField::new("Name", FieldType::String));
    schema.add_field(SchemaField::new("Value", FieldType::UInt32));
    schema.set_key_field("ID");

    let parser = DbcParser::parse_bytes(&data).unwrap();
    let parser = parser.with_schema(schema).unwrap();
    let record_set = parser.parse_records().unwrap();

    // Generate keys to look up
    let keys: Vec<u32> = (0..1000).collect();

    group.bench_function("hashmap_key_lookup", |b| {
        b.iter(|| {
            for &key in &keys {
                black_box(record_set.get_record_by_key(key));
            }
        })
    });

    // Clone record set and create sorted key map
    let mut sorted_record_set = record_set.clone();
    sorted_record_set.create_sorted_key_map().unwrap();

    group.bench_function("binary_search_key_lookup", |b| {
        b.iter(|| {
            for &key in &keys {
                black_box(sorted_record_set.get_record_by_key_binary_search(key));
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_parsing,
    benchmark_string_lookups,
    benchmark_key_lookups
);
criterion_main!(benches);
