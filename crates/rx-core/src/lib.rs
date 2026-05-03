//! Core AST and emitters for `rx`.
//!
//! This crate intentionally stays below the public builder API. It owns the
//! canonical pattern representation and standard regex emission behavior.

/// Canonical pattern representation for the regular-language core.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pattern {
    Literal(String),
}

impl Pattern {
    /// Construct a literal pattern.
    pub fn literal(value: impl Into<String>) -> Self {
        Self::Literal(value.into())
    }

    /// Emit this pattern as a compact standard regex string.
    pub fn to_regex(&self) -> String {
        match self {
            Self::Literal(value) => emit_literal(value),
        }
    }
}

fn emit_literal(value: &str) -> String {
    let mut emitted = String::with_capacity(value.len());

    for ch in value.chars() {
        if is_regex_meta(ch) {
            emitted.push('\\');
        }
        emitted.push(ch);
    }

    emitted
}

fn is_regex_meta(ch: char) -> bool {
    matches!(
        ch,
        '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}' | '^' | '$'
    )
}

#[cfg(test)]
mod tests {
    use super::Pattern;

    #[test]
    fn emits_plain_literal_unchanged() {
        assert_eq!(Pattern::literal("abc123").to_regex(), "abc123");
    }

    #[test]
    fn escapes_regex_metacharacters_in_literals() {
        assert_eq!(
            Pattern::literal(r#"\.+*?()|[]{}^$"#).to_regex(),
            r#"\\\.\+\*\?\(\)\|\[\]\{\}\^\$"#
        );
    }

    #[test]
    fn preserves_unicode_literal_text() {
        assert_eq!(Pattern::literal("cafe").to_regex(), "cafe");
    }
}
