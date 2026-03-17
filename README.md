# hypernote-mdx

A Rust parser for MDX (Markdown + JSX), built for the [hypernote](https://github.com/futurepaul/hypernote-pages) format. A Rust port of [zig-mdx](https://github.com/futurepaul/zig-mdx). Parses MDX source into a compact AST, serializes to JSON, and renders back to canonical source.

## Usage

```rust
// Parse MDX source into AST
let ast = hypernote_mdx::parse(source);

// Optional parser behavior (for example, emoji shortcode normalization)
let ast = hypernote_mdx::parse_with_options(
  source,
  &hypernote_mdx::ParseOptions {
    normalize_emoji_shortcodes: true,
  },
);

// Serialize AST to JSON (for crossing FFI boundaries)
let json = hypernote_mdx::serialize_tree(&ast);

// Render AST back to canonical MDX source
let roundtripped = hypernote_mdx::render(&ast);
```

### CLI

```sh
cargo run -- example.hnmd
```

Prints the tokenized AST and tree structure for a given `.hnmd` file.

## Development

This repo includes a `justfile` for common test and timing workflows.

```sh
just test
just test-one markdown_ast_snapshot_is_stable
just test-suite ast_snapshots
just time
just time-one markdown_ast_snapshot_is_stable
just time-suite ast_snapshots
just test-build-timings
just bench
just bench-one markdown_ast_snapshot_is_stable
just bench-suite ast_snapshots
```

`just bench*` recipes use [`hyperfine`](https://github.com/sharkdp/hyperfine). Install it with:

```sh
cargo install hyperfine
```

## Autoresearch

This worktree also supports an `autoresearch`-style optimization loop for `src/` only.

```sh
just research-baseline
just research-run "reduce repeated token slicing"
just research-plot
just research-serve
```

Open `http://localhost:8765` after starting `just research-serve` to watch the progress chart and recent experiment table in your browser.

### Fuzzing

A `cargo-fuzz` harness lives under [`fuzz/`](/Users/futurepaul/dev/sec/hypernote-mdx/fuzz/Cargo.toml). The first target stresses the main library path on arbitrary input:

- `parse`
- `serialize_tree`
- `render`
- parse/render round-trips
- the `normalize_emoji_shortcodes` parser option

Install the tool once:

```sh
cargo install cargo-fuzz
```

Then run either:

```sh
just fuzz
```

or directly:

```sh
cargo fuzz run parse_render_serialize -- -max_total_time=10
```

Starter corpus seeds live in `fuzz/corpus/parse_render_serialize/`.
The fuzz dictionary lives in `fuzz/dictionaries/mdx.dict`, and `just fuzz` uses it by default so libFuzzer can mutate common Markdown/MDX syntax more intelligently.
`just fuzz` runs against a temporary corpus copy so it does not churn the repo; use `just fuzz-corpus` when you intentionally want libFuzzer to grow the checked-in corpus.

## What it parses

**Markdown:** headings, paragraphs, bold, italic, inline code, code blocks, links, images, blockquotes, ordered/unordered lists, horizontal rules, hard breaks.

**MDX extensions:** JSX elements (`<Card>...</Card>`), self-closing JSX (`<Button />`), JSX fragments (`<>...</>`), JSX attributes (literal and expression), inline expressions (`{name}`), flow expressions, ESM imports/exports.

**Frontmatter:** YAML (`---`) and JSON (`` ```hnmd ```) frontmatter blocks.

## AST design

The AST uses a flat, index-based representation inspired by Zig's data-oriented design. Nodes reference tokens and children by index into parallel arrays rather than using heap-allocated tree pointers. This keeps the structure compact and cache-friendly.

```rust
let ast = hypernote_mdx::parse("# Hello **world**");

// Walk children by index
let children = ast.children(node_idx);

// Get source text for any token
let text = ast.token_slice(token_idx);

// Find the deepest node at a byte offset
let node = ast.node_at_offset(42);
```

## License

MIT
