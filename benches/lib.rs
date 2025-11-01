use std::collections::HashMap;

use bitcask::Bitcask;
use bytes::{Bytes, buf};
use criterion::{Criterion, criterion_group, criterion_main};
use tempfile::tempdir;

const KB: usize = 1 << 10;

fn random_bytes(size: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; size];
    rand::fill(bytes.as_mut_slice());
    bytes
}

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

fn bench_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("get");

    group.bench_function("constant_key_value_size", |b| {
        let dir = tempdir().unwrap();
        let mut bitcask = Bitcask::open(&dir, Default::default()).unwrap();

        let bufsize = 2 * KB;
        let itemsize = 64;
        let num_items = bufsize / itemsize;

        let keys = Bytes::from_owner(random_bytes(bufsize));
        let values = Bytes::from_owner(random_bytes(bufsize));

        for i in (0..num_items).map(|n| n * itemsize) {
            bitcask
                .put(
                    keys.slice(i..(i + itemsize)),
                    values.slice(i..(i + itemsize)),
                )
                .unwrap();
        }

        b.iter(|| {
            let i = rand::random_range(0..num_items) * itemsize;
            let value = bitcask.get(keys.slice(i..(i + itemsize))).unwrap();
            std::hint::black_box(value);
        });

        std::fs::remove_dir_all(dir).unwrap();
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = bench_put, bench_get
);

criterion_main!(benches);
