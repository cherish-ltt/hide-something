use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use hide_something::{hide_decrypt, hide_encrypt};

/// A fixed carrier template for all benchmarks.
const TEMPLATE: &str = "The quick brown fox jumps over the lazy dog. 12345!";

/// Benchmark hide_encrypt with various input sizes.
fn bench_encrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("encrypt");

    for size in [0, 16, 64, 256, 1024, 4096] {
        let input = "a".repeat(size);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(size),
            &input,
            |b, data| {
                b.iter(|| {
                    let _ = hide_encrypt(data, TEMPLATE);
                })
            },
        );
    }
    group.finish();
}

/// Benchmark hide_decrypt with hidden texts generated from the same sizes.
fn bench_decrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("decrypt");

    for size in [0, 16, 64, 256, 1024, 4096] {
        let input = "a".repeat(size);
        // Pre‑generate the hidden text so that we measure only decryption.
        let hidden = hide_encrypt(&input, TEMPLATE).unwrap();

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(size),
            &hidden,
            |b, data| {
                b.iter(|| {
                    let _ = hide_decrypt(data);
                })
            },
        );
    }
    group.finish();
}

/// Combined benchmark that measures the round‑trip (encrypt + decrypt) end‑to‑end.
fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");

    for size in [0, 16, 64, 256, 1024, 4096] {
        let input = "a".repeat(size);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(size),
            &input,
            |b, data| {
                b.iter(|| {
                    let hidden = hide_encrypt(data, TEMPLATE).unwrap();
                    let _ = hide_decrypt(&hidden).unwrap();
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_encrypt,
    bench_decrypt,
    bench_roundtrip
);
criterion_main!(benches);