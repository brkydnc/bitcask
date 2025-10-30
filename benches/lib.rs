use bitcask::Bitcask;
use bytes::Bytes;
use criterion::{Criterion, criterion_group, criterion_main};
use tempfile::tempdir;

fn bench_put(c: &mut Criterion) {
    let mut group = c.benchmark_group("put");

    group.bench_function("constant_key_value_size", |b| {
        let dir = tempdir().unwrap();
        let mut bitcask = Bitcask::open(&dir, Default::default()).unwrap();

        let key = Box::leak(vec![0u8; 64].into_boxed_slice());
        let value = Box::leak(vec![0u8; 64].into_boxed_slice());

        rand::fill(key);
        rand::fill(value);

        b.iter(|| {
            bitcask
                .put(Bytes::from_static(key), Bytes::from_static(value))
                .unwrap()
        });

        std::fs::remove_dir_all(dir).unwrap();
    });
}

criterion_group!(put, bench_put);
criterion_main!(put);
