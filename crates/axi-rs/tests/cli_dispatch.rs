//! CLI dispatch + slim help — locked integration contract.
//!
//! Source of truth: `spec/axi-protocol-prototype.md` § B "Test contract".
//!
//! These tests assert against a TESTABLE SEAM rather than process stdout:
//! `App::execute(&[String]) -> Outcome`, where `Outcome` exposes the public
//! fields `stdout: String`, `stderr: String`, and `code: i32`. Build refactors
//! the current `run()` (which prints directly) into `execute()` computing an
//! `Outcome`, with a thin `run()` that prints it and returns `code`.
//!
//! Commands are registered with a one-line summary — `App::command(name,
//! summary, handler)` — because the slim `--help` index lists each registered
//! command by summary. This file will not compile until that seam exists; that
//! is the intended red state.

use axi::App;
use serde_json::json;

/// A distinctive value returned by `home()` so tests can prove the slim `--help`
/// index does NOT echo the fat ambient/home payload.
const AMBIENT_SENTINEL: &str = "AMBIENT_BLOB_DO_NOT_SHOW_IN_HELP";

fn app() -> App {
    App::new("axi", "the axi agent-facing surface", "0.0.1")
        .home(|| json!({ "ready": true, "ambient": AMBIENT_SENTINEL }))
        .command("echo", "echo the args back as TOON", |args| {
            Ok(json!({ "args": args }))
        })
}

#[test]
fn when_no_args_then_prints_home_as_toon_on_stdout_exit_0() {
    // Arrange
    let app = app();

    // Act
    let outcome = app.execute(&[]);

    // Assert — home() encoded as TOON on stdout, clean exit, nothing on stderr.
    assert_eq!(outcome.code, 0);
    assert!(outcome.stdout.contains("ready: true"));
    assert!(outcome.stderr.trim().is_empty());
}

#[test]
fn when_version_flag_then_prints_name_and_version_exit_0_and_runs_no_command() {
    // Arrange — echo is registered; --version must short-circuit before it runs.
    let app = app();

    // Act
    let outcome = app.execute(&["--version".to_string()]);

    // Assert
    assert_eq!(outcome.code, 0);
    assert_eq!(outcome.stdout.trim(), "axi 0.0.1");
    // No command ran: echo's output shape ("args...") must be absent.
    assert!(!outcome.stdout.contains("args"));
    assert!(outcome.stderr.trim().is_empty());
}

#[test]
fn when_help_flag_then_prints_slim_index_with_command_summaries_and_no_ambient_blob() {
    // Arrange
    let app = app();

    // Act
    let outcome = app.execute(&["--help".to_string()]);

    // Assert — slim index carries name, description, and each command's summary.
    assert_eq!(outcome.code, 0);
    assert!(outcome.stdout.contains("axi"));
    assert!(outcome.stdout.contains("the axi agent-facing surface"));
    assert!(outcome.stdout.contains("echo"));
    assert!(outcome.stdout.contains("echo the args back as TOON"));
    // The fat ambient/home payload MUST NOT leak into the slim help index.
    assert!(!outcome.stdout.contains(AMBIENT_SENTINEL));
    assert!(outcome.stderr.trim().is_empty());
}

#[test]
fn when_registered_command_runs_then_round_trips_args_to_toon_stdout_exit_0() {
    // Arrange
    let app = app();

    // Act
    let outcome = app.execute(&["echo".to_string(), "a".to_string(), "b".to_string()]);

    // Assert — {"args":["a","b"]} encodes to the inline scalar-array TOON form.
    assert_eq!(outcome.code, 0);
    assert_eq!(outcome.stdout.trim(), "args[2]: a,b");
    assert!(outcome.stderr.trim().is_empty());
}

#[test]
fn when_unknown_command_then_exit_3_and_stderr_is_toon_not_found_error_with_suggestion() {
    // Arrange
    let app = app();

    // Act
    let outcome = app.execute(&["nope".to_string()]);

    // Assert — not-found class exits 3; the structured error goes to stderr as
    // TOON carrying `code: NOT_FOUND` and an actionable `--help` suggestion.
    assert_eq!(outcome.code, 3);
    assert!(outcome.stderr.contains("code: NOT_FOUND"));
    assert!(outcome.stderr.contains("--help"));
    assert!(outcome.stdout.trim().is_empty());
}
