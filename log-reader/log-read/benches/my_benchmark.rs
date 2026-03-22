use criterion::{Criterion, criterion_group, criterion_main};
use data_proc::read_data;
use std::hint::black_box;
use std::path::Path;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample_size 10");
    group.sample_size(10);
    group.bench_function("main", |b| {
        b.iter(|| read_data(black_box(Path::new("/home/dtzi/rs-conc/log1.json")), black_box(3)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
