use crate::alg::Method;
use crate::input::{ComputeType, InputVariant, PipelineType};
use bench_helper::runner::cpu_mix;
use orx_concurrent_iter::*;
use orx_criterion::Experiment;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::hint::black_box;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Agg {
    pub sum: u64,
    pub xor_sum: u64,
    pub count: u64,
}

impl Agg {
    #[inline(always)]
    pub fn from_val(v: u64) -> Self {
        Self {
            sum: v,
            xor_sum: v,
            count: 1,
        }
    }
}

#[inline(always)]
pub fn merge(a: Agg, b: Agg) -> Agg {
    Agg {
        sum: a.sum.wrapping_add(b.sum),
        xor_sum: a.xor_sum ^ b.xor_sum,
        count: a.count + b.count,
    }
}

// ---------------------------------------------------------------------------
// Large / Allocating Output (mirrors benches/con_iter_of_iter.rs)
// ---------------------------------------------------------------------------

const NUM_NUMBERS: usize = 4;
const NUM_VECTORS: usize = 4;
const LEN_VECTORS: usize = 4;

#[derive(Clone, Debug)]
pub struct LargeOutput {
    name: String,
    numbers: [i64; NUM_NUMBERS],
    vectors: Vec<Vec<i64>>,
}

fn compute_alloc(val: u64) -> LargeOutput {
    let idx = val as usize;
    let prefix = match idx % 7 {
        0 => "zero-",
        3 => "three-",
        _ => "sth-",
    };
    let fib_val = bench_helper::runner::fib(50, val);
    let name = format!("{}-fib-{}", prefix, fib_val);

    let mut numbers = [0i64; NUM_NUMBERS];
    for (i, x) in numbers.iter_mut().enumerate() {
        *x = match (idx * 7 + i) % 3 {
            0 => idx as i64 + i as i64,
            _ => idx as i64 - i as i64,
        };
    }

    let mut vectors = Vec::with_capacity(NUM_VECTORS);
    for i in 0..NUM_VECTORS {
        let mut vec = Vec::with_capacity(LEN_VECTORS);
        for j in 0..(idx % LEN_VECTORS) {
            vec.push(idx as i64 - i as i64 + j as i64);
        }
        vectors.push(vec);
    }

    LargeOutput {
        name,
        numbers,
        vectors,
    }
}

fn large_output_to_hash(out: &LargeOutput) -> u64 {
    let mut h = out.name.len() as u64;
    for &num in &out.numbers {
        h = h
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(num as u64);
    }
    for vec in &out.vectors {
        h ^= vec.len() as u64;
        for &v in vec {
            h = h.rotate_left(5) ^ (v as u64);
        }
    }
    h
}

// ---------------------------------------------------------------------------
// Compute functions per item
// ---------------------------------------------------------------------------

#[inline(always)]
fn compute_light(x: u64) -> u64 {
    let x = black_box(x);
    x.rotate_left(13) ^ 0x5555_AAAA_3333_CCCC ^ (x.wrapping_mul(0x9E37_79B9_7F4A_7C15))
}

#[inline(always)]
fn compute_medium(x: u64) -> u64 {
    cpu_mix(50, black_box(x ^ 0xDEAD_BEEF_CAFE_0123))
}

#[inline(always)]
fn compute_heavy(x: u64) -> u64 {
    cpu_mix(500, black_box(x ^ 0xFEED_FACE_CAFE_BABE))
}

#[inline(always)]
fn compute_variable(x: u64) -> u64 {
    if x % 16 == 0 {
        cpu_mix(500, black_box(x))
    } else {
        compute_light(x)
    }
}

#[inline(always)]
fn apply_compute(compute_type: ComputeType, val: u64) -> u64 {
    match compute_type {
        ComputeType::Light => compute_light(val),
        ComputeType::Medium => compute_medium(val),
        ComputeType::Heavy => compute_heavy(val),
        ComputeType::Variable => compute_variable(val),
        ComputeType::Alloc => {
            let out = compute_alloc(val);
            large_output_to_hash(&out)
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn get_num_threads() -> usize {
    std::env::var("ORX_NUM_THREADS")
        .or_else(|_| std::env::var("RAYON_NUM_THREADS"))
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
        })
}

// ---------------------------------------------------------------------------
// Experiment Implementation
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct InputData {
    data: Vec<u64>,
}

pub struct Exp;

impl Experiment for Exp {
    type InputFactors = InputVariant;
    type AlgFactors = Method;
    type Input = InputData;
    type Output = Agg;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        const SEED: u64 = 0x5426_1234_BEEF_CAFE;
        let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ (input_variant.n as u64));
        let data: Vec<u64> = (0..input_variant.n)
            .map(|_| rng.random_range(1..=1_000_000))
            .collect();
        InputData { data }
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let compute = input_variant.compute;
        let pipeline = input_variant.pipeline;
        let data = &input.data;

        match alg_variant {
            Method::Seq => execute_seq(data, pipeline, compute),
            Method::Rayon => execute_rayon(data, pipeline, compute),
            Method::ConIterSingle => execute_con_iter_single(data, pipeline, compute),
            Method::ConIterChunk(chunk_size) => {
                execute_con_iter_chunk(data, pipeline, compute, *chunk_size)
            }
        }
    }

    fn expected_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        Some(execute_seq(
            &input.data,
            input_variant.pipeline,
            input_variant.compute,
        ))
    }
}

// ---------------------------------------------------------------------------
// Execution Strategies
// ---------------------------------------------------------------------------

fn execute_seq(data: &[u64], pipeline: PipelineType, compute: ComputeType) -> Agg {
    match pipeline {
        PipelineType::FilterMap => data
            .iter()
            .copied()
            .filter(|&x| x % 2 == 0)
            .map(|x| apply_compute(compute, x))
            .map(Agg::from_val)
            .fold(Agg::default(), merge),
        PipelineType::Map => data
            .iter()
            .copied()
            .map(|x| apply_compute(compute, x))
            .map(Agg::from_val)
            .fold(Agg::default(), merge),
    }
}

fn execute_rayon(data: &[u64], pipeline: PipelineType, compute: ComputeType) -> Agg {
    use rayon::prelude::*;
    match pipeline {
        PipelineType::FilterMap => data
            .iter()
            .copied()
            .filter(|&x| x % 2 == 0)
            .par_bridge()
            .map(|x| apply_compute(compute, x))
            .map(Agg::from_val)
            .reduce(Agg::default, merge),
        PipelineType::Map => data
            .iter()
            .copied()
            .par_bridge()
            .map(|x| apply_compute(compute, x))
            .map(Agg::from_val)
            .reduce(Agg::default, merge),
    }
}

fn execute_con_iter_single(data: &[u64], pipeline: PipelineType, compute: ComputeType) -> Agg {
    let num_threads = get_num_threads();
    match pipeline {
        PipelineType::FilterMap => {
            let iter = data.iter().copied().filter(|&x| x % 2 == 0);
            let con_iter = iter.iter_into_con_iter();
            run_threads(num_threads, || {
                let mut local = Agg::default();
                while let Some(x) = con_iter.next() {
                    let v = apply_compute(compute, x);
                    local = merge(local, Agg::from_val(v));
                }
                local
            })
        }
        PipelineType::Map => {
            let iter = data.iter().copied();
            let con_iter = iter.iter_into_con_iter();
            run_threads(num_threads, || {
                let mut local = Agg::default();
                while let Some(x) = con_iter.next() {
                    let v = apply_compute(compute, x);
                    local = merge(local, Agg::from_val(v));
                }
                local
            })
        }
    }
}

fn execute_con_iter_chunk(
    data: &[u64],
    pipeline: PipelineType,
    compute: ComputeType,
    chunk_size: usize,
) -> Agg {
    let num_threads = get_num_threads();
    match pipeline {
        PipelineType::FilterMap => {
            let iter = data.iter().copied().filter(|&x| x % 2 == 0);
            let con_iter = iter.iter_into_con_iter();
            run_threads(num_threads, || {
                let mut local = Agg::default();
                let mut puller = con_iter.chunk_puller(chunk_size);
                while let Some(chunk) = puller.pull() {
                    for x in chunk {
                        let v = apply_compute(compute, x);
                        local = merge(local, Agg::from_val(v));
                    }
                }
                local
            })
        }
        PipelineType::Map => {
            let iter = data.iter().copied();
            let con_iter = iter.iter_into_con_iter();
            run_threads(num_threads, || {
                let mut local = Agg::default();
                let mut puller = con_iter.chunk_puller(chunk_size);
                while let Some(chunk) = puller.pull() {
                    for x in chunk {
                        let v = apply_compute(compute, x);
                        local = merge(local, Agg::from_val(v));
                    }
                }
                local
            })
        }
    }
}

fn run_threads<F>(num_threads: usize, f: F) -> Agg
where
    F: Fn() -> Agg + Send + Sync,
{
    std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(num_threads);
        for _ in 0..num_threads {
            handles.push(s.spawn(&f));
        }
        let mut total = Agg::default();
        for h in handles {
            let local = h.join().expect("thread join failed");
            total = merge(total, local);
        }
        total
    })
}
