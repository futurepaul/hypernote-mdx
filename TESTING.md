# Testing

This document is for maintainers working on parser behavior, safety, and regressions. If you are looking for the consumer-facing API overview, start with [README.md](README.md).

## Goals

The test surface is meant to cover:

- parsing correctness for Markdown, MDX, and Hypernote-specific constructs
- semantic accessor behavior
- round-trip rendering and JSON serialization
- malformed-input and malformed-AST no-panic behavior
- regression fixtures for pathological parser cases
- fuzz coverage for parser, renderer, and serializer entry points

## Day-To-Day Commands

Use `cargo test` directly or the shortcuts in [`justfile`](justfile):

```sh
cargo test
just test

just test-one markdown_ast_snapshot_is_stable
just test-suite semantic_accessors
```

Useful focused commands:

```sh
cargo test --test semantic_accessors
cargo test --test malformed_input
cargo test --test ast_safety
cargo test --test pathological_fixtures
```

## Debug Binaries

The repo includes two small binaries for manual inspection:

```sh
cargo run --bin mdx-parse -- path/to/file.hnmd
cargo run --bin mdx-view -- path/to/file.hnmd
```

- `mdx-parse` prints AST-oriented debug output.
- `mdx-view` renders a terminal-oriented view of the parsed document.

These are useful when a fixture is failing and you want a quick manual read on what the parser produced.

## Pathological Fixtures

Pathological fixtures live under [`tests/pathological/`](tests/pathological/). They exist for parser behaviors that have historically been crashy, non-terminating, or otherwise fragile.

The fixture-specific notes live in [`tests/pathological/README.md`](tests/pathological/README.md).

When a production bug produces a minimal nasty input, prefer adding a fixture and a targeted regression test instead of relying only on a unit test string literal.

## Fuzzing

The fuzz harness lives under [`fuzz/`](fuzz/). The current target exercises:

- `parse`
- `serialize_tree`
- `render`
- parse/render round-trips
- `normalize_emoji_shortcodes`

Install the tool once:

```sh
cargo install cargo-fuzz
```

Then run one of:

```sh
just fuzz
just fuzz-corpus
just fuzz-build
```

Or invoke libFuzzer directly:

```sh
cargo fuzz run parse_render_serialize -- -dict=fuzz/dictionaries/mdx.dict -max_total_time=10
```

Important details:

- [`fuzz/dictionaries/mdx.dict`](fuzz/dictionaries/mdx.dict) gives libFuzzer MD/MDX-aware tokens.
- [`fuzz/corpus/parse_render_serialize/`](fuzz/corpus/parse_render_serialize/) contains the checked-in starter corpus.
- `just fuzz` runs against a temporary corpus copy so the repo does not churn during exploratory fuzzing.
- `just fuzz-corpus` lets libFuzzer grow the checked-in corpus intentionally.

When fuzzing finds a failure, look under `fuzz/artifacts/parse_render_serialize/` and replay the reproducer with:

```sh
cargo fuzz run parse_render_serialize fuzz/artifacts/parse_render_serialize/<artifact-file>
```

## What Good Regression Coverage Looks Like

When adding parser features or fixing bugs, prefer coverage that includes:

- a focused test per feature or bug
- one mixed-document case that combines the new behavior with existing syntax
- malformed-input tests for obvious broken forms
- semantic accessor tests when new parser-owned structure is exposed publicly
- end-to-end tests when behavior matters through `render()` or `serialize_tree()`

The best additions usually prove both parser correctness and downstream usability.

## Cross-Links

- Consumer API overview: [README.md](README.md)
- Internal architecture and parser boundary: [DESIGN.md](DESIGN.md)
- Benchmarking and autoresearch workflow: [AUTORESEARCH.md](AUTORESEARCH.md)
