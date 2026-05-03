use crate::Pattern;

/// Which regex failed to compile for a behavior-preservation sample check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampleRegexSide {
    Legacy,
    Generated,
}

/// Error returned when a sample behavior check cannot compile one side.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SampleCheckError {
    pub side: SampleRegexSide,
    pub regex: String,
    pub message: String,
}

/// Match result for one sample input checked against both regexes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SampleInputCheck {
    pub input: String,
    pub legacy_matches: bool,
    pub generated_matches: bool,
}

/// Behavior-preservation report for a set of sample inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SampleBehaviorReport {
    pub legacy_regex: String,
    pub generated_regex: String,
    pub checks: Vec<SampleInputCheck>,
}

impl SampleInputCheck {
    /// Whether the generated regex matched this sample the same way as the legacy regex.
    pub fn is_preserved(&self) -> bool {
        self.legacy_matches == self.generated_matches
    }
}

impl SampleBehaviorReport {
    /// Whether every provided sample preserved legacy match behavior.
    pub fn is_preserved(&self) -> bool {
        self.checks.iter().all(SampleInputCheck::is_preserved)
    }

    /// Sample checks where the generated regex did not preserve legacy behavior.
    pub fn mismatches(&self) -> impl Iterator<Item = &SampleInputCheck> {
        self.checks.iter().filter(|check| !check.is_preserved())
    }
}

/// Compare a legacy regex and a validated pattern against sample inputs using Rust regex semantics.
pub fn check_sample_inputs(
    legacy_regex: &str,
    pattern: &Pattern,
    samples: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<SampleBehaviorReport, SampleCheckError> {
    check_generated_regex_sample_inputs(legacy_regex, &pattern.to_regex(), samples)
}

/// Compare two regex strings against sample inputs using Rust regex semantics.
pub fn check_generated_regex_sample_inputs(
    legacy_regex: &str,
    generated_regex: &str,
    samples: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<SampleBehaviorReport, SampleCheckError> {
    let legacy = compile_sample_regex(SampleRegexSide::Legacy, legacy_regex)?;
    let generated = compile_sample_regex(SampleRegexSide::Generated, generated_regex)?;

    let checks = samples
        .into_iter()
        .map(|sample| {
            let input = sample.as_ref().to_string();
            SampleInputCheck {
                legacy_matches: legacy.is_match(&input),
                generated_matches: generated.is_match(&input),
                input,
            }
        })
        .collect();

    Ok(SampleBehaviorReport {
        legacy_regex: legacy_regex.to_string(),
        generated_regex: generated_regex.to_string(),
        checks,
    })
}

fn compile_sample_regex(
    side: SampleRegexSide,
    input: &str,
) -> Result<regex::Regex, SampleCheckError> {
    regex::Regex::new(input).map_err(|error| SampleCheckError {
        side,
        regex: input.to_string(),
        message: error.to_string(),
    })
}
