# hypernote-mdx autoresearch

This worktree runs an `autoresearch`-style optimization loop for `hypernote-mdx`.

The goal is simple: lower the benchmarked wall-clock runtime of the full test suite.

## Setup

This loop runs in the dedicated git worktree on branch `autoresearch/master-baseline`.

Before the loop begins:

1. Confirm the branch is clean.
2. Read these files for context:
   - `README.md`
   - `program.md`
   - `justfile`
   - `src/**/*.rs`
3. Do not read `tests/**` for modification ideas unless you need behavioral context.
4. The first measurement must always be the baseline:
   - `just research-baseline`
5. Serve the browser view if the human wants live progress:
   - `just research-serve`

## Scope

What you CAN modify:
- Files under `src/` only.

What you MUST NOT modify:
- Anything under `tests/`
- `Cargo.toml`
- `Cargo.lock`
- `README.md`
- `justfile`
- `pyproject.toml`
- Anything under `research/`
- `program.md`

The ban on modifying tests is absolute. Do not edit, delete, weaken, skip, filter, rename, or regenerate tests.

## Metric

The official score is the benchmarked runtime of the whole test suite, excluding compile time.

The measurement harness works like this:

1. Run `cargo test --no-run --quiet` as a build gate.
2. Measure build time separately as a diagnostic.
3. Benchmark `cargo test --quiet` with `hyperfine`.
4. Use `hyperfine --prepare 'cargo test --no-run --quiet'` so compilation is outside the measured window.

Primary score:
- `suite_mean_seconds`

Secondary diagnostics:
- `suite_stddev_seconds`
- `build_seconds`

Lower is better.

## Simplicity criterion

Keep changes small, local, and understandable.

Prefer:
- Removing work
- Avoiding needless allocations
- Avoiding repeated scans
- Tightening hot loops
- Early exits
- Reusing computed values

Avoid:
- Large refactors
- New dependencies
- Feature work
- API churn
- Benchmark gaming

If an optimization adds complexity for a negligible gain, discard it.

## Logging

Results are stored in untracked `research/results.tsv`.

Columns:

1. `commit`
2. `suite_seconds`
3. `suite_stddev_seconds`
4. `build_seconds`
5. `status`
6. `description`

Statuses:
- `keep`
- `discard`
- `crash`

The browser view is generated at `research/index.html` and `research/progress.png`.

## Experiment loop

The first run is always the baseline on the current clean branch.

After that, loop forever:

1. Inspect the current kept state.
2. Make one small optimization in `src/`.
3. Commit the change with a short descriptive message.
4. Run:
   - `just research-run "short description"`
5. Read the harness output.
6. If status is `keep`, advance the branch and continue.
7. If status is `discard` or `crash`, immediately revert the experiment commit:
   - `git reset --hard HEAD~1`
8. Continue without asking the human whether to proceed.

Each experiment should contain exactly one main idea.

## Guardrails

- Do not change tests.
- Do not change the benchmark harness.
- Do not score compile time.
- Do not leave the branch dirty between experiments.
- Do not batch multiple ideas into one commit.

## Timeouts

Use the harness defaults unless the human instructs otherwise.

If the build gate or benchmark step times out, treat the experiment as a crash.
