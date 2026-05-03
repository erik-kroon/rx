#[test]
fn public_literal_emits_standard_regex() {
    let pattern = rx::literal("readable");

    assert_eq!(pattern.to_regex(), "readable");
}

#[test]
fn public_literal_output_is_compact_and_deterministic() {
    let pattern = rx::literal("a.b+(c)");

    assert_eq!(pattern.to_regex(), r#"a\.b\+\(c\)"#);
    assert_eq!(pattern.to_regex(), r#"a\.b\+\(c\)"#);
}

#[test]
fn public_regex_emission_can_reuse_existing_string_buffer() {
    let pattern = rx::sequence([
        rx::start_text(),
        rx::literal("a.b"),
        rx::one_or_more(rx::set([rx::ascii::digit()])),
        rx::end_text(),
    ]);
    let mut output = String::with_capacity(64);

    pattern.write_regex(&mut output);
    assert_eq!(output, r#"^a\.b[0-9]+$"#);

    output.clear();
    pattern
        .write_regex_for(rx::Dialect::RustRegex, &mut output)
        .unwrap();
    assert_eq!(output, r#"^a\.b[0-9]+$"#);
}

#[test]
fn public_set_emits_compact_deterministic_regex() {
    let pattern = rx::set([
        rx::ascii::digit(),
        rx::char('_'),
        rx::range('a', 'z').unwrap(),
    ]);

    assert_eq!(pattern.to_regex(), "[0-9_a-z]");
    assert_eq!(pattern.to_regex(), "[0-9_a-z]");
}

#[test]
fn public_set_builder_concentrates_common_character_workflows() {
    let pattern = rx::Set::new().ascii_word().chars("./-").one_or_more();

    assert_eq!(pattern.to_regex(), r#"[A-Za-z0-9_./-]+"#);
    assert_eq!(
        pattern.to_rx(),
        r#"one_or_more(
    set(
        ascii.word,
        chars("./-")
    )
)"#
    );
}

#[test]
fn public_chars_builder_adapts_into_set_pattern() {
    let pattern = rx::set(rx::chars("._/-"));

    assert_eq!(pattern.to_regex(), r#"[._/-]"#);
}

#[test]
fn public_set_deduplicates_exact_items() {
    let pattern = rx::set([rx::char('b'), rx::char('a'), rx::char('b')]);

    assert_eq!(pattern.to_regex(), "[ba]");
}

#[test]
fn public_set_escapes_class_metacharacters() {
    let pattern = rx::set([rx::char(']'), rx::char('-'), rx::char('^'), rx::char('\\')]);

    assert_eq!(pattern.to_regex(), r#"[\]\^\\-]"#);
}

#[test]
fn public_range_rejects_invalid_bounds() {
    assert_eq!(
        rx::range('z', 'a'),
        Err(rx::Error::InvalidRange {
            start: 'z',
            end: 'a'
        })
    );
}

#[test]
fn public_ascii_class_names_are_explicit() {
    assert_eq!(rx::set([rx::ascii::word()]).to_regex(), "[A-Za-z0-9_]");
    assert_eq!(rx::set([rx::ascii::alnum()]).to_regex(), "[A-Za-z0-9]");
    assert_eq!(rx::set([rx::ascii::alpha()]).to_regex(), "[A-Za-z]");
    assert_eq!(rx::set([rx::ascii::digit()]).to_regex(), "[0-9]");
    assert_eq!(
        rx::set([rx::ascii::whitespace()]).to_regex(),
        r#"[\t\n\f\r ]"#
    );
}

#[test]
fn public_unicode_class_names_are_explicitly_rejected() {
    assert_eq!(
        rx::unicode::word(),
        Err(rx::Error::UnsupportedUnicodeClass(rx::UnicodeClass::Word))
    );
    assert_eq!(
        rx::unicode::letter(),
        Err(rx::Error::UnsupportedUnicodeClass(rx::UnicodeClass::Letter))
    );
}

#[test]
fn public_identifier_style_example_emits_compact_regex() {
    let head = rx::set([rx::ascii::alpha(), rx::char('_')]);
    let tail = rx::set([rx::ascii::alnum(), rx::char('_')]).zero_or_more();
    let pattern = rx::sequence([rx::start_text(), head, tail, rx::end_text()]);

    assert_eq!(pattern.to_regex(), "^[A-Za-z_][A-Za-z0-9_]*$");
}

#[test]
fn public_ascii_namespace_owns_identifier_policy() {
    let pattern = rx::sequence([rx::start_text(), rx::ascii::identifier(), rx::end_text()]);

    assert_eq!(pattern.to_regex(), "^[A-Za-z_][A-Za-z0-9_]*$");
}

#[test]
fn public_either_emits_alternation_with_sequence_grouping() {
    let method = rx::either([rx::literal("GET"), rx::literal("POST")]);
    let pattern = rx::sequence([rx::start_text(), method, rx::literal(" /"), rx::end_text()]);

    assert_eq!(pattern.to_regex(), r#"^(?:GET|POST) /$"#);
    assert_eq!(
        pattern.to_rx(),
        r#"sequence(
    start_text,
    either(
        literal("GET"),
        literal("POST")
    ),
    literal(" /"),
    end_text
)"#
    );
    assert_eq!(
        pattern.explain(),
        "Match a sequence of 4 patterns:\nMatch the start of the text.\nMatch one of 2 alternatives:\nMatch the literal text \"GET\".\nMatch the literal text \"POST\".\nMatch the literal text \" /\".\nMatch the end of the text."
    );
}

#[test]
fn public_path_piece_style_example_emits_compact_regex() {
    let pattern = rx::set([
        rx::ascii::alnum(),
        rx::char('_'),
        rx::char('.'),
        rx::char('/'),
        rx::char('-'),
    ])
    .one_or_more();

    assert_eq!(pattern.to_regex(), r#"[A-Za-z0-9_./-]+"#);
}

#[test]
fn public_path_piece_style_example_pretty_prints_readable_rx() {
    let pattern = rx::set([
        rx::ascii::alnum(),
        rx::char('_'),
        rx::char('.'),
        rx::char('/'),
        rx::char('-'),
    ])
    .one_or_more();

    assert_eq!(
        pattern.to_rx(),
        r#"one_or_more(
    set(
        ascii.alnum,
        chars("_./-")
    )
)"#
    );
    assert_eq!(
        pattern.explain(),
        "Repeat the next pattern one or more times.\nMatch one character from the set: ASCII alphanumeric characters A-Z, a-z, and 0-9, \"_\", \".\", \"/\", \"-\"."
    );
}

#[test]
fn public_captures_and_named_captures_emit_standard_regex() {
    let scheme = rx::named_capture(
        "scheme",
        rx::sequence([rx::literal("http"), rx::optional(rx::literal("s"))]),
    )
    .unwrap();
    let host = rx::named_capture(
        "host",
        rx::set([rx::ascii::alnum(), rx::char('.'), rx::char('-')]).one_or_more(),
    )
    .unwrap();
    let pattern = rx::sequence([
        rx::start_text(),
        scheme,
        rx::literal("://"),
        host,
        rx::end_text(),
    ]);

    assert_eq!(
        pattern.to_regex(),
        r#"^(?<scheme>https?)://(?<host>[A-Za-z0-9.-]+)$"#
    );
}

#[test]
fn public_emit_uses_explicit_dialect_selection() {
    let pattern = rx::sequence([
        rx::start_text(),
        rx::named_capture("id", rx::set([rx::ascii::digit()]).one_or_more()).unwrap(),
        rx::end_text(),
    ]);

    assert_eq!(
        pattern.emit(rx::Dialect::RustRegex),
        Ok(r#"^(?<id>[0-9]+)$"#.to_string())
    );
    assert_eq!(pattern.to_regex(), r#"^(?<id>[0-9]+)$"#);
}

#[test]
fn public_emit_rejects_named_captures_for_posix_ere() {
    let pattern = rx::named_capture("id", rx::literal("x")).unwrap();

    assert_eq!(
        pattern.emit(rx::Dialect::PosixEre),
        Err(rx::Error::UnsupportedDialectFeature {
            dialect: rx::Dialect::PosixEre,
            feature: rx::Feature::NamedCapture,
        })
    );
}

#[test]
fn public_pretty_and_explanation_cover_literals_anchors_captures_and_repeats() {
    let pattern = rx::sequence([
        rx::start_text(),
        rx::named_capture(
            "id",
            rx::set([rx::ascii::alpha(), rx::char('_')]).one_or_more(),
        )
        .unwrap(),
        rx::literal("-"),
        rx::capture(rx::repeat(rx::set([rx::ascii::digit()]), 3)),
        rx::end_text(),
    ]);

    assert_eq!(
        pattern.to_rx(),
        r#"sequence(
    start_text,
    named_capture("id",
        one_or_more(
            set(
                ascii.alpha,
                char("_")
            )
        )
    ),
    literal("-"),
    capture(
        repeat(
            set(
                ascii.digit
            ),
            3
        )
    ),
    end_text
)"#
    );
    assert_eq!(
        pattern.explain(),
        "Match a sequence of 5 patterns:\nMatch the start of the text.\nCapture the next pattern as \"id\".\nRepeat the next pattern one or more times.\nMatch one character from the set: ASCII alphabetic characters A-Z and a-z, \"_\".\nMatch the literal text \"-\".\nCapture the next pattern.\nRepeat the next pattern exactly 3 times.\nMatch one character from the set: ASCII digits 0-9.\nMatch the end of the text."
    );
}

#[test]
fn public_repeat_shapes_emit_mvp_quantifiers() {
    let digit = rx::set([rx::ascii::digit()]);

    assert_eq!(rx::zero_or_more(digit.clone()).to_regex(), "[0-9]*");
    assert_eq!(rx::one_or_more(digit.clone()).to_regex(), "[0-9]+");
    assert_eq!(rx::optional(digit.clone()).to_regex(), "[0-9]?");
    assert_eq!(
        rx::repeat_between(digit.clone(), 2, 4).unwrap().to_regex(),
        "[0-9]{2,4}"
    );
    assert_eq!(rx::repeat(digit, 3).to_regex(), "[0-9]{3}");
}

#[test]
fn public_rejects_invalid_repeat_bounds_and_capture_names() {
    assert_eq!(
        rx::repeat_between(rx::literal("x"), 3, 2),
        Err(rx::Error::InvalidRepeatBounds { min: 3, max: 2 })
    );
    assert_eq!(
        rx::named_capture("123-id", rx::literal("x")),
        Err(rx::Error::InvalidCaptureName("123-id".to_string()))
    );
}

#[test]
fn public_legacy_parser_handles_prd_path_piece_example() {
    let pattern = rx::parse_legacy_regex(r#"[\w._/-]+"#).unwrap();

    assert_eq!(pattern.to_regex(), r#"[A-Za-z0-9_./-]+"#);
    assert_eq!(
        pattern.to_rx(),
        r#"one_or_more(
    set(
        ascii.word,
        chars("./-")
    )
)"#
    );
}

#[test]
fn public_legacy_parser_handles_literals_escapes_anchors_captures_and_repeats() {
    let pattern = rx::parse_legacy_regex(r#"^(?<id>[A-Za-z_]\w{2,4})-(\d+)\?$"#).unwrap();

    assert_eq!(
        pattern.to_regex(),
        r#"^(?<id>[A-Za-z_][A-Za-z0-9_]{2,4})-([0-9]+)\?$"#
    );
}

#[test]
fn public_legacy_parser_handles_alternation() {
    let pattern = rx::parse_legacy_regex(r#"^(GET|POST) /$"#).unwrap();

    assert_eq!(pattern.to_regex(), r#"^(GET|POST) /$"#);
    assert_eq!(
        pattern.to_rx(),
        r#"sequence(
    start_text,
    capture(
        either(
            literal("GET"),
            literal("POST")
        )
    ),
    literal(" /"),
    end_text
)"#
    );
}

#[test]
fn public_sample_input_checks_report_preserved_behavior() {
    let pattern = rx::parse_legacy_regex(r#"^[A-Za-z_]\w{2,4}$"#).unwrap();
    let report = rx::check_sample_inputs(
        r#"^[A-Za-z_]\w{2,4}$"#,
        &pattern,
        ["abc", "_id1", "9abc", "abcdef"],
    )
    .unwrap();

    assert_eq!(report.generated_regex, r#"^[A-Za-z_][A-Za-z0-9_]{2,4}$"#);
    assert!(report.is_preserved());
    assert_eq!(report.mismatches().count(), 0);
    assert_eq!(
        report.checks,
        vec![
            rx::SampleInputCheck {
                input: "abc".to_string(),
                legacy_matches: true,
                generated_matches: true,
            },
            rx::SampleInputCheck {
                input: "_id1".to_string(),
                legacy_matches: true,
                generated_matches: true,
            },
            rx::SampleInputCheck {
                input: "9abc".to_string(),
                legacy_matches: false,
                generated_matches: false,
            },
            rx::SampleInputCheck {
                input: "abcdef".to_string(),
                legacy_matches: false,
                generated_matches: false,
            },
        ]
    );
}

#[test]
fn public_sample_input_checks_report_mismatches_between_regex_strings() {
    let report =
        rx::check_generated_regex_sample_inputs(r#"^cat$"#, r#"^dog$"#, ["cat", "dog", "bird"])
            .unwrap();
    let mismatches = report
        .mismatches()
        .map(|check| check.input.as_str())
        .collect::<Vec<_>>();

    assert!(!report.is_preserved());
    assert_eq!(mismatches, ["cat", "dog"]);
}

#[test]
fn public_sample_input_checks_report_regex_compile_errors() {
    let error = rx::check_generated_regex_sample_inputs("[", "ok", ["sample"]).unwrap_err();

    assert_eq!(error.side, rx::SampleRegexSide::Legacy);
    assert_eq!(error.regex, "[");
    assert!(error.message.contains("unclosed character class"));
}

#[test]
fn public_legacy_parser_reports_unsupported_compatibility_constructs() {
    let error = rx::parse_legacy_regex(r#"(\w+)\1"#).unwrap_err();

    assert_eq!(
        error.kind,
        rx::ParseErrorKind::UnsupportedFeature(rx::UnsupportedFeature::Backreference)
    );
    assert_eq!(error.span, rx::SourceSpan { start: 5, end: 7 });
    assert!(error.message.contains("backreferences"));
    assert!(error.suggestion.contains("MVP safe-core"));
}

#[test]
fn public_legacy_parser_reports_actionable_structural_errors() {
    let error = rx::parse_legacy_regex("[z-a").unwrap_err();

    assert_eq!(
        error.kind,
        rx::ParseErrorKind::InvalidRange {
            start: 'z',
            end: 'a'
        }
    );
    assert_eq!(error.span, rx::SourceSpan { start: 1, end: 4 });
    assert!(error.suggestion.contains("lower character first"));
}

#[test]
fn public_legacy_linter_reports_structured_diagnostics() {
    let diagnostics = rx::lint_legacy_regex(r#"(?<123-id>[z-a\w\.])\1"#);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.kind
        == rx::LintDiagnosticKind::InvalidCaptureName("123-id".to_string())
        && diagnostic.span == rx::SourceSpan { start: 3, end: 9 }));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.kind
        == rx::LintDiagnosticKind::AmbiguousCharacterClass {
            class: rx::LegacyCharacterClass::Word
        }));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.kind
        == rx::LintDiagnosticKind::UnnecessaryEscape { escaped: '.' }
        && diagnostic.message.contains("does not need escaping")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.kind
        == rx::LintDiagnosticKind::InvalidRange {
            start: 'z',
            end: 'a'
        }));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.kind
        == rx::LintDiagnosticKind::UnsupportedFeature(rx::UnsupportedFeature::Backreference)));
}
