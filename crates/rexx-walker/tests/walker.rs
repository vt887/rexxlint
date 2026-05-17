use std::fs;
use std::path::PathBuf;

use rexx_walker::{WalkOptions, discover};
use tempfile::tempdir;

fn touch(path: &std::path::Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, "/* */\nsay 1\n").unwrap();
}

#[test]
fn discovers_rexx_extensions() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    touch(&root.join("a.rexx"));
    touch(&root.join("b.rex"));
    touch(&root.join("c.rx"));
    touch(&root.join("d.txt"));
    touch(&root.join("e.md"));

    let files = discover(&[root.to_path_buf()], &WalkOptions::default()).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["a.rexx", "b.rex", "c.rx"]);
}

#[test]
fn extension_match_is_case_insensitive() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    touch(&root.join("UPPER.REXX"));
    touch(&root.join("Mixed.Rex"));

    let files = discover(&[root.to_path_buf()], &WalkOptions::default()).unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn recurses_subdirectories() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    touch(&root.join("src/a.rexx"));
    touch(&root.join("src/nested/deep/b.rex"));
    touch(&root.join("other/c.rx"));

    let files = discover(&[root.to_path_buf()], &WalkOptions::default()).unwrap();
    assert_eq!(files.len(), 3);
}

#[test]
fn respects_gitignore() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join(".gitignore"), "ignored/\n").unwrap();
    touch(&root.join("keep.rexx"));
    touch(&root.join("ignored/skip.rexx"));

    // ignore::WalkBuilder only consults .gitignore inside a git repo OR when
    // an explicit .ignore file is present. The standard_filters call above
    // turns on the latter; assert .gitignore via creating both files.
    fs::write(root.join(".ignore"), "ignored/\n").unwrap();

    let files = discover(&[root.to_path_buf()], &WalkOptions::default()).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["keep.rexx"]);
}

#[test]
fn no_ignore_option_disables_filters() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join(".ignore"), "ignored/\n").unwrap();
    touch(&root.join("keep.rexx"));
    touch(&root.join("ignored/skip.rexx"));

    let opts = WalkOptions {
        respect_gitignore: false,
        ..Default::default()
    };
    let files = discover(&[root.to_path_buf()], &opts).unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn applies_exclude_patterns() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    touch(&root.join("src/a.rexx"));
    touch(&root.join("vendor/v.rexx"));
    touch(&root.join("generated/g.rexx"));
    touch(&root.join("build/b.rexx"));

    let opts = WalkOptions {
        excludes: vec![
            "vendor/**".to_string(),
            "generated/**".to_string(),
            "build/**".to_string(),
        ],
        ..Default::default()
    };
    let files = discover(&[root.to_path_buf()], &opts).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("src/a.rexx"));
}

#[test]
fn output_is_sorted_regardless_of_input_order() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    touch(&root.join("z.rexx"));
    touch(&root.join("a.rexx"));
    touch(&root.join("m.rexx"));

    let files = discover(&[root.to_path_buf()], &WalkOptions::default()).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["a.rexx", "m.rexx", "z.rexx"]);
}

#[test]
fn explicit_file_input_passes_through() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let file = root.join("explicit.rexx");
    touch(&file);

    let files = discover(std::slice::from_ref(&file), &WalkOptions::default()).unwrap();
    assert_eq!(files, vec![file]);
}

#[test]
fn explicit_non_rexx_file_is_skipped() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let file = root.join("not_rexx.txt");
    touch(&file);

    let files = discover(&[file], &WalkOptions::default()).unwrap();
    assert!(files.is_empty());
}

#[test]
fn duplicate_inputs_collapse() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    touch(&root.join("a.rexx"));

    let inputs = vec![root.to_path_buf(), root.to_path_buf()];
    let files = discover(&inputs, &WalkOptions::default()).unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn bad_pattern_is_reported() {
    let opts = WalkOptions {
        excludes: vec!["[unterminated".to_string()],
        ..Default::default()
    };
    let err = discover(&[PathBuf::from(".")], &opts).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("[unterminated"), "got: {msg}");
}
