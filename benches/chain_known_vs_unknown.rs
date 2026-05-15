use criterion::{Criterion, criterion_group, criterion_main};
use orx_concurrent_iter::*;
use orx_criterion::{Experiment, Factors};
use std::hint::black_box;

#[derive(Clone, Copy)]
struct Input {
    n_i_exp: usize,
    n_j_exp: usize,
}

impl Factors for Input {
    fn factor_names() -> Vec<&'static str> {
        vec!["n_i", "n_j"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![format!("2e{}", self.n_i_exp), format!("2e{}", self.n_j_exp)]
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
    fn consume_known(i: &[usize], j: &[usize]) -> u64 {
        let chain = i.con_iter().chain(j);

        let mut sum = 0u64;
        while let Some(x) = chain.next() {
            sum = sum.wrapping_add(*x as u64);
        }
        black_box(sum)
    }

    fn consume_unknown(i: &[usize], j: &[usize]) -> u64 {
        let chain = i.con_iter().chain_inexact(j);

        let mut sum = 0u64;
        while let Some(x) = chain.next() {
            sum = sum.wrapping_add(*x as u64);
        }
        black_box(sum)
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
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let (i, j) = input;
        match alg_variant {
            Method::KnownLen => Self::consume_known(i, j),
            Method::UnknownLen => Self::consume_unknown(i, j),
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
        },
        Input {
            n_i_exp: 16,
            n_j_exp: 12,
        },
        Input {
            n_i_exp: 16,
            n_j_exp: 16,
        },
    ];

    let variants = [Method::KnownLen, Method::UnknownLen];

    Exp.bench(c, "chain_known_vs_unknown", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
