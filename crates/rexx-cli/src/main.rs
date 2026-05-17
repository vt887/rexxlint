use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use rayon::prelude::*;
use rexx_analyzer::lint;
use rexx_cli::{FileOutcome, render_json_multi, render_sarif_multi, render_text_multi};
use rexx_config::resolve_config;
use rexx_formatter::{apply_fixes, format_rexx_with_profile_name};
use rexx_walker::{WalkOptions, discover};
use similar::TextDiff;

// ── Version helpers ───────────────────────────────────────────────────────────

fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn get_build_date() -> &'static str {
    option_env!("REXXLINT_BUILD_DATE").unwrap_or("unknown")
}

fn get_bitness() -> usize {
    std::mem::size_of::<usize>() * 8
}

fn get_license() -> &'static str {
    env!("CARGO_PKG_LICENSE")
}

fn version_line(version: &str, date: &str, bits: usize) -> String {
    format!("rexxlint: {version} {date} ({bits} bit)")
}

fn help_text(version: &str, date: &str, bits: usize) -> String {
    let license = get_license();
    format!(
        r#"{ver}

This is a Rexx linter and formatter for deterministic CLI and CI workflows.
It is distributed under the terms of the {license} license and comes with NO WARRANTY.
See the LICENSE file for details.

To run rexxlint:
  rexxlint <command> [switches] [paths...]

Commands:
  check   Lint one or more Rexx files or directories
  format  Format one or more Rexx files or directories

Common switches:
  --help, -h                 show this message
  --version, -V              display rexxlint version and exit
  --jobs=N                   number of parallel workers (default: all CPUs)
  --no-ignore                disable .gitignore / .ignore filtering

Check switches:
  --output=MODE              output mode: text|json|sarif (default: text)

Format switches:
  --profile=PROFILE          formatting profile: standard|mainframe-compatible|minimal
  --check                    exit 1 if any file would be reformatted (no writes)
  --diff                     show unified diff for each file that would change

Examples:
  rexxlint check .
  rexxlint check src/ lib/
  rexxlint check --output=json src/
  rexxlint format .
  rexxlint format --check .
  rexxlint format --diff src/
  rexxlint format --profile=standard src/main.rexx
"#,
        ver = version_line(version, date, bits),
    )
}

// ── CLI definition ────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "rexxlint",
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true
)]
struct Cli {
    #[arg(short = 'h', long = "help", action = clap::ArgAction::SetTrue, global = true)]
    help: bool,

    #[arg(short = 'V', long = "version", action = clap::ArgAction::SetTrue, global = true)]
    version: bool,

    #[arg(long, global = true, value_name = "N")]
    jobs: Option<usize>,

    #[arg(long, global = true)]
    no_ignore: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Lint one or more Rexx files or directories
    Check {
        paths: Vec<PathBuf>,
        #[arg(long, value_enum, default_value = "text", value_name = "MODE")]
        output: OutputMode,
        /// Apply safe auto-fixes and report remaining diagnostics
        #[arg(long)]
        fix: bool,
        /// Apply safe auto-fixes silently (no diagnostic output)
        #[arg(long, conflicts_with = "fix")]
        fix_only: bool,
    },
    /// Format one or more Rexx files or directories
    Format {
        paths: Vec<PathBuf>,
        #[arg(long)]
        check: bool,
        #[arg(long)]
        diff: bool,
        /// Also apply safe lint auto-fixes after formatting
        #[arg(long)]
        fix: bool,
        #[arg(long, default_value = "mainframe-compatible", value_name = "PROFILE")]
        profile: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputMode {
    Text,
    Json,
    Sarif,
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e:#}");
        process::exit(2);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let version = get_version();
    let date = get_build_date();
    let bits = get_bitness();

    if cli.version {
        println!("{}", version_line(version, date, bits));
        return Ok(());
    }
    if cli.help || cli.command.is_none() {
        print!("{}", help_text(version, date, bits));
        return Ok(());
    }

    // Build rayon thread pool
    let jobs = cli.jobs.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    });
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build()
        .context("failed to build thread pool")?;

    let respect_gitignore = !cli.no_ignore;

    match cli.command.unwrap() {
        Command::Check {
            paths,
            output,
            fix,
            fix_only,
        } => run_check(&paths, &output, fix, fix_only, respect_gitignore, &pool),
        Command::Format {
            paths,
            check,
            diff,
            fix,
            profile,
        } => run_format(&paths, check, diff, fix, &profile, respect_gitignore, &pool),
    }
}

// ── check ─────────────────────────────────────────────────────────────────────

fn run_check(
    paths: &[PathBuf],
    output: &OutputMode,
    apply_fix: bool,
    fix_only: bool,
    respect_gitignore: bool,
    pool: &rayon::ThreadPool,
) -> Result<()> {
    let config = resolve_config(&std::env::current_dir()?);
    let excludes = config.files.map(|f| f.exclude).unwrap_or_default();

    let opts = WalkOptions {
        excludes,
        respect_gitignore,
        ..Default::default()
    };
    let files = discover(paths, &opts)?;

    let mut outcomes: Vec<(PathBuf, FileOutcome)> = pool.install(|| {
        files
            .par_iter()
            .map(|path| {
                let outcome = lint_file(path);
                (path.clone(), outcome)
            })
            .collect()
    });

    outcomes.sort_by(|a, b| a.0.cmp(&b.0));

    let file_outcomes: Vec<FileOutcome> = outcomes.into_iter().map(|(_, o)| o).collect();

    // Apply fixes when requested (--fix or --fix-only).
    if apply_fix || fix_only {
        for outcome in &file_outcomes {
            let has_fixes = outcome.diagnostics.iter().any(|d| d.fix.is_some());
            if !has_fixes {
                continue;
            }
            let source = match read_utf8_lossy(&outcome.path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!(
                        "warning: cannot apply fixes to {}: {e}",
                        outcome.path.display()
                    );
                    continue;
                }
            };
            let patched = apply_fixes(&source, &outcome.diagnostics);
            if patched != source {
                if let Err(e) = write_atomic(&outcome.path, &patched) {
                    eprintln!(
                        "error: could not write fixes to {}: {e}",
                        outcome.path.display()
                    );
                } else if !fix_only {
                    eprintln!("fixed: {}", outcome.path.display());
                }
            }
        }
    }

    if fix_only {
        return Ok(());
    }

    let has_any = file_outcomes.iter().any(|o| !o.diagnostics.is_empty());
    if has_any {
        let rendered = match output {
            OutputMode::Text => render_text_multi(&file_outcomes),
            OutputMode::Json => render_json_multi(&file_outcomes)?,
            OutputMode::Sarif => render_sarif_multi(&file_outcomes)?,
        };
        println!("{rendered}");
    }

    if has_any {
        process::exit(1);
    }
    Ok(())
}

fn lint_file(path: &Path) -> FileOutcome {
    let source = match read_utf8_lossy(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("warning: skipping {}: {e}", path.display());
            return FileOutcome {
                path: path.to_path_buf(),
                diagnostics: vec![],
            };
        }
    };
    FileOutcome {
        path: path.to_path_buf(),
        diagnostics: lint(&source),
    }
}

// ── format ────────────────────────────────────────────────────────────────────

struct FormatOutcome {
    path: PathBuf,
    /// None = no change needed
    diff_text: Option<String>,
    would_change: bool,
}

fn run_format(
    paths: &[PathBuf],
    check: bool,
    show_diff: bool,
    apply_lint_fixes: bool,
    profile: &str,
    respect_gitignore: bool,
    pool: &rayon::ThreadPool,
) -> Result<()> {
    let config = resolve_config(&std::env::current_dir()?);
    let excludes = config.files.map(|f| f.exclude).unwrap_or_default();

    let opts = WalkOptions {
        excludes,
        respect_gitignore,
        ..Default::default()
    };
    let files = discover(paths, &opts)?;

    let mut outcomes: Vec<(PathBuf, FormatOutcome)> = pool.install(|| {
        files
            .par_iter()
            .map(|path| {
                let outcome = format_file(path, check || show_diff, show_diff, profile);
                (path.clone(), outcome)
            })
            .collect()
    });

    outcomes.sort_by(|a, b| a.0.cmp(&b.0));

    let mut any_change = false;
    for (_, outcome) in outcomes {
        if outcome.would_change {
            any_change = true;
            if check {
                println!("would reformat: {}", outcome.path.display());
            }
            if let Some(diff) = outcome.diff_text {
                print!("{diff}");
            }
        }

        // After formatting (which already wrote the file), apply lint fixes if requested.
        if apply_lint_fixes && !check && !show_diff {
            let source = match read_utf8_lossy(&outcome.path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let diagnostics = rexx_analyzer::lint(&source);
            let patched = apply_fixes(&source, &diagnostics);
            if patched != source {
                match write_atomic(&outcome.path, &patched) {
                    Ok(()) => {}
                    Err(e) => eprintln!(
                        "error: could not write fixes to {}: {e}",
                        outcome.path.display()
                    ),
                }
            }
        }
    }

    if any_change && (check || show_diff) {
        process::exit(1);
    }
    Ok(())
}

fn format_file(path: &Path, dry_run: bool, show_diff: bool, profile: &str) -> FormatOutcome {
    let source = match read_utf8_lossy(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("warning: skipping {}: {e}", path.display());
            return FormatOutcome {
                path: path.to_path_buf(),
                diff_text: None,
                would_change: false,
            };
        }
    };

    let formatted = match format_rexx_with_profile_name(&source, profile) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: formatting failed for {}: {e}", path.display());
            return FormatOutcome {
                path: path.to_path_buf(),
                diff_text: None,
                would_change: false,
            };
        }
    };

    if formatted == source {
        return FormatOutcome {
            path: path.to_path_buf(),
            diff_text: None,
            would_change: false,
        };
    }

    let diff_text = if show_diff {
        let label = path.to_string_lossy();
        let diff = TextDiff::from_lines(&source, &formatted);
        let hunks = diff.unified_diff().header(&label, &label).to_string();
        Some(hunks)
    } else {
        None
    };

    if !dry_run && let Err(e) = write_atomic(path, &formatted) {
        eprintln!("error: could not write {}: {e}", path.display());
        return FormatOutcome {
            path: path.to_path_buf(),
            diff_text: None,
            would_change: false,
        };
    }

    FormatOutcome {
        path: path.to_path_buf(),
        diff_text,
        would_change: true,
    }
}

/// Write `content` to `path` atomically via a temp file in the same directory.
/// Preserves file permissions on Unix.
fn write_atomic(path: &Path, content: &str) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));

    // Capture original permissions before creating the temp file
    #[cfg(unix)]
    let orig_mode = {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|m| m.permissions().mode())
            .unwrap_or(0o644)
    };

    let mut tmp =
        tempfile::NamedTempFile::new_in(parent).context("failed to create temporary file")?;

    use std::io::Write as _;
    tmp.write_all(content.as_bytes())
        .context("failed to write to temporary file")?;
    tmp.flush().context("failed to flush temporary file")?;

    // Restore permissions before persist
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(orig_mode);
        fs::set_permissions(tmp.path(), perms).context("failed to set file permissions")?;
    }

    tmp.persist(path)
        .context("failed to replace original file")?;
    Ok(())
}

// ── I/O helper ────────────────────────────────────────────────────────────────

fn read_utf8_lossy(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    match String::from_utf8(bytes) {
        Ok(s) => Ok(s),
        Err(e) => {
            // Invalid UTF-8: fall back to lossy conversion, treating it as a
            // per-file warning rather than aborting the whole run.
            eprintln!(
                "warning: {} contains invalid UTF-8 — processing with replacement characters",
                path.display()
            );
            Ok(String::from_utf8_lossy(e.as_bytes()).into_owned())
        }
    }
}
