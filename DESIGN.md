# Design

This document explains the internal shape of `hypernote-mdx`: what the crate owns, how the AST is organized, and why the public semantic layer exists.

For consumer usage, start with [README.md](README.md). For testing and fuzzing workflows, see [TESTING.md](TESTING.md).

## Crate Boundary

`hypernote-mdx` owns:

- tokenization and parsing of Markdown + MDX source
- a compact AST representation
- parser-owned semantic extraction that downstream Rust code would otherwise duplicate
- canonical rendering
- semantic JSON serialization
- source-position helpers for diagnostics

`hypernote-mdx` does not own:

- app-specific component registries
- prop schemas for a particular application
- action semantics
- renderer-facing document models
- runtime evaluation of MDX expressions

That boundary is important. The crate should remove parser-adjacent glue from downstreams without drifting into application policy.

## Processing Layers

The crate is easiest to understand as four layers:

1. `tokenizer.rs`: source text to tokens and byte starts
2. `parser.rs`: tokens to `Ast`
3. `semantic.rs`: typed, parser-owned views over raw AST structure
4. `render.rs` and `tree_builder.rs`: canonical rendering and semantic JSON serialization

The semantic layer is the key recent addition. It lets downstream Rust code ask for things like “the semantic link,” “the semantic JSX element,” or “plain text with explicit expression policy” without re-decoding token layout or `extra_data`.

## AST Layout

The AST is intentionally flat and index-based.

- `Ast::token_tags` stores token kinds.
- `Ast::token_starts` stores byte offsets into the original source.
- `Ast::nodes` stores AST nodes.
- `Ast::extra_data` stores compact auxiliary data for nodes that need more than a single token or child range.

Nodes refer to tokens and auxiliary data by index rather than using pointer-heavy tree structures. This keeps the representation compact and cheap to traverse.

Common examples:

- headings store level plus child range
- links/images store child range plus URL token
- JSX elements store name token, attribute range, and child range
- frontmatter stores format plus content token range
- tables store row count and alignments in `extra_data`

## Why The Semantic Layer Exists

The raw AST is efficient, but it is not the best direct interface for downstream code. Before the semantic layer, consumers often had to rebuild logic like:

- token slicing and trimming
- JSX string unquoting and entity decoding
- attribute type coercion
- frontmatter content slicing
- expression source extraction
- child slicing for links, images, and JSX

That duplication is exactly the kind of parser-adjacent glue the crate should own. The semantic layer keeps the raw AST compact while exposing stable, typed helpers on `Ast`.

Examples of public semantic access:

- `code_block_info`
- `link_view`
- `image_view`
- `expression_info`
- `frontmatter_view`
- `jsx_attribute_views`
- `jsx_element_view`
- `plain_text*`
- `line_col`
- `node_position`

## Why JSON Still Exists

`serialize_tree()` is still useful, but it is not the preferred in-process Rust API.

The current intended layering is:

- Rust consumers should prefer typed semantic accessors on `Ast`.
- JSON should exist for process boundaries, language boundaries, snapshotting, and tools that specifically want a serialized semantic tree.

`serialize_tree()` now dogfoods the same semantic helper layer where practical so the crate is not maintaining parallel semantic interpretations.

## Rendering

`render()` turns the AST back into canonical MDX source. This is intentionally parser-owned behavior rather than a general-purpose formatter with many style knobs.

The goal is stable, correct emission for the syntax the parser understands, not configurable pretty-printing policy.

## Source Positions And Diagnostics

Parser errors are recorded in `ast.errors` with byte offsets. Downstream validators often need user-facing locations, so the crate also exposes:

- `line_col(byte_offset)` for a one-based `line:column` mapping
- `node_position(node_idx)` for the start of a node span

That keeps source mapping near the parser, where the original source text and offsets already live.

## Safety Model

Malformed source input should produce parse errors, not panics.

The public library flow is exercised around:

- malformed Markdown / MDX input
- malformed JSX attributes
- unclosed expressions and frontmatter
- render / `serialize_tree()` no-panic behavior
- invalid-index and malformed-AST accessor safety

Manual AST construction is not the primary workflow, but the accessor layer is intentionally defensive so obviously bad indices or malformed node/data combinations degrade to empty values or `None` rather than panicking.

## When To Add New Public Semantics

A good candidate for a new parser-owned semantic helper usually has all of these properties:

- downstreams are duplicating it
- it depends on parser-internal layout knowledge
- it is still parser-scoped rather than app-scoped
- it can be tested directly and through existing consumer-facing paths

Recent examples include `jsx_element_view` and source-position helpers: both remove low-level AST plumbing from downstream applications without forcing application policy into the parser crate.

## Cross-Links

- Consumer API overview: [README.md](README.md)
- Test and fuzz workflow: [TESTING.md](TESTING.md)
- Performance experiments and maintainers’ optimization loop: [AUTORESEARCH.md](AUTORESEARCH.md)
