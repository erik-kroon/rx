use std::process::Command;

fn rx() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rx"))
}

#[test]
fn explain_outputs_readable_form_explanation_and_warnings() {
    let output = rx()
        .args(["explain", r#"[\w\._/-]+"#])
        .output()
        .expect("run rx explain");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Readable rx:"));
    assert!(stdout.contains("one_or_more("));
    assert!(stdout.contains("ascii.word"));
    assert!(stdout.contains("Explanation:"));
    assert!(stdout.contains("Repeat the next pattern one or more times."));
    assert!(stdout.contains("Warnings:"));
    assert!(stdout.contains("does not need escaping inside a character class"));
    assert!(stdout.contains("target-dependent ASCII or Unicode semantics"));
}

#[test]
fn convert_outputs_supported_legacy_regex_forms() {
    let output = rx()
        .args(["convert", r#"^[A-Za-z_][A-Za-z0-9_]*$"#])
        .output()
        .expect("run rx convert");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Readable rx:"));
    assert!(stdout.contains("sequence("));
    assert!(stdout.contains("start_text"));
    assert!(stdout.contains("Rust builder:"));
    assert!(stdout.contains("Macro:"));
    assert!(stdout.contains("rx::pattern! {"));
    assert!(stdout.contains("Generated regex:"));
    assert!(stdout.contains("^[A-Za-z_][A-Za-z0-9_]*$"));
}

#[test]
fn convert_reports_unsupported_legacy_regex_diagnostics() {
    let output = rx()
        .args(["convert", r#"(\w+)\1"#])
        .output()
        .expect("run rx convert failure");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to parse legacy regex"));
    assert!(stderr.contains("error compatibility in legacy regex"));
    assert!(stderr.contains("backreferences are outside the MVP safe core"));
}

#[test]
fn emit_outputs_standard_regex_from_readable_input() {
    let output = rx()
        .args([
            "emit",
            r#"one_or_more(set(ascii.alnum, chars("._/-")))"#,
            "--dialect",
            "rust-regex",
        ])
        .output()
        .expect("run rx emit");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "[A-Za-z0-9._/-]+\n"
    );
}

#[test]
fn emit_reads_pattern_definition_file() {
    let path =
        std::env::temp_dir().join(format!("rx-cli-test-{}-path-piece.rx", std::process::id()));
    std::fs::write(
        &path,
        r#"pattern path_piece =
    one_or_more(
        set(
            ascii.alnum,
            chars("._/-")
        )
    )
"#,
    )
    .unwrap();

    let output = rx()
        .args(["emit", path.to_str().unwrap()])
        .output()
        .expect("run rx emit file");

    let _ = std::fs::remove_file(path);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "[A-Za-z0-9._/-]+\n"
    );
}

#[test]
fn check_validates_named_pattern_file() {
    let path = std::env::temp_dir().join(format!("rx-cli-test-{}-check.rx", std::process::id()));
    std::fs::write(
        &path,
        r#"pattern path_piece = one_or_more(set(ascii.alnum, chars("._/-")))
pattern identifier = sequence(start_text, set(ascii.alpha, char("_")), zero_or_more(set(ascii.alnum, char("_"))), end_text)
"#,
    )
    .unwrap();

    let output = rx()
        .args(["check", path.to_str().unwrap()])
        .output()
        .expect("run rx check");

    let _ = std::fs::remove_file(path);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("OK: 2 patterns"));
    assert!(stdout.contains("- path_piece"));
    assert!(stdout.contains("- identifier"));
}

#[test]
fn explain_reads_named_pattern_file() {
    let path = std::env::temp_dir().join(format!("rx-cli-test-{}-explain.rx", std::process::id()));
    std::fs::write(
        &path,
        r#"pattern path_piece =
    one_or_more(
        set(
            ascii.alnum,
            chars("._/-")
        )
    )
"#,
    )
    .unwrap();

    let output = rx()
        .args(["explain", path.to_str().unwrap()])
        .output()
        .expect("run rx explain file");

    let _ = std::fs::remove_file(path);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Pattern: path_piece"));
    assert!(stdout.contains("Readable rx:"));
    assert!(stdout.contains("Explanation:"));
    assert!(stdout.contains("ASCII alphanumeric characters"));
}

#[test]
fn emit_multiple_named_patterns_with_names() {
    let path = std::env::temp_dir().join(format!("rx-cli-test-{}-multi.rx", std::process::id()));
    std::fs::write(
        &path,
        r#"pattern path_piece = one_or_more(set(ascii.alnum, chars("._/-")))
pattern identifier = sequence(start_text, set(ascii.alpha, char("_")), zero_or_more(set(ascii.alnum, char("_"))), end_text)
"#,
    )
    .unwrap();

    let output = rx()
        .args(["emit", path.to_str().unwrap()])
        .output()
        .expect("run rx emit multi file");

    let _ = std::fs::remove_file(path);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "path_piece: [A-Za-z0-9._/-]+\nidentifier: ^[A-Za-z_][A-Za-z0-9_]*$\n"
    );
}

#[test]
fn check_reports_malformed_file_location() {
    let path = std::env::temp_dir().join(format!("rx-cli-test-{}-bad.rx", std::process::id()));
    std::fs::write(
        &path,
        r#"pattern path_piece =
    one_or_more(
        set(ascii.unknown)
    )
"#,
    )
    .unwrap();

    let output = rx()
        .args(["check", path.to_str().unwrap()])
        .output()
        .expect("run rx check malformed file");

    let _ = std::fs::remove_file(path);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to parse readable rx"));
    assert!(stderr.contains("error compatibility in readable rx at line"));
    assert!(stderr.contains("unsupported set item `ascii.unknown`"));
}

#[test]
fn explain_reports_actionable_parse_errors() {
    let output = rx()
        .args(["explain", r#"(\w+)\1"#])
        .output()
        .expect("run rx explain failure");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to parse legacy regex"));
    assert!(stderr.contains("error compatibility in legacy regex"));
    assert!(stderr.contains("backreferences are outside the MVP safe core"));
    assert!(stderr.contains("suggestion:"));
}

#[test]
fn emit_reports_actionable_readable_parse_errors() {
    let output = rx()
        .args(["emit", r#"set(ascii.unknown)"#])
        .output()
        .expect("run rx emit failure");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to parse readable rx"));
    assert!(stderr.contains("error compatibility in readable rx"));
}

#[test]
fn emit_accepts_target_alias_for_pcre2() {
    let output = rx()
        .args([
            "emit",
            r#"named_capture("id", one_or_more(set(ascii.digit)))"#,
            "--target",
            "pcre2",
        ])
        .output()
        .expect("run rx emit pcre2");

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "(?<id>[0-9]+)\n");
}

#[test]
fn emit_reports_dialect_feature_errors() {
    let output = rx()
        .args([
            "emit",
            r#"named_capture("id", literal("x"))"#,
            "--dialect",
            "posix-ere",
        ])
        .output()
        .expect("run rx emit posix failure");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("cannot emit for PosixEre"));
    assert!(stderr.contains("NamedCapture"));
}

#[test]
fn emit_reports_unknown_dialects_before_parsing_input() {
    let output = rx()
        .args(["emit", r#"literal("x")"#, "--dialect", "javascript"])
        .output()
        .expect("run rx emit unknown dialect");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("unsupported dialect `javascript`"));
    assert!(stderr.contains("rust-regex for full 0.1 support"));
}

#[test]
fn commands_report_usage_for_wrong_arity() {
    let output = rx()
        .args(["emit", r#"literal("x")"#, "--dialect"])
        .output()
        .expect("run rx emit wrong arity");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("usage:"));
    assert!(stderr.contains("rx convert '<legacy-regex>'"));
    assert!(stderr.contains("rx emit '<readable-rx>'"));
}

#[test]
fn check_reports_file_read_errors() {
    let missing =
        std::env::temp_dir().join(format!("rx-cli-test-{}-missing.rx", std::process::id()));
    let _ = std::fs::remove_file(&missing);

    let output = rx()
        .args(["check", missing.to_str().unwrap()])
        .output()
        .expect("run rx check missing file");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to read"));
    assert!(stderr.contains(missing.to_str().unwrap()));
}
