use crate::charset::{AsciiClass, CharSetItem};
use crate::diagnostic::{
    Diagnostic, DiagnosticCategory, DiagnosticSeverity, DiagnosticSourceFamily, Error, SourceSpan,
    ToDiagnostic,
};
use crate::pattern::Pattern;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadablePatternArtifact {
    pattern: Pattern,
    span: SourceSpan,
    rust_builder_code: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadablePatternDefinitionArtifact {
    name: String,
    name_span: SourceSpan,
    pattern_span: SourceSpan,
    artifact: ReadablePatternArtifact,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadableParseError {
    pub span: SourceSpan,
    pub category: DiagnosticCategory,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReadablePatternDefinition {
    pub(crate) name: String,
    pub(crate) name_span: SourceSpan,
    pub(crate) pattern: ReadablePattern,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReadablePattern {
    pub span: SourceSpan,
    kind: ReadablePatternKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ReadablePatternKind {
    Literal(String),
    Set(Vec<ReadableSetItem>),
    Sequence(Vec<ReadablePattern>),
    Either(Vec<ReadablePattern>),
    ZeroOrMore(Box<ReadablePattern>),
    OneOrMore(Box<ReadablePattern>),
    Optional(Box<ReadablePattern>),
    Repeat {
        pattern: Box<ReadablePattern>,
        count: usize,
    },
    RepeatBetween {
        pattern: Box<ReadablePattern>,
        min: usize,
        max: usize,
    },
    StartText,
    EndText,
    Capture(Box<ReadablePattern>),
    NamedCapture {
        name: String,
        pattern: Box<ReadablePattern>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReadableSetItem {
    span: SourceSpan,
    kind: ReadableSetItemKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ReadableSetItemKind {
    Char(char),
    Chars(String),
    Range { start: char, end: char },
    Ascii(AsciiClass),
}

pub fn parse_readable_rx_artifact(
    input: &str,
) -> Result<ReadablePatternArtifact, ReadableParseError> {
    ReadableParser::new(input).parse()?.lower()
}

pub fn parse_readable_rx_file_artifacts(
    input: &str,
) -> Result<Vec<ReadablePatternDefinitionArtifact>, ReadableParseError> {
    ReadableParser::new(input)
        .parse_file()?
        .into_iter()
        .map(ReadablePatternDefinition::lower)
        .collect()
}

pub fn parse_readable_rx(input: &str) -> Result<Pattern, ReadableParseError> {
    parse_readable_rx_artifact(input).map(|artifact| artifact.into_pattern())
}

pub fn parse_readable_rx_file(input: &str) -> Result<Vec<(String, Pattern)>, ReadableParseError> {
    Ok(parse_readable_rx_file_artifacts(input)?
        .into_iter()
        .map(|definition| (definition.name, definition.artifact.into_pattern()))
        .collect())
}

impl ReadablePatternArtifact {
    pub fn pattern(&self) -> &Pattern {
        &self.pattern
    }

    pub fn into_pattern(self) -> Pattern {
        self.pattern
    }

    pub fn span(&self) -> SourceSpan {
        self.span
    }

    pub fn rust_builder_code(&self) -> &str {
        &self.rust_builder_code
    }
}

impl ReadablePatternDefinitionArtifact {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_span(&self) -> SourceSpan {
        self.name_span
    }

    pub fn pattern_span(&self) -> SourceSpan {
        self.pattern_span
    }

    pub fn artifact(&self) -> &ReadablePatternArtifact {
        &self.artifact
    }

    pub fn into_parts(self) -> (String, ReadablePatternArtifact) {
        (self.name, self.artifact)
    }
}

impl ReadablePatternDefinition {
    fn lower(self) -> Result<ReadablePatternDefinitionArtifact, ReadableParseError> {
        let pattern_span = self.pattern.span;
        Ok(ReadablePatternDefinitionArtifact {
            name: self.name,
            name_span: self.name_span,
            pattern_span,
            artifact: self.pattern.lower()?,
        })
    }
}

impl ReadablePattern {
    fn lower(&self) -> Result<ReadablePatternArtifact, ReadableParseError> {
        Ok(ReadablePatternArtifact {
            pattern: self.lower_pattern()?,
            span: self.span,
            rust_builder_code: self.rust_builder_code(),
        })
    }

    fn lower_pattern(&self) -> Result<Pattern, ReadableParseError> {
        match &self.kind {
            ReadablePatternKind::Literal(value) => Ok(Pattern::literal(value)),
            ReadablePatternKind::Set(items) => {
                let mut lowered = Vec::with_capacity(items.len());
                for item in items {
                    item.lower_into(&mut lowered)?;
                }
                Ok(Pattern::set(lowered))
            }
            ReadablePatternKind::Sequence(patterns) => patterns
                .iter()
                .map(ReadablePattern::lower_pattern)
                .collect::<Result<Vec<_>, _>>()
                .map(Pattern::sequence),
            ReadablePatternKind::Either(patterns) => patterns
                .iter()
                .map(ReadablePattern::lower_pattern)
                .collect::<Result<Vec<_>, _>>()
                .map(Pattern::either),
            ReadablePatternKind::ZeroOrMore(pattern) => {
                Ok(Pattern::zero_or_more(pattern.lower_pattern()?))
            }
            ReadablePatternKind::OneOrMore(pattern) => {
                Ok(Pattern::one_or_more(pattern.lower_pattern()?))
            }
            ReadablePatternKind::Optional(pattern) => Ok(Pattern::optional(pattern.lower_pattern()?)),
            ReadablePatternKind::Repeat { pattern, count } => {
                Ok(Pattern::repeat_exactly(pattern.lower_pattern()?, *count))
            }
            ReadablePatternKind::RepeatBetween { pattern, min, max } => {
                Pattern::repeat_between(pattern.lower_pattern()?, *min, *max).map_err(|_| {
                    ReadableParseError::validation(
                        self.span,
                        format!("invalid repeat bounds: min {min} is greater than max {max}"),
                        "Use bounds where the lower number is not greater than the upper number.",
                    )
                })
            }
            ReadablePatternKind::StartText => Ok(Pattern::start_text()),
            ReadablePatternKind::EndText => Ok(Pattern::end_text()),
            ReadablePatternKind::Capture(pattern) => Ok(Pattern::capture(pattern.lower_pattern()?)),
            ReadablePatternKind::NamedCapture { name, pattern } => {
                Pattern::named_capture(name, pattern.lower_pattern()?).map_err(|error| {
                    let message = match error {
                        Error::InvalidCaptureName(_) => format!(
                            "invalid capture name {name:?}: capture names must start with a letter or underscore and contain only letters, digits, or underscore"
                        ),
                        other => format!("invalid named capture: {other:?}"),
                    };
                    ReadableParseError::validation(
                        self.span,
                        message,
                        "Use a capture name that starts with a letter or underscore and contains only letters, digits, or underscore.",
                    )
                })
            }
        }
    }

    fn rust_builder_code(&self) -> String {
        match &self.kind {
            ReadablePatternKind::Literal(value) => {
                format!("::rx::literal({})", rust_string_literal(value))
            }
            ReadablePatternKind::Set(items) => format!(
                "::rx::set([{}])",
                items
                    .iter()
                    .flat_map(ReadableSetItem::rust_builder_code)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ReadablePatternKind::Sequence(patterns) => format!(
                "::rx::sequence([{}])",
                patterns
                    .iter()
                    .map(ReadablePattern::rust_builder_code)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ReadablePatternKind::Either(patterns) => format!(
                "::rx::either([{}])",
                patterns
                    .iter()
                    .map(ReadablePattern::rust_builder_code)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ReadablePatternKind::ZeroOrMore(pattern) => {
                format!("::rx::zero_or_more({})", pattern.rust_builder_code())
            }
            ReadablePatternKind::OneOrMore(pattern) => {
                format!("::rx::one_or_more({})", pattern.rust_builder_code())
            }
            ReadablePatternKind::Optional(pattern) => {
                format!("::rx::optional({})", pattern.rust_builder_code())
            }
            ReadablePatternKind::Repeat { pattern, count } => {
                format!("::rx::repeat({}, {count})", pattern.rust_builder_code())
            }
            ReadablePatternKind::RepeatBetween { pattern, min, max } => format!(
                "::rx::repeat_between({}, {min}, {max}).expect(\"rx macro validated repeat bounds\")",
                pattern.rust_builder_code()
            ),
            ReadablePatternKind::StartText => "::rx::start_text()".to_string(),
            ReadablePatternKind::EndText => "::rx::end_text()".to_string(),
            ReadablePatternKind::Capture(pattern) => {
                format!("::rx::capture({})", pattern.rust_builder_code())
            }
            ReadablePatternKind::NamedCapture { name, pattern } => format!(
                "::rx::named_capture({}, {}).expect(\"rx macro validated capture name\")",
                rust_string_literal(name),
                pattern.rust_builder_code()
            ),
        }
    }
}

impl ReadableSetItem {
    fn lower_into(&self, output: &mut Vec<CharSetItem>) -> Result<(), ReadableParseError> {
        match &self.kind {
            ReadableSetItemKind::Char(ch) => output.push(CharSetItem::literal(*ch)),
            ReadableSetItemKind::Chars(value) => {
                output.extend(value.chars().map(CharSetItem::literal));
            }
            ReadableSetItemKind::Range { start, end } => {
                let item = CharSetItem::range(*start, *end).map_err(|_| {
                    ReadableParseError::validation(
                        self.span,
                        format!("range start '{start}' is greater than end '{end}'"),
                        "Put the lower character first or escape the hyphen.",
                    )
                })?;
                output.push(item);
            }
            ReadableSetItemKind::Ascii(class) => output.push(CharSetItem::ascii(*class)),
        }
        Ok(())
    }

    fn rust_builder_code(&self) -> Vec<String> {
        match &self.kind {
            ReadableSetItemKind::Char(ch) => {
                vec![format!("::rx::char({})", rust_char_literal(*ch))]
            }
            ReadableSetItemKind::Chars(value) => value
                .chars()
                .map(|ch| format!("::rx::char({})", rust_char_literal(ch)))
                .collect(),
            ReadableSetItemKind::Range { start, end } => vec![format!(
                "::rx::range({}, {}).expect(\"rx macro validated range\")",
                rust_char_literal(*start),
                rust_char_literal(*end)
            )],
            ReadableSetItemKind::Ascii(class) => vec![class.rust_builder_path().to_string()],
        }
    }
}

fn rust_string_literal(value: &str) -> String {
    format!("{value:?}")
}

fn rust_char_literal(value: char) -> String {
    format!("{value:?}")
}

struct ReadableParser<'a> {
    input: &'a str,
    chars: Vec<(usize, char)>,
    pos: usize,
}

impl<'a> ReadableParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().collect(),
            pos: 0,
        }
    }

    fn parse(mut self) -> Result<ReadablePattern, ReadableParseError> {
        self.skip_pattern_definition()?;
        let pattern = self.parse_pattern()?;
        self.skip_ws();
        if self.peek().is_some() {
            return Err(self.error("unexpected trailing readable rx input"));
        }
        Ok(pattern)
    }

    fn parse_file(mut self) -> Result<Vec<ReadablePatternDefinition>, ReadableParseError> {
        let mut patterns = Vec::new();
        loop {
            self.skip_ws();
            if self.peek().is_none() {
                break;
            }
            self.expect_word("pattern")?;
            self.skip_ws();
            let name_start = self.current_offset();
            let name = self.expect_identifier()?;
            let name_span = SourceSpan {
                start: name_start,
                end: self.current_offset(),
            };
            self.skip_ws();
            self.expect_char('=')?;
            let pattern = self.parse_pattern()?;
            patterns.push(ReadablePatternDefinition {
                name,
                name_span,
                pattern,
            });
        }
        if patterns.is_empty() {
            return Err(self.error("readable rx file must define at least one pattern"));
        }
        Ok(patterns)
    }

    fn skip_pattern_definition(&mut self) -> Result<(), ReadableParseError> {
        self.skip_ws();
        if !self.starts_with_word("pattern") {
            return Ok(());
        }
        self.expect_word("pattern")?;
        self.skip_ws();
        self.expect_identifier()?;
        self.skip_ws();
        self.expect_char('=')?;
        Ok(())
    }

    fn parse_pattern(&mut self) -> Result<ReadablePattern, ReadableParseError> {
        self.skip_ws();
        let start = self.current_offset();
        if self.consume_word("start_text") {
            return Ok(self.pattern(start, ReadablePatternKind::StartText));
        }
        if self.consume_word("end_text") {
            return Ok(self.pattern(start, ReadablePatternKind::EndText));
        }

        let name = self.expect_identifier()?;
        let kind = match name.as_str() {
            "literal" => {
                self.expect_char('(')?;
                let value = self.expect_string()?;
                self.expect_char(')')?;
                ReadablePatternKind::Literal(value)
            }
            "sequence" => ReadablePatternKind::Sequence(self.parse_pattern_args()?),
            "either" => ReadablePatternKind::Either(self.parse_pattern_args()?),
            "set" => ReadablePatternKind::Set(self.parse_set_args()?),
            "zero_or_more" => {
                ReadablePatternKind::ZeroOrMore(Box::new(self.parse_unary_pattern()?))
            }
            "one_or_more" => ReadablePatternKind::OneOrMore(Box::new(self.parse_unary_pattern()?)),
            "optional" => ReadablePatternKind::Optional(Box::new(self.parse_unary_pattern()?)),
            "repeat" => {
                self.expect_char('(')?;
                let pattern = self.parse_pattern()?;
                self.expect_char(',')?;
                let count = self.expect_usize()?;
                self.expect_char(')')?;
                ReadablePatternKind::Repeat {
                    pattern: Box::new(pattern),
                    count,
                }
            }
            "repeat_between" => {
                self.expect_char('(')?;
                let pattern = self.parse_pattern()?;
                self.expect_char(',')?;
                let min = self.expect_usize()?;
                self.expect_char(',')?;
                let max = self.expect_usize()?;
                self.expect_char(')')?;
                ReadablePatternKind::RepeatBetween {
                    pattern: Box::new(pattern),
                    min,
                    max,
                }
            }
            "capture" => ReadablePatternKind::Capture(Box::new(self.parse_unary_pattern()?)),
            "named_capture" => {
                self.expect_char('(')?;
                let name = self.expect_string()?;
                self.expect_char(',')?;
                let pattern = self.parse_pattern()?;
                self.expect_char(')')?;
                ReadablePatternKind::NamedCapture {
                    name,
                    pattern: Box::new(pattern),
                }
            }
            _ => {
                return Err(self.compatibility_error(
                    &format!(
                        "unknown readable rx pattern `{name}`: unsupported construct `{name}` in MVP readable rx syntax"
                    ),
                    "Use an MVP readable rx construct or keep this regex outside automatic conversion.",
                ));
            }
        };
        Ok(self.pattern(start, kind))
    }

    fn parse_unary_pattern(&mut self) -> Result<ReadablePattern, ReadableParseError> {
        self.expect_char('(')?;
        let pattern = self.parse_pattern()?;
        self.expect_char(')')?;
        Ok(pattern)
    }

    fn parse_pattern_args(&mut self) -> Result<Vec<ReadablePattern>, ReadableParseError> {
        self.expect_char('(')?;
        let mut patterns = Vec::new();
        loop {
            self.skip_ws();
            if self.consume_char(')') {
                break;
            }
            patterns.push(self.parse_pattern()?);
            self.skip_ws();
            if self.consume_char(')') {
                break;
            }
            self.expect_char(',')?;
        }
        Ok(patterns)
    }

    fn parse_set_args(&mut self) -> Result<Vec<ReadableSetItem>, ReadableParseError> {
        self.expect_char('(')?;
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.consume_char(')') {
                break;
            }
            items.push(self.parse_set_item()?);
            self.skip_ws();
            if self.consume_char(')') {
                break;
            }
            self.expect_char(',')?;
        }
        Ok(items)
    }

    fn parse_set_item(&mut self) -> Result<ReadableSetItem, ReadableParseError> {
        self.skip_ws();
        let start = self.current_offset();
        if let Some(class) = self.consume_ascii_class() {
            return Ok(self.set_item(start, ReadableSetItemKind::Ascii(class)));
        }

        let name = self.expect_identifier()?;
        let kind = match name.as_str() {
            "char" => {
                self.expect_char('(')?;
                let value = self.expect_single_char_string("char")?;
                self.expect_char(')')?;
                ReadableSetItemKind::Char(value)
            }
            "chars" => {
                self.expect_char('(')?;
                let value = self.expect_string()?;
                self.expect_char(')')?;
                ReadableSetItemKind::Chars(value)
            }
            "range" => {
                self.expect_char('(')?;
                let start = self.expect_single_char_string("range start")?;
                self.expect_char(',')?;
                let end = self.expect_single_char_string("range end")?;
                self.expect_char(')')?;
                ReadableSetItemKind::Range { start, end }
            }
            _ => {
                return Err(self.compatibility_error(
                    &format!("unsupported set item `{name}` in MVP readable rx syntax"),
                    "Use char, chars, range, or an explicit ascii class.",
                ));
            }
        };
        Ok(self.set_item(start, kind))
    }

    fn consume_ascii_class(&mut self) -> Option<AsciiClass> {
        let saved = self.pos;
        self.skip_ws();
        let start = self.current_offset();
        if !self.input[start..].starts_with("ascii") {
            return None;
        }
        while self.current_offset() < start + "ascii".len() {
            self.bump();
        }
        self.skip_ws();
        let separated = self.consume_char('.') || self.consume_double_colon();
        if !separated {
            self.pos = saved;
            return None;
        }
        let name = self.expect_identifier().ok()?;
        if let Some(class) = AsciiClass::from_readable_name(&name) {
            return Some(class);
        }
        self.pos = saved;
        None
    }

    fn consume_double_colon(&mut self) -> bool {
        self.skip_ws();
        let saved = self.pos;
        if self.consume_char(':') && self.consume_char(':') {
            true
        } else {
            self.pos = saved;
            false
        }
    }

    fn expect_single_char_string(&mut self, context: &str) -> Result<char, ReadableParseError> {
        let value = self.expect_string()?;
        let mut chars = value.chars();
        let Some(ch) = chars.next() else {
            return Err(self.error(&format!("{context} must contain one character")));
        };
        if chars.next().is_some() {
            return Err(self.error(&format!("{context} must contain one character")));
        }
        Ok(ch)
    }

    fn expect_string(&mut self) -> Result<String, ReadableParseError> {
        self.skip_ws();
        self.expect_char('"')?;
        let mut value = String::new();
        while let Some((_, ch)) = self.bump() {
            match ch {
                '"' => return Ok(value),
                '\\' => {
                    let Some((_, escaped)) = self.bump() else {
                        return Err(self.error("unterminated string escape"));
                    };
                    value.push(match escaped {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\\' => '\\',
                        '"' => '"',
                        other => other,
                    });
                }
                ch => value.push(ch),
            }
        }
        Err(self.error("unterminated string"))
    }

    fn expect_usize(&mut self) -> Result<usize, ReadableParseError> {
        self.skip_ws();
        let mut value = String::new();
        while let Some((_, ch)) = self.peek() {
            if ch.is_ascii_digit() {
                value.push(ch);
                self.bump();
            } else {
                break;
            }
        }
        if value.is_empty() {
            return Err(self.error("expected number"));
        }
        value
            .parse()
            .map_err(|_| self.error("number is too large for this platform"))
    }

    fn expect_identifier(&mut self) -> Result<String, ReadableParseError> {
        self.skip_ws();
        let mut value = String::new();
        while let Some((_, ch)) = self.peek() {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.') {
                value.push(ch);
                self.bump();
            } else {
                break;
            }
        }
        if value.is_empty() {
            Err(self.error("expected identifier"))
        } else {
            Ok(value)
        }
    }

    fn expect_word(&mut self, expected: &str) -> Result<(), ReadableParseError> {
        if self.consume_word(expected) {
            Ok(())
        } else {
            Err(self.error(&format!("expected `{expected}`")))
        }
    }

    fn consume_word(&mut self, expected: &str) -> bool {
        self.skip_ws();
        if !self.input[self.current_offset()..].starts_with(expected) {
            return false;
        }
        let end = self.current_offset() + expected.len();
        if self
            .input
            .get(end..)
            .and_then(|rest| rest.chars().next())
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.'))
        {
            return false;
        }
        while self.current_offset() < end {
            self.bump();
        }
        true
    }

    fn starts_with_word(&self, expected: &str) -> bool {
        let offset = self
            .chars
            .iter()
            .skip(self.pos)
            .find(|(_, ch)| !ch.is_whitespace())
            .map(|(offset, _)| *offset)
            .unwrap_or(self.input.len());
        self.input[offset..].starts_with(expected)
    }

    fn expect_char(&mut self, expected: char) -> Result<(), ReadableParseError> {
        self.skip_ws();
        if self.consume_char(expected) {
            Ok(())
        } else {
            Err(self.error(&format!("expected `{expected}`")))
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        self.skip_ws();
        if matches!(self.peek(), Some((_, ch)) if ch == expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some((_, ch)) if ch.is_whitespace()) {
            self.bump();
        }
    }

    fn bump(&mut self) -> Option<(usize, char)> {
        let current = self.peek();
        if current.is_some() {
            self.pos += 1;
        }
        current
    }

    fn peek(&self) -> Option<(usize, char)> {
        self.chars.get(self.pos).copied()
    }

    fn current_offset(&self) -> usize {
        self.peek()
            .map(|(offset, _)| offset)
            .unwrap_or(self.input.len())
    }

    fn pattern(&self, start: usize, kind: ReadablePatternKind) -> ReadablePattern {
        ReadablePattern {
            span: SourceSpan {
                start,
                end: self.current_offset(),
            },
            kind,
        }
    }

    fn set_item(&self, start: usize, kind: ReadableSetItemKind) -> ReadableSetItem {
        ReadableSetItem {
            span: SourceSpan {
                start,
                end: self.current_offset(),
            },
            kind,
        }
    }

    fn error(&self, message: &str) -> ReadableParseError {
        ReadableParseError::syntax(
            SourceSpan {
                start: self.current_offset(),
                end: self.current_offset(),
            },
            message,
            "Fix the readable rx syntax at this location.",
        )
    }

    fn compatibility_error(&self, message: &str, suggestion: &str) -> ReadableParseError {
        ReadableParseError {
            span: SourceSpan {
                start: self.current_offset(),
                end: self.current_offset(),
            },
            category: DiagnosticCategory::Compatibility,
            message: message.to_string(),
            suggestion: Some(suggestion.to_string()),
        }
    }
}

impl ReadableParseError {
    pub fn syntax(span: SourceSpan, message: impl Into<String>, suggestion: &str) -> Self {
        Self {
            span,
            category: DiagnosticCategory::Syntax,
            message: message.into(),
            suggestion: Some(suggestion.to_string()),
        }
    }

    pub fn validation(span: SourceSpan, message: impl Into<String>, suggestion: &str) -> Self {
        Self {
            span,
            category: DiagnosticCategory::Validation,
            message: message.into(),
            suggestion: Some(suggestion.to_string()),
        }
    }
}

impl ToDiagnostic for ReadableParseError {
    fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic {
            span: self.span,
            category: self.category,
            severity: DiagnosticSeverity::Error,
            message: self.message.clone(),
            suggestion: self.suggestion.clone(),
            source_family: DiagnosticSourceFamily::ReadableRx,
        }
    }
}
