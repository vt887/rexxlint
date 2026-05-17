//! Recursive Rexx-file discovery for rexxlint.
//!
//! Wraps `ignore::WalkBuilder` with extension filtering (`.rexx`, `.rex`, `.rx`)
//! and config-driven exclude patterns. Output is always sorted lexicographically
//! for deterministic downstream processing.

use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use thiserror::Error;

const REXX_EXTENSIONS: &[&str] = &["rexx", "rex", "rx"];

#[derive(Debug, Error)]
pub enum WalkError {
    #[error("invalid exclude pattern '{pattern}': {source}")]
    BadPattern {
        pattern: String,
        #[source]
        source: globset::Error,
    },
    #[error("failed to walk '{path}': {source}")]
    Walk {
        path: PathBuf,
        #[source]
        source: ignore::Error,
    },
}

#[derive(Debug, Clone)]
pub struct WalkOptions {
    pub excludes: Vec<String>,
    pub respect_gitignore: bool,
    pub follow_links: bool,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            excludes: Vec::new(),
            respect_gitignore: true,
            follow_links: false,
        }
    }
}

/// Discover Rexx files under `inputs`.
///
/// Each input may be a file (passed through if its extension matches) or a
/// directory (recursively traversed). Output is deduplicated and sorted.
pub fn discover(inputs: &[PathBuf], opts: &WalkOptions) -> Result<Vec<PathBuf>, WalkError> {
    let excludes = build_globset(&opts.excludes)?;
    let mut out: Vec<PathBuf> = Vec::new();

    for input in inputs {
        if input.is_file() {
            if has_rexx_extension(input) && !excluded(input, input, &excludes) {
                out.push(input.clone());
            }
            continue;
        }

        let mut builder = WalkBuilder::new(input);
        builder
            .follow_links(opts.follow_links)
            .git_ignore(opts.respect_gitignore)
            .git_global(opts.respect_gitignore)
            .git_exclude(opts.respect_gitignore)
            .ignore(opts.respect_gitignore)
            .hidden(false)
            .standard_filters(opts.respect_gitignore);

        for entry in builder.build() {
            let entry = entry.map_err(|e| WalkError::Walk {
                path: input.clone(),
                source: e,
            })?;
            let path = entry.path();
            if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                continue;
            }
            if !has_rexx_extension(path) {
                continue;
            }
            if excluded(path, input, &excludes) {
                continue;
            }
            out.push(path.to_path_buf());
        }
    }

    out.sort();
    out.dedup();
    Ok(out)
}

fn build_globset(patterns: &[String]) -> Result<GlobSet, WalkError> {
    if patterns.is_empty() {
        return Ok(GlobSet::empty());
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern).map_err(|e| WalkError::BadPattern {
            pattern: pattern.clone(),
            source: e,
        })?;
        builder.add(glob);
    }
    builder.build().map_err(|e| WalkError::BadPattern {
        pattern: patterns.join(","),
        source: e,
    })
}

fn has_rexx_extension(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => REXX_EXTENSIONS
            .iter()
            .any(|target| ext.eq_ignore_ascii_case(target)),
        None => false,
    }
}

fn excluded(path: &Path, root: &Path, set: &GlobSet) -> bool {
    if set.is_empty() {
        return false;
    }
    // Match both the absolute path and the path relative to the walk root so
    // patterns like "vendor/**" work whether the user passed "." or an
    // absolute directory.
    if set.is_match(path) {
        return true;
    }
    if let Ok(rel) = path.strip_prefix(root) {
        return set.is_match(rel);
    }
    false
}
