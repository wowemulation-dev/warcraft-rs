//! hash benchmarks

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use wow_mpq::{hash_string, hash_type, jenkins_hash};

fn bench_hash_string_short(c: &mut Criterion) {
    let filename = "file.txt";

    c.bench_function("hash_string_short", |b| {
        b.iter(|| hash_string(black_box(filename), black_box(hash_type::TABLE_OFFSET)));
    });
}

fn bench_hash_string_medium(c: &mut Criterion) {
    let filename = "units\\human\\footman\\footman.mdx";

    c.bench_function("hash_string_medium", |b| {
        b.iter(|| hash_string(black_box(filename), black_box(hash_type::TABLE_OFFSET)));
    });
}

fn bench_hash_string_long(c: &mut Criterion) {
    let filename = "folder1\\folder2\\folder3\\folder4\\folder5\\folder6\\folder7\\folder8\\very_long_filename.txt";

    c.bench_function("hash_string_long", |b| {
        b.iter(|| hash_string(black_box(filename), black_box(hash_type::TABLE_OFFSET)));
    });
}

fn bench_hash_all_types(c: &mut Criterion) {
    let filename = "war3map.j";

    c.bench_function("hash_all_types", |b| {
        b.iter(|| {
            let h0 = hash_string(filename, hash_type::TABLE_OFFSET);
            let h1 = hash_string(filename, hash_type::NAME_A);
            let h2 = hash_string(filename, hash_type::NAME_B);
            let h3 = hash_string(filename, hash_type::FILE_KEY);
            black_box((h0, h1, h2, h3));
        });
    });
}

fn bench_jenkins_hash_short(c: &mut Criterion) {
    let filename = "file.txt";

    c.bench_function("jenkins_hash_short", |b| {
        b.iter(|| jenkins_hash(black_box(filename)));
    });
}

fn bench_jenkins_hash_long(c: &mut Criterion) {
    let filename = "folder1\\folder2\\folder3\\folder4\\folder5\\folder6\\folder7\\folder8\\very_long_filename.txt";

    c.bench_function("jenkins_hash_long", |b| {
        b.iter(|| jenkins_hash(black_box(filename)));
    });
}

fn bench_hash_with_path_conversion(c: &mut Criterion) {
    let filename = "path/to/some/file/with/forward/slashes.txt";

    c.bench_function("hash_with_path_conversion", |b| {
        b.iter(|| hash_string(black_box(filename), black_box(hash_type::TABLE_OFFSET)));
    });
}

fn bench_hash_case_conversion(c: &mut Criterion) {
    let filename = "MiXeD_CaSe_FiLeNaMe.TxT";

    c.bench_function("hash_case_conversion", |b| {
        b.iter(|| hash_string(black_box(filename), black_box(hash_type::TABLE_OFFSET)));
    });
}

criterion_group!(
    benches,
    bench_hash_string_short,
    bench_hash_string_medium,
    bench_hash_string_long,
    bench_hash_all_types,
    bench_jenkins_hash_short,
    bench_jenkins_hash_long,
    bench_hash_with_path_conversion,
    bench_hash_case_conversion
);
criterion_main!(benches);
