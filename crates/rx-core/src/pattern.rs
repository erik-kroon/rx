use crate::charset::{CharSet, CharSetItem, UnicodeClass};
use crate::diagnostic::{Dialect, Error};

/// Canonical pattern representation for the regular-language core.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pattern {
    pub(crate) kind: PatternKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PatternKind {
    Literal(String),
    Set(CharSet),
    Sequence(Vec<Pattern>),
    Either(Vec<Pattern>),
    Repeat {
        pattern: Box<Pattern>,
        min: usize,
        max: Option<usize>,
    },
    StartText,
    EndText,
    Capture {
        name: Option<String>,
        pattern: Box<Pattern>,
    },
}

impl Pattern {
    pub fn literal(value: impl Into<String>) -> Self {
        Self {
            kind: PatternKind::Literal(value.into()),
        }
    }

    pub fn set(items: impl IntoIterator<Item = CharSetItem>) -> Self {
        Self {
            kind: PatternKind::Set(CharSet::new(items)),
        }
    }

    pub fn sequence(patterns: impl IntoIterator<Item = Pattern>) -> Self {
        Self {
            kind: PatternKind::Sequence(patterns.into_iter().collect()),
        }
    }

    pub fn either(patterns: impl IntoIterator<Item = Pattern>) -> Self {
        Self {
            kind: PatternKind::Either(patterns.into_iter().collect()),
        }
    }

    pub fn zero_or_more(pattern: Pattern) -> Self {
        Self::repeat(pattern, 0, None)
    }

    pub fn one_or_more(pattern: Pattern) -> Self {
        Self::repeat(pattern, 1, None)
    }

    pub fn optional(pattern: Pattern) -> Self {
        Self::repeat(pattern, 0, Some(1))
    }

    pub fn repeat_between(pattern: Pattern, min: usize, max: usize) -> Result<Self, Error> {
        if min > max {
            return Err(Error::InvalidRepeatBounds { min, max });
        }

        Ok(Self::repeat(pattern, min, Some(max)))
    }

    pub fn repeat_exactly(pattern: Pattern, count: usize) -> Self {
        Self::repeat(pattern, count, Some(count))
    }

    pub fn start_text() -> Self {
        Self {
            kind: PatternKind::StartText,
        }
    }

    pub fn end_text() -> Self {
        Self {
            kind: PatternKind::EndText,
        }
    }

    pub fn capture(pattern: Pattern) -> Self {
        Self {
            kind: PatternKind::Capture {
                name: None,
                pattern: Box::new(pattern),
            },
        }
    }

    pub fn named_capture(name: impl Into<String>, pattern: Pattern) -> Result<Self, Error> {
        let name = name.into();
        if !crate::diagnostic::is_valid_capture_name(&name) {
            return Err(Error::InvalidCaptureName(name));
        }

        Ok(Self {
            kind: PatternKind::Capture {
                name: Some(name),
                pattern: Box::new(pattern),
            },
        })
    }

    pub fn unicode_class(class: UnicodeClass) -> Result<Self, Error> {
        Err(Error::UnsupportedUnicodeClass(class))
    }

    pub fn to_regex(&self) -> String {
        self.emit(Dialect::RustRegex)
            .expect("RustRegex supports all currently constructible safe-core patterns")
    }

    pub fn write_regex(&self, output: &mut String) {
        self.write_regex_for(Dialect::RustRegex, output)
            .expect("RustRegex supports all currently constructible safe-core patterns");
    }

    pub fn emit(&self, dialect: Dialect) -> Result<String, Error> {
        crate::emit::emit_pattern(self, dialect)
    }

    pub fn write_regex_for(&self, dialect: Dialect, output: &mut String) -> Result<(), Error> {
        crate::emit::write_pattern(self, dialect, output)
    }

    pub fn to_rx(&self) -> String {
        let mut output = String::new();
        crate::pretty::write_rx_pattern(self, 0, &mut output);
        output
    }

    pub fn explain(&self) -> String {
        let mut lines = Vec::new();
        crate::explain::explain_pattern(self, &mut lines);
        lines.join("\n")
    }

    pub(crate) fn kind(&self) -> &PatternKind {
        &self.kind
    }

    fn repeat(pattern: Pattern, min: usize, max: Option<usize>) -> Self {
        Self {
            kind: PatternKind::Repeat {
                pattern: Box::new(pattern),
                min,
                max,
            },
        }
    }
}
