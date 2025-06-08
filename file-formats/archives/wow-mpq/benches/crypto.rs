use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use wow_mpq::crypto::{ENCRYPTION_TABLE, decrypt_block, decrypt_dword, encrypt_block};

fn bench_encryption_table_access(c: &mut Criterion) {
    c.bench_function("encryption_table_access", |b| {
        b.iter(|| {
            // Access various parts of the encryption table
            let sum = ENCRYPTION_TABLE[0]
                + ENCRYPTION_TABLE[0x100]
                + ENCRYPTION_TABLE[0x200]
                + ENCRYPTION_TABLE[0x300]
                + ENCRYPTION_TABLE[0x400];
            black_box(sum);
        });
    });
}

fn bench_encrypt_block(c: &mut Criterion) {
    let mut data = vec![0x12345678u32; 1024]; // 4KB of data
    let key = 0xC1EB1CEF;

    c.bench_function("encrypt_block_4kb", |b| {
        b.iter(|| {
            encrypt_block(&mut data, black_box(key));
        });
    });
}

fn bench_decrypt_block(c: &mut Criterion) {
    let mut data = vec![0x12345678u32; 1024]; // 4KB of data
    let key = 0xC1EB1CEF;

    c.bench_function("decrypt_block_4kb", |b| {
        b.iter(|| {
            decrypt_block(&mut data, black_box(key));
        });
    });
}

fn bench_decrypt_dword(c: &mut Criterion) {
    let value = 0x6DBB9D94;
    let key = 0xC1EB1CEF;

    c.bench_function("decrypt_dword", |b| {
        b.iter(|| {
            black_box(decrypt_dword(black_box(value), black_box(key)));
        });
    });
}

fn bench_round_trip(c: &mut Criterion) {
    let original_data = vec![0x12345678u32; 1024]; // 4KB
    let key = 0xC1EB1CEF;

    c.bench_function("encrypt_decrypt_round_trip_4kb", |b| {
        b.iter(|| {
            let mut data = original_data.clone();
            encrypt_block(&mut data, key);
            decrypt_block(&mut data, key);
            black_box(data);
        });
    });
}

criterion_group!(
    benches,
    bench_encryption_table_access,
    bench_encrypt_block,
    bench_decrypt_block,
    bench_decrypt_dword,
    bench_round_trip
);
criterion_main!(benches);
