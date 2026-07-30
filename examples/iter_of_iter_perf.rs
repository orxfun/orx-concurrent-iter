use orx_concurrent_iter::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::env;
use std::hint::black_box;
use std::sync::Mutex;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Workload — mirrors examples/reduce.rs in orx-parallel
// ---------------------------------------------------------------------------

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 0xBEEF_CAFE_1234_5678;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ len as u64);
    (0..len).map(|_| rng.random_range(0..150_u64)).collect()
}

fn fibonacci(n: u64) -> u64 {
    let mut a = 0u64;
    let mut b = 1u64;
    for _ in 0..n {
        let c = a.wrapping_add(b);
        a = b;
        b = c;
    }
    a
}

fn map_item(x: &u64) -> u64 {
    7 * x + 1000
}

fn reduce_pair(a: u64, b: u64) -> u64 {
    let f = fibonacci(a % 5);
    let g = a.wrapping_add(f);
    g.wrapping_add(b).wrapping_sub(f)
}

// ---------------------------------------------------------------------------
// Sequential baseline
// ---------------------------------------------------------------------------

fn run_seq(input: &[u64]) -> Option<u64> {
    black_box(input)
        .iter()
        .filter(|&&x| x % 2 == 0)
        .map(map_item)
        .reduce(reduce_pair)
}

// ---------------------------------------------------------------------------
// Parallel via ConIterOfIter — single-element pull
//
// The source iterator is Filter<Iter<u64>>, wrapped in ConIterOfIter.
// Every call to con_iter.next() must acquire a MutHandle spin-lock.
// With N threads all spinning on the same AtomicU8, contention grows with N.
// ---------------------------------------------------------------------------

fn run_con_iter_single(input: &[u64], num_threads: usize) -> Option<u64> {
    let con_iter = black_box(input)
        .iter()
        .filter(|&&x| x % 2 == 0)
        .map(map_item)
        .iter_into_con_iter();

    let partial_results: Mutex<Vec<u64>> = Mutex::new(Vec::with_capacity(num_threads));

    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(|| {
                let mut local: Option<u64> = None;
                while let Some(x) = con_iter.next() {
                    local = Some(match local {
                        None => x,
                        Some(acc) => reduce_pair(acc, x),
                    });
                }
                if let Some(v) = local {
                    partial_results.lock().unwrap().push(v);
                }
            });
        }
    });

    partial_results
        .into_inner()
        .unwrap()
        .into_iter()
        .reduce(reduce_pair)
}

// ---------------------------------------------------------------------------
// Parallel via ConIterOfIter — chunk pull
//
// Same spinlock bottleneck, but each lock acquisition covers a chunk of
// elements rather than a single one, reducing lock frequency.
// ---------------------------------------------------------------------------

fn run_con_iter_chunk(input: &[u64], num_threads: usize, chunk_size: usize) -> Option<u64> {
    let con_iter = black_box(input)
        .iter()
        .filter(|&&x| x % 2 == 0)
        .map(map_item)
        .iter_into_con_iter();

    let partial_results: Mutex<Vec<u64>> = Mutex::new(Vec::with_capacity(num_threads));

    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(|| {
                let mut puller = con_iter.chunk_puller(chunk_size);
                let mut local: Option<u64> = None;
                while let Some(chunk) = puller.pull() {
                    for x in chunk {
                        local = Some(match local {
                            None => x,
                            Some(acc) => reduce_pair(acc, x),
                        });
                    }
                }
                if let Some(v) = local {
                    partial_results.lock().unwrap().push(v);
                }
            });
        }
    });

    partial_results
        .into_inner()
        .unwrap()
        .into_iter()
        .reduce(reduce_pair)
}

// ---------------------------------------------------------------------------
// CLI argument parsing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Method {
    Seq,
    ConIterSingle,
    ConIterChunk,
}

impl Method {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "seq" => Some(Self::Seq),
            "con-iter-single" => Some(Self::ConIterSingle),
            "con-iter-chunk" => Some(Self::ConIterChunk),
            _ => None,
        }
    }
}

struct Args {
    method: Method,
    n: u32,
    num_threads: usize,
    chunk_size: usize,
    warmup: usize,
    runs: usize,
}

impl Args {
    fn parse() -> Self {
        let args: Vec<String> = env::args().collect();
        let get = |flag: &str, default: &str| -> String {
            args.windows(2)
                .find(|w| w[0] == flag)
                .map(|w| w[1].clone())
                .unwrap_or_else(|| default.to_string())
        };

        let method_str = get("--method", "con-iter-single");
        let method = Method::from_str(&method_str).unwrap_or_else(|| {
            eprintln!(
                "Unknown method '{}'. Valid: seq | con-iter-single | con-iter-chunk",
                method_str
            );
            std::process::exit(1);
        });

        Args {
            method,
            n: get("--n", "18").parse().expect("--n must be a u32"),
            num_threads: get("--num-threads", "4")
                .parse()
                .expect("--num-threads must be usize"),
            chunk_size: get("--chunk-size", "64")
                .parse()
                .expect("--chunk-size must be usize"),
            warmup: get("--warmup", "3")
                .parse()
                .expect("--warmup must be usize"),
            runs: get("--runs", "10").parse().expect("--runs must be usize"),
        }
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn run_once(input: &[u64], args: &Args) -> Option<u64> {
    match args.method {
        Method::Seq => run_seq(input),
        Method::ConIterSingle => run_con_iter_single(input, args.num_threads),
        Method::ConIterChunk => run_con_iter_chunk(input, args.num_threads, args.chunk_size),
    }
}

fn main() {
    let args = Args::parse();
    let len = 1usize << args.n;

    println!(
        "method={:?}  n=2^{}={}  num_threads={}  chunk_size={}  warmup={}  runs={}",
        args.method, args.n, len, args.num_threads, args.chunk_size, args.warmup, args.runs
    );
    println!("\nNote: con-iter-single and con-iter-chunk wrap Filter<Iter<u64>> in");
    println!("      ConIterOfIter, which serializes element generation via a spin-lock.");
    println!("      Performance is expected to degrade with more threads due to contention.\n");

    println!("Generating {} inputs...", len);
    let input = inputs(len);

    println!("Warming up ({} runs)...", args.warmup);
    for _ in 0..args.warmup {
        let _ = black_box(run_once(&input, &args));
    }

    println!("Running {} timed iterations...", args.runs);
    let mut durations: Vec<Duration> = Vec::with_capacity(args.runs);
    let mut result: Option<u64> = None;

    for _ in 0..args.runs {
        let t0 = Instant::now();
        result = black_box(run_once(&input, &args));
        durations.push(t0.elapsed());
    }

    let total_ns: u64 = durations.iter().map(|d| d.as_nanos() as u64).sum();
    let avg_ns = total_ns / args.runs as u64;
    let min_ns = durations.iter().map(|d| d.as_nanos() as u64).min().unwrap();
    let max_ns = durations.iter().map(|d| d.as_nanos() as u64).max().unwrap();

    println!("\n--- Results ---");
    println!("avg: {:.3} ms", avg_ns as f64 / 1_000_000.0);
    println!("min: {:.3} ms", min_ns as f64 / 1_000_000.0);
    println!("max: {:.3} ms", max_ns as f64 / 1_000_000.0);
    println!("output: {:?}", result);
}
