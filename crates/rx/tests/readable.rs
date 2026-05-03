#[test]
fn readable_parser_accepts_pattern_definition_wrappers_for_single_expression() {
    let pattern = rx::parse_readable_rx(
        r#"
            pattern identifier =
                sequence(
                    start_text,
                    set(ascii.alpha, char("_")),
                    zero_or_more(set(ascii.alnum, char("_"))),
                    end_text
                )
        "#,
    )
    .unwrap();

    assert_eq!(pattern.to_regex(), "^[A-Za-z_][A-Za-z0-9_]*$");
}

#[test]
fn readable_parser_accepts_dot_and_double_colon_ascii_namespaces() {
    let dot = rx::parse_readable_rx(r#"set(ascii.word, ascii.alnum)"#).unwrap();
    let double_colon = rx::parse_readable_rx(r#"set(ascii::word, ascii::alnum)"#).unwrap();

    assert_eq!(dot.to_regex(), "[A-Za-z0-9_]");
    assert_eq!(double_colon.to_regex(), "[A-Za-z0-9_]");
}

#[test]
fn readable_parser_lowers_string_escapes_in_literals_and_sets() {
    let pattern = rx::parse_readable_rx(
        r#"sequence(literal("line\n"), set(char("\""), char("\\"), chars("\t\r")))"#,
    )
    .unwrap();

    assert_eq!(pattern.to_regex(), "line\n[\"\\\\\t\r]");
    assert_eq!(
        pattern.to_rx(),
        "sequence(\n    literal(\"line\\n\"),\n    set(\n        chars(\"\\\"\\\\\\t\\r\")\n    )\n)"
    );
}

#[test]
fn readable_parser_preserves_file_definition_names_and_order() {
    let patterns = rx::parse_readable_rx_file(
        r#"
            pattern path_piece = one_or_more(set(ascii.alnum, chars("._/-")))
            pattern id = sequence(start_text, set(ascii.alpha, char("_")), end_text)
        "#,
    )
    .unwrap();

    let emitted = patterns
        .iter()
        .map(|(name, pattern)| (name.as_str(), pattern.to_regex()))
        .collect::<Vec<_>>();

    assert_eq!(
        emitted,
        vec![
            ("path_piece", "[A-Za-z0-9._/-]+".to_string()),
            ("id", "^[A-Za-z_]$".to_string()),
        ]
    );
}

#[test]
fn readable_file_syntax_exposes_source_spans_for_tools() {
    let definitions = rx_core::parse_readable_rx_file_artifacts(
        "pattern first = literal(\"a\")\npattern second = set(ascii.digit)\n",
    )
    .unwrap();

    assert_eq!(definitions.len(), 2);
    assert_eq!(definitions[0].name(), "first");
    assert_eq!(
        definitions[0].name_span(),
        rx::SourceSpan { start: 8, end: 13 }
    );
    assert_eq!(
        definitions[0].pattern_span(),
        rx::SourceSpan { start: 16, end: 28 }
    );
    assert_eq!(definitions[1].name(), "second");
    assert_eq!(
        definitions[1].name_span(),
        rx::SourceSpan { start: 37, end: 43 }
    );
    assert_eq!(
        definitions[1].pattern_span(),
        rx::SourceSpan { start: 46, end: 62 }
    );
}

#[test]
fn readable_artifact_exposes_lowered_pattern_span_and_builder_code() {
    let artifact = rx_core::parse_readable_rx_artifact(
        r#"named_capture("id", repeat_between(set(range("a", "z")), 2, 4))"#,
    )
    .unwrap();

    assert_eq!(artifact.pattern().to_regex(), "(?<id>[a-z]{2,4})");
    assert_eq!(artifact.span(), rx::SourceSpan { start: 0, end: 63 });
    assert_eq!(
        artifact.rust_builder_code(),
        "::rx::named_capture(\"id\", ::rx::repeat_between(::rx::set([::rx::range('a', 'z').expect(\"rx macro validated range\")]), 2, 4).expect(\"rx macro validated repeat bounds\")).expect(\"rx macro validated capture name\")"
    );
}

#[test]
fn readable_parser_accepts_either_alternation() {
    let artifact =
        rx_core::parse_readable_rx_artifact(r#"either(literal("cat"), literal("dog"))"#).unwrap();

    assert_eq!(artifact.pattern().to_regex(), "cat|dog");
    assert_eq!(
        artifact.rust_builder_code(),
        "::rx::either([::rx::literal(\"cat\"), ::rx::literal(\"dog\")])"
    );
}

#[test]
fn readable_parser_rejects_invalid_repeat_bounds_during_lowering() {
    let error = rx::parse_readable_rx(r#"repeat_between(literal("x"), 4, 2)"#).unwrap_err();

    assert_eq!(error.span, rx::SourceSpan { start: 0, end: 34 });
    assert!(error.message.contains("invalid repeat bounds"));
}

#[test]
fn readable_parser_rejects_invalid_named_capture_during_lowering() {
    let error = rx::parse_readable_rx(r#"named_capture("123", literal("x"))"#).unwrap_err();

    assert_eq!(error.span, rx::SourceSpan { start: 0, end: 34 });
    assert!(error.message.contains("invalid capture name"));
}

#[test]
fn readable_parser_reports_unknown_patterns_and_trailing_input() {
    let unknown = rx::parse_readable_rx("boundary_word").unwrap_err();
    assert!(
        unknown
            .message
            .contains("unsupported construct `boundary_word` in MVP readable rx syntax")
            || unknown
                .message
                .contains("unknown readable rx pattern `boundary_word`")
    );

    let trailing = rx::parse_readable_rx("literal(\"a\") literal(\"b\")").unwrap_err();
    assert!(trailing
        .message
        .contains("unexpected trailing readable rx input"));
}

#[test]
fn readable_parser_reports_set_item_arity_and_range_errors() {
    let char_error = rx::parse_readable_rx(r#"set(char("ab"))"#).unwrap_err();
    assert!(char_error
        .message
        .contains("char must contain one character"));

    let range_error = rx::parse_readable_rx(r#"set(range("z", "a"))"#).unwrap_err();
    assert!(range_error
        .message
        .contains("range start 'z' is greater than end 'a'"));
}

#[test]
fn readable_file_parser_rejects_empty_files_and_missing_pattern_keyword() {
    let empty = rx::parse_readable_rx_file(" \n\t ").unwrap_err();
    assert!(empty
        .message
        .contains("readable rx file must define at least one pattern"));

    let missing_keyword = rx::parse_readable_rx_file("literal(\"x\")").unwrap_err();
    assert!(missing_keyword.message.contains("expected `pattern`"));
}
