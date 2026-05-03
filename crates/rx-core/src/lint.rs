use crate::diagnostic::LintDiagnostic;

pub fn lint_legacy_regex(input: &str) -> Vec<LintDiagnostic> {
    crate::legacy::lint_legacy_regex(input)
}
