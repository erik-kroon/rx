use proptest::prelude::*;
use rx_core::{AsciiClass, CharSetItem, Pattern};

fn safe_literal_char() -> impl Strategy<Value = char> {
    prop_oneof![
        prop::sample::select(('a'..='z').collect::<Vec<_>>()),
        prop::sample::select(('A'..='Z').collect::<Vec<_>>()),
        prop::sample::select(('0'..='9').collect::<Vec<_>>()),
        Just('_'),
        Just('-'),
        Just('/'),
        Just(':'),
        Just(' '),
    ]
}

fn regex_meta_char() -> impl Strategy<Value = char> {
    prop_oneof![
        Just('\\'),
        Just('.'),
        Just('+'),
        Just('*'),
        Just('?'),
        Just('('),
        Just(')'),
        Just('|'),
        Just('['),
        Just(']'),
        Just('{'),
        Just('}'),
        Just('^'),
        Just('$'),
    ]
}

fn readable_string_literal() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            safe_literal_char().prop_map(|ch| ch.to_string()),
            Just("\\n".to_string()),
            Just("\\t".to_string()),
            Just("\\r".to_string()),
            Just("\\\\".to_string()),
            Just("\\\"".to_string()),
        ],
        0..8,
    )
    .prop_map(|parts| parts.concat())
}

fn literal_text() -> impl Strategy<Value = String> {
    prop::collection::vec(prop_oneof![safe_literal_char(), regex_meta_char()], 0..8)
        .prop_map(|chars| chars.into_iter().collect())
}

fn legacy_atom() -> impl Strategy<Value = String> {
    prop_oneof![
        prop::collection::vec(safe_literal_char(), 1..6)
            .prop_map(|chars| chars.into_iter().collect()),
        Just("\\d".to_string()),
        Just("\\w".to_string()),
        Just("\\s".to_string()),
        Just("[a-z]".to_string()),
        Just("[A-Z]".to_string()),
        Just("[0-9_]".to_string()),
        Just("[\\w\\.]".to_string()),
        Just("^".to_string()),
        Just("$".to_string()),
    ]
}

fn legacy_regex() -> impl Strategy<Value = String> {
    let quantified = (
        legacy_atom(),
        prop_oneof![Just(""), Just("*"), Just("+"), Just("?")],
    )
        .prop_map(|(atom, quantifier)| format!("{atom}{quantifier}"));
    prop::collection::vec(quantified, 1..8).prop_map(|parts| parts.concat())
}

fn readable_pattern() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        readable_string_literal().prop_map(|value| format!("literal(\"{value}\")")),
        Just("start_text".to_string()),
        Just("end_text".to_string()),
        Just("set(ascii.digit)".to_string()),
        Just("set(ascii.alnum, char(\"_\"))".to_string()),
        Just("set(range(\"a\", \"z\"), chars(\"_-/\"))".to_string()),
    ];

    leaf.prop_recursive(4, 32, 3, |inner| {
        prop_oneof![
            inner
                .clone()
                .prop_map(|pattern| format!("zero_or_more({pattern})")),
            inner
                .clone()
                .prop_map(|pattern| format!("one_or_more({pattern})")),
            inner
                .clone()
                .prop_map(|pattern| format!("optional({pattern})")),
            (inner.clone(), 0usize..5)
                .prop_map(|(pattern, count)| { format!("repeat({pattern}, {count})") }),
            (inner.clone(), 0usize..4, 4usize..8).prop_map(|(pattern, min, max)| {
                format!("repeat_between({pattern}, {min}, {max})")
            }),
            prop::collection::vec(inner.clone(), 2..4)
                .prop_map(|patterns| { format!("sequence({})", patterns.join(", ")) }),
            prop::collection::vec(inner.clone(), 2..4)
                .prop_map(|patterns| { format!("either({})", patterns.join(", ")) }),
            inner.prop_map(|pattern| format!("capture({pattern})")),
        ]
    })
}

fn core_pattern() -> impl Strategy<Value = Pattern> {
    let leaf = prop_oneof![
        literal_text().prop_map(Pattern::literal),
        Just(Pattern::set([CharSetItem::ascii(AsciiClass::Digit)])),
        Just(Pattern::set([
            CharSetItem::ascii(AsciiClass::Alpha),
            CharSetItem::literal('_'),
        ])),
        Just(Pattern::set([
            CharSetItem::range('a', 'z').expect("valid range"),
            CharSetItem::literal('-'),
            CharSetItem::literal(']'),
            CharSetItem::literal('\\'),
        ])),
        Just(Pattern::start_text()),
        Just(Pattern::end_text()),
    ];

    leaf.prop_recursive(4, 32, 3, |inner| {
        prop_oneof![
            inner.clone().prop_map(Pattern::zero_or_more),
            inner.clone().prop_map(Pattern::one_or_more),
            inner.clone().prop_map(Pattern::optional),
            (inner.clone(), 0usize..5)
                .prop_map(|(pattern, count)| { Pattern::repeat_exactly(pattern, count) }),
            (inner.clone(), 0usize..4, 4usize..8).prop_map(|(pattern, min, max)| {
                Pattern::repeat_between(pattern, min, max).expect("generated valid bounds")
            }),
            prop::collection::vec(inner.clone(), 0..4).prop_map(Pattern::sequence),
            prop::collection::vec(inner.clone(), 1..4).prop_map(Pattern::either),
            inner.prop_map(Pattern::capture),
        ]
    })
}

proptest! {
    #[test]
    fn safe_legacy_regex_parses_and_emits_compilable_regex(input in legacy_regex()) {
        let pattern = rx_core::parse_legacy_regex(&input)
            .expect("generated legacy subset should parse");
        let emitted = pattern.to_regex();

        regex::Regex::new(&emitted)
            .expect("emitted legacy regex should compile with Rust regex");
        prop_assert_eq!(pattern.to_regex(), emitted);
    }

    #[test]
    fn readable_rx_parses_and_emits_compilable_regex(input in readable_pattern()) {
        let pattern = rx_core::parse_readable_rx(&input)
            .expect("generated readable rx should parse");
        let emitted = pattern.to_regex();

        regex::Regex::new(&emitted)
            .expect("emitted readable regex should compile with Rust regex");
        prop_assert_eq!(rx_core::parse_readable_rx(&pattern.to_rx()).unwrap().to_regex(), emitted);
    }

    #[test]
    fn safe_core_emission_is_compilable_deterministic_and_readable_round_trips(pattern in core_pattern()) {
        let emitted = pattern.to_regex();

        regex::Regex::new(&emitted)
            .expect("safe-core emission should compile with Rust regex");
        prop_assert_eq!(pattern.to_regex(), emitted.clone());
        prop_assert_eq!(rx_core::parse_readable_rx(&pattern.to_rx()).unwrap().to_regex(), emitted);
    }
}
