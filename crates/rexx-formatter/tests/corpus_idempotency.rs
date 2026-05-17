use rexx_formatter::format_rexx;
use std::path::PathBuf;

fn corpus_dir(subdir: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/corpus/formatter")
        .join(subdir)
}

fn read_rexx_files(dir: &PathBuf) -> Vec<(String, String)> {
    let mut files = Vec::new();
    if !dir.exists() {
        return files;
    }
    for entry in std::fs::read_dir(dir).expect("failed to read dir") {
        let entry = entry.expect("bad entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("rexx") {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("failed to read {}", path.display()));
            files.push((name, content));
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
}

/// format(format(x)) == format(x) for every corpus file.
#[test]
fn corpus_valid_idempotency() {
    let dir = corpus_dir("valid");
    let files = read_rexx_files(&dir);
    assert!(!files.is_empty(), "no valid corpus files found in {dir:?}");
    for (name, src) in &files {
        let once = format_rexx(src);
        let twice = format_rexx(&once);
        assert_eq!(
            once, twice,
            "format is NOT idempotent for {name}:\nfirst pass:\n{once}\nsecond pass:\n{twice}"
        );
    }
}

/// Formatter must not panic or corrupt files for malformed inputs.
#[test]
fn corpus_malformed_no_panic() {
    let dir = corpus_dir("malformed");
    let files = read_rexx_files(&dir);
    assert!(
        !files.is_empty(),
        "no malformed corpus files found in {dir:?}"
    );
    for (name, src) in &files {
        let result = std::panic::catch_unwind(|| format_rexx(src));
        assert!(
            result.is_ok(),
            "formatter panicked on malformed input {name}"
        );
        // Idempotency must also hold for malformed inputs.
        let once = format_rexx(src);
        let twice = format_rexx(&once);
        assert_eq!(once, twice, "format is NOT idempotent for malformed {name}");
    }
}

/// Legacy files must survive formatting without corruption.
#[test]
fn corpus_legacy_idempotency() {
    let dir = corpus_dir("legacy");
    let files = read_rexx_files(&dir);
    assert!(!files.is_empty(), "no legacy corpus files found in {dir:?}");
    for (name, src) in &files {
        let once = format_rexx(src);
        let twice = format_rexx(&once);
        assert_eq!(once, twice, "format is NOT idempotent for legacy {name}");
        // Strings must survive intact.
        for line in src.lines() {
            if line.contains("'") {
                let quoted: Vec<&str> = line.split('\'').collect();
                if quoted.len() >= 3 {
                    let literal = quoted[1];
                    assert!(
                        once.contains(literal),
                        "string literal {literal:?} lost after formatting {name}"
                    );
                }
            }
        }
    }
}
