# hypernote-mdx

A Rust parser for MDX (Markdown + JSX), built for the [hypernote](https://github.com/nicksimpsonx/hypernote-spec) format. Parses MDX source into a compact AST, serializes to JSON, and renders back to canonical source.

## Usage

```rust
// Parse MDX source into AST
let ast = hypernote_mdx::parse(source);

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
