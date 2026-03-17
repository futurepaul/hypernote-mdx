set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
  @just --list

[private]
_require-hyperfine:
  @command -v hyperfine >/dev/null 2>&1 || { \
    echo "hyperfine not found in PATH."; \
    echo "Install it with: cargo install hyperfine"; \
    exit 1; \
  }

[private]
_require-cargo-fuzz:
  @cargo fuzz --help >/dev/null 2>&1 || { \
    echo "cargo-fuzz not found."; \
    echo "Install it with: cargo install cargo-fuzz"; \
    exit 1; \
  }

# Run the full test suite.
test:
  cargo test

# Run one exact test by name with serialized execution.
test-one name:
  cargo test {{name}} -- --exact --test-threads=1

# Run one integration test target by file name, e.g. `just test-suite ast_snapshots`.
test-suite suite:
  cargo test --test {{suite}}

# Time one full `cargo test` run with the system `time` command.
time:
  /usr/bin/time -lp cargo test

# Time one exact test by name.
time-one name:
  /usr/bin/time -lp cargo test {{name}} -- --exact --test-threads=1

# Time one integration test target by file name.
time-suite suite:
  /usr/bin/time -lp cargo test --test {{suite}}

# Generate Cargo compile-time timings in target/cargo-timings/.
test-build-timings:
  cargo test --timings

# Benchmark the full test suite command with hyperfine.
bench runs="10" warmup="1": _require-hyperfine
  hyperfine --warmup {{warmup}} --runs {{runs}} 'cargo test'

# Benchmark one integration test target.
bench-suite suite runs="10" warmup="1": _require-hyperfine
  hyperfine --warmup {{warmup}} --runs {{runs}} 'cargo test --test {{suite}}'

# Benchmark one exact test with serialized execution for cleaner numbers.
bench-one name runs="10" warmup="1": _require-hyperfine
  hyperfine --warmup {{warmup}} --runs {{runs}} 'cargo test {{name}} -- --exact --test-threads=1'

# Run the baseline autoresearch measurement.
research-baseline runs="10" warmup="1" batch_count="10": _require-hyperfine
  uv run python research/run_experiment.py --description baseline --status keep --runs {{runs}} --warmup {{warmup}} --batch-count {{batch_count}}

# Run one autoresearch experiment from the current commit.
research-run description runs="10" warmup="1" batch_count="10": _require-hyperfine
  uv run python research/run_experiment.py --description '{{description}}' --status auto --runs {{runs}} --warmup {{warmup}} --batch-count {{batch_count}}

# Start a fresh autoresearch campaign by deleting prior results and logs.
research-reset:
  rm -rf research/results.tsv research/progress.png research/index.html research/logs research/tmp

# Rebuild the browser-facing progress report from results.tsv.
research-plot:
  uv run python research/plot_progress.py

# Serve the autoresearch report directory in a browser-friendly way.
research-serve port="8765":
  cd research && python3 -m http.server {{port}}

# Run the parser no-panic fuzz target against a temporary corpus copy.
fuzz seconds="10": _require-cargo-fuzz
  tmpdir=$(mktemp -d); \
  trap 'rm -rf "$tmpdir"' EXIT; \
  cp fuzz/corpus/parse_render_serialize/* "$tmpdir"/; \
  cargo fuzz run parse_render_serialize "$tmpdir" -- -dict=fuzz/dictionaries/mdx.dict -max_total_time={{seconds}}

# Run fuzzing against the checked-in corpus and let libFuzzer grow it.
fuzz-corpus seconds="10": _require-cargo-fuzz
  cargo fuzz run parse_render_serialize -- -dict=fuzz/dictionaries/mdx.dict -max_total_time={{seconds}}

# Build the fuzz target without starting a fuzzing session.
fuzz-build: _require-cargo-fuzz
  cargo fuzz build parse_render_serialize
