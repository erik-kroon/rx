use crate::charset::UnicodeClass;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub span: SourceSpan,
    pub category: DiagnosticCategory,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub suggestion: Option<String>,
    pub source_family: DiagnosticSourceFamily,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticCategory {
    Syntax,
    Validation,
    Compatibility,
    Lint,
    Migration,
    Dialect,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticSourceFamily {
    Core,
    LegacyRegex,
    ReadableRx,
    RustRegexMigration,
    Dialect,
}
pub trait ToDiagnostic {
    fn to_diagnostic(&self) -> Diagnostic;
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dialect {
    RustRegex,
    Pcre2,
    PosixEre,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Feature {
    NamedCapture,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidRange { start: char, end: char },
    InvalidRepeatBounds { min: usize, max: usize },
    InvalidCaptureName(String),
    UnsupportedUnicodeClass(UnicodeClass),
    UnsupportedDialectFeature { dialect: Dialect, feature: Feature },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    pub span: SourceSpan,
    pub kind: ParseErrorKind,
    pub message: String,
    pub suggestion: String,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseErrorKind {
    UnexpectedEnd,
    UnexpectedToken(char),
    UnclosedGroup,
    UnclosedClass,
    EmptyRepeat,
    InvalidRepeatBounds { min: usize, max: usize },
    InvalidRange { start: char, end: char },
    UnsupportedFeature(UnsupportedFeature),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LintDiagnostic {
    pub span: SourceSpan,
    pub kind: LintDiagnosticKind,
    pub message: String,
    pub suggestion: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LintDiagnosticKind {
    UnnecessaryEscape { escaped: char },
    AmbiguousCharacterClass { class: LegacyCharacterClass },
    InvalidRange { start: char, end: char },
    InvalidCaptureName(String),
    UnsupportedFeature(UnsupportedFeature),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyCharacterClass {
    Word,
    Digit,
    Whitespace,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnsupportedFeature {
    Alternation,
    Backreference,
    Conditional,
    EngineSpecificEscape,
    LookBehind,
    LookAhead,
    NonCapturingGroup,
    RecursivePattern,
    ClassNegation,
    Wildcard,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustRegexSuggestion {
    pub span: SourceSpan,
    pub literal_span: SourceSpan,
    pub constructor: RustRegexConstructor,
    pub regex: String,
    pub replacement: Option<ReplacementSuggestion>,
    pub diagnostics: Vec<SuggestionDiagnostic>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RustRegexConstructor {
    RegexNew,
    RegexBuilderNew,
    BytesRegexNew,
    BytesRegexBuilderNew,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplacementSuggestion {
    pub builder: String,
    pub macro_form: String,
    pub generated_regex: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuggestionDiagnostic {
    pub span: SourceSpan,
    pub kind: SuggestionDiagnosticKind,
    pub message: String,
    pub suggestion: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuggestionDiagnosticKind {
    UnsupportedRustRegexConstruction,
    InvalidRustStringLiteral,
    UnsupportedRegex(ParseErrorKind),
    LegacyRegexLint(LintDiagnosticKind),
}
impl Diagnostic {
    pub fn source_location(&self, source: &str) -> SourceLocation {
        source_location(source, self.span.start)
    }

    pub fn render(&self) -> String {
        let mut output = format!(
            "{} {} in {} at bytes {}..{}: {}",
            self.severity.as_str(),
            self.category.as_str(),
            self.source_family.as_str(),
            self.span.start,
            self.span.end,
            self.message
        );
        if let Some(suggestion) = &self.suggestion {
            output.push_str("\nsuggestion: ");
            output.push_str(suggestion);
        }
        output
    }

    pub fn render_with_source(&self, source: &str) -> String {
        let SourceLocation { line, column } = self.source_location(source);
        let mut output = format!(
            "{} {} in {} at line {line}, column {column} (bytes {}..{}): {}",
            self.severity.as_str(),
            self.category.as_str(),
            self.source_family.as_str(),
            self.span.start,
            self.span.end,
            self.message
        );
        if let Some(suggestion) = &self.suggestion {
            output.push_str("\nsuggestion: ");
            output.push_str(suggestion);
        }
        output
    }
}
pub fn source_location(input: &str, offset: usize) -> SourceLocation {
    let mut line = 1;
    let mut column = 1;
    for (index, ch) in input.char_indices() {
        if index >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    SourceLocation { line, column }
}
impl DiagnosticCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticCategory::Syntax => "syntax",
            DiagnosticCategory::Validation => "validation",
            DiagnosticCategory::Compatibility => "compatibility",
            DiagnosticCategory::Lint => "lint",
            DiagnosticCategory::Migration => "migration",
            DiagnosticCategory::Dialect => "dialect",
        }
    }
}
impl DiagnosticSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
        }
    }
}
impl DiagnosticSourceFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticSourceFamily::Core => "core",
            DiagnosticSourceFamily::LegacyRegex => "legacy regex",
            DiagnosticSourceFamily::ReadableRx => "readable rx",
            DiagnosticSourceFamily::RustRegexMigration => "Rust regex migration",
            DiagnosticSourceFamily::Dialect => "dialect",
        }
    }
}
impl ToDiagnostic for ParseError {
    fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic {
            span: self.span,
            category: parse_error_category(&self.kind),
            severity: DiagnosticSeverity::Error,
            message: self.message.clone(),
            suggestion: Some(self.suggestion.clone()),
            source_family: DiagnosticSourceFamily::LegacyRegex,
        }
    }
}
impl ToDiagnostic for LintDiagnostic {
    fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic {
            span: self.span,
            category: lint_diagnostic_category(&self.kind),
            severity: DiagnosticSeverity::Warning,
            message: self.message.clone(),
            suggestion: Some(self.suggestion.clone()),
            source_family: DiagnosticSourceFamily::LegacyRegex,
        }
    }
}
impl ToDiagnostic for SuggestionDiagnostic {
    fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic {
            span: self.span,
            category: suggestion_diagnostic_category(&self.kind),
            severity: DiagnosticSeverity::Warning,
            message: self.message.clone(),
            suggestion: Some(self.suggestion.clone()),
            source_family: DiagnosticSourceFamily::RustRegexMigration,
        }
    }
}
impl ToDiagnostic for Error {
    fn to_diagnostic(&self) -> Diagnostic {
        let category = match self {
            Error::UnsupportedDialectFeature { .. } => DiagnosticCategory::Dialect,
            Error::UnsupportedUnicodeClass(_) => DiagnosticCategory::Compatibility,
            Error::InvalidRange { .. }
            | Error::InvalidRepeatBounds { .. }
            | Error::InvalidCaptureName(_) => DiagnosticCategory::Validation,
        };
        Diagnostic {
            span: SourceSpan { start: 0, end: 0 },
            category,
            severity: DiagnosticSeverity::Error,
            message: core_error_message(self),
            suggestion: core_error_suggestion(self),
            source_family: DiagnosticSourceFamily::Core,
        }
    }
}
pub fn parse_error_category(kind: &ParseErrorKind) -> DiagnosticCategory {
    match kind {
        ParseErrorKind::UnsupportedFeature(_) => DiagnosticCategory::Compatibility,
        ParseErrorKind::InvalidRepeatBounds { .. } | ParseErrorKind::InvalidRange { .. } => {
            DiagnosticCategory::Validation
        }
        ParseErrorKind::UnexpectedEnd
        | ParseErrorKind::UnexpectedToken(_)
        | ParseErrorKind::UnclosedGroup
        | ParseErrorKind::UnclosedClass
        | ParseErrorKind::EmptyRepeat => DiagnosticCategory::Syntax,
    }
}
pub fn lint_diagnostic_category(kind: &LintDiagnosticKind) -> DiagnosticCategory {
    match kind {
        LintDiagnosticKind::UnsupportedFeature(_) => DiagnosticCategory::Compatibility,
        _ => DiagnosticCategory::Lint,
    }
}
pub fn suggestion_diagnostic_category(kind: &SuggestionDiagnosticKind) -> DiagnosticCategory {
    match kind {
        SuggestionDiagnosticKind::UnsupportedRegex(parse_kind) => parse_error_category(parse_kind),
        SuggestionDiagnosticKind::LegacyRegexLint(lint_kind) => lint_diagnostic_category(lint_kind),
        _ => DiagnosticCategory::Migration,
    }
}
fn core_error_message(error: &Error) -> String {
    match error {
        Error::InvalidRange { start, end } => {
            format!("invalid range: start '{start}' is greater than end '{end}'")
        }
        Error::InvalidRepeatBounds { min, max } => {
            format!("invalid repeat bounds: min {min} is greater than max {max}")
        }
        Error::InvalidCaptureName(name) => format!("invalid capture name {name:?}"),
        Error::UnsupportedUnicodeClass(class) => {
            format!("unsupported Unicode class {class:?} in the MVP safe core")
        }
        Error::UnsupportedDialectFeature { dialect, feature } => {
            format!("cannot emit for {dialect:?}: unsupported feature {feature:?}")
        }
    }
}
fn core_error_suggestion(error: &Error) -> Option<String> {
    let suggestion = match error {
        Error::InvalidRange { .. } => "Put the lower character first or escape the hyphen.",
        Error::InvalidRepeatBounds { .. } => {
            "Use bounds where the lower number is not greater than the upper number."
        }
        Error::InvalidCaptureName(_) => {
            "Use a capture name that starts with a letter or underscore and contains only letters, digits, or underscore."
        }
        Error::UnsupportedUnicodeClass(_) => {
            "Use an explicit ASCII class or keep Unicode behavior outside automatic conversion."
        }
        Error::UnsupportedDialectFeature { .. } => {
            "Choose a dialect that supports this feature or rewrite the pattern."
        }
    };
    Some(suggestion.to_string())
}
pub(crate) fn is_valid_capture_name(name: &str) -> bool {
    let mut c = name.chars();
    matches!(c.next(), Some('A'..='Z' | 'a'..='z' | '_'))
        && c.all(|x| matches!(x,'A'..='Z'|'a'..='z'|'0'..='9'|'_'))
}
impl Dialect {
    pub(crate) fn supports(self, feature: Feature) -> bool {
        matches!(
            (self, feature),
            (Dialect::RustRegex | Dialect::Pcre2, Feature::NamedCapture)
        )
    }
}
