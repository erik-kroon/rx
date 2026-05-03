use rx_core::{
    analyze_legacy_regex, lint_legacy_regex, parse_readable_rx, Diagnostic, DiagnosticCategory,
    DiagnosticSeverity, DiagnosticSourceFamily, Dialect, SourceSpan, ToDiagnostic,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommandOptions {
    dialect: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandResult {
    readable: Option<String>,
    regex: Option<String>,
    explanation: Option<String>,
    diagnostics: Vec<DiagnosticDto>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LintResult {
    diagnostics: Vec<DiagnosticDto>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticDto {
    severity: String,
    category: String,
    source_family: String,
    message: String,
    suggestion: Option<String>,
    span: SpanDto,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SpanDto {
    start: usize,
    end: usize,
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(js_name = explainRegex)]
pub fn explain_regex(input: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let (options, option_diagnostics) = decode_options(options)?;
    if !option_diagnostics.is_empty() {
        return to_js(&CommandResult::diagnostics(option_diagnostics));
    }

    let dialect = resolve_dialect(&options);
    let mut analysis = analyze_legacy_regex(input);
    let mut diagnostics = analysis
        .unsupported_diagnostics
        .drain(..)
        .map(|diagnostic| diagnostic.to_diagnostic())
        .chain(
            analysis
                .lint_diagnostics
                .drain(..)
                .map(|diagnostic| diagnostic.to_diagnostic()),
        )
        .collect::<Vec<_>>();

    match analysis.parse_result {
        Ok(pattern) => match pattern.emit(dialect) {
            Ok(regex) => to_js(&CommandResult {
                readable: Some(pattern.to_rx()),
                regex: Some(regex),
                explanation: Some(pattern.explain()),
                diagnostics: diagnostics.into_iter().map(DiagnosticDto::from).collect(),
            }),
            Err(error) => {
                diagnostics.push(error.to_diagnostic());
                to_js(&CommandResult::diagnostics(diagnostics))
            }
        },
        Err(error) => {
            diagnostics.push(error.to_diagnostic());
            to_js(&CommandResult::diagnostics(diagnostics))
        }
    }
}

#[wasm_bindgen(js_name = lintRegex)]
pub fn lint_regex(input: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let (_options, option_diagnostics) = decode_options(options)?;
    if !option_diagnostics.is_empty() {
        return to_js(&LintResult {
            diagnostics: option_diagnostics
                .into_iter()
                .map(DiagnosticDto::from)
                .collect(),
        });
    }

    to_js(&LintResult {
        diagnostics: lint_legacy_regex(input)
            .into_iter()
            .map(|diagnostic| diagnostic.to_diagnostic().into())
            .collect(),
    })
}

#[wasm_bindgen(js_name = parseRegex)]
pub fn parse_regex(input: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let (options, option_diagnostics) = decode_options(options)?;
    if !option_diagnostics.is_empty() {
        return to_js(&CommandResult::diagnostics(option_diagnostics));
    }

    let dialect = resolve_dialect(&options);
    let mut analysis = analyze_legacy_regex(input);
    let mut diagnostics = analysis
        .unsupported_diagnostics
        .drain(..)
        .map(|diagnostic| diagnostic.to_diagnostic())
        .chain(
            analysis
                .lint_diagnostics
                .drain(..)
                .map(|diagnostic| diagnostic.to_diagnostic()),
        )
        .collect::<Vec<_>>();

    match analysis.parse_result {
        Ok(pattern) => match pattern.emit(dialect) {
            Ok(regex) => to_js(&CommandResult {
                readable: Some(pattern.to_rx()),
                regex: Some(regex),
                explanation: None,
                diagnostics: diagnostics.into_iter().map(DiagnosticDto::from).collect(),
            }),
            Err(error) => {
                diagnostics.push(error.to_diagnostic());
                to_js(&CommandResult::diagnostics(diagnostics))
            }
        },
        Err(error) => {
            diagnostics.push(error.to_diagnostic());
            to_js(&CommandResult::diagnostics(diagnostics))
        }
    }
}

#[wasm_bindgen(js_name = emitRx)]
pub fn emit_rx(input: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let (options, option_diagnostics) = decode_options(options)?;
    if !option_diagnostics.is_empty() {
        return to_js(&CommandResult::diagnostics(option_diagnostics));
    }

    let dialect = resolve_dialect(&options);
    match parse_readable_rx(input) {
        Ok(pattern) => match pattern.emit(dialect) {
            Ok(regex) => to_js(&CommandResult {
                readable: Some(pattern.to_rx()),
                regex: Some(regex),
                explanation: None,
                diagnostics: Vec::new(),
            }),
            Err(error) => to_js(&CommandResult::diagnostics([error.to_diagnostic()])),
        },
        Err(error) => to_js(&CommandResult::diagnostics([error.to_diagnostic()])),
    }
}

#[wasm_bindgen(js_name = formatRx)]
pub fn format_rx(input: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let (_options, option_diagnostics) = decode_options(options)?;
    if !option_diagnostics.is_empty() {
        return to_js(&CommandResult::diagnostics(option_diagnostics));
    }

    match parse_readable_rx(input) {
        Ok(pattern) => to_js(&CommandResult {
            readable: Some(pattern.to_rx()),
            regex: None,
            explanation: None,
            diagnostics: Vec::new(),
        }),
        Err(error) => to_js(&CommandResult::diagnostics([error.to_diagnostic()])),
    }
}

impl CommandResult {
    fn diagnostics(diagnostics: impl IntoIterator<Item = Diagnostic>) -> Self {
        Self {
            readable: None,
            regex: None,
            explanation: None,
            diagnostics: diagnostics.into_iter().map(DiagnosticDto::from).collect(),
        }
    }
}

impl From<Diagnostic> for DiagnosticDto {
    fn from(diagnostic: Diagnostic) -> Self {
        Self {
            severity: diagnostic.severity.as_str().to_string(),
            category: diagnostic.category.as_str().to_string(),
            source_family: diagnostic.source_family.as_str().to_string(),
            message: diagnostic.message,
            suggestion: diagnostic.suggestion,
            span: diagnostic.span.into(),
        }
    }
}

impl From<SourceSpan> for SpanDto {
    fn from(span: SourceSpan) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

fn decode_options(options: JsValue) -> Result<(CommandOptions, Vec<Diagnostic>), JsValue> {
    if options.is_null() || options.is_undefined() {
        return Ok((CommandOptions::default(), Vec::new()));
    }

    let options = serde_wasm_bindgen::from_value(options)
        .map_err(|error| JsValue::from_str(&format!("invalid rx options: {error}")))?;
    let diagnostics = validate_options(&options);
    Ok((options, diagnostics))
}

fn validate_options(options: &CommandOptions) -> Vec<Diagnostic> {
    match options.dialect.as_deref() {
        Some(value) if parse_dialect(value).is_none() => {
            vec![Diagnostic {
                span: SourceSpan { start: 0, end: 0 },
                category: DiagnosticCategory::Dialect,
                severity: DiagnosticSeverity::Error,
                message: format!("unsupported dialect `{value}`"),
                suggestion: Some(
                    "Use rust-regex for full 0.1 support; pcre2 and posix-ere are limited compatibility targets."
                        .to_string(),
                ),
                source_family: DiagnosticSourceFamily::Dialect,
            }]
        }
        _ => Vec::new(),
    }
}

fn resolve_dialect(options: &CommandOptions) -> Dialect {
    options
        .dialect
        .as_deref()
        .and_then(parse_dialect)
        .unwrap_or(Dialect::RustRegex)
}

fn parse_dialect(value: &str) -> Option<Dialect> {
    match value {
        "rust" | "rust-regex" | "rust_regex" => Some(Dialect::RustRegex),
        "pcre2" => Some(Dialect::Pcre2),
        "posix-ere" | "posix_ere" => Some(Dialect::PosixEre),
        _ => None,
    }
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value)
        .map_err(|error| JsValue::from_str(&format!("failed to serialize rx result: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dialect_accepts_supported_names() {
        assert_eq!(parse_dialect("rust-regex"), Some(Dialect::RustRegex));
        assert_eq!(parse_dialect("pcre2"), Some(Dialect::Pcre2));
        assert_eq!(parse_dialect("posix-ere"), Some(Dialect::PosixEre));
        assert_eq!(parse_dialect("javascript"), None);
    }
}
