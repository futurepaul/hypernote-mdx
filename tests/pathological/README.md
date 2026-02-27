# Pathological parser fixtures

These fixtures were captured from a real `pika` crash where malformed hypernote content containing unescaped `<<` could trigger non-terminating parse/tokenization behavior.

Run any fixture with:

```bash
cargo run --bin mdx-parse tests/pathological/<fixture>.hnmd
```

Fixtures:
- `01_caption_unclosed_heredoc_like.hnmd`
- `02_minimal_double_angle.hnmd`
- `03_card_caption_shell_heredoc.hnmd`
- `04_double_angle_after_text.hnmd`
