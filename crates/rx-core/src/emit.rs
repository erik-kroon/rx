use std::fmt::{self, Write};

use crate::charset::{CharSet, CharSetItem};
use crate::diagnostic::{Dialect, Error, Feature};
use crate::pattern::{Pattern, PatternKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompactnessPolicy {
    Compact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FeatureSupport {
    dialect: Dialect,
}

pub(crate) struct EmissionContext<'a, W: Write> {
    feature_support: FeatureSupport,
    compactness: CompactnessPolicy,
    output: &'a mut W,
}

pub(crate) fn emit_pattern(pattern: &Pattern, dialect: Dialect) -> Result<String, Error> {
    let mut output = String::new();
    write_pattern(pattern, dialect, &mut output)?;
    Ok(output)
}

pub(crate) fn write_pattern(
    pattern: &Pattern,
    dialect: Dialect,
    output: &mut impl Write,
) -> Result<(), Error> {
    EmissionContext::new(dialect, output).write_pattern(pattern)
}

impl FeatureSupport {
    fn for_dialect(dialect: Dialect) -> Self {
        Self { dialect }
    }

    fn supports(self, feature: Feature) -> bool {
        self.dialect.supports(feature)
    }

    fn dialect(self) -> Dialect {
        self.dialect
    }
}

impl<'a, W: Write> EmissionContext<'a, W> {
    fn new(dialect: Dialect, output: &'a mut W) -> Self {
        Self {
            feature_support: FeatureSupport::for_dialect(dialect),
            compactness: CompactnessPolicy::Compact,
            output,
        }
    }

    fn write_pattern(&mut self, pattern: &Pattern) -> Result<(), Error> {
        match pattern.kind() {
            PatternKind::Literal(value) => self.write_fmt(|output| write_literal(value, output)),
            PatternKind::Set(set) => self.write_fmt(|output| write_set(set, output)),
            PatternKind::Sequence(patterns) => {
                for pattern in patterns {
                    self.write_sequence_item(pattern)?;
                }
                Ok(())
            }
            PatternKind::Either(patterns) => self.write_either(patterns),
            PatternKind::Repeat { pattern, min, max } => self.write_repeat(pattern, *min, *max),
            PatternKind::StartText => self.write_fmt(|output| output.write_char('^')),
            PatternKind::EndText => self.write_fmt(|output| output.write_char('$')),
            PatternKind::Capture { name, pattern } => self.write_capture(name.as_deref(), pattern),
        }
    }

    fn write_sequence_item(&mut self, pattern: &Pattern) -> Result<(), Error> {
        if matches!(pattern.kind(), PatternKind::Either(_)) {
            self.write_fmt(|output| output.write_str("(?:"))?;
            self.write_pattern(pattern)?;
            self.write_fmt(|output| output.write_char(')'))
        } else {
            self.write_pattern(pattern)
        }
    }

    fn write_either(&mut self, patterns: &[Pattern]) -> Result<(), Error> {
        for (index, pattern) in patterns.iter().enumerate() {
            if index > 0 {
                self.write_fmt(|output| output.write_char('|'))?;
            }
            self.write_pattern(pattern)?;
        }
        Ok(())
    }

    fn write_repeat(
        &mut self,
        pattern: &Pattern,
        min: usize,
        max: Option<usize>,
    ) -> Result<(), Error> {
        self.write_repeat_atom(pattern)?;
        self.write_fmt(|output| match (min, max) {
            (0, None) => output.write_char('*'),
            (1, None) => output.write_char('+'),
            (0, Some(1)) => output.write_char('?'),
            (min, Some(max)) if min == max => write!(output, "{{{min}}}"),
            (min, Some(max)) => write!(output, "{{{min},{max}}}"),
            (min, None) => write!(output, "{{{min},}}"),
        })
    }

    fn write_repeat_atom(&mut self, pattern: &Pattern) -> Result<(), Error> {
        match (self.compactness, pattern.kind()) {
            (CompactnessPolicy::Compact, PatternKind::Literal(value))
                if value.chars().count() == 1 =>
            {
                self.write_fmt(|output| write_literal(value, output))
            }
            (CompactnessPolicy::Compact, PatternKind::Set(_) | PatternKind::Capture { .. }) => {
                self.write_pattern(pattern)
            }
            _ => {
                self.write_fmt(|output| output.write_str("(?:"))?;
                self.write_pattern(pattern)?;
                self.write_fmt(|output| output.write_char(')'))
            }
        }
    }

    fn write_capture(&mut self, name: Option<&str>, pattern: &Pattern) -> Result<(), Error> {
        if name.is_some() && !self.feature_support.supports(Feature::NamedCapture) {
            return Err(Error::UnsupportedDialectFeature {
                dialect: self.feature_support.dialect(),
                feature: Feature::NamedCapture,
            });
        }

        match name {
            Some(name) => {
                self.write_fmt(|output| write!(output, "(?<{name}>"))?;
                self.write_pattern(pattern)?;
                self.write_fmt(|output| output.write_char(')'))
            }
            None => {
                self.write_fmt(|output| output.write_char('('))?;
                self.write_pattern(pattern)?;
                self.write_fmt(|output| output.write_char(')'))
            }
        }
    }

    fn write_fmt(&mut self, write: impl FnOnce(&mut W) -> fmt::Result) -> Result<(), Error> {
        write(self.output).expect("writing regex into String cannot fail");
        Ok(())
    }
}

pub(crate) fn write_literal(value: &str, output: &mut impl Write) -> fmt::Result {
    for ch in value.chars() {
        if is_regex_meta(ch) {
            output.write_char('\\')?;
        }
        output.write_char(ch)?;
    }
    Ok(())
}

fn write_set(set: &CharSet, output: &mut impl Write) -> fmt::Result {
    output.write_char('[')?;
    let mut has_literal_hyphen = false;
    for item in &set.items {
        match item {
            CharSetItem::Literal('-') => has_literal_hyphen = true,
            CharSetItem::Literal(ch) => write_set_literal(*ch, output)?,
            CharSetItem::Range { start, end } => {
                write_set_literal(*start, output)?;
                output.write_char('-')?;
                write_set_literal(*end, output)?;
            }
            CharSetItem::Ascii(class) => output.write_str(class.regex_fragment())?,
        }
    }
    if has_literal_hyphen {
        output.write_char('-')?;
    }
    output.write_char(']')
}

fn write_set_literal(ch: char, output: &mut impl Write) -> fmt::Result {
    if matches!(ch, '\\' | ']' | '-' | '^') {
        output.write_char('\\')?;
    }
    output.write_char(ch)
}

pub(crate) fn is_regex_meta(ch: char) -> bool {
    matches!(
        ch,
        '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}' | '^' | '$'
    )
}
