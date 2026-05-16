use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use rexx_analyzer::lint;
use rexx_cli::{render_json, render_sarif, render_text};
use rexx_config::{default_profile, resolve_config};
use rexx_diagnostics::Diagnostic;
use rexx_formatter::format_rexx_with_profile;

#[derive(Parser, Debug)]
#[command(name = "rexxlint", version, about = "Rexx linter and formatter CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, value_enum, default_value = "text", help = "Output format")]
    output: Output,

    #[arg(long, help = "Path to configuration file")]
    config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Lint files or stdin
    Check {
        /// Files to check
        paths: Vec<PathBuf>,
        /// Read from stdin
        #[arg(long)]
        stdin: bool,
    },
    /// Format files or stdin
    Format {
        /// Files to format
        paths: Vec<PathBuf>,
        /// Read from stdin
        #[arg(long)]
        stdin: bool,
        /// Check if files are formatted without overwriting
        #[arg(long)]
        check: bool,
        /// Overwrite files in place
        #[arg(long)]
        inplace: bool,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum Output {
    Text,
    Json,
    Sarif,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(2);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Resolve config
    let config = if let Some(path) = &cli.config {
        rexx_config::load_config(path)?
    } else {
        resolve_config(&std::env::current_dir()?)
    };

    match cli.command {
        Commands::Check { paths, stdin } => {
            let mut _all_diagnostics: Vec<Diagnostic> = Vec::new();
            let mut has_errors = false;

            if stdin {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                let diagnostics = lint(&buffer);
                if !diagnostics.is_empty() {
                    has_errors = true;
                    println!("{}", render_output("stdin", &diagnostics, &cli.output)?);
                }
            } else {
                for path in paths {
                    let source = fs::read_to_string(&path)?;
                    let diagnostics = lint(&source);
                    if !diagnostics.is_empty() {
                        has_errors = true;
                        println!(
                            "{}",
                            render_output(&path.to_string_lossy(), &diagnostics, &cli.output)?
                        );
                    }
                }
            }

            if has_errors {
                process::exit(1);
            }
        }
        Commands::Format {
            paths,
            stdin,
            check,
            inplace,
        } => {
            let profile = config.formatting.unwrap_or_else(default_profile);
            let mut has_changes = false;

            if stdin {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                let formatted = format_rexx_with_profile(&buffer, profile.clone());
                if check {
                    if formatted != buffer {
                        has_changes = true;
                    }
                } else {
                    print!("{}", formatted);
                }
            } else {
                for path in paths {
                    let buffer = fs::read_to_string(&path)?;
                    let formatted = format_rexx_with_profile(&buffer, profile.clone());

                    if formatted != buffer {
                        has_changes = true;
                        if check {
                            // in check mode we just track that there's a difference
                        } else if inplace {
                            fs::write(&path, &formatted)?;
                        } else {
                            print!("{}", formatted);
                        }
                    }
                }
            }

            if check && has_changes {
                process::exit(1);
            }
        }
    }

    Ok(())
}

fn render_output(
    path: &str,
    diagnostics: &[rexx_diagnostics::Diagnostic],
    format: &Output,
) -> Result<String> {
    match format {
        Output::Text => Ok(render_text(path, diagnostics)),
        Output::Json => Ok(render_json(path, diagnostics)?),
        Output::Sarif => Ok(render_sarif(path, diagnostics)?),
    }
}
