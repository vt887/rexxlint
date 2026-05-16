use std::fs;
use std::io;
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use rexx_analyzer::lint;
use rexx_cli::{render_json, render_sarif, render_text};
use rexx_formatter::format_rexx_with_profile_name;

fn get_version() -> io::Result<String> {
    let version = std::fs::read_to_string("VERSION")?.trim().to_string();
    Ok(version)
}

fn get_build_date() -> String {
    option_env!("REXXLINT_BUILD_DATE")
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // fallback to current date in ISO format
            use chrono::prelude::*;
            Local::now().format("%Y-%m-%d").to_string()
        })
}

fn get_bitness() -> usize {
    std::mem::size_of::<usize>() * 8
}

fn custom_help(version: &str, date: &str, bits: usize) -> String {
    format!(
        r#"rexxlint: {version} {date} ({bits} bit)

This is a Rexx linter and formatter for deterministic CLI and CI workflows.
It is distributed under the terms MIT OR Apache-2.0 and comes with NO WARRANTY.
See the LICENSE file for details.

To run rexxlint:
  rexxlint [switches] [program]

where switches are:
  --help, -h                 show this message
  --version, -V              display rexxlint version and exit
  --fix                      apply formatting changes in-place
  --format                   print formatted output to stdout
  --output=MODE              output mode: text|json|sarif (default: text)
  --profile=PROFILE          formatting profile (default: mainframe-compatible)

"program" is the Rexx file to lint or format

Examples:
  rexxlint program.rexx
  rexxlint --fix program.rexx
  rexxlint --format program.rexx
  rexxlint --output=json program.rexx
"#
    )
}

#[derive(Debug, Clone, ValueEnum)]
enum Output {
    Text,
    Json,
    Sarif,
}

#[derive(Parser, Debug)]
#[command(
    name = "rexxlint",
    version,
    about = "Rexx linter and formatter CLI",
    arg_required_else_help = true,
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    path: PathBuf,
    #[arg(long, help = "Apply formatting fixes to the file in place")]
    fix: bool,
    #[arg(long, help = "Print formatted output to stdout")]
    format: bool,
    #[arg(long, value_enum, default_value = "text", help = "Output format")]
    output: Output,
    #[arg(
        long,
        default_value = "mainframe-compatible",
        help = "Formatting profile"
    )]
    profile: String,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let version = get_version().unwrap_or_else(|_| "unknown".to_string());
    let date = get_build_date();
    let bits = get_bitness();
    if args.len() == 1 || args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", custom_help(&version, &date, bits));
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--version" || arg == "-V" || arg == "-v") {
        println!(
            "rexxlint: {} {} ({} bit)",
            version,
            date,
            bits
        );
        return Ok(());
    }

    let cli = Cli::parse();
    let mut source = fs::read_to_string(&cli.path)?;

    if cli.format || cli.fix {
        source = format_rexx_with_profile_name(&source, &cli.profile)?;
        if cli.fix {
            fs::write(&cli.path, &source)?;
        } else {
            print!("{source}");
            return Ok(());
        }
    }

    let diagnostics = lint(&source);
    let path = cli.path.to_string_lossy().to_string();
    let output = match cli.output {
        Output::Text => render_text(&path, &diagnostics),
        Output::Json => render_json(&diagnostics)?,
        Output::Sarif => render_sarif(&path, &diagnostics)?,
    };

    if !output.is_empty() {
        println!("{output}");
    }

    Ok(())
}
