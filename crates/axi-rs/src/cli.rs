//! Command-first CLI dispatch — the `runAxiCli` equivalent.
//!
//! Shape: `<bin> <command> [args...] [--flags]`. With no command, the home view
//! prints (the ambient/discovery output). `--help`/`--version` are handled up
//! front. An unknown command returns a structured [`AxiError`], not a panic.
//!
//! The testable seam is [`App::execute`], which computes an [`Outcome`]
//! (`stdout`, `stderr`, `code`) without touching the process streams. [`App::run`]
//! is a thin printer over it, so behavior can be asserted without spawning a
//! subprocess.
//!
//! We deliberately reserve NO `update` self-update command (cadre-context
//! decisions/2026-08-29-review-surface-rust-backend-keep-axi-protocol): the AXI
//! JS framework shipped one for free; we do not.

use serde_json::Value;

use crate::error::{exit_code_for, AxiError};
use crate::toon;

/// A command handler: receives the args that follow the command name, returns a
/// value to encode or a structured error.
pub type CommandFn = Box<dyn Fn(&[String]) -> Result<Value, AxiError>>;

/// A registered command: name, one-line summary (shown in the slim `--help`
/// index), and its handler.
struct Command {
    name: String,
    summary: String,
    handler: CommandFn,
}

/// The computed result of an invocation: what would go to stdout, to stderr, and
/// the process exit code — without actually writing to those streams.
pub struct Outcome {
    pub stdout: String,
    pub stderr: String,
    pub code: i32,
}

/// One agent-facing CLI built on the AXI protocol.
pub struct App {
    pub name: String,
    pub description: String,
    pub version: String,
    home: Box<dyn Fn() -> Value>,
    commands: Vec<Command>,
}

/// What a parsed invocation resolves to, before any handler runs.
enum Dispatch<'a> {
    Version,
    Help,
    Home,
    Command(&'a str, Vec<String>),
}

impl App {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        App {
            name: name.into(),
            description: description.into(),
            version: version.into(),
            home: Box::new(|| Value::Null),
            commands: Vec::new(),
        }
    }

    /// Set the home/ambient view printed when no command is given.
    pub fn home(mut self, f: impl Fn() -> Value + 'static) -> Self {
        self.home = Box::new(f);
        self
    }

    /// Register a command handler with a one-line summary for the slim help index.
    pub fn command(
        mut self,
        name: impl Into<String>,
        summary: impl Into<String>,
        f: impl Fn(&[String]) -> Result<Value, AxiError> + 'static,
    ) -> Self {
        self.commands.push(Command {
            name: name.into(),
            summary: summary.into(),
            handler: Box::new(f),
        });
        self
    }

    /// Register the opt-in `setup hooks` command wired to the hook installer.
    pub fn setup_hooks(self) -> Self {
        self.command(
            "setup",
            "install opt-in session-start hooks (setup hooks)",
            crate::hooks::setup_command,
        )
    }

    /// Compute the [`Outcome`] for an invocation without touching stdout/stderr.
    pub fn execute(&self, argv: &[String]) -> Outcome {
        match self.parse(argv) {
            Dispatch::Version => Outcome {
                stdout: format!("{} {}", self.name, self.version),
                stderr: String::new(),
                code: 0,
            },
            Dispatch::Help => Outcome {
                stdout: self.help_text(),
                stderr: String::new(),
                code: 0,
            },
            Dispatch::Home => Outcome {
                stdout: encode_value(&(self.home)()),
                stderr: String::new(),
                code: 0,
            },
            Dispatch::Command(name, rest) => match self.lookup(name) {
                Some(cmd) => match (cmd.handler)(&rest) {
                    Ok(value) => Outcome {
                        stdout: encode_value(&value),
                        stderr: String::new(),
                        code: 0,
                    },
                    Err(err) => Outcome {
                        stdout: String::new(),
                        stderr: toon::encode(&err.to_value()),
                        code: exit_code_for(&err),
                    },
                },
                None => {
                    let err = AxiError::not_found(format!("Unknown command: {name}"))
                        .suggest(format!("Run `{} --help` for available commands", self.name));
                    Outcome {
                        stdout: String::new(),
                        stderr: toon::encode(&err.to_value()),
                        code: exit_code_for(&err),
                    }
                }
            },
        }
    }

    /// Run against argv (excluding the binary name); prints the outcome and
    /// returns the process exit code. Thin printer over [`App::execute`].
    pub fn run(&self, argv: &[String]) -> i32 {
        let outcome = self.execute(argv);
        if !outcome.stdout.is_empty() {
            println!("{}", outcome.stdout);
        }
        if !outcome.stderr.is_empty() {
            eprintln!("{}", outcome.stderr);
        }
        outcome.code
    }

    fn lookup(&self, name: &str) -> Option<&Command> {
        self.commands.iter().find(|c| c.name == name)
    }

    /// The slim `--help` index: name, description, and each command's summary.
    /// Deliberately excludes the fat ambient/home payload.
    fn help_text(&self) -> String {
        let mut out = format!("{} — {}\n", self.name, self.description);
        if self.commands.is_empty() {
            return out;
        }
        out.push_str("\nCommands:\n");
        let width = self
            .commands
            .iter()
            .map(|c| c.name.len())
            .max()
            .unwrap_or(0);
        for cmd in &self.commands {
            out.push_str(&format!(
                "  {:width$}  {}\n",
                cmd.name,
                cmd.summary,
                width = width
            ));
        }
        out
    }

    fn parse<'a>(&self, argv: &'a [String]) -> Dispatch<'a> {
        // Flags short-circuit before any command runs: version, then help.
        if argv
            .iter()
            .any(|a| a == "--version" || a == "-v" || a == "-V")
        {
            return Dispatch::Version;
        }
        if argv.iter().any(|a| a == "--help" || a == "-h") {
            return Dispatch::Help;
        }
        // First non-flag token is the command; everything else (flags and later
        // positionals) passes through to the handler.
        match argv.iter().position(|a| !a.starts_with('-')) {
            None => Dispatch::Home,
            Some(idx) => {
                let mut rest: Vec<String> = Vec::with_capacity(argv.len() - 1);
                rest.extend_from_slice(&argv[..idx]);
                rest.extend_from_slice(&argv[idx + 1..]);
                Dispatch::Command(&argv[idx], rest)
            }
        }
    }
}

/// Encode a value as TOON, treating null as empty output (no home/result to show).
fn encode_value(value: &Value) -> String {
    if value.is_null() {
        return String::new();
    }
    toon::encode(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn app() -> App {
        App::new("axi", "test app", "0.0.1")
            .home(|| json!({"ready": true}))
            .command("echo", "echo the args back", |args| Ok(json!({"args": args})))
    }

    #[test]
    fn no_args_is_home() {
        assert!(matches!(app().parse(&[]), Dispatch::Home));
    }

    #[test]
    fn version_flag_short_circuits() {
        let a = app();
        assert!(matches!(a.parse(&["--version".into()]), Dispatch::Version));
        assert!(matches!(a.parse(&["-v".into()]), Dispatch::Version));
    }

    #[test]
    fn help_flag_dispatches_to_help() {
        let a = app();
        assert!(matches!(a.parse(&["--help".into()]), Dispatch::Help));
        assert!(matches!(a.parse(&["-h".into()]), Dispatch::Help));
    }

    #[test]
    fn command_first_with_passthrough_args() {
        let a = app();
        let argv = vec!["echo".to_string(), "hello".to_string(), "--loud".to_string()];
        match a.parse(&argv) {
            Dispatch::Command(name, rest) => {
                assert_eq!(name, "echo");
                assert_eq!(rest, vec!["hello".to_string(), "--loud".to_string()]);
            }
            _ => panic!("expected a command dispatch"),
        }
    }

    #[test]
    fn unknown_command_is_not_found() {
        let outcome = app().execute(&["nope".to_string()]);
        assert_eq!(outcome.code, 3);
        assert!(outcome.stderr.contains("code: NOT_FOUND"));
    }

    #[test]
    fn known_command_succeeds() {
        let outcome = app().execute(&["echo".to_string(), "hi".to_string()]);
        assert_eq!(outcome.code, 0);
        assert_eq!(outcome.stdout.trim(), "args[1]: hi");
    }

    #[test]
    fn help_lists_commands_without_home_payload() {
        let outcome = app().execute(&["--help".to_string()]);
        assert_eq!(outcome.code, 0);
        assert!(outcome.stdout.contains("echo the args back"));
        assert!(!outcome.stdout.contains("ready"));
    }
}
