use crate::charset::{CharSet, CharSetItem};
use crate::pattern::{Pattern, PatternKind};
pub(crate) fn write_rx_pattern(p: &Pattern, indent: usize, out: &mut String) {
    match p.kind() {
        PatternKind::Literal(v) => out.push_str(&format!("literal(\"{}\")", escape_rx(v))),
        PatternKind::Set(s) => write_rx_set(s, indent, out),
        PatternKind::Sequence(ps) => write_rx_call("sequence", ps, indent, out),
        PatternKind::Either(ps) => write_rx_call("either", ps, indent, out),
        PatternKind::Repeat { pattern, min, max } => match (*min, *max) {
            (0, None) => write_rx_unary_call("zero_or_more", pattern, indent, out),
            (1, None) => write_rx_unary_call("one_or_more", pattern, indent, out),
            (0, Some(1)) => write_rx_unary_call("optional", pattern, indent, out),
            (a, Some(b)) if a == b => write_rx_repeat_call("repeat", pattern, &[a], indent, out),
            (a, Some(b)) => write_rx_repeat_call("repeat_between", pattern, &[a, b], indent, out),
            (a, None) => write_rx_repeat_call("repeat_at_least", pattern, &[a], indent, out),
        },
        PatternKind::StartText => out.push_str("start_text"),
        PatternKind::EndText => out.push_str("end_text"),
        PatternKind::Capture { name, pattern } => match name {
            Some(n) => {
                out.push_str(&format!("named_capture(\"{}\",\n", escape_rx(n)));
                out.push_str(&" ".repeat(indent + 4));
                write_rx_pattern(pattern, indent + 4, out);
                out.push('\n');
                out.push_str(&" ".repeat(indent));
                out.push(')')
            }
            None => write_rx_unary_call("capture", pattern, indent, out),
        },
    }
}
fn write_rx_set(set: &CharSet, indent: usize, out: &mut String) {
    out.push_str("set(");
    if set.items.is_empty() {
        out.push(')');
        return;
    }
    out.push('\n');
    let mut first = true;
    let mut lits = String::new();
    for item in &set.items {
        match item {
            CharSetItem::Literal(ch) => lits.push(*ch),
            CharSetItem::Range { .. } | CharSetItem::Ascii(_) => {
                flush(&mut lits, indent + 4, &mut first, out);
                sep(indent + 4, &mut first, out);
                match item {
                    CharSetItem::Range { start, end } => out.push_str(&format!(
                        "range(\"{}\", \"{}\")",
                        escape_rx(&start.to_string()),
                        escape_rx(&end.to_string())
                    )),
                    CharSetItem::Ascii(class) => out.push_str(class.readable_qualified_name()),
                    _ => {}
                }
            }
        }
    }
    flush(&mut lits, indent + 4, &mut first, out);
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push(')')
}
fn flush(lits: &mut String, indent: usize, first: &mut bool, out: &mut String) {
    if lits.is_empty() {
        return;
    }
    sep(indent, first, out);
    if lits.chars().count() == 1 {
        out.push_str(&format!("char(\"{}\")", escape_rx(lits)))
    } else {
        out.push_str(&format!("chars(\"{}\")", escape_rx(lits)))
    }
    lits.clear()
}
fn sep(indent: usize, first: &mut bool, out: &mut String) {
    if !*first {
        out.push_str(",\n")
    }
    out.push_str(&" ".repeat(indent));
    *first = false
}
fn write_rx_call(name: &str, patterns: &[Pattern], indent: usize, out: &mut String) {
    out.push_str(name);
    out.push('(');
    if patterns.is_empty() {
        out.push(')');
        return;
    }
    out.push('\n');
    for (i, p) in patterns.iter().enumerate() {
        if i > 0 {
            out.push_str(",\n")
        }
        out.push_str(&" ".repeat(indent + 4));
        write_rx_pattern(p, indent + 4, out)
    }
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push(')')
}
fn write_rx_unary_call(name: &str, p: &Pattern, indent: usize, out: &mut String) {
    out.push_str(name);
    out.push_str("(\n");
    out.push_str(&" ".repeat(indent + 4));
    write_rx_pattern(p, indent + 4, out);
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push(')')
}
fn write_rx_repeat_call(
    name: &str,
    p: &Pattern,
    counts: &[usize],
    indent: usize,
    out: &mut String,
) {
    out.push_str(name);
    out.push_str("(\n");
    out.push_str(&" ".repeat(indent + 4));
    write_rx_pattern(p, indent + 4, out);
    for c in counts {
        out.push_str(&format!(",\n{}{c}", " ".repeat(indent + 4)))
    }
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push(')')
}
pub(crate) fn escape_rx(v: &str) -> String {
    let mut e = String::new();
    for ch in v.chars() {
        match ch {
            '\\' => e.push_str("\\\\"),
            '"' => e.push_str(r#"\""#),
            '\n' => e.push_str(r#"\n"#),
            '\r' => e.push_str(r#"\r"#),
            '\t' => e.push_str(r#"\t"#),
            c => e.push(c),
        }
    }
    e
}
pub(crate) fn write_rust_builder(pattern: &Pattern) -> String {
    pattern.to_rx()
}
