#![no_main]

use hypernote_mdx::{ParseOptions, parse, parse_with_options, render, serialize_tree};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data);

    // Main parser path.
    let ast = parse(source.as_ref());
    let json = serialize_tree(&ast);
    let rendered = render(&ast);
    let _: serde_json::Value =
        serde_json::from_str(&json).expect("serialized tree should remain valid JSON");

    // Canonical render path should also stay parseable and serializable.
    let roundtrip_ast = parse(&rendered);
    let roundtrip_json = serialize_tree(&roundtrip_ast);
    let _roundtrip_rendered = render(&roundtrip_ast);
    let _: serde_json::Value = serde_json::from_str(&roundtrip_json)
        .expect("round-tripped serialized tree should remain valid JSON");

    // Exercise the optional normalization path too.
    let normalized_ast = parse_with_options(
        source.as_ref(),
        &ParseOptions {
            normalize_emoji_shortcodes: true,
        },
    );
    let normalized_json = serialize_tree(&normalized_ast);
    let normalized_rendered = render(&normalized_ast);
    let _: serde_json::Value = serde_json::from_str(&normalized_json)
        .expect("normalized serialized tree should remain valid JSON");

    let normalized_roundtrip_ast = parse(&normalized_rendered);
    let normalized_roundtrip_json = serialize_tree(&normalized_roundtrip_ast);
    let _: serde_json::Value = serde_json::from_str(&normalized_roundtrip_json)
        .expect("normalized round-tripped serialized tree should remain valid JSON");
});
