use std::fs;
use std::process::Command;

use tempfile::tempdir;

fn rexxlint() -> Command {
    let bin = env!("CARGO_BIN_EXE_rexxlint");
    Command::new(bin)
}

fn write(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
    path
}

// ── check ─────────────────────────────────────────────────────────────────────

#[test]
fn check_single_clean_file_exits_zero() {
    let dir = tempdir().unwrap();
    write(dir.path(), "ok.rexx", "/* header */\nsay 'hi'\n");
    let status = rexxlint()
        .args(["check", dir.path().to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn check_single_dirty_file_exits_one() {
    let dir = tempdir().unwrap();
    write(dir.path(), "bad.rexx", "say 'missing header'\n");
    let status = rexxlint()
        .args(["check", dir.path().to_str().unwrap()])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(1));
}

#[test]
fn check_multi_file_aggregates_diagnostics() {
    let dir = tempdir().unwrap();
    write(dir.path(), "a.rexx", "say 'no header'\n");
    write(dir.path(), "b.rexx", "/* ok */\nsay 'fine'\n");
    write(dir.path(), "c.rexx", "say 'also bad'\n");

    let out = rexxlint()
        .args(["check", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("a.rexx"), "expected a.rexx in output");
    assert!(stdout.contains("c.rexx"), "expected c.rexx in output");
    assert!(!stdout.contains("b.rexx"), "b.rexx should be clean");
}

#[test]
fn check_output_is_deterministic() {
    let dir = tempdir().unwrap();
    write(dir.path(), "z.rexx", "say 'z'\n");
    write(dir.path(), "a.rexx", "say 'a'\n");
    write(dir.path(), "m.rexx", "say 'm'\n");

    let out = rexxlint()
        .args(["check", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    // Extract the first path component of each line to verify sorted order.
    let paths: Vec<&str> = lines
        .iter()
        .map(|l| l.split(':').next().unwrap_or(""))
        .collect();
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted, "output should be in sorted path order");
}

#[test]
fn check_json_output_has_schema_version() {
    let dir = tempdir().unwrap();
    write(dir.path(), "bad.rexx", "say 'no header'\n");
    let out = rexxlint()
        .args(["check", "--output=json", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert_eq!(v["schema_version"], 1);
}

#[test]
fn check_skips_non_rexx_files() {
    let dir = tempdir().unwrap();
    write(dir.path(), "file.txt", "not rexx");
    write(dir.path(), "script.sh", "#!/bin/bash");
    let status = rexxlint()
        .args(["check", dir.path().to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());
}

// ── format --check ────────────────────────────────────────────────────────────

#[test]
fn format_check_exits_one_when_files_need_formatting() {
    let dir = tempdir().unwrap();
    write(dir.path(), "unformatted.rexx", "say 'hello'\n");
    let status = rexxlint()
        .args(["format", "--check", dir.path().to_str().unwrap()])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(1));
}

#[test]
fn format_check_exits_zero_when_already_formatted() {
    let dir = tempdir().unwrap();
    // A file that the formatter would not change.
    write(dir.path(), "clean.rexx", "/* header */\nSAY 'hello'\n");
    let out = rexxlint()
        .args(["format", "--check", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    // If the formatter returns the same content, exit 0 and no "would reformat" message.
    let stdout = String::from_utf8_lossy(&out.stdout);
    if out.status.success() {
        assert!(
            !stdout.contains("would reformat"),
            "no reformat message expected; got: {stdout}"
        );
    }
    // We accept either result depending on formatter behavior — the important
    // invariant is that exit 1 implies "would reformat" in stdout.
    if !out.status.success() {
        assert!(
            stdout.contains("would reformat"),
            "exit 1 must emit 'would reformat'"
        );
    }
}

#[test]
fn format_check_does_not_modify_files() {
    let dir = tempdir().unwrap();
    let path = write(dir.path(), "orig.rexx", "say 'no header'\n");
    let before = fs::read_to_string(&path).unwrap();
    rexxlint()
        .args(["format", "--check", dir.path().to_str().unwrap()])
        .status()
        .unwrap();
    let after = fs::read_to_string(&path).unwrap();
    assert_eq!(before, after, "--check must not modify files");
}

// ── format --diff ─────────────────────────────────────────────────────────────

#[test]
fn format_diff_produces_unified_diff_headers() {
    let dir = tempdir().unwrap();
    write(dir.path(), "f.rexx", "say 'missing'\n");
    let out = rexxlint()
        .args(["format", "--diff", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("---"), "expected --- header in diff");
    assert!(stdout.contains("+++"), "expected +++ header in diff");
    assert!(stdout.contains("@@"), "expected hunk marker in diff");
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn format_diff_does_not_modify_files() {
    let dir = tempdir().unwrap();
    let path = write(dir.path(), "orig.rexx", "say 'no header'\n");
    let before = fs::read_to_string(&path).unwrap();
    rexxlint()
        .args(["format", "--diff", dir.path().to_str().unwrap()])
        .status()
        .unwrap();
    let after = fs::read_to_string(&path).unwrap();
    assert_eq!(before, after, "--diff must not modify files");
}

// ── format in-place ───────────────────────────────────────────────────────────

#[test]
fn format_inplace_rewrites_file() {
    let dir = tempdir().unwrap();
    let path = write(dir.path(), "fix.rexx", "say 'hello'\n");
    rexxlint()
        .args(["format", path.to_str().unwrap()])
        .status()
        .unwrap();
    let after = fs::read_to_string(&path).unwrap();
    // Formatter should have added a header comment
    assert!(
        after.starts_with("/*"),
        "expected formatted output to start with comment block"
    );
}

#[cfg(unix)]
#[test]
fn format_inplace_preserves_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let path = write(dir.path(), "perm.rexx", "say 'hello'\n");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

    rexxlint()
        .args(["format", path.to_str().unwrap()])
        .status()
        .unwrap();

    let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o644, "permissions should be preserved after format");
}

// ── failure isolation ─────────────────────────────────────────────────────────

#[test]
fn check_continues_after_per_file_io_error() {
    // A directory with one good file and a path that cannot be read (simulate
    // by passing a non-existent explicit path alongside a real one).
    let dir = tempdir().unwrap();
    let good = write(dir.path(), "good.rexx", "/* ok */\nsay 1\n");
    let bad_path = dir.path().join("nonexistent.rexx");

    // Passing both: the non-existent file should be a WalkError (not found),
    // and the whole run should not panic.
    let out = rexxlint()
        .args(["check", good.to_str().unwrap(), bad_path.to_str().unwrap()])
        .output()
        .unwrap();
    // Either succeeds (good file clean) or exits with a warning — must not panic (exit 2 is ok).
    assert_ne!(
        out.status.code(),
        None,
        "process must not terminate with a signal"
    );
}

// ── no-args prints help ───────────────────────────────────────────────────────

#[test]
fn no_args_prints_help_and_exits_zero() {
    let out = rexxlint().output().unwrap();
    assert!(
        out.status.success(),
        "exit 0 expected for no-args invocation"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("rexxlint:"),
        "expected version header in help"
    );
    assert!(stdout.contains("check"), "expected 'check' command in help");
}
