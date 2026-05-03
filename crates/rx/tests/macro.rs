#[test]
fn pattern_macro_matches_builder_output() {
    let macro_pattern = rx::pattern! {
        one_or_more(
            set(
                ascii::alnum,
                chars("._/-")
            )
        )
    };
    let builder_pattern = rx::set([
        rx::ascii::alnum(),
        rx::char('.'),
        rx::char('_'),
        rx::char('/'),
        rx::char('-'),
    ])
    .one_or_more();

    assert_eq!(macro_pattern.to_regex(), builder_pattern.to_regex());
    assert_eq!(macro_pattern.to_regex(), "[A-Za-z0-9._/-]+");
}

#[test]
fn regex_macro_emits_static_regex_string() {
    const IDENTIFIER: &str = rx::regex! {
        sequence(
            start_text,
            set(ascii::alpha, char("_")),
            zero_or_more(set(ascii.alnum, char("_"))),
            end_text
        )
    };

    assert_eq!(IDENTIFIER, "^[A-Za-z_][A-Za-z0-9_]*$");
}

#[test]
fn macro_dsl_accepts_either_alternation() {
    let macro_pattern = rx::pattern! {
        sequence(
            start_text,
            either(literal("GET"), literal("POST")),
            literal(" /"),
            end_text
        )
    };

    assert_eq!(macro_pattern.to_regex(), r#"^(?:GET|POST) /$"#);
}

#[test]
fn macro_compile_fail_diagnostics_are_useful() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/macro_invalid_range.rs");
    tests.compile_fail("tests/ui/macro_invalid_capture_name.rs");
    tests.compile_fail("tests/ui/macro_unsupported_construct.rs");
    tests.compile_fail("tests/ui/macro_malformed_syntax.rs");
}
