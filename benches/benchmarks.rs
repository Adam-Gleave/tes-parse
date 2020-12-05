use criterion::{criterion_group, criterion_main, Criterion};
use std::fs::File;
use std::path::PathBuf;
use tes_parse::read_plugin;

fn read_benchmark(c: &mut Criterion) {
    let path = PathBuf::from(format!(
        "{}{}",
        env!("CARGO_MANIFEST_DIR"),
        "/data/Skyrim.esm"
    ));

    c.bench_function("read_plugin", move |b| {
        b.iter_with_setup(|| File::open(&path).unwrap(), |file| read_plugin(file))
    });
}

criterion_group!(benches, read_benchmark);
criterion_main!(benches);
