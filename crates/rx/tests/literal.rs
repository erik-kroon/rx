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
