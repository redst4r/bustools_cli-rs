use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bustools_cli::multinomial::{multinomial_sample, multinomial_sample_binary_search};

/*
fn criterion_benchmark_multinomial(c: &mut Criterion) {
    // use statrs::distribution::Binomial as Binomial_statrs;
    // use probability::distribution::Binomial as Binomial_prob;
    use bustools::multinomial::{ multinomial_sample_statrs};
    // c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));

    // use bustools::io::parse_ecmatrix;

    // let fname = "/tmp/matrix.ec";
    // c.bench_function("ec1", |b| b.iter(|| parse_ecmatrix(black_box(fname.to_string()))));
    // c.bench_function("ec2", |b| b.iter(|| parse_ecmatrix2(black_box(fname.to_string()))));


    // multinomial benchmarks
    // c.bench_function("MN1", |b| b.iter(|| test_multinomial(black_box( 1000))));
    // c.bench_function("Mn2", |b| b.iter(|| test_multinomial_stats(black_box(1000))));

    // multinomial benchmarks, my implementation
    let n = 100000;
    let dim = 10000;
    let p: Vec<f64> = (1..dim).map(|x| x as f64).collect();

    c.bench_function("MN1", |b| b.iter(|| multinomial_sample(black_box( n),  black_box( p.clone()))));
    c.bench_function("MN2", |b| b.iter(|| multinomial_sample_statrs(black_box( n),  black_box( p.clone()))));
    // c.bench_function("Mn2", |b| b.iter(|| test_multinomial_stats(black_box(1000))));

    /*
    comparing the Binomial sampling algorthims in 
    statrs and probability crates
    1. statrs uses sum of bernoulli, which is very slow for large N
    2. probability uses inverse cdf sampling, much faster
    */
    // use probability::prelude::*;
    // use rand::distributions::Distribution;
    // let N = 10000;
    // let p = 0.001;

    // let mut source = source::default(42);
    // let mut r = rand::thread_rng();

    // let b1 = Binomial_prob::new(N, p);
    // let b2 = Binomial_statrs::new(p, N as u64).unwrap();

    // c.bench_function("Binomial prob", |b| b.iter(|| b1.sample(&mut source)));
    // c.bench_function("Binomial statrs", |b| b.iter(|| b2.sample(&mut r)));

}
 */
#[allow(dead_code)]
fn multinomial_speed(c: &mut Criterion){

    use probability::prelude::*;

    fn binary_search_dummy(N: u64, d: u64){

        let p: Vec<_> = (1..d).map(|x| x as f64).collect();

        let mut random_source = source::default(4);   
        multinomial_sample_binary_search(N, &p, &mut random_source);
    }

    fn binomial_dummy(N: u64, d: u64){
        let p: Vec<_> = (1..d).map(|x| x as f64).collect();

        let mut random_source = source::default(4);   
        multinomial_sample(N, &p, &mut random_source);
    }

    let dims = vec![10_000, 100_000, 1_000_000];
    let N = 1_000_000;
    for d in dims{

        let name = format!("Binary, dim {}", d);
        c.bench_function(&name, |b| b.iter(|| 
            binary_search_dummy(black_box(N), 
                            black_box(d), 
            )));

        let name = format!("Binomial, dim {}", d);
        c.bench_function(&name, |b| b.iter(|| 
            binomial_dummy(black_box(N), 
                            black_box(d), 
            )));
    }

}


criterion_group!(benches, multinomial_speed);
criterion_main!(benches);