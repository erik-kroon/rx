use std::{env, fs, process};

use rx::ToDiagnostic;

fn main() {
    let code = match run(env::args().skip(1).collect()) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{error}");
            1
        }
    };
    process::exit(code);
}

fn run(args: Vec<String>) -> Result<(), String> {
    match args.as_slice() {
        [] => Err(usage()),
        [command, input] if command == "explain" => explain(input),
        [command, input] if command == "convert" => convert(input),
        [command, input] if command == "check" => check(input),
        [command, input] if command == "emit" => emit(input, rx::Dialect::RustRegex),
        [command, input, flag, dialect] if command == "emit" && flag == "--dialect" => {
            emit(input, parse_dialect(dialect)?)
        }
        [command, input, flag, dialect] if command == "emit" && flag == "--target" => {
            emit(input, parse_dialect(dialect)?)
        }
        [command, ..]
            if command == "check"
                || command == "convert"
                || command == "explain"
                || command == "emit" =>
        {
            Err(usage())
        }
        _ => Err(usage()),
    }
}

fn explain(input: &str) -> Result<(), String> {
    if let Ok(source) = fs::read_to_string(input) {
        let patterns = rx::parse_readable_rx_file(&source)
            .map_err(|error| format_readable_parse_error(&source, error))?;
        for (index, named) in patterns.iter().enumerate() {
            if index > 0 {
                println!();
            }
            println!("Pattern: {}", named.0);
            println!();
            println!("Readable rx:");
            println!("{}", named.1.to_rx());
            println!();
            println!("Explanation:");
            println!("{}", named.1.explain());
        }
        return Ok(());
    }

    let warnings = rx::lint_legacy_regex(input);
    let pattern = rx::parse_legacy_regex(input).map_err(format_parse_error)?;

    println!("Readable rx:");
    println!("{}", pattern.to_rx());
    println!();
    println!("Explanation:");
    println!("{}", pattern.explain());

    if !warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in warnings {
            println!("- {}", warning.to_diagnostic().render());
        }
    }

    Ok(())
}

fn convert(input: &str) -> Result<(), String> {
    let pattern = rx::parse_legacy_regex(input).map_err(format_parse_error)?;
    let readable = pattern.to_rx();

    println!("Readable rx:");
    println!("{readable}");
    println!();
    println!("Rust builder:");
    println!("{readable}");
    println!();
    println!("Macro:");
    println!("rx::pattern! {{");
    for line in readable.lines() {
        println!("    {line}");
    }
    println!("}}");
    println!();
    println!("Generated regex:");
    println!("{}", pattern.to_regex());

    let warnings = rx::lint_legacy_regex(input);
    if !warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in warnings {
            println!("- {}", warning.to_diagnostic().render());
        }
    }

    Ok(())
}

fn check(input: &str) -> Result<(), String> {
    let source =
        fs::read_to_string(input).map_err(|error| format!("failed to read `{input}`: {error}"))?;
    let patterns = rx::parse_readable_rx_file(&source)
        .map_err(|error| format_readable_parse_error(&source, error))?;
    println!(
        "OK: {} pattern{}",
        patterns.len(),
        if patterns.len() == 1 { "" } else { "s" }
    );
    for pattern in patterns {
        println!("- {}", pattern.0);
    }
    Ok(())
}

fn emit(input: &str, dialect: rx::Dialect) -> Result<(), String> {
    let readable = read_readable_input(input)?;
    match readable {
        ReadableInput::Expression(source) => {
            let pattern = rx::parse_readable_rx(&source)
                .map_err(|error| format_readable_parse_error(&source, error))?;
            let emitted = pattern.emit(dialect).map_err(format_emit_error)?;
            println!("{emitted}");
        }
        ReadableInput::File(source) => {
            let patterns = rx::parse_readable_rx_file(&source)
                .map_err(|error| format_readable_parse_error(&source, error))?;
            for named in patterns {
                let emitted = named.1.emit(dialect).map_err(format_emit_error)?;
                if patterns_have_single_definition(&source) {
                    println!("{emitted}");
                } else {
                    println!("{}: {emitted}", named.0);
                }
            }
        }
    }
    Ok(())
}

enum ReadableInput {
    Expression(String),
    File(String),
}

fn read_readable_input(input: &str) -> Result<ReadableInput, String> {
    fs::read_to_string(input)
        .or_else(|error| {
            if looks_like_readable_pattern(input) {
                Ok(input.to_string())
            } else {
                Err(format!("failed to read `{input}`: {error}"))
            }
        })
        .map(|source| {
            if fs::metadata(input).is_ok() {
                ReadableInput::File(source)
            } else {
                ReadableInput::Expression(source)
            }
        })
}

fn patterns_have_single_definition(source: &str) -> bool {
    source
        .split_whitespace()
        .filter(|token| *token == "pattern")
        .count()
        == 1
}

fn looks_like_readable_pattern(input: &str) -> bool {
    input.contains('(') || input.contains("ascii.") || input == "start_text" || input == "end_text"
}

fn parse_dialect(value: &str) -> Result<rx::Dialect, String> {
    match value {
        "rust" | "rust-regex" | "rust_regex" => Ok(rx::Dialect::RustRegex),
        "pcre2" => Ok(rx::Dialect::Pcre2),
        "posix-ere" | "posix_ere" => Ok(rx::Dialect::PosixEre),
        _ => Err(format!(
            "unsupported dialect `{value}`. Use rust-regex for full 0.1 support; pcre2 and posix-ere are limited compatibility targets."
        )),
    }
}

fn format_parse_error(error: rx::ParseError) -> String {
    format!(
        "failed to parse legacy regex\n{}",
        error.to_diagnostic().render()
    )
}

fn format_emit_error(error: rx::Error) -> String {
    error.to_diagnostic().render()
}

fn format_readable_parse_error(input: &str, error: rx::ReadableParseError) -> String {
    let diagnostic = error.to_diagnostic();
    let rx::SourceLocation { line, column } = diagnostic.source_location(input);
    format!(
        "failed to parse readable rx at line {line}, column {column}\n{}",
        diagnostic.render_with_source(input)
    )
}

fn usage() -> String {
    "usage:\n  rx explain '<legacy-regex>'\n  rx explain path.rx\n  rx convert '<legacy-regex>'\n  rx check path.rx\n  rx emit '<readable-rx>' [--dialect rust-regex|pcre2|posix-ere]\n  rx emit path.rx [--dialect rust-regex|pcre2|posix-ere]\n\nrust-regex is the fully supported 0.1 dialect; pcre2 and posix-ere are limited compatibility targets.".to_string()
}
