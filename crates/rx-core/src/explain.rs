use crate::charset::{CharSet, CharSetItem};
use crate::pattern::{Pattern, PatternKind};
pub(crate) fn explain_pattern(p: &Pattern, lines: &mut Vec<String>) {
    match p.kind() {
        PatternKind::Literal(v) => lines.push(format!("Match the literal text \"{v}\".")),
        PatternKind::Set(s) => lines.push(format!(
            "Match one character from the set: {}.",
            explain_set_contents(s)
        )),
        PatternKind::Sequence(ps) => {
            lines.push(format!("Match a sequence of {} patterns:", ps.len()));
            for p in ps {
                explain_pattern(p, lines)
            }
        }
        PatternKind::Either(ps) => {
            lines.push(format!("Match one of {} alternatives:", ps.len()));
            for p in ps {
                explain_pattern(p, lines)
            }
        }
        PatternKind::Repeat { pattern, min, max } => {
            lines.push(format!(
                "Repeat the next pattern {}.",
                explain_repeat(*min, *max)
            ));
            explain_pattern(pattern, lines)
        }
        PatternKind::StartText => lines.push("Match the start of the text.".into()),
        PatternKind::EndText => lines.push("Match the end of the text.".into()),
        PatternKind::Capture { name, pattern } => {
            match name {
                Some(n) => lines.push(format!("Capture the next pattern as \"{n}\".")),
                None => lines.push("Capture the next pattern.".into()),
            }
            explain_pattern(pattern, lines)
        }
    }
}
fn explain_set_contents(set: &CharSet) -> String {
    set.items
        .iter()
        .map(|i| match i {
            CharSetItem::Literal(ch) => format!("\"{}\"", explain_char(*ch)),
            CharSetItem::Range { start, end } => format!(
                "characters from \"{}\" through \"{}\"",
                explain_char(*start),
                explain_char(*end)
            ),
            CharSetItem::Ascii(class) => class.explanation().to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
fn explain_repeat(min: usize, max: Option<usize>) -> String {
    match (min, max) {
        (0, None) => "zero or more times".into(),
        (1, None) => "one or more times".into(),
        (0, Some(1)) => "zero or one time".into(),
        (a, Some(b)) if a == b => format!("exactly {a} times"),
        (a, Some(b)) => format!("between {a} and {b} times"),
        (a, None) => format!("at least {a} times"),
    }
}
fn explain_char(ch: char) -> String {
    match ch {
        '\n' => r#"\n"#.into(),
        '\r' => r#"\r"#.into(),
        '\t' => r#"\t"#.into(),
        c => c.to_string(),
    }
}
