use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Parse input on a background thread with a hard timeout.
/// Panics if parsing takes longer than the deadline.
fn parse_with_timeout(label: &str, source: &str, timeout: Duration) {
    let src = source.to_string();
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let ast = hypernote_mdx::parse(&src);
        let _ = tx.send(ast);
    });

    match rx.recv_timeout(timeout) {
        Ok(ast) => {
            // Parsing finished — just verify we got *something* back and errors are bounded.
            assert!(
                !ast.nodes.is_empty(),
                "{label}: parser returned an empty AST"
            );
            assert!(
                ast.errors.len() <= 4096,
                "{label}: error list blew up ({} errors)",
                ast.errors.len()
            );
        }
        Err(_) => {
            panic!("{label}: parser did not terminate within {timeout:?}");
        }
    }
}

const TIMEOUT: Duration = Duration::from_secs(2);

// ── Fixtures from tests/pathological/ ──────────────────────────────────

#[test]
fn fixture_01_caption_unclosed_heredoc_like() {
    let source = include_str!("pathological/01_caption_unclosed_heredoc_like.hnmd");
    parse_with_timeout("01_caption_unclosed_heredoc_like", source, TIMEOUT);
}

#[test]
fn fixture_02_minimal_double_angle() {
    let source = include_str!("pathological/02_minimal_double_angle.hnmd");
    parse_with_timeout("02_minimal_double_angle", source, TIMEOUT);
}

#[test]
fn fixture_03_card_caption_shell_heredoc() {
    let source = include_str!("pathological/03_card_caption_shell_heredoc.hnmd");
    parse_with_timeout("03_card_caption_shell_heredoc", source, TIMEOUT);
}

#[test]
fn fixture_04_double_angle_after_text() {
    let source = include_str!("pathological/04_double_angle_after_text.hnmd");
    parse_with_timeout("04_double_angle_after_text", source, TIMEOUT);
}

// ── Synthetic worst-cases ──────────────────────────────────────────────

#[test]
fn deeply_nested_unclosed_brackets() {
    // 500 opening [ with no close — should bail, not loop
    let source = "[".repeat(500);
    parse_with_timeout("deeply_nested_unclosed_brackets", &source, TIMEOUT);
}

#[test]
fn many_stray_angle_brackets() {
    let source = "< ".repeat(200);
    parse_with_timeout("many_stray_angle_brackets", &source, TIMEOUT);
}

#[test]
fn double_angle_variations() {
    let cases = [
        "<<",
        "<<EOF",
        "<< EOF",
        "a <<EOF\nb\nEOF",
        "<<<<<<",
        "text <<< more text",
    ];
    for (i, src) in cases.iter().enumerate() {
        parse_with_timeout(&format!("double_angle_{i}"), src, TIMEOUT);
    }
}

#[test]
fn unclosed_jsx_with_inner_content() {
    let source = "<Foo>\nsome text\nmore text\n";
    parse_with_timeout("unclosed_jsx_with_inner_content", source, TIMEOUT);
}

#[test]
fn mismatched_jsx_tags() {
    let source = "<Foo>\ntext\n</Bar>\n";
    parse_with_timeout("mismatched_jsx_tags", source, TIMEOUT);
}

#[test]
fn pipe_flood() {
    // Lots of pipes that don't form a valid table
    let source = "|||||||||||||||||||||\n";
    parse_with_timeout("pipe_flood", source, TIMEOUT);
}

#[test]
fn malformed_table_separator() {
    let source = "| A |\n| not-a-separator |\n| B |\n";
    parse_with_timeout("malformed_table_separator", source, TIMEOUT);
}

#[test]
fn unclosed_emphasis_flood() {
    let source = "* ".repeat(300);
    parse_with_timeout("unclosed_emphasis_flood", &source, TIMEOUT);
}

#[test]
fn unclosed_strong_flood() {
    let source = "** ".repeat(300);
    parse_with_timeout("unclosed_strong_flood", &source, TIMEOUT);
}

#[test]
fn mixed_unclosed_formatting() {
    let source = "**bold *italic `code [link\n".repeat(50);
    parse_with_timeout("mixed_unclosed_formatting", &source, TIMEOUT);
}

#[test]
fn expression_without_closing_brace() {
    let source = "text {unclosed expression here\n";
    parse_with_timeout("expression_without_closing_brace", source, TIMEOUT);
}

#[test]
fn huge_single_line() {
    // 100KB of text on one line
    let source = "a".repeat(100_000);
    parse_with_timeout("huge_single_line", &source, TIMEOUT);
}

#[test]
fn many_blank_lines() {
    let source = "\n".repeat(10_000);
    parse_with_timeout("many_blank_lines", &source, TIMEOUT);
}

#[test]
fn table_cell_with_unclosed_link() {
    let source = "| [unclosed |\n| --- |\n| val |\n";
    parse_with_timeout("table_cell_with_unclosed_link", source, TIMEOUT);
}
