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
