use std::io::Write as _;
use std::process::{Command, Stdio};

fn rexxlint() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rexxlint"))
}

fn run_with_stdin(args: &[&str], input: &[u8]) -> std::process::Output {
    let mut child = rexxlint()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn rexxlint");
    child
        .stdin
        .take()
        .expect("stdin handle")
        .write_all(input)
        .expect("write stdin");
    child.wait_with_output().expect("wait failed")
}

// ── format --stdin ────────────────────────────────────────────────────────────

#[test]
fn format_stdin_outputs_formatted_source() {
    let out = run_with_stdin(&["format", "--stdin"], b"say 'hi'\n");
    assert!(
        out.status.success(),
        "expected exit 0; got {:?}",
        out.status
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.starts_with("/* The first line"),
        "expected header comment; got:\n{stdout}"
    );
    assert!(stdout.contains("SAY 'hi'"), "got:\n{stdout}");
}

#[test]
fn format_stdin_already_formatted_exits_zero() {
    let src = "/* The first line of a REXX exec must always be a comment. */\nSAY 'hi'\n";
    let out = run_with_stdin(&["format", "--stdin"], src.as_bytes());
    assert!(
        out.status.success(),
        "exit 0 expected; got {:?}",
        out.status
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.as_ref(), src);
}

#[test]
fn format_stdin_check_exits_one_on_change() {
    let out = run_with_stdin(&["format", "--stdin", "--check"], b"say 'hi'\n");
    assert_eq!(out.status.code(), Some(1), "expected exit 1");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("would reformat"),
        "expected 'would reformat'; got:\n{stdout}"
    );
}

#[test]
fn format_stdin_check_exits_zero_when_clean() {
    let src = "/* The first line of a REXX exec must always be a comment. */\nSAY 'hi'\n";
    let out = run_with_stdin(&["format", "--stdin", "--check"], src.as_bytes());
    assert!(
        out.status.success(),
        "expected exit 0; got {:?}",
        out.status
    );
}

#[test]
fn format_stdin_diff_shows_diff_and_exits_one() {
    let out = run_with_stdin(&["format", "--stdin", "--diff"], b"say 'hi'\n");
    assert_eq!(out.status.code(), Some(1), "expected exit 1");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("@@"),
        "expected unified diff; got:\n{stdout}"
    );
}

#[test]
fn format_stdin_path_appears_in_diff_header() {
    let out = run_with_stdin(
        &["format", "--stdin", "--diff", "--path", "my_file.rexx"],
        b"say 'hi'\n",
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("my_file.rexx"),
        "expected virtual path in diff; got:\n{stdout}"
    );
}

// ── check --stdin ─────────────────────────────────────────────────────────────

#[test]
fn check_stdin_emits_json_diagnostics() {
    let out = run_with_stdin(&["check", "--stdin", "--output", "json"], b"say 'hi'\n");
    assert_eq!(out.status.code(), Some(1), "expected exit 1 (diagnostics)");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("expected valid JSON output");
    let file = json["files"][0]["file"].as_str().unwrap_or("");
    assert_eq!(file, "<stdin>", "default virtual path should be <stdin>");
}

#[test]
fn check_stdin_path_overrides_default_virtual_path() {
    let out = run_with_stdin(
        &["check", "--stdin", "--output", "json", "--path", "foo.rexx"],
        b"say 'hi'\n",
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let file = json["files"][0]["file"].as_str().unwrap_or("");
    assert_eq!(file, "foo.rexx");
}

#[test]
fn check_stdin_clean_exits_zero() {
    let src = b"/* The first line of a REXX exec must always be a comment. */\nSAY 'hi'\n";
    let out = run_with_stdin(&["check", "--stdin"], src);
    assert!(
        out.status.success(),
        "expected exit 0; got {:?}",
        out.status
    );
    assert!(out.stdout.is_empty(), "expected no output for clean file");
}

#[test]
fn check_stdin_text_output() {
    let out = run_with_stdin(&["check", "--stdin"], b"say 'hi'\n");
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("R001"),
        "expected R001 diagnostic; got:\n{stdout}"
    );
}

// ── error cases ───────────────────────────────────────────────────────────────

#[test]
fn stdin_with_paths_rejected() {
    let out = run_with_stdin(&["format", "--stdin", "some.rexx"], b"");
    assert_eq!(
        out.status.code(),
        Some(2),
        "expected exit 2 for invalid args; got {:?}",
        out.status
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("positional paths") || stderr.contains("Error"),
        "expected error message; got:\n{stderr}"
    );
}

#[test]
fn check_stdin_fix_rejected() {
    let out = run_with_stdin(&["check", "--stdin", "--fix"], b"say 'hi'\n");
    assert_eq!(
        out.status.code(),
        Some(2),
        "expected exit 2; got {:?}",
        out.status
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("fix") || stderr.contains("Error"),
        "expected error about --fix; got:\n{stderr}"
    );
}

// ── edge cases ────────────────────────────────────────────────────────────────

#[test]
fn empty_stdin_format_inserts_header() {
    let out = run_with_stdin(&["format", "--stdin"], b"");
    assert!(
        out.status.success(),
        "expected exit 0; got {:?}",
        out.status
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.starts_with("/*"),
        "expected header comment for empty input; got:\n{stdout}"
    );
}

#[test]
fn empty_stdin_check_json_has_r001() {
    let out = run_with_stdin(&["check", "--stdin", "--output", "json"], b"");
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("R001"), "expected R001; got:\n{stdout}");
}

#[test]
fn invalid_utf8_stdin_format_no_panic() {
    // 0xFF bytes are invalid UTF-8; the CLI must not panic.
    let input = b"\xFF\xFE/* hi */\nsay 'x'\n";
    let result = std::panic::catch_unwind(|| run_with_stdin(&["format", "--stdin"], input));
    assert!(result.is_ok(), "formatter panicked on invalid UTF-8 stdin");
    let out = result.unwrap();
    assert_ne!(
        out.status.code(),
        Some(2),
        "fatal error on invalid UTF-8; stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("invalid UTF-8") || out.status.success(),
        "expected UTF-8 warning; got stderr:\n{stderr}"
    );
}
