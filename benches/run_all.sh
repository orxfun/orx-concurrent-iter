#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BENCH_DIR="$ROOT_DIR/benches"
CRITERION_DIR="$ROOT_DIR/target/criterion"
OUT_FILE="$BENCH_DIR/results.csv"

shopt -s nullglob
bench_files=("$BENCH_DIR"/*.rs)
shopt -u nullglob

if [[ ${#bench_files[@]} -eq 0 ]]; then
    echo "No benchmark files (*.rs) found in $BENCH_DIR" >&2
    exit 1
fi

IFS=$'\n' bench_files=($(printf '%s\n' "${bench_files[@]}" | sort))
unset IFS

tmp_file="$(mktemp)"
trap 'rm -f "$tmp_file"' EXIT

expected_header=""

for bench_file in "${bench_files[@]}"; do
    bench_name="$(basename "$bench_file" .rs)"

    echo "Running benchmark: $bench_name"
    cargo bench --bench "$bench_name"

    summary_csv="$CRITERION_DIR/$bench_name/summary_${bench_name}.csv"
    if [[ ! -f "$summary_csv" ]]; then
        echo "Missing summary csv: $summary_csv" >&2
        exit 1
    fi

    header="$(head -n 1 "$summary_csv")"

    if [[ -z "$expected_header" ]]; then
        expected_header="$header"
        printf 'bench,%s\n' "$header" > "$tmp_file"
    elif [[ "$header" != "$expected_header" ]]; then
        echo "Header mismatch in $summary_csv" >&2
        echo "Expected: $expected_header" >&2
        echo "Found:    $header" >&2
        exit 1
    fi

    tail -n +2 "$summary_csv" | sed "s/^/${bench_name},/" >> "$tmp_file"
done

mv "$tmp_file" "$OUT_FILE"
trap - EXIT

echo "Combined benchmark results written to $OUT_FILE"
