use crate::diagnostic::Error;

const ASCII_CLASSES: &[AsciiClass] = &[
    AsciiClass::Word,
    AsciiClass::Alnum,
    AsciiClass::Alpha,
    AsciiClass::Digit,
    AsciiClass::Whitespace,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharSet {
    pub(crate) items: Vec<CharSetItem>,
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CharSetItem {
    Literal(char),
    Range { start: char, end: char },
    Ascii(AsciiClass),
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AsciiClass {
    Word,
    Alnum,
    Alpha,
    Digit,
    Whitespace,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnicodeClass {
    Word,
    Letter,
}
impl CharSet {
    pub(crate) fn new(input: impl IntoIterator<Item = CharSetItem>) -> Self {
        let mut items = Vec::new();
        for item in input {
            if item.is_redundant_with(&items) {
                continue;
            }
            if let CharSetItem::Ascii(c) = item {
                items.retain(|e| !c.contains_item(e));
            }
            if !items.contains(&item) {
                items.push(item)
            }
        }
        Self { items }
    }
}
impl CharSetItem {
    pub fn literal(value: char) -> Self {
        Self::Literal(value)
    }
    pub fn range(start: char, end: char) -> Result<Self, Error> {
        if start > end {
            Err(Error::InvalidRange { start, end })
        } else {
            Ok(Self::Range { start, end })
        }
    }
    pub fn ascii(class: AsciiClass) -> Self {
        Self::Ascii(class)
    }

    pub(crate) fn is_contained_in_ascii_class(&self, class: AsciiClass) -> bool {
        match self {
            CharSetItem::Literal(ch) => class.contains_char(*ch),
            CharSetItem::Range { start, end } => {
                start.is_ascii()
                    && end.is_ascii()
                    && (*start..=*end).all(|ch| class.contains_char(ch))
            }
            CharSetItem::Ascii(existing) => class.contains_class(*existing),
        }
    }

    fn is_redundant_with(&self, existing: &[CharSetItem]) -> bool {
        match self {
            CharSetItem::Literal(ch) => existing.iter().any(|item| item.contains_char(*ch)),
            CharSetItem::Range { start, end } => existing.iter().any(|item| {
                matches!(item, CharSetItem::Range { start: existing_start, end: existing_end } if existing_start <= start && end <= existing_end)
            }),
            CharSetItem::Ascii(class) => existing.iter().any(|item| match item {
                CharSetItem::Ascii(existing) => existing.contains_class(*class),
                _ => item == self,
            }),
        }
    }

    fn contains_char(&self, ch: char) -> bool {
        match self {
            CharSetItem::Literal(existing) => *existing == ch,
            CharSetItem::Range { start, end } => *start <= ch && ch <= *end,
            CharSetItem::Ascii(class) => class.contains_char(ch),
        }
    }
}

impl AsciiClass {
    pub(crate) fn all() -> &'static [AsciiClass] {
        ASCII_CLASSES
    }

    pub(crate) fn from_readable_name(name: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|class| class.readable_name() == name)
    }

    pub(crate) fn regex_fragment(self) -> &'static str {
        match self {
            AsciiClass::Word => "A-Za-z0-9_",
            AsciiClass::Alnum => "A-Za-z0-9",
            AsciiClass::Alpha => "A-Za-z",
            AsciiClass::Digit => "0-9",
            AsciiClass::Whitespace => r#"\t\n\f\r "#,
        }
    }

    pub(crate) fn readable_name(self) -> &'static str {
        match self {
            AsciiClass::Word => "word",
            AsciiClass::Alnum => "alnum",
            AsciiClass::Alpha => "alpha",
            AsciiClass::Digit => "digit",
            AsciiClass::Whitespace => "whitespace",
        }
    }

    pub(crate) fn readable_qualified_name(self) -> &'static str {
        match self {
            AsciiClass::Word => "ascii.word",
            AsciiClass::Alnum => "ascii.alnum",
            AsciiClass::Alpha => "ascii.alpha",
            AsciiClass::Digit => "ascii.digit",
            AsciiClass::Whitespace => "ascii.whitespace",
        }
    }

    pub(crate) fn rust_builder_path(self) -> &'static str {
        match self {
            AsciiClass::Word => "::rx::ascii::word()",
            AsciiClass::Alnum => "::rx::ascii::alnum()",
            AsciiClass::Alpha => "::rx::ascii::alpha()",
            AsciiClass::Digit => "::rx::ascii::digit()",
            AsciiClass::Whitespace => "::rx::ascii::whitespace()",
        }
    }

    pub(crate) fn explanation(self) -> &'static str {
        match self {
            AsciiClass::Word => "ASCII word characters A-Z, a-z, 0-9, and _",
            AsciiClass::Alnum => "ASCII alphanumeric characters A-Z, a-z, and 0-9",
            AsciiClass::Alpha => "ASCII alphabetic characters A-Z and a-z",
            AsciiClass::Digit => "ASCII digits 0-9",
            AsciiClass::Whitespace => {
                "ASCII whitespace tab, line feed, form feed, carriage return, and space"
            }
        }
    }

    pub(crate) fn contains_char(self, ch: char) -> bool {
        match self {
            AsciiClass::Word => ch.is_ascii_alphanumeric() || ch == '_',
            AsciiClass::Alnum => ch.is_ascii_alphanumeric(),
            AsciiClass::Alpha => ch.is_ascii_alphabetic(),
            AsciiClass::Digit => ch.is_ascii_digit(),
            AsciiClass::Whitespace => matches!(ch, '\t' | '\n' | '\u{000c}' | '\r' | ' '),
        }
    }

    pub(crate) fn contains_class(self, other: AsciiClass) -> bool {
        matches!(
            (self, other),
            (
                AsciiClass::Word,
                AsciiClass::Word | AsciiClass::Alnum | AsciiClass::Alpha | AsciiClass::Digit
            ) | (
                AsciiClass::Alnum,
                AsciiClass::Alnum | AsciiClass::Alpha | AsciiClass::Digit
            ) | (AsciiClass::Alpha, AsciiClass::Alpha)
                | (AsciiClass::Digit, AsciiClass::Digit)
                | (AsciiClass::Whitespace, AsciiClass::Whitespace)
        )
    }

    fn contains_item(self, item: &CharSetItem) -> bool {
        item.is_contained_in_ascii_class(self)
    }
}
