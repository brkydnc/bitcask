use bitcask::{Bitcask, Error, OpenOptions};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

pub fn write_only_benchmark(c: &mut Criterion) {}

criterion_group!(benches, write_only_benchmark);
criterion_main!(benches);
