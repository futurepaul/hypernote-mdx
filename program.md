# hypernote-mdx autoresearch

This worktree runs an `autoresearch`-style optimization loop for `hypernote-mdx`.

The goal is simple: lower the benchmarked wall-clock runtime of the full test suite.

## Setup

This loop runs in the dedicated git worktree on branch `autoresearch/master-baseline`.

Before the loop begins:

1. Confirm the branch is clean.
2. If starting a new campaign, clear old results first:
   - `just research-reset`
3. Read these files for context:
   - `README.md`
   - `program.md`
   - `justfile`
   - `src/**/*.rs`
4. Do not read `tests/**` for modification ideas unless you need behavioral context.
5. The first measurement must always be the baseline:
   - `just research-baseline`
6. Serve the browser view if the human wants live progress:
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
3. Use `hyperfine` to benchmark a batch runner that executes `cargo test --quiet -- --test-threads=1` multiple times per sample.
4. Use `hyperfine --prepare 'cargo test --no-run --quiet'` so compilation is outside the measured window.
5. Keep `--test-threads=1` in the benchmark command so the score is stable even if individual test targets have parallel temp-file races.
6. Normalize the batch timing back to `seconds per full suite run` for logging and charting.
7. Use enough batch executions that every scored experiment includes many timed suite runs, typically 10 to 100 total.

Primary score:
- `suite_mean_seconds`

Secondary diagnostics:
- `suite_stddev_seconds`
- `bench_runs`
- `build_seconds`

Lower is better.

## Measurement discipline

Do not make keep/discard decisions from tiny samples.

Rules:
- The official score must come from the harness, not from ad hoc commands.
- The official `hyperfine` measurement must benchmark batched suite runs, not single tiny suite samples.
- The total timed benchmark should include many full-suite executions, typically 10 to 100.
- Compile time is never part of the score.
- The official measured command is the serialized test-harness variant, not the default parallel-per-binary variant.

Per-target diagnosis is allowed, but it is diagnostic only.

Do not treat separate `cargo test --test ...` command timings as a trustworthy breakdown of suite runtime. Cargo invocation overhead can dominate those numbers. If you need a hotspot hint, prefer timing compiled test binaries directly and treat the result as advisory.

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
4. `bench_runs`
5. `build_seconds`
6. `status`
7. `description`

Statuses:
- `keep`
- `discard`
- `crash`

The browser view is generated at `research/index.html` and `research/progress.png`.

`bench_runs` means the total number of timed full-suite executions used for that experiment.

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
- Do not lower the benchmark budget to save time.
- Do not mix results from different measurement policies in one campaign.
- Do not leave the branch dirty between experiments.
- Do not batch multiple ideas into one commit.

## Timeouts

Use the harness defaults unless the human instructs otherwise.

If the build gate or benchmark step times out, treat the experiment as a crash.
