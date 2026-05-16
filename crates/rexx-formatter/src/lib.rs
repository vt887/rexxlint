use rexx_config::{FormattingProfile, ProfileError, default_profile, load_profile};

const KEYWORDS: &[&str] = &[
    "do",
    "end",
    "select",
    "when",
    "then",
    "otherwise",
    "if",
    "else",
    "return",
    "exit",
    "interpret",
    "say",
];

pub fn format_rexx(input: &str) -> String {
    format_rexx_with_profile(input, default_profile())
}

pub fn format_rexx_with_profile_name(
    input: &str,
    profile_name: &str,
) -> Result<String, ProfileError> {
    let profile = load_profile(profile_name)?;
    Ok(format_rexx_with_profile(input, profile))
}

pub fn format_rexx_with_profile(input: &str, profile: FormattingProfile) -> String {
    let mut lines: Vec<String> = input
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect();

    if let Some((idx, line)) = lines
        .iter()
        .enumerate()
        .find(|(_, line)| !line.trim().is_empty())
    {
        let trimmed = line.trim();
        let comment_ok = trimmed.starts_with("/*") && trimmed.ends_with("*/");
        if !comment_ok {
            lines.insert(
                idx,
                "/* The first line of a REXX exec must always be a comment. */".to_string(),
            );
        }
    } else {
        lines.push("/* The first line of a REXX exec must always be a comment. */".to_string());
    }

    let mut out = Vec::with_capacity(lines.len());
    let mut blank = false;
    let mut in_block_comment = false;
    for line in lines {
        let lower = line.trim().to_ascii_lowercase();
        let leading = if lower == "end" || lower.starts_with("otherwise") {
            0
        } else {
            4
        };

        let normalized = if line.trim().is_empty() {
            if blank {
                continue;
            }
            blank = true;
            String::new()
        } else {
            blank = false;
            let mut body = line.trim().replace('\t', "    ");
            if profile.uppercase_keywords {
                body = normalize_keywords_upper(&body, &mut in_block_comment);
            }
            format!("{}{}", " ".repeat(leading), body)
        };
        out.push(normalized);
    }

    if !out.is_empty() {
        out[0] = out[0].trim_start().to_string();
    }

    out.join("\n") + "\n"
}

fn normalize_keywords_upper(line: &str, in_block_comment: &mut bool) -> String {
    let mut out = String::with_capacity(line.len());
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0usize;
    let mut in_single = false;
    let mut in_double = false;

    while i < chars.len() {
        let ch = chars[i];
        let next = chars.get(i + 1).copied();

        if *in_block_comment {
            out.push(ch);
            if ch == '*' && next == Some('/') {
                out.push('/');
                i += 2;
                *in_block_comment = false;
                continue;
            }
            i += 1;
            continue;
        }

        if !in_single && !in_double && ch == '/' && next == Some('*') {
            out.push('/');
            out.push('*');
            i += 2;
            *in_block_comment = true;
            continue;
        }

        if !in_double && ch == '\'' {
            // doubled quote inside string is an escape — skip both chars
            if in_single && i + 1 < chars.len() && chars[i + 1] == '\'' {
                out.push(ch);
                out.push('\'');
                i += 2;
                continue;
            }
            in_single = !in_single;
            out.push(ch);
            i += 1;
            continue;
        }
        if !in_single && ch == '"' {
            if in_double && i + 1 < chars.len() && chars[i + 1] == '"' {
                out.push(ch);
                out.push('"');
                i += 2;
                continue;
            }
            in_double = !in_double;
            out.push(ch);
            i += 1;
            continue;
        }

        if in_single || in_double {
            out.push(ch);
            i += 1;
            continue;
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = i;
            i += 1;
            while i < chars.len() {
                let c = chars[i];
                if c.is_ascii_alphanumeric() || c == '_' {
                    i += 1;
                } else {
                    break;
                }
            }
            let word: String = chars[start..i].iter().collect();
            let lower = word.to_ascii_lowercase();
            if KEYWORDS.contains(&lower.as_str()) {
                out.push_str(&word.to_ascii_uppercase());
            } else {
                out.push_str(&word);
            }
            continue;
        }

        out.push(ch);
        i += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    use rexx_config::MAINFRAME_COMPATIBLE;

    use super::{format_rexx, format_rexx_with_profile};

    #[test]
    fn inserts_first_line_comment() {
        let out = format_rexx("say 'x'\n");
        assert!(out.starts_with("/* The first line of a REXX exec must always be a comment. */"));
    }

    #[test]
    fn mainframe_profile_uppercases_keywords_conservatively() {
        let src = "/* ok */\nsay 'do not touch'\ndo\n/* keep comment do */\nend\n";
        let out = format_rexx_with_profile(src, MAINFRAME_COMPATIBLE);
        assert!(out.contains("SAY 'do not touch'"));
        assert!(out.contains("DO"));
        assert!(out.contains("/* keep comment do */"));
        assert!(out.contains("END"));
    }

    #[test]
    fn mainframe_profile_tabs_expanded_and_deterministic() {
        let src = "say 'x'\n\treturn\n";
        let a = format_rexx_with_profile(src, MAINFRAME_COMPATIBLE);
        let b = format_rexx_with_profile(src, MAINFRAME_COMPATIBLE);
        assert_eq!(a, b);
        assert!(!a.contains('\t'));
    }
}
