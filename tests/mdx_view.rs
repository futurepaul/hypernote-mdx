use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_path(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    path.push(format!(
        "hypernote-mdx-{}-{}{}",
        std::process::id(),
        nanos,
        suffix
    ));
    path
}

fn run_mdx_view_on(source: &str) -> std::process::Output {
    let path = unique_temp_path(".hnmd");
    fs::write(&path, source).expect("should write temp input");

    let output = Command::new(env!("CARGO_BIN_EXE_mdx-view"))
        .arg(&path)
        .output()
        .expect("should run mdx-view");

    fs::remove_file(&path).expect("should clean up temp input");
    output
}

fn strip_ansi(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(next) = chars.next() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
                continue;
            }
        }

        output.push(ch);
    }

    output
}

#[test]
fn mdx_view_requires_a_path_argument() {
    let output = Command::new(env!("CARGO_BIN_EXE_mdx-view"))
        .output()
        .expect("should run mdx-view");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("Usage:"));
    assert!(stderr.contains("<file.md|file.hnmd>"));
}

#[test]
fn mdx_view_reports_missing_files_cleanly() {
    let path = unique_temp_path(".hnmd");
    let output = Command::new(env!("CARGO_BIN_EXE_mdx-view"))
        .arg(&path)
        .output()
        .expect("should run mdx-view");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("Error reading"));
    assert!(stderr.contains(path.to_string_lossy().as_ref()));
}

#[test]
fn mdx_view_renders_heading_and_checklist() {
    let output = run_mdx_view_on("# Title\n\n- [x] Done\n- [ ] Todo\n");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("\u{1b}["));
    assert_eq!(
        "# Title\n\n  * [x] Done\n  * [ ] Todo\n",
        strip_ansi(&stdout)
    );
}

#[test]
fn mdx_view_renders_tables_with_box_drawing_and_alignment() {
    let output = run_mdx_view_on("| Left | Right |\n| :--- | ---: |\n| a | 42 |\n");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let plain = strip_ansi(&stdout);

    assert!(plain.contains("┌"));
    assert!(plain.contains("┐"));
    assert!(plain.contains("└"));
    assert!(plain.contains("┘"));
    assert_eq!(
        "┌──────┬───────┐\n│ Left │ Right │\n├──────┼───────┤\n│ a    │    42 │\n└──────┴───────┘\n",
        plain
    );
}
