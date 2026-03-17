# Autoresearch

This document covers performance-oriented maintainer workflows: timing, benchmarking, and the local autoresearch loop.

If you are looking for consumer-facing API docs, start with [README.md](README.md). For normal correctness testing and fuzzing, see [TESTING.md](TESTING.md).

## Purpose

The autoresearch workflow exists to make parser performance work more disciplined:

- establish a baseline
- run controlled experiments from the current worktree
- keep or reject changes based on measured behavior
- visualize the progression over time

It is a maintainer tool, not part of the public library surface.

## Prerequisites

Some commands use external tools:

```sh
cargo install hyperfine
```

The autoresearch scripts themselves are run via `uv`, and the report viewer uses `python3 -m http.server`.

## Timing And Benchmark Commands

The repo’s [`justfile`](justfile) includes lightweight performance helpers:

```sh
just time
just time-one markdown_ast_snapshot_is_stable
just time-suite ast_snapshots

just test-build-timings

just bench
just bench-one markdown_ast_snapshot_is_stable
just bench-suite ast_snapshots
```

What they do:

- `time*` uses the system `time` command around `cargo test`
- `test-build-timings` asks Cargo for compile-time timing reports
- `bench*` uses `hyperfine` for repeated command benchmarking

These are useful for quick checks before or after a parser change even if you are not running the full autoresearch loop.

## Autoresearch Commands

The higher-level experiment loop is:

```sh
just research-baseline
just research-run "reduce repeated token slicing"
just research-plot
just research-serve
```

There is also a reset helper:

```sh
just research-reset
```

Typical flow:

1. Run `just research-baseline` to capture the current state.
2. Make one focused change.
3. Run `just research-run "<description>"`.
4. Repeat until you have a useful sequence of kept/rejected experiments.
5. Run `just research-plot`.
6. Run `just research-serve` and inspect the generated report in a browser.

## Scripts And Outputs

Scripts live under [`research/`](research/):

- [`research/run_experiment.py`](research/run_experiment.py)
- [`research/run_suite_batch.py`](research/run_suite_batch.py)
- [`research/plot_progress.py`](research/plot_progress.py)

Generated outputs typically include:

- `research/results.tsv`
- `research/progress.png`
- `research/index.html`
- `research/logs/`
- `research/tmp/`

Open `http://localhost:8765` after `just research-serve` if you keep the default port.

## Recommended Discipline

This workflow is most useful when you keep experiments narrow:

- change one parser concern at a time
- keep a short description that names the actual hypothesis
- avoid mixing behavior changes with performance work
- rerun correctness tests before trusting a “faster” result

For parser work, speed without semantic stability is not a win.

## Cross-Links

- Consumer docs: [README.md](README.md)
- Correctness tests, fuzzing, and debug binaries: [TESTING.md](TESTING.md)
- Internal architecture and parser boundary: [DESIGN.md](DESIGN.md)
