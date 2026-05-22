use criterion::{Criterion, criterion_group, criterion_main};
use orx_concurrent_iter::*;
use orx_criterion::{Experiment, Factors};
use std::hint::black_box;

#[derive(Clone, Copy)]
struct Input {
    n_i_exp: usize,
    n_j_exp: usize,
    num_threads: usize,
}

impl Factors for Input {
    fn factor_names() -> Vec<&'static str> {
        vec!["n_i", "n_j", "num_threads"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("2e{}", self.n_i_exp),
            format!("2e{}", self.n_j_exp),
            self.num_threads.to_string(),
        ]
    }
}

#[derive(Clone, Copy)]
enum Method {
    KnownLen,
    UnknownLen,
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["chain-type"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            match self {
                Self::KnownLen => "known-len",
                Self::UnknownLen => "unknown-len",
            }
            .to_string(),
        ]
    }
}

struct Exp;

impl Exp {
    fn consume_known(i: &[usize], j: &[usize], num_threads: usize) -> u64 {
        let chain = i.con_iter().chain(j);

        std::thread::scope(|s| {
            let mut handles = Vec::with_capacity(num_threads);
            for _ in 0..num_threads {
                handles.push(s.spawn(|| {
                    let mut sum = 0u64;
                    while let Some(x) = chain.next() {
                        sum = sum.wrapping_add(*x as u64);
                    }
                    sum
                }));
            }

            let sum = handles
                .into_iter()
                .map(|handle| handle.join().expect("failed to join thread"))
                .fold(0u64, |acc, value| acc.wrapping_add(value));

            black_box(sum)
        })
    }

    fn consume_unknown(i: &[usize], j: &[usize], num_threads: usize) -> u64 {
        let chain = i.con_iter().chain_inexact(j);

        std::thread::scope(|s| {
            let mut handles = Vec::with_capacity(num_threads);
            for _ in 0..num_threads {
                handles.push(s.spawn(|| {
                    let mut sum = 0u64;
                    while let Some(x) = chain.next() {
                        sum = sum.wrapping_add(*x as u64);
                    }
                    sum
                }));
            }

            let sum = handles
                .into_iter()
                .map(|handle| handle.join().expect("failed to join thread"))
                .fold(0u64, |acc, value| acc.wrapping_add(value));

            black_box(sum)
        })
    }
}

impl Experiment for Exp {
    type InputFactors = Input;
    type AlgFactors = Method;
    type Input = (Vec<usize>, Vec<usize>);
    type Output = u64;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        let n_i = 1usize << input_variant.n_i_exp;
        let n_j = 1usize << input_variant.n_j_exp;

        let i: Vec<usize> = (0..n_i).map(|x| x % 1024).collect();
        let j: Vec<usize> = (0..n_j).map(|x| (x + 7) % 2048).collect();

        (i, j)
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let (i, j) = input;
        match alg_variant {
            Method::KnownLen => Self::consume_known(i, j, input_variant.num_threads),
            Method::UnknownLen => Self::consume_unknown(i, j, input_variant.num_threads),
        }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        let (i, j) = input;
        Some(
            i.iter()
                .chain(j.iter())
                .fold(0u64, |acc, x| acc.wrapping_add(*x as u64)),
        )
    }

    fn validate_output(&self, _: &Self::InputFactors, _: &Self::Input, _: &Self::Output) {}
}

fn run(c: &mut Criterion) {
    let treatments = [
        Input {
            n_i_exp: 12,
            n_j_exp: 12,
            num_threads: 1,
        },
        Input {
            n_i_exp: 16,
            n_j_exp: 12,
            num_threads: 1,
        },
        Input {
            n_i_exp: 16,
            n_j_exp: 16,
            num_threads: 1,
        },
        Input {
            n_i_exp: 12,
            n_j_exp: 12,
            num_threads: 16,
        },
        Input {
            n_i_exp: 16,
            n_j_exp: 12,
            num_threads: 16,
        },
        Input {
            n_i_exp: 16,
            n_j_exp: 16,
            num_threads: 16,
        },
        Input {
            n_i_exp: 12,
            n_j_exp: 12,
            num_threads: 32,
        },
        Input {
            n_i_exp: 16,
            n_j_exp: 12,
            num_threads: 32,
        },
        Input {
            n_i_exp: 16,
            n_j_exp: 16,
            num_threads: 32,
        },
    ];

    let variants = [Method::KnownLen, Method::UnknownLen];

    Exp.bench(c, "chain_known_vs_unknown", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
