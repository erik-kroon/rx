use crate::diagnostic::{
    RustRegexConstructor, RustRegexSuggestion, SourceSpan, SuggestionDiagnostic,
    SuggestionDiagnosticKind,
};
use crate::legacy::analyze_legacy_regex;

/// Discover common Rust `regex` crate construction forms and suggest `rx` replacements.
pub fn suggest_rust_regex_replacements(source: &str) -> Vec<RustRegexSuggestion> {
    suggest_rust_regex_replacements_in_range(
        source,
        SourceSpan {
            start: 0,
            end: source.len(),
        },
    )
}

/// Discover replacement suggestions in a byte range, preserving absolute source spans.
pub fn suggest_rust_regex_replacements_in_range(
    source: &str,
    range: SourceSpan,
) -> Vec<RustRegexSuggestion> {
    if range.start > range.end || range.end > source.len() {
        return Vec::new();
    }

    let Some(slice) = source.get(range.start..range.end) else {
        return Vec::new();
    };

    let mut suggestions = Vec::new();
    for constructor in RUST_REGEX_CONSTRUCTORS {
        let mut offset = 0;
        while let Some(found) = slice[offset..].find(constructor.needle) {
            let start = range.start + offset + found;
            if !has_rust_path_boundary(source, start) {
                offset += found + constructor.needle.len();
                continue;
            }
            let argument_start = start + constructor.needle.len();
            let literal_start = skip_rust_whitespace(source, argument_start);
            if let Some((regex, literal_end)) = parse_string_literal(source, literal_start) {
                let analysis = analyze_legacy_regex(&regex);
                let diagnostics =
                    analysis_diagnostics(&analysis, regex_source_start(source, literal_start));
                suggestions.push(RustRegexSuggestion {
                    span: SourceSpan {
                        start,
                        end: literal_end,
                    },
                    literal_span: SourceSpan {
                        start: literal_start,
                        end: literal_end,
                    },
                    constructor: constructor.kind,
                    regex,
                    replacement: analysis.replacement,
                    diagnostics,
                });
                offset = literal_end.saturating_sub(range.start);
            } else {
                offset += found + constructor.needle.len();
            }
        }
    }

    suggestions.sort_by_key(|suggestion| suggestion.span.start);
    suggestions
}

#[derive(Clone, Copy)]
struct RustRegexConstructorPattern {
    needle: &'static str,
    kind: RustRegexConstructor,
}

const RUST_REGEX_CONSTRUCTORS: &[RustRegexConstructorPattern] = &[
    RustRegexConstructorPattern {
        needle: "Regex::new(",
        kind: RustRegexConstructor::RegexNew,
    },
    RustRegexConstructorPattern {
        needle: "regex::Regex::new(",
        kind: RustRegexConstructor::RegexNew,
    },
    RustRegexConstructorPattern {
        needle: "RegexBuilder::new(",
        kind: RustRegexConstructor::RegexBuilderNew,
    },
    RustRegexConstructorPattern {
        needle: "regex::RegexBuilder::new(",
        kind: RustRegexConstructor::RegexBuilderNew,
    },
    RustRegexConstructorPattern {
        needle: "bytes::Regex::new(",
        kind: RustRegexConstructor::BytesRegexNew,
    },
    RustRegexConstructorPattern {
        needle: "regex::bytes::Regex::new(",
        kind: RustRegexConstructor::BytesRegexNew,
    },
    RustRegexConstructorPattern {
        needle: "bytes::RegexBuilder::new(",
        kind: RustRegexConstructor::BytesRegexBuilderNew,
    },
    RustRegexConstructorPattern {
        needle: "regex::bytes::RegexBuilder::new(",
        kind: RustRegexConstructor::BytesRegexBuilderNew,
    },
];

fn analysis_diagnostics(
    analysis: &crate::legacy::LegacyRegexAnalysis,
    regex_source_start: usize,
) -> Vec<SuggestionDiagnostic> {
    let mut diagnostics = Vec::new();
    if let Err(error) = &analysis.parse_result {
        diagnostics.push(SuggestionDiagnostic {
            span: offset_span(error.span, regex_source_start),
            kind: SuggestionDiagnosticKind::UnsupportedRegex(error.kind.clone()),
            message: error.message.clone(),
            suggestion: error.suggestion.clone(),
        });
    }
    diagnostics.extend(
        analysis
            .lint_diagnostics
            .iter()
            .map(|diagnostic| SuggestionDiagnostic {
                span: offset_span(diagnostic.span, regex_source_start),
                kind: SuggestionDiagnosticKind::LegacyRegexLint(diagnostic.kind.clone()),
                message: diagnostic.message.clone(),
                suggestion: diagnostic.suggestion.clone(),
            }),
    );
    diagnostics
}

fn offset_span(span: SourceSpan, offset: usize) -> SourceSpan {
    SourceSpan {
        start: offset + span.start,
        end: offset + span.end,
    }
}

fn skip_rust_whitespace(source: &str, start: usize) -> usize {
    let Some(rest) = source.get(start..) else {
        return start;
    };
    rest.char_indices()
        .find_map(|(offset, ch)| (!ch.is_whitespace()).then_some(start + offset))
        .unwrap_or(source.len())
}

fn has_rust_path_boundary(source: &str, start: usize) -> bool {
    let Some(prefix) = source.get(..start) else {
        return false;
    };
    let Some(ch) = prefix.chars().next_back() else {
        return true;
    };
    !(ch == '_' || ch == ':' || ch.is_ascii_alphanumeric())
}

fn regex_source_start(source: &str, literal_start: usize) -> usize {
    if source.as_bytes().get(literal_start) == Some(&b'r') {
        let Some(rest) = source.get(literal_start + 1..) else {
            return literal_start + 1;
        };
        let hashes = rest.bytes().take_while(|byte| *byte == b'#').count();
        return literal_start + 1 + hashes + 1;
    }

    literal_start + 1
}

fn parse_string_literal(source: &str, start: usize) -> Option<(String, usize)> {
    let rest = source.get(start..)?;
    let mut chars = rest.char_indices();
    let (_, first) = chars.next()?;
    if first == 'r' {
        return parse_raw_string_literal(source, start);
    }
    if first != '"' {
        return None;
    }

    let mut value = String::new();
    let mut escaped = false;
    for (offset, ch) in chars {
        if escaped {
            value.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some((value, start + offset + ch.len_utf8())),
            ch => value.push(ch),
        }
    }
    None
}

fn parse_raw_string_literal(source: &str, start: usize) -> Option<(String, usize)> {
    let rest = source.get(start + 1..)?;
    let hashes = rest.bytes().take_while(|byte| *byte == b'#').count();
    if rest.as_bytes().get(hashes) != Some(&b'"') {
        return None;
    }

    let value_start = start + 1 + hashes + 1;
    let value = source.get(value_start..)?;
    let terminator = format!("\"{}", "#".repeat(hashes));
    let terminator_start = value.find(&terminator)?;
    Some((
        value[..terminator_start].to_string(),
        value_start + terminator_start + terminator.len(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_qualified_and_bytes_regex_constructors_in_source_order() {
        let source = r##"
let a = regex::bytes::Regex::new(r#"[A-Z]+"#).unwrap();
let b = regex::RegexBuilder::new( "[0-9]+" ).build().unwrap();
let c = bytes::RegexBuilder::new(r"\w+").build().unwrap();
"##;

        let suggestions = suggest_rust_regex_replacements(source);

        assert_eq!(
            suggestions
                .iter()
                .map(|suggestion| (suggestion.constructor, suggestion.regex.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (RustRegexConstructor::BytesRegexNew, "[A-Z]+"),
                (RustRegexConstructor::RegexBuilderNew, "[0-9]+"),
                (RustRegexConstructor::BytesRegexBuilderNew, r"\w+"),
            ]
        );
    }

    #[test]
    fn preserves_absolute_spans_for_range_discovery_with_raw_strings() {
        let source = r#"before
let id = regex::Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
after
"#;
        let range_start = source.find("regex::Regex::new").unwrap();
        let range = SourceSpan {
            start: range_start,
            end: source.find("after").unwrap(),
        };

        let suggestions = suggest_rust_regex_replacements_in_range(source, range);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].constructor, RustRegexConstructor::RegexNew);
        assert_eq!(suggestions[0].regex, "^[A-Za-z_][A-Za-z0-9_]*$");
        assert_eq!(
            source.get(suggestions[0].span.start..range_start + 17),
            Some("regex::Regex::new")
        );
        assert!(suggestions[0].literal_span.start > range.start);
    }
}
