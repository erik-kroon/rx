use proc_macro::TokenStream;
use rx_core::{parse_readable_rx_artifact, DiagnosticCategory, ReadableParseError, ToDiagnostic};

#[proc_macro]
pub fn pattern(input: TokenStream) -> TokenStream {
    expand(input, MacroOutput::Pattern)
}

#[proc_macro]
pub fn regex(input: TokenStream) -> TokenStream {
    expand(input, MacroOutput::Regex)
}

enum MacroOutput {
    Pattern,
    Regex,
}

fn expand(input: TokenStream, output: MacroOutput) -> TokenStream {
    match parse_readable_rx_artifact(&input.to_string()) {
        Ok(artifact) => match output {
            MacroOutput::Pattern => artifact
                .rust_builder_code()
                .parse()
                .expect("macro expansion is valid Rust"),
            MacroOutput::Regex => {
                let regex = rust_string_literal(&artifact.pattern().to_regex());
                regex.parse().expect("regex string literal is valid Rust")
            }
        },
        Err(error) => compile_error(&format_macro_parse_error(error)),
    }
}

fn format_macro_parse_error(error: ReadableParseError) -> String {
    let diagnostic = error.to_diagnostic();

    if matches!(diagnostic.category, DiagnosticCategory::Validation) {
        if diagnostic.message.starts_with("range start ") {
            return format!("invalid rx pattern: {}", diagnostic.message);
        }
        return diagnostic.message;
    }

    if matches!(diagnostic.category, DiagnosticCategory::Compatibility) {
        let message = diagnostic
            .message
            .strip_prefix("unknown readable rx pattern `")
            .and_then(|rest| rest.strip_suffix('`'))
            .map(|unsupported| format!("unsupported construct `{unsupported}` in MVP macro DSL"))
            .or_else(|| {
                error
                    .message
                    .strip_prefix("unsupported construct `")
                    .and_then(|rest| rest.strip_suffix("` in MVP readable rx syntax"))
                    .map(|unsupported| {
                        format!("unsupported construct `{unsupported}` in MVP macro DSL")
                    })
            })
            .or_else(|| {
                error
                    .message
                    .split_once(": unsupported construct `")
                    .and_then(|(_, rest)| rest.strip_suffix("` in MVP readable rx syntax"))
                    .map(|unsupported| {
                        format!("unsupported construct `{unsupported}` in MVP macro DSL")
                    })
            })
            .unwrap_or(diagnostic.message);
        return format!(
            "invalid rx macro input at byte {}: {}",
            diagnostic.span.start, message
        );
    }

    let message = if diagnostic.message.starts_with("expected ") {
        format!("malformed syntax: {}", diagnostic.message)
    } else if let Some(unsupported) = error
        .message
        .strip_prefix("unknown readable rx pattern `")
        .and_then(|rest| rest.strip_suffix('`'))
    {
        format!("unsupported construct `{unsupported}` in MVP macro DSL")
    } else {
        diagnostic
            .message
            .replace(" in MVP readable rx syntax", " in MVP macro DSL")
    };

    format!(
        "invalid rx macro input at byte {}: {message}",
        diagnostic.span.start
    )
}

fn compile_error(message: &str) -> TokenStream {
    format!("compile_error!({})", rust_string_literal(message))
        .parse()
        .expect("compile_error expansion is valid Rust")
}

fn rust_string_literal(value: &str) -> String {
    format!("{value:?}")
}
