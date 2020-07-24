use criterion::BenchmarkId;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{prelude::SliceRandom, Rng};
#[cfg(feature = "benchmark")]
use sss_rs::num_primes;
use sss_rs::*;

fn benchmark_sss_distribute(c: &mut Criterion) {
    let secret_length = 1024;
    let chunk_size = 8;
    let size = 10;
    let threshold = 10;

    let mut rng = rand::thread_rng();
    let secret: Vec<u8> = (0..1024).map(|_| rng.gen()).collect();

    c.bench_function(
        &format!(
            "distribute-{}-{}-{}-{}",
            secret_length, chunk_size, size, threshold
        ),
        |b| {
            b.iter(|| {
                distribute(
                    black_box(&secret),
                    black_box(chunk_size),
                    black_box(size),
                    black_box(threshold),
                );
            })
        },
    );
}

fn benchmark_sss_reconstruct(c: &mut Criterion) {
    let secret_length = 1024;
    let chunk_size = 8;
    let size = 10;
    let threshold = 10;

    let mut rng = rand::thread_rng();
    let secret: Vec<u8> = (0..1024).map(|_| rng.gen()).collect();
    let mut shares = distribute(&secret, chunk_size, size, threshold);
    shares.shuffle(&mut rng);

    c.bench_function(
        &format!(
            "reconstruct-{}-{}-{}-{}",
            secret_length, chunk_size, size, threshold
        ),
        |b| {
            b.iter(|| {
                reconstruct(black_box(&shares[0..threshold])).unwrap();
            })
        },
    );
}

#[cfg(feature = "benchmark")]
pub fn benchmark_new_prime(c: &mut Criterion) {
    c.bench_function("new prime 256", |b| {
        b.iter(|| {
            num_primes::Generator::new_prime(256);
        })
    });
}

#[cfg(feature = "benchmark")]
pub fn benchmark_beta(c: &mut Criterion) {
    let secret_length = 1024;
    let chunk_size = 8;
    let size = 10;
    let threshold = 10;

    let mut rng = rand::thread_rng();
    let secret: Vec<u8> = (0..1024).map(|_| rng.gen()).collect();
    let mut shares = distribute(&secret, chunk_size, size, threshold);
    shares.shuffle(&mut rng);

    let parameter_string = format!("{}-{}-{}-{}", secret_length, chunk_size, size, threshold);
    let input = (shares[0].prime(), &shares[0], &shares);
    c.bench_with_input(
        BenchmarkId::new("beta", parameter_string),
        &input,
        |b, (prime, current, shares)| {
            b.iter(|| {
                beta_wrapper(prime, current, shares);
            })
        },
    );
}

criterion_group!(benches, benchmark_sss_distribute, benchmark_sss_reconstruct);
#[cfg(feature = "benchmark")]
criterion_group!(simple, benchmark_new_prime, benchmark_beta,);

#[cfg(feature = "benchmark")]
criterion_main!(benches, simple);

#[cfg(not(feature = "benchmark"))]
criterion_main!(benches);
