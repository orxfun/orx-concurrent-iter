use criterion::{Criterion, criterion_group, criterion_main};
use orx_concurrent_iter::*;
use orx_criterion::{Experiment, Factors};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

const NUM_NUMBERS: usize = 4;
const NUM_VECTORS: usize = 4;
const LEN_VECTORS: usize = 4;
const SEED: u64 = 5426;
const FIB_UPPER_BOUND: u32 = 999;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct LargeOutput {
    name: String,
    numbers: [i64; NUM_NUMBERS],
    vectors: Vec<Vec<i64>>,
}

fn to_large_output(idx: usize) -> LargeOutput {
    let prefix = match idx % 7 {
        0 => "zero-",
        3 => "three-",
        _ => "sth-",
    };
    let fib = fibonacci(&(idx as u32));
    let name = format!("{}-fib-{}", prefix, fib);

    let mut numbers = [0i64; NUM_NUMBERS];
    for (i, x) in numbers.iter_mut().enumerate() {
        *x = match (idx * 7 + i) % 3 {
            0 => idx as i64 + i as i64,
            _ => idx as i64 - i as i64,
        };
    }

    let mut vectors = vec![];
    for i in 0..NUM_VECTORS {
        let mut vec = vec![];
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

fn fibonacci(n: &u32) -> u32 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..*n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

fn validate(expected: &[LargeOutput], unsorted_result: Vec<LargeOutput>) {
    let mut sorted_result = unsorted_result;
    sorted_result.sort();
    assert_eq!(expected.len(), sorted_result.len());
    assert_eq!(expected, sorted_result);
}

fn inputs(len: usize) -> Vec<usize> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|_| rng.random_range(0..FIB_UPPER_BOUND) as usize)
        .collect()
}

fn seq(inputs: &[usize]) -> Vec<LargeOutput> {
    inputs
        .iter()
        .filter(|x| *x % 3 > 0)
        .map(|x| x + 1)
        .map(to_large_output)
        .collect()
}

fn rayon(inputs: &[usize]) -> Vec<LargeOutput> {
    use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
    inputs
        .iter()
        .filter(|x| *x % 3 > 0)
        .map(|x| x + 1)
        .par_bridge()
        .into_par_iter()
        .map(to_large_output)
        .collect()
}

fn con_iter(inputs: &[usize], num_threads: usize, chunk_size: usize) -> Vec<LargeOutput> {
    let iter = inputs.iter().filter(|x| *x % 3 > 0).map(|x| x + 1);
    let con_iter = iter.iter_into_con_iter();

    std::thread::scope(|s| {
        let mut handles = vec![];
        for _ in 0..num_threads {
            let thread_vec = match chunk_size {
                1 => s.spawn(|| {
                    let mut vec = vec![];
                    while let Some(x) = con_iter.next() {
                        vec.push(to_large_output(x));
                    }
                    vec
                }),
                _ => s.spawn(|| {
                    let mut vec = vec![];
                    let mut chunk_iter = con_iter.chunk_puller(chunk_size);
                    while let Some(chunk) = chunk_iter.pull() {
                        vec.extend(chunk.map(to_large_output));
                    }
                    vec
                }),
            };
            handles.push(thread_vec);
        }

        let mut vec = vec![];
        for x in handles {
            vec.extend(x.join().expect("failed to join the thread"));
        }
        vec
    })
}

#[derive(Clone)]
struct Input {
    len: usize,
}

impl Factors for Input {
    fn factor_names() -> Vec<&'static str> {
        vec!["len"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![self.len.to_string()]
    }

    fn factor_levels_short(&self) -> Vec<String> {
        vec![self.len.to_string()]
    }
}

#[derive(Clone, Copy, Debug)]
enum Method {
    Seq,
    Rayon,
    ConcurrentIter,
    ConcurrentIterChunk,
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            match self {
                Self::Seq => "seq",
                Self::Rayon => "rayon",
                Self::ConcurrentIter => "orx",
                Self::ConcurrentIterChunk => "orx-c64",
            }
            .to_string(),
        ]
    }

    fn factor_levels_short(&self) -> Vec<String> {
        self.factor_levels()
    }
}

struct Exp;

impl Experiment for Exp {
    type InputFactors = Input;
    type AlgFactors = Method;
    type Input = (Vec<usize>, Vec<LargeOutput>);
    type Output = Vec<LargeOutput>;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        let input = inputs(input_variant.len);
        let mut expected = seq(&input);
        expected.sort();
        (input, expected)
    }

    fn execute(
        &mut self,
        _input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let (values, _) = input;
        match alg_variant {
            Method::Seq => seq(values),
            Method::Rayon => rayon(values),
            Method::ConcurrentIter => con_iter(values, 8, 1),
            Method::ConcurrentIterChunk => con_iter(values, 8, 64),
        }
    }

    fn validate_output(
        &self,
        _input_variant: &Self::InputFactors,
        input: &Self::Input,
        output: &Self::Output,
    ) {
        let (_, expected) = input;
        validate(expected, output.clone());
    }
}

fn con_iter_of_iter(c: &mut Criterion) {
    let treatments = vec![Input { len: 4096 }, Input { len: 65_536 }];
    let variants = vec![
        Method::Seq,
        Method::Rayon,
        Method::ConcurrentIter,
        Method::ConcurrentIterChunk,
    ];

    Exp.bench(c, "con_iter_of_iter", &treatments, &variants);
}

criterion_group!(benches, con_iter_of_iter);
criterion_main!(benches);
