use std::hint::black_box;
use std::time::{Duration, Instant};

use rx_core::{parse_legacy_regex, parse_readable_rx, AsciiClass, CharSetItem, Pattern};

const ITERATIONS: usize = 100_000;
const READABLE_PATTERN: &str =
    r#"one_or_more(set(ascii.alnum, chars("._/-"), range("a", "f"), char(":")))"#;
const LEGACY_PATTERN: &str = r#"[\w._/\-a-f:]+"#;

fn main() {
    run(
        "readable_parse",
        || {
            let pattern = parse_readable_rx(black_box(READABLE_PATTERN)).expect("readable pattern");
            black_box(pattern.to_regex());
        },
        ITERATIONS,
    );
    run(
        "legacy_parse",
        || {
            let pattern = parse_legacy_regex(black_box(LEGACY_PATTERN)).expect("legacy pattern");
            black_box(pattern.to_regex());
        },
        ITERATIONS,
    );
    run(
        "builder_emit",
        || {
            let pattern = Pattern::one_or_more(Pattern::set([
                CharSetItem::ascii(AsciiClass::Alnum),
                CharSetItem::literal('.'),
                CharSetItem::literal('_'),
                CharSetItem::literal('/'),
                CharSetItem::literal('-'),
                CharSetItem::range('a', 'f').expect("valid range"),
                CharSetItem::literal(':'),
            ]));
            let mut output = String::new();
            pattern.write_regex(&mut output);
            black_box(output);
        },
        ITERATIONS,
    );
}

fn run(label: &str, mut workload: impl FnMut(), iterations: usize) {
    for _ in 0..1_000 {
        workload();
    }

    let start = Instant::now();
    for _ in 0..iterations {
        workload();
    }
    let elapsed = start.elapsed();
    println!(
        "{label}: {elapsed:?} total, {:?}/iter",
        per_iter(elapsed, iterations)
    );
}

fn per_iter(elapsed: Duration, iterations: usize) -> Duration {
    Duration::from_nanos((elapsed.as_nanos() / iterations as u128) as u64)
}
