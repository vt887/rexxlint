use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormattingProfile {
    pub name: &'static str,
    pub line_length_soft: usize,
    pub line_length_hard: usize,
    pub uppercase_keywords: bool,
    pub tabs_forbidden: bool,
}

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("unknown formatting profile: {0}")]
    UnknownProfile(String),
}

pub const MAINFRAME_COMPATIBLE: FormattingProfile = FormattingProfile {
    name: "mainframe-compatible",
    line_length_soft: 72,
    line_length_hard: 80,
    uppercase_keywords: true,
    tabs_forbidden: true,
};

pub const STANDARD: FormattingProfile = FormattingProfile {
    name: "standard",
    line_length_soft: 100,
    line_length_hard: 200,
    uppercase_keywords: false,
    tabs_forbidden: false,
};

pub const MINIMAL: FormattingProfile = FormattingProfile {
    name: "minimal",
    line_length_soft: 200,
    line_length_hard: 200,
    uppercase_keywords: false,
    tabs_forbidden: false,
};

pub fn load_profile(name: &str) -> Result<FormattingProfile, ProfileError> {
    match name {
        "mainframe-compatible" | "mainframe" => Ok(MAINFRAME_COMPATIBLE),
        "standard" => Ok(STANDARD),
        "minimal" => Ok(MINIMAL),
        _ => Err(ProfileError::UnknownProfile(name.to_string())),
    }
}

pub fn default_profile() -> FormattingProfile {
    MAINFRAME_COMPATIBLE
}

#[cfg(test)]
mod tests {
    use super::{MAINFRAME_COMPATIBLE, default_profile, load_profile};

    #[test]
    fn loads_mainframe_profile() {
        let p = load_profile("mainframe-compatible").expect("profile exists");
        assert_eq!(p, MAINFRAME_COMPATIBLE);
    }

    #[test]
    fn default_is_mainframe() {
        assert_eq!(default_profile(), MAINFRAME_COMPATIBLE);
    }
}
