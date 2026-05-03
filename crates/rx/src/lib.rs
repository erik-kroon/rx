//! Public Rust API for readable regex construction.

pub use rx_macros::{pattern, regex};

pub use rx_core::{
    AsciiClass, Diagnostic, DiagnosticCategory, DiagnosticSeverity, DiagnosticSourceFamily,
    Dialect, Error, Feature, LegacyCharacterClass, LintDiagnostic, LintDiagnosticKind, ParseError,
    ParseErrorKind, ReadableParseError, ReadablePatternArtifact, ReadablePatternDefinitionArtifact,
    SampleBehaviorReport, SampleCheckError, SampleInputCheck, SampleRegexSide, SourceLocation,
    SourceSpan, ToDiagnostic, UnicodeClass, UnsupportedFeature,
};

/// A typed readable regex pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pattern {
    inner: rx_core::Pattern,
}

/// Public character set item for validated set construction.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SetItem {
    inner: rx_core::CharSetItem,
}

/// Public builder for character sets.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Set {
    items: Vec<SetItem>,
}

impl Pattern {
    /// Emit this pattern as a compact standard regex string.
    pub fn to_regex(&self) -> String {
        self.inner.to_regex()
    }

    /// Emit this pattern into an existing string buffer.
    pub fn write_regex(&self, output: &mut String) {
        self.inner.write_regex(output);
    }

    /// Emit this pattern for a specific regex dialect.
    pub fn emit(&self, dialect: Dialect) -> Result<String, Error> {
        self.inner.emit(dialect)
    }

    /// Emit this pattern for a specific regex dialect into an existing string buffer.
    pub fn write_regex_for(&self, dialect: Dialect, output: &mut String) -> Result<(), Error> {
        self.inner.write_regex_for(dialect, output)
    }

    /// Pretty-print this pattern as readable rx source.
    pub fn to_rx(&self) -> String {
        self.inner.to_rx()
    }

    /// Explain this pattern in plain English.
    pub fn explain(&self) -> String {
        self.inner.explain()
    }

    /// Repeat this pattern zero or more times.
    pub fn zero_or_more(self) -> Self {
        rx_core::Pattern::zero_or_more(self.inner).into()
    }

    /// Repeat this pattern one or more times.
    pub fn one_or_more(self) -> Self {
        rx_core::Pattern::one_or_more(self.inner).into()
    }

    /// Make this pattern optional.
    pub fn optional(self) -> Self {
        rx_core::Pattern::optional(self.inner).into()
    }

    /// Repeat this pattern between the provided bounds.
    pub fn repeat_between(self, min: usize, max: usize) -> Result<Self, Error> {
        rx_core::Pattern::repeat_between(self.inner, min, max).map(Into::into)
    }

    /// Repeat this pattern exactly the provided number of times.
    pub fn repeat(self, count: usize) -> Self {
        rx_core::Pattern::repeat_exactly(self.inner, count).into()
    }
}

impl From<rx_core::Pattern> for Pattern {
    fn from(inner: rx_core::Pattern) -> Self {
        Self { inner }
    }
}

impl Set {
    /// Start an empty character set builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one set item.
    pub fn item(mut self, item: SetItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add one literal character to the set.
    pub fn char(self, value: char) -> Self {
        self.item(char(value))
    }

    /// Add each character in the provided string as a literal set item.
    pub fn chars(mut self, value: impl AsRef<str>) -> Self {
        self.items.extend(value.as_ref().chars().map(char));
        self
    }

    /// Add a validated character range to the set.
    pub fn range(self, start: char, end: char) -> Result<Self, Error> {
        Ok(self.item(range(start, end)?))
    }

    /// Add ASCII word characters: `[A-Za-z0-9_]`.
    pub fn ascii_word(self) -> Self {
        self.item(ascii::word())
    }

    /// Add ASCII alphanumeric characters: `[A-Za-z0-9]`.
    pub fn ascii_alnum(self) -> Self {
        self.item(ascii::alnum())
    }

    /// Add ASCII alphabetic characters: `[A-Za-z]`.
    pub fn ascii_alpha(self) -> Self {
        self.item(ascii::alpha())
    }

    /// Add ASCII decimal digits: `[0-9]`.
    pub fn ascii_digit(self) -> Self {
        self.item(ascii::digit())
    }

    /// Add ASCII whitespace characters.
    pub fn ascii_whitespace(self) -> Self {
        self.item(ascii::whitespace())
    }

    /// Finish the set as a pattern.
    pub fn into_pattern(self) -> Pattern {
        rx_core::Pattern::set(self.items.into_iter().map(|item| item.inner)).into()
    }

    /// Finish the set and repeat it zero or more times.
    pub fn zero_or_more(self) -> Pattern {
        self.into_pattern().zero_or_more()
    }

    /// Finish the set and repeat it one or more times.
    pub fn one_or_more(self) -> Pattern {
        self.into_pattern().one_or_more()
    }

    /// Finish the set and make it optional.
    pub fn optional(self) -> Pattern {
        self.into_pattern().optional()
    }

    /// Finish the set and repeat it between the provided bounds.
    pub fn repeat_between(self, min: usize, max: usize) -> Result<Pattern, Error> {
        self.into_pattern().repeat_between(min, max)
    }

    /// Finish the set and repeat it exactly the provided number of times.
    pub fn repeat(self, count: usize) -> Pattern {
        self.into_pattern().repeat(count)
    }
}

impl IntoIterator for Set {
    type Item = SetItem;
    type IntoIter = std::vec::IntoIter<SetItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<const N: usize> From<[SetItem; N]> for Set {
    fn from(items: [SetItem; N]) -> Self {
        Self {
            items: Vec::from(items),
        }
    }
}

impl From<Vec<SetItem>> for Set {
    fn from(items: Vec<SetItem>) -> Self {
        Self { items }
    }
}

/// Construct a pattern that matches the provided text literally.
pub fn literal(value: impl Into<String>) -> Pattern {
    rx_core::Pattern::literal(value).into()
}

/// Construct a pattern that matches one character from the provided set.
pub fn set(items: impl IntoIterator<Item = SetItem>) -> Pattern {
    rx_core::Pattern::set(items.into_iter().map(|item| item.inner)).into()
}

/// Start a character set builder.
pub fn set_builder() -> Set {
    Set::new()
}

/// Construct a character set builder from literal characters.
pub fn chars(value: impl AsRef<str>) -> Set {
    Set::new().chars(value)
}

/// Construct a sequence that emits each pattern in order.
pub fn sequence(patterns: impl IntoIterator<Item = Pattern>) -> Pattern {
    rx_core::Pattern::sequence(patterns.into_iter().map(|pattern| pattern.inner)).into()
}

/// Construct an alternation that matches any one of the provided patterns.
pub fn either(patterns: impl IntoIterator<Item = Pattern>) -> Pattern {
    rx_core::Pattern::either(patterns.into_iter().map(|pattern| pattern.inner)).into()
}

/// Repeat a pattern zero or more times.
pub fn zero_or_more(pattern: Pattern) -> Pattern {
    pattern.zero_or_more()
}

/// Repeat a pattern one or more times.
pub fn one_or_more(pattern: Pattern) -> Pattern {
    pattern.one_or_more()
}

/// Make a pattern optional.
pub fn optional(pattern: Pattern) -> Pattern {
    pattern.optional()
}

/// Repeat a pattern between the provided bounds.
pub fn repeat_between(pattern: Pattern, min: usize, max: usize) -> Result<Pattern, Error> {
    pattern.repeat_between(min, max)
}

/// Repeat a pattern exactly the provided number of times.
pub fn repeat(pattern: Pattern, count: usize) -> Pattern {
    pattern.repeat(count)
}

/// Parse a focused MVP subset of legacy regex syntax.
pub fn parse_legacy_regex(input: &str) -> Result<Pattern, ParseError> {
    rx_core::parse_legacy_regex(input).map(Into::into)
}

/// Parse readable rx syntax and lower it through validated core constructors.
pub fn parse_readable_rx(input: &str) -> Result<Pattern, ReadableParseError> {
    rx_core::parse_readable_rx(input).map(Into::into)
}

/// Parse readable rx syntax into a validated artifact with spans and generated builder code.
pub fn parse_readable_rx_artifact(
    input: &str,
) -> Result<ReadablePatternArtifact, ReadableParseError> {
    rx_core::parse_readable_rx_artifact(input)
}

/// Parse a readable rx file and lower every definition through validated core constructors.
pub fn parse_readable_rx_file(input: &str) -> Result<Vec<(String, Pattern)>, ReadableParseError> {
    rx_core::parse_readable_rx_file(input).map(|patterns| {
        patterns
            .into_iter()
            .map(|(name, pattern)| (name, pattern.into()))
            .collect()
    })
}

/// Parse a readable rx file into validated definition artifacts.
pub fn parse_readable_rx_file_artifacts(
    input: &str,
) -> Result<Vec<ReadablePatternDefinitionArtifact>, ReadableParseError> {
    rx_core::parse_readable_rx_file_artifacts(input)
}

/// Lint legacy regex syntax with structured diagnostics for tools.
pub fn lint_legacy_regex(input: &str) -> Vec<LintDiagnostic> {
    rx_core::lint_legacy_regex(input)
}

/// Compare a legacy regex and a validated pattern against sample inputs using Rust regex semantics.
pub fn check_sample_inputs(
    legacy_regex: &str,
    pattern: &Pattern,
    samples: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<SampleBehaviorReport, SampleCheckError> {
    rx_core::check_sample_inputs(legacy_regex, &pattern.inner, samples)
}

/// Compare two regex strings against sample inputs using Rust regex semantics.
pub fn check_generated_regex_sample_inputs(
    legacy_regex: &str,
    generated_regex: &str,
    samples: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<SampleBehaviorReport, SampleCheckError> {
    rx_core::check_generated_regex_sample_inputs(legacy_regex, generated_regex, samples)
}

/// Construct a start-of-text anchor.
pub fn start_text() -> Pattern {
    rx_core::Pattern::start_text().into()
}

/// Construct an end-of-text anchor.
pub fn end_text() -> Pattern {
    rx_core::Pattern::end_text().into()
}

/// Construct a numbered capture.
pub fn capture(pattern: Pattern) -> Pattern {
    rx_core::Pattern::capture(pattern.inner).into()
}

/// Construct a validated named capture.
pub fn named_capture(name: impl Into<String>, pattern: Pattern) -> Result<Pattern, Error> {
    rx_core::Pattern::named_capture(name, pattern.inner).map(Into::into)
}

/// Construct a literal character set item.
pub fn char(value: char) -> SetItem {
    rx_core::CharSetItem::literal(value).into()
}

/// Construct a validated character range set item.
pub fn range(start: char, end: char) -> Result<SetItem, Error> {
    rx_core::CharSetItem::range(start, end).map(Into::into)
}

impl From<rx_core::CharSetItem> for SetItem {
    fn from(inner: rx_core::CharSetItem) -> Self {
        Self { inner }
    }
}

/// Explicit ASCII character class constructors.
pub mod ascii {
    use crate::{Pattern, Set, SetItem};

    /// ASCII word characters: `[A-Za-z0-9_]`.
    pub fn word() -> SetItem {
        rx_core::CharSetItem::ascii(rx_core::AsciiClass::Word).into()
    }

    /// ASCII alphanumeric characters: `[A-Za-z0-9]`.
    pub fn alnum() -> SetItem {
        rx_core::CharSetItem::ascii(rx_core::AsciiClass::Alnum).into()
    }

    /// ASCII alphabetic characters: `[A-Za-z]`.
    pub fn alpha() -> SetItem {
        rx_core::CharSetItem::ascii(rx_core::AsciiClass::Alpha).into()
    }

    /// ASCII decimal digits: `[0-9]`.
    pub fn digit() -> SetItem {
        rx_core::CharSetItem::ascii(rx_core::AsciiClass::Digit).into()
    }

    /// ASCII whitespace characters: tab, line feed, form feed, carriage return, and space.
    pub fn whitespace() -> SetItem {
        rx_core::CharSetItem::ascii(rx_core::AsciiClass::Whitespace).into()
    }

    /// One ASCII word character as a pattern.
    pub fn word_char() -> Pattern {
        Set::new().ascii_word().into_pattern()
    }

    /// One ASCII digit as a pattern.
    pub fn digit_char() -> Pattern {
        Set::new().ascii_digit().into_pattern()
    }

    /// Identifier shape with an ASCII alphabetic or `_` head and ASCII alphanumeric or `_` tail.
    pub fn identifier() -> Pattern {
        crate::sequence([
            Set::new().ascii_alpha().char('_').into_pattern(),
            Set::new().ascii_alnum().char('_').zero_or_more(),
        ])
    }
}

/// Unicode class concepts that are explicit, not silently mapped to ASCII.
pub mod unicode {
    use crate::{Error, Pattern, UnicodeClass};

    /// Unicode word semantics are not part of the MVP safe core emitter.
    pub fn word() -> Result<Pattern, Error> {
        rx_core::Pattern::unicode_class(UnicodeClass::Word).map(Into::into)
    }

    /// Unicode letter semantics are not part of the MVP safe core emitter.
    pub fn letter() -> Result<Pattern, Error> {
        rx_core::Pattern::unicode_class(UnicodeClass::Letter).map(Into::into)
    }
}

/// Common imports for day-to-day `rx` usage.
pub mod prelude {
    pub use crate::{
        ascii, capture, char, chars, check_generated_regex_sample_inputs, check_sample_inputs,
        either, end_text, lint_legacy_regex, literal, named_capture, optional, parse_legacy_regex,
        parse_readable_rx, parse_readable_rx_artifact, parse_readable_rx_file,
        parse_readable_rx_file_artifacts, pattern, range, regex, repeat, repeat_between, sequence,
        set, set_builder, start_text, unicode, zero_or_more, Diagnostic, DiagnosticCategory,
        DiagnosticSeverity, DiagnosticSourceFamily, Dialect, Error, Feature, LegacyCharacterClass,
        LintDiagnostic, LintDiagnosticKind, ParseError, ParseErrorKind, Pattern,
        ReadableParseError, ReadablePatternArtifact, ReadablePatternDefinitionArtifact,
        SampleBehaviorReport, SampleCheckError, SampleInputCheck, SampleRegexSide, Set, SetItem,
        SourceSpan, ToDiagnostic, UnsupportedFeature,
    };
}
