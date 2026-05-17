use std::path::PathBuf;

use rexx_diagnostics::{Diagnostic, Severity};
use serde::Serialize;

// ── Public data model ─────────────────────────────────────────────────────────

/// Per-file result produced by the lint/format pipeline.
#[derive(Debug)]
pub struct FileOutcome {
    pub path: PathBuf,
    pub diagnostics: Vec<Diagnostic>,
}

// ── Text rendering ────────────────────────────────────────────────────────────

pub fn render_text(path: &str, diagnostics: &[Diagnostic]) -> String {
    render_text_multi(&[FileOutcome {
        path: PathBuf::from(path),
        diagnostics: diagnostics.to_vec(),
    }])
}

pub fn render_text_multi(outcomes: &[FileOutcome]) -> String {
    outcomes
        .iter()
        .flat_map(|o| {
            let path = o.path.to_string_lossy().into_owned();
            o.diagnostics.iter().map(move |d| {
                format!(
                    "{}:{}:{} {} {}",
                    path, d.span.start_line, d.span.start_col, d.code, d.message
                )
            })
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ── JSON rendering ────────────────────────────────────────────────────────────

pub fn render_json(path: &str, diagnostics: &[Diagnostic]) -> Result<String, serde_json::Error> {
    render_json_multi(&[FileOutcome {
        path: PathBuf::from(path),
        diagnostics: diagnostics.to_vec(),
    }])
}

pub fn render_json_multi(outcomes: &[FileOutcome]) -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct JsonFile<'a> {
        file: String,
        diagnostics: &'a [Diagnostic],
    }

    #[derive(Serialize)]
    struct JsonOutput<'a> {
        schema_version: u32,
        files: Vec<JsonFile<'a>>,
    }

    let files = outcomes
        .iter()
        .filter(|o| !o.diagnostics.is_empty())
        .map(|o| JsonFile {
            file: o.path.to_string_lossy().into_owned(),
            diagnostics: &o.diagnostics,
        })
        .collect();

    serde_json::to_string_pretty(&JsonOutput {
        schema_version: 1,
        files,
    })
}

// ── SARIF rendering ───────────────────────────────────────────────────────────

pub fn render_sarif(path: &str, diagnostics: &[Diagnostic]) -> Result<String, serde_json::Error> {
    render_sarif_multi(&[FileOutcome {
        path: PathBuf::from(path),
        diagnostics: diagnostics.to_vec(),
    }])
}

pub fn render_sarif_multi(outcomes: &[FileOutcome]) -> Result<String, serde_json::Error> {
    let rules = vec![
        rule("R001", "Missing first-line comment", "error"),
        rule("R002", "Unclosed block comment", "error"),
        rule("R003", "Unmatched DO/END", "error"),
        rule("R004", "Unmatched SELECT/END", "error"),
        rule("R005", "Duplicate labels", "warning"),
        rule("R006", "Unreachable code", "warning"),
        rule("R007", "Unsafe INTERPRET", "warning"),
        rule("R008", "Inconsistent keyword casing", "warning"),
        rule("R009", "Line too long", "warning"),
        rule(
            "R010",
            "Tabs forbidden (mainframe-compatible profile)",
            "error",
        ),
    ];

    let results: Vec<SarifResult<'_>> = outcomes
        .iter()
        .flat_map(|o| {
            let path_str = o.path.to_string_lossy().into_owned();
            o.diagnostics.iter().map(move |d| SarifResult {
                rule_id: &d.code,
                level: severity_to_level(d.severity),
                message: SarifMessage {
                    text: d.message.clone(),
                },
                locations: vec![SarifLocation {
                    physical_location: SarifPhysicalLocation {
                        artifact_location: SarifArtifactLocation {
                            uri: path_str.clone(),
                        },
                        region: SarifRegion {
                            start_line: d.span.start_line as usize,
                            start_column: d.span.start_col as usize,
                        },
                    },
                }],
            })
        })
        .collect();

    let sarif = Sarif {
        schema_version: 1,
        version: "2.1.0",
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "rexxlint",
                    rules,
                },
            },
            results,
        }],
    };

    serde_json::to_string_pretty(&sarif)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn severity_to_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
    }
}

fn rule(id: &'static str, name: &'static str, level: &'static str) -> SarifRule<'static> {
    SarifRule {
        id,
        short_description: SarifMessage {
            text: name.to_string(),
        },
        help: SarifMessage {
            text: format!("Rule {id}: {name}"),
        },
        default_configuration: SarifRuleConfig { level },
    }
}

// ── SARIF types ───────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct Sarif<'a> {
    schema_version: u32,
    version: &'a str,
    runs: Vec<SarifRun<'a>>,
}

#[derive(Serialize)]
struct SarifRun<'a> {
    tool: SarifTool<'a>,
    results: Vec<SarifResult<'a>>,
}

#[derive(Serialize)]
struct SarifTool<'a> {
    driver: SarifDriver<'a>,
}

#[derive(Serialize)]
struct SarifDriver<'a> {
    name: &'a str,
    rules: Vec<SarifRule<'a>>,
}

#[derive(Serialize)]
struct SarifRule<'a> {
    id: &'a str,
    #[serde(rename = "shortDescription")]
    short_description: SarifMessage,
    help: SarifMessage,
    #[serde(rename = "defaultConfiguration")]
    default_configuration: SarifRuleConfig<'a>,
}

#[derive(Serialize)]
struct SarifRuleConfig<'a> {
    level: &'a str,
}

#[derive(Serialize)]
struct SarifResult<'a> {
    #[serde(rename = "ruleId")]
    rule_id: &'a str,
    level: &'a str,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
struct SarifRegion {
    #[serde(rename = "startLine")]
    start_line: usize,
    #[serde(rename = "startColumn")]
    start_column: usize,
}
