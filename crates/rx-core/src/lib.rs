//! Core AST and emitters for `rx`.
//!
//! This crate intentionally stays below the public builder API. It owns the
//! canonical pattern representation and standard regex emission behavior.

mod behavior;
mod charset;
mod diagnostic;
mod emit;
mod explain;
mod legacy;
mod lint;
mod migration;
mod pattern;
mod pretty;
mod syntax;

pub use behavior::{
    check_generated_regex_sample_inputs, check_sample_inputs, SampleBehaviorReport,
    SampleCheckError, SampleInputCheck, SampleRegexSide,
};
pub use charset::{AsciiClass, CharSet, CharSetItem, UnicodeClass};
pub use diagnostic::{
    Diagnostic, DiagnosticCategory, DiagnosticSeverity, DiagnosticSourceFamily, Dialect, Error,
    Feature, LegacyCharacterClass, LintDiagnostic, LintDiagnosticKind, ParseError, ParseErrorKind,
    ReplacementSuggestion, RustRegexConstructor, RustRegexSuggestion, SourceLocation, SourceSpan,
    SuggestionDiagnostic, SuggestionDiagnosticKind, ToDiagnostic, UnsupportedFeature,
};
pub use legacy::{analyze_legacy_regex, parse_legacy_regex, LegacyRegexAnalysis};
pub use lint::lint_legacy_regex;
pub use migration::{suggest_rust_regex_replacements, suggest_rust_regex_replacements_in_range};
pub use pattern::Pattern;
pub use syntax::{
    parse_readable_rx, parse_readable_rx_artifact, parse_readable_rx_file,
    parse_readable_rx_file_artifacts, ReadableParseError, ReadablePatternArtifact,
    ReadablePatternDefinitionArtifact,
};
