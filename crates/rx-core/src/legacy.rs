use crate::charset::{AsciiClass, CharSetItem};
use crate::diagnostic::{
    LegacyCharacterClass, LintDiagnostic, LintDiagnosticKind, ParseError, ParseErrorKind,
    ReplacementSuggestion, SourceSpan, UnsupportedFeature,
};
use crate::pattern::{Pattern, PatternKind};
use crate::pretty::write_rust_builder;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyRegexAnalysis {
    pub parse_result: Result<Pattern, ParseError>,
    pub lint_diagnostics: Vec<LintDiagnostic>,
    pub unsupported_diagnostics: Vec<ParseError>,
    pub replacement: Option<ReplacementSuggestion>,
}

struct LegacyFactModel {
    parse_result: Result<Pattern, ParseError>,
    lint_diagnostics: Vec<LintDiagnostic>,
    unsupported_diagnostics: Vec<ParseError>,
}

/// Analyze legacy regex syntax once for parsing, linting, compatibility, and migration.
pub fn analyze_legacy_regex(input: &str) -> LegacyRegexAnalysis {
    let facts = LegacyParser::new(input).analyze();
    let replacement = facts
        .parse_result
        .as_ref()
        .ok()
        .map(|pattern| ReplacementSuggestion {
            builder: write_rust_builder(pattern),
            macro_form: pattern.to_rx(),
            generated_regex: pattern.to_regex(),
        });

    LegacyRegexAnalysis {
        parse_result: facts.parse_result,
        lint_diagnostics: facts.lint_diagnostics,
        unsupported_diagnostics: facts.unsupported_diagnostics,
        replacement,
    }
}

/// Parse a focused MVP subset of legacy regex syntax into the canonical AST.
pub fn parse_legacy_regex(input: &str) -> Result<Pattern, ParseError> {
    analyze_legacy_regex(input).parse_result
}

pub fn lint_legacy_regex(input: &str) -> Vec<LintDiagnostic> {
    analyze_legacy_regex(input).lint_diagnostics
}

struct LegacyParser<'a> {
    input: &'a str,
    pos: usize,
    peeked: Option<(usize, char)>,
    lint_diagnostics: Vec<LintDiagnostic>,
    unsupported_diagnostics: Vec<ParseError>,
}

impl<'a> LegacyParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            peeked: None,
            lint_diagnostics: Vec::new(),
            unsupported_diagnostics: Vec::new(),
        }
    }

    fn analyze(mut self) -> LegacyFactModel {
        let parse_result = self.parse_alternation(None);
        if parse_result.is_err() {
            self.pos = 0;
            self.peeked = None;
            self.collect_recoverable_facts();
        }
        LegacyFactModel {
            parse_result,
            lint_diagnostics: self.lint_diagnostics,
            unsupported_diagnostics: self.unsupported_diagnostics,
        }
    }

    fn collect_recoverable_facts(&mut self) {
        while let Some((start, ch)) = self.bump() {
            match ch {
                '[' => self.collect_class_facts(),
                '(' if self.consume('?') => self.collect_group_extension_facts(start),
                '\\' => {
                    self.collect_escape_fact(start, false);
                }
                _ => {}
            }
        }
    }

    fn collect_group_extension_facts(&mut self, start: usize) {
        match self.bump() {
            Some((_, '<')) if matches!(self.peek(), Some((_, '=' | '!'))) => {
                self.bump();
                let _ = self.unsupported_at(start, 3, UnsupportedFeature::LookBehind);
            }
            Some((_, '<')) => {
                let name_start = self.current_offset();
                while let Some((end, ch)) = self.bump() {
                    if ch == '>' {
                        let name = self.input[name_start..end].to_string();
                        if !crate::diagnostic::is_valid_capture_name(&name) {
                            self.push_invalid_capture_name_lint(name_start, &name);
                        }
                        return;
                    }
                }
            }
            Some((_, '=' | '!')) => {
                let _ = self.unsupported_at(start, 2, UnsupportedFeature::LookAhead);
            }
            Some((_, ':')) => {
                let _ = self.unsupported_at(start, 2, UnsupportedFeature::NonCapturingGroup);
            }
            Some((_, 'R')) => {
                let _ = self.unsupported_at(start, 2, UnsupportedFeature::RecursivePattern);
            }
            Some((_, '(')) => {
                let _ = self.unsupported_at(start, 3, UnsupportedFeature::Conditional);
            }
            Some((_, _)) | None => {
                let _ = self.unsupported_at(start, 1, UnsupportedFeature::EngineSpecificEscape);
            }
        }
    }

    fn collect_class_facts(&mut self) {
        let mut previous_literal: Option<(usize, char)> = None;
        while let Some((start, ch)) = self.bump() {
            match ch {
                ']' => return,
                '\\' => {
                    previous_literal = self
                        .collect_escape_fact(start, true)
                        .map(|literal| (start, literal));
                }
                '-' => {
                    if let Some((range_start, left)) = previous_literal {
                        if let Some((_, right)) = self.peek() {
                            if right != ']' {
                                let right = if right == '\\' {
                                    let (escape_start, _) =
                                        self.bump().expect("peeked escape exists");
                                    self.collect_escape_fact(escape_start, true)
                                } else {
                                    self.bump();
                                    Some(right)
                                };
                                if let Some(right) = right {
                                    if left > right {
                                        self.push_invalid_range_lint(
                                            range_start,
                                            self.current_offset(),
                                            left,
                                            right,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    previous_literal = Some((start, '-'));
                }
                _ => previous_literal = Some((start, ch)),
            }
        }
    }

    fn collect_escape_fact(&mut self, start: usize, in_class: bool) -> Option<char> {
        let (_, escaped) = self.bump()?;
        match escaped {
            'w' => {
                self.push_ambiguous_class_lint(start, escaped, LegacyCharacterClass::Word);
                None
            }
            '0'..='9' => {
                let _ = self.unsupported_at(
                    start,
                    1 + escaped.len_utf8(),
                    UnsupportedFeature::Backreference,
                );
                None
            }
            '.' if in_class => {
                self.push_unnecessary_escape_lint(start, escaped);
                Some('.')
            }
            ch => Some(ch),
        }
    }

    fn parse_alternation(&mut self, terminator: Option<char>) -> Result<Pattern, ParseError> {
        let mut alternatives = vec![self.parse_sequence(terminator)?];
        while self.consume('|') {
            alternatives.push(self.parse_sequence(terminator)?);
        }
        Ok(if alternatives.len() == 1 {
            alternatives.pop().expect("length checked")
        } else {
            Pattern::either(alternatives)
        })
    }

    fn parse_sequence(&mut self, terminator: Option<char>) -> Result<Pattern, ParseError> {
        let mut patterns = Vec::new();
        while let Some((_, ch)) = self.peek() {
            if Some(ch) == terminator || ch == '|' {
                break;
            }
            let atom = self.parse_atom()?;
            push_sequence_pattern(&mut patterns, self.parse_repeat(atom)?);
        }
        Ok(if patterns.len() == 1 {
            patterns.pop().expect("length checked")
        } else {
            Pattern::sequence(patterns)
        })
    }

    fn parse_atom(&mut self) -> Result<Pattern, ParseError> {
        let (start, ch) = self.bump().ok_or_else(|| self.unexpected_end())?;
        match ch {
            '(' => self.parse_group(start),
            '[' => self.parse_class(start),
            '\\' => self.parse_escape(start),
            '^' => Ok(Pattern::start_text()),
            '$' => Ok(Pattern::end_text()),
            '.' => Err(self.unsupported_at(start, 1, UnsupportedFeature::Wildcard)),
            '|' => Err(self.error(
                SourceSpan {
                    start,
                    end: start + ch.len_utf8(),
                },
                ParseErrorKind::UnexpectedToken(ch),
                "unexpected legacy regex alternation token".to_string(),
                "Put a supported pattern on both sides of `|`.".to_string(),
            )),
            ')' | ']' => Err(self.error(
                SourceSpan {
                    start,
                    end: start + ch.len_utf8(),
                },
                ParseErrorKind::UnexpectedToken(ch),
                format!("unexpected legacy regex token `{ch}`"),
                "Remove the token or use a supported MVP construct.".to_string(),
            )),
            ch => Ok(Pattern::literal(ch.to_string())),
        }
    }

    fn parse_group(&mut self, start: usize) -> Result<Pattern, ParseError> {
        if self.consume('?') {
            if self.consume('<') {
                if matches!(self.peek(), Some((_, '=' | '!'))) {
                    self.bump();
                    return Err(self.unsupported_at(start, 3, UnsupportedFeature::LookBehind));
                }
                let name_start = self.current_offset();
                let name = self.read_until('>').ok_or_else(|| self.unexpected_end())?;
                let name_is_valid = crate::diagnostic::is_valid_capture_name(&name);
                if !name_is_valid {
                    self.push_invalid_capture_name_lint(name_start, &name);
                }
                let pattern = self.parse_alternation(Some(')'))?;
                if !self.consume(')') {
                    return Err(self.error(
                        SourceSpan {
                            start,
                            end: self.input.len(),
                        },
                        ParseErrorKind::UnclosedGroup,
                        "capture group is missing `)`".to_string(),
                        "Close the capture group with `)`.".to_string(),
                    ));
                }
                if !name_is_valid {
                    return Err(self.error(
                        SourceSpan {
                            start: name_start,
                            end: name_start + name.len(),
                        },
                        ParseErrorKind::UnsupportedFeature(
                            UnsupportedFeature::EngineSpecificEscape,
                        ),
                        "invalid capture name".to_string(),
                        "Use a capture name that starts with a letter or underscore.".to_string(),
                    ));
                }
                return Pattern::named_capture(name, pattern).map_err(|_| {
                    unreachable!("capture name validity checked before construction")
                });
            }
            if self.consume('=') || self.consume('!') {
                return Err(self.unsupported_at(start, 2, UnsupportedFeature::LookAhead));
            }
            if self.consume(':') {
                return Err(self.unsupported_at(start, 2, UnsupportedFeature::NonCapturingGroup));
            }
            if self.consume('R') {
                return Err(self.unsupported_at(start, 2, UnsupportedFeature::RecursivePattern));
            }
            if self.consume('(') {
                return Err(self.unsupported_at(start, 3, UnsupportedFeature::Conditional));
            }
            return Err(self.unsupported_at(start, 1, UnsupportedFeature::EngineSpecificEscape));
        }
        let pattern = self.parse_alternation(Some(')'))?;
        if !self.consume(')') {
            return Err(self.error(
                SourceSpan {
                    start,
                    end: self.input.len(),
                },
                ParseErrorKind::UnclosedGroup,
                "capture group is missing `)`".to_string(),
                "Close the capture group with `)`.".to_string(),
            ));
        }
        Ok(Pattern::capture(pattern))
    }

    fn parse_class(&mut self, start: usize) -> Result<Pattern, ParseError> {
        if self.consume('^') {
            return Err(self.unsupported_at(start, 2, UnsupportedFeature::ClassNegation));
        }

        let mut items = Vec::new();
        while let Some((item_start, ch)) = self.bump() {
            if ch == ']' {
                return Ok(Pattern::set(items));
            }
            let item = if ch == '\\' {
                self.class_escape(item_start)?
            } else {
                CharSetItem::literal(ch)
            };
            if self.consume('-') {
                if matches!(self.peek(), Some((_, ']'))) {
                    items.push(item);
                    items.push(CharSetItem::literal('-'));
                    continue;
                }
                let Some((end_start, end_ch)) = self.bump() else {
                    items.push(item);
                    items.push(CharSetItem::literal('-'));
                    break;
                };
                let end_item = if end_ch == '\\' {
                    self.class_escape(end_start)?
                } else {
                    CharSetItem::literal(end_ch)
                };
                if let (CharSetItem::Literal(range_start), CharSetItem::Literal(range_end)) =
                    (item, end_item)
                {
                    let range = match CharSetItem::range(range_start, range_end) {
                        Ok(range) => range,
                        Err(_) => {
                            self.push_invalid_range_lint(
                                item_start,
                                self.current_offset(),
                                range_start,
                                range_end,
                            );
                            let error = self.error(
                                SourceSpan {
                                    start: item_start,
                                    end: self.current_offset(),
                                },
                                ParseErrorKind::InvalidRange {
                                    start: range_start,
                                    end: range_end,
                                },
                                "invalid character range".to_string(),
                                "Put the lower character first or escape the hyphen.".to_string(),
                            );
                            self.collect_class_facts();
                            return Err(error);
                        }
                    };
                    items.push(range);
                } else {
                    items.push(item);
                    items.push(CharSetItem::literal('-'));
                    items.push(end_item);
                }
            } else {
                items.push(item);
            }
        }

        Err(self.error(
            SourceSpan {
                start,
                end: self.input.len(),
            },
            ParseErrorKind::UnclosedClass,
            "character class is missing `]`".to_string(),
            "Close the character class with `]`.".to_string(),
        ))
    }

    fn parse_escape(&mut self, start: usize) -> Result<Pattern, ParseError> {
        let (_, escaped) = self.bump().ok_or_else(|| self.unexpected_end())?;
        match escaped {
            'd' => Ok(Pattern::set([CharSetItem::ascii(AsciiClass::Digit)])),
            'w' => {
                self.push_ambiguous_class_lint(start, escaped, LegacyCharacterClass::Word);
                Ok(Pattern::set([CharSetItem::ascii(AsciiClass::Word)]))
            }
            's' => Ok(Pattern::set([CharSetItem::ascii(AsciiClass::Whitespace)])),
            '0'..='9' => Err(self.unsupported_at(
                start,
                1 + escaped.len_utf8(),
                UnsupportedFeature::Backreference,
            )),
            ch => Ok(Pattern::literal(ch.to_string())),
        }
    }

    fn class_escape(&mut self, start: usize) -> Result<CharSetItem, ParseError> {
        let (_, escaped) = self.bump().ok_or_else(|| self.unexpected_end())?;
        match escaped {
            'd' => Ok(CharSetItem::ascii(AsciiClass::Digit)),
            'w' => {
                self.push_ambiguous_class_lint(start, escaped, LegacyCharacterClass::Word);
                Ok(CharSetItem::ascii(AsciiClass::Word))
            }
            's' => Ok(CharSetItem::ascii(AsciiClass::Whitespace)),
            '0'..='9' => Err(self.unsupported_at(
                start,
                1 + escaped.len_utf8(),
                UnsupportedFeature::Backreference,
            )),
            '.' => {
                self.push_unnecessary_escape_lint(start, escaped);
                Ok(CharSetItem::literal('.'))
            }
            ch => Ok(CharSetItem::literal(ch)),
        }
    }

    fn parse_repeat(&mut self, pattern: Pattern) -> Result<Pattern, ParseError> {
        match self.peek().map(|(_, ch)| ch) {
            Some('*') => {
                self.bump();
                Ok(Pattern::zero_or_more(pattern))
            }
            Some('+') => {
                self.bump();
                Ok(Pattern::one_or_more(pattern))
            }
            Some('?') => {
                self.bump();
                Ok(Pattern::optional(pattern))
            }
            Some('{') => self.parse_braced_repeat(pattern),
            _ => Ok(pattern),
        }
    }

    fn parse_braced_repeat(&mut self, pattern: Pattern) -> Result<Pattern, ParseError> {
        self.consume('{');
        let min = self.parse_number().ok_or_else(|| self.unexpected_end())?;
        let max = if self.consume(',') {
            self.parse_number().ok_or_else(|| self.unexpected_end())?
        } else {
            min
        };
        if !self.consume('}') {
            return Err(self.unexpected_end());
        }
        Pattern::repeat_between(pattern, min, max).map_err(|_| {
            self.error(
                SourceSpan {
                    start: self.current_offset(),
                    end: self.current_offset(),
                },
                ParseErrorKind::InvalidRepeatBounds { min, max },
                "repeat lower bound is greater than upper bound".to_string(),
                "Use bounds where the lower number is not greater than the upper number."
                    .to_string(),
            )
        })
    }

    fn parse_number(&mut self) -> Option<usize> {
        let start = self.current_offset();
        while matches!(self.peek(), Some((_, ch)) if ch.is_ascii_digit()) {
            self.bump();
        }
        if self.current_offset() == start {
            return None;
        }
        self.input[start..self.current_offset()].parse().ok()
    }

    fn read_until(&mut self, terminator: char) -> Option<String> {
        let start = self.current_offset();
        while let Some((offset, ch)) = self.bump() {
            if ch == terminator {
                return Some(self.input[start..offset].to_string());
            }
        }
        None
    }

    fn consume(&mut self, expected: char) -> bool {
        if matches!(self.peek(), Some((_, ch)) if ch == expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn bump(&mut self) -> Option<(usize, char)> {
        let current = self.peek()?;
        self.pos += current.1.len_utf8();
        self.peeked = None;
        Some(current)
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        if self.peeked.is_none() {
            self.peeked = self.input[self.pos..]
                .chars()
                .next()
                .map(|ch| (self.pos, ch));
        }
        self.peeked
    }

    fn current_offset(&self) -> usize {
        self.pos
    }

    fn unsupported_at(
        &mut self,
        start: usize,
        len: usize,
        feature: UnsupportedFeature,
    ) -> ParseError {
        let message = match feature {
            UnsupportedFeature::Backreference => "backreferences are outside the MVP safe core",
            _ => "legacy regex feature is outside the MVP safe core",
        };
        let error = self.error(
            SourceSpan {
                start,
                end: start + len,
            },
            ParseErrorKind::UnsupportedFeature(feature),
            message.to_string(),
            "Rewrite the pattern using MVP safe-core constructs or keep the original regex outside automatic conversion.".to_string(),
        );
        let lint = LintDiagnostic {
            span: error.span,
            kind: LintDiagnosticKind::UnsupportedFeature(feature),
            message: message.to_string(),
            suggestion: error.suggestion.clone(),
        };
        if !self
            .lint_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.span == lint.span && diagnostic.kind == lint.kind)
        {
            self.lint_diagnostics.push(lint);
        }
        if !self
            .unsupported_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.span == error.span && diagnostic.kind == error.kind)
        {
            self.unsupported_diagnostics.push(error.clone());
        }
        error
    }

    fn push_ambiguous_class_lint(
        &mut self,
        start: usize,
        escaped: char,
        class: LegacyCharacterClass,
    ) {
        self.lint_diagnostics.push(LintDiagnostic {
            span: SourceSpan {
                start,
                end: start + 1 + escaped.len_utf8(),
            },
            kind: LintDiagnosticKind::AmbiguousCharacterClass { class },
            message: format!("`\\{escaped}` has target-dependent ASCII or Unicode semantics"),
            suggestion: "Use an explicit ASCII or Unicode rx class before conversion.".to_string(),
        });
    }

    fn push_unnecessary_escape_lint(&mut self, start: usize, escaped: char) {
        self.lint_diagnostics.push(LintDiagnostic {
            span: SourceSpan {
                start,
                end: start + 1 + escaped.len_utf8(),
            },
            kind: LintDiagnosticKind::UnnecessaryEscape { escaped },
            message: format!("`\\{escaped}` does not need escaping inside a character class"),
            suggestion: format!("Use `{escaped}` inside the character class."),
        });
    }

    fn push_invalid_range_lint(&mut self, start: usize, end: usize, left: char, right: char) {
        self.lint_diagnostics.push(LintDiagnostic {
            span: SourceSpan { start, end },
            kind: LintDiagnosticKind::InvalidRange {
                start: left,
                end: right,
            },
            message: "character class range has invalid bounds".to_string(),
            suggestion: "Put the lower character first or escape the hyphen.".to_string(),
        });
    }

    fn push_invalid_capture_name_lint(&mut self, start: usize, name: &str) {
        self.lint_diagnostics.push(LintDiagnostic {
            span: SourceSpan {
                start,
                end: start + name.len(),
            },
            kind: LintDiagnosticKind::InvalidCaptureName(name.to_string()),
            message: format!("invalid capture name `{name}`"),
            suggestion: "Use a capture name beginning with a letter or underscore.".to_string(),
        });
    }

    fn unexpected_end(&self) -> ParseError {
        self.error(
            SourceSpan {
                start: self.input.len(),
                end: self.input.len(),
            },
            ParseErrorKind::UnexpectedEnd,
            "unexpected end of legacy regex".to_string(),
            "Complete the pattern.".to_string(),
        )
    }

    fn error(
        &self,
        span: SourceSpan,
        kind: ParseErrorKind,
        message: String,
        suggestion: String,
    ) -> ParseError {
        ParseError {
            span,
            kind,
            message,
            suggestion,
        }
    }
}

fn push_sequence_pattern(patterns: &mut Vec<Pattern>, pattern: Pattern) {
    match pattern.kind {
        PatternKind::Literal(value) => {
            if let Some(Pattern {
                kind: PatternKind::Literal(existing),
            }) = patterns.last_mut()
            {
                existing.push_str(&value);
            } else {
                patterns.push(Pattern {
                    kind: PatternKind::Literal(value),
                });
            }
        }
        kind => patterns.push(Pattern { kind }),
    }
}

#[cfg(test)]
mod tests {
    use super::analyze_legacy_regex;
    use crate::diagnostic::{
        LegacyCharacterClass, LintDiagnosticKind, ParseErrorKind, SourceSpan, UnsupportedFeature,
    };

    #[test]
    fn analysis_returns_parse_lints_and_replacement_from_one_contract() {
        let analysis = analyze_legacy_regex(r#"[\w\.]+"#);

        assert!(analysis.parse_result.is_ok());
        assert!(analysis.replacement.is_some());
        assert!(analysis.unsupported_diagnostics.is_empty());
        assert!(analysis.lint_diagnostics.iter().any(|diagnostic| {
            diagnostic.kind
                == LintDiagnosticKind::AmbiguousCharacterClass {
                    class: LegacyCharacterClass::Word,
                }
        }));
        assert!(analysis.lint_diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == LintDiagnosticKind::UnnecessaryEscape { escaped: '.' }
                && diagnostic.span == SourceSpan { start: 3, end: 5 }
        }));
    }

    #[test]
    fn analysis_keeps_unsupported_parse_and_lint_diagnostics_aligned() {
        let analysis = analyze_legacy_regex(r#"(\w+)\1"#);

        assert_eq!(
            analysis.parse_result.as_ref().unwrap_err().kind,
            ParseErrorKind::UnsupportedFeature(UnsupportedFeature::Backreference)
        );
        assert_eq!(analysis.unsupported_diagnostics.len(), 1);
        assert!(analysis.replacement.is_none());
        assert!(analysis.lint_diagnostics.iter().any(|diagnostic| {
            diagnostic.kind
                == LintDiagnosticKind::UnsupportedFeature(UnsupportedFeature::Backreference)
                && diagnostic.span == SourceSpan { start: 5, end: 7 }
        }));
    }
}
