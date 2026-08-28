# ConIterOfIter Performance Benchmark

A benchmark suite designed to evaluate and optimize the performance of `ConIterOfIter` across various input characteristics, computation types, pipeline patterns, and thread counts.

## Matrix Dimensions

### 1. Methods (`features`)
- `seq`: Sequential baseline (`Iterator::map / filter / reduce`).
- `rayon`: Rayon's `par_bridge` baseline.
- `con-iter-single`: `ConIterOfIter::next()` (pull 1 element per lock acquisition).
- `con-iter-c16`: `ConIterOfIter::chunk_puller(16)` (pull 16 elements per lock acquisition).
- `con-iter-c64`: `ConIterOfIter::chunk_puller(64)` (pull 64 elements per lock acquisition).
- `con-iter-c256`: `ConIterOfIter::chunk_puller(256)` (pull 256 elements per lock acquisition).
- `con-iter-c1024`: `ConIterOfIter::chunk_puller(1024)` (pull 1024 elements per lock acquisition).

### 2. Input Characteristics (`InputVariant`)
- **Size (`n`)**: Sequence length (e.g. `16_384`, `65_536`, `262_144`, `1_048_576`).
- **Compute Type (`compute`)**:
  - `Light`: Fast bitwise/arithmetic operations (~few ns). Directly exposes lock contention and spinlock pull overhead.
  - `Medium`: Moderate CPU mixing (`cpu_mix(50)`).
  - `Heavy`: Compute-heavy workloads (`cpu_mix(500)`).
  - `Variable`: Skewed workload (10% heavy, 90% light) to evaluate dynamic load balancing.
  - `Alloc`: Allocates complex heap objects (`String` and nested `Vec`s) to test memory allocation under concurrency.
- **Pipeline Type (`pipeline`)**:
  - `FilterMap`: `filter(pred).map(fn)` — filters ~50% of items during sequential iterator evaluation under lock.
  - `Map`: `map(fn)` — direct transformation of every element.

## How to Run

### Using the Runner Script (Runs all features across 1, 2, 4, 8, 16 threads)
```bash
./run.sh
```

### Using `bench-runner`
```bash
cargo run --release --manifest-path ../../bench-runner/Cargo.toml -- \
    --path . \
    --path-result results/results.csv \
    --warmup-runs 5 \
    --actual-runs 20 \
    --threads 4 \
    --threads 8
```

### Running a Specific Method Manually
```bash
ORX_NUM_THREADS=8 cargo run --release --features con-iter-c64 -- \
    --warmup-runs 5 \
    --actual-runs 20 \
    --run-mode run
```
