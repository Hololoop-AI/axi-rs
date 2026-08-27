//! Command-first CLI dispatch — the `runAxiCli` equivalent.
//!
//! Shape: `<bin> <command> [args...] [--flags]`. With no command, the home view
//! prints (the ambient/discovery output). `--help`/`--version` are handled up
//! front. An unknown command returns a structured [`AxiError`], not a panic.
//!
//! We deliberately reserve NO `update` self-update command (cadre-context
//! decisions/2026-08-29-review-surface-rust-backend-keep-axi-protocol): the AXI
//! JS framework shipped one for free; we do not.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::error::{exit_code_for, AxiError};
use crate::toon;

/// A command handler: receives the args that follow the command name, returns a
/// value to encode or a structured error.
pub type CommandFn = Box<dyn Fn(&[String]) -> Result<Value, AxiError>>;

/// One agent-facing CLI built on the AXI protocol.
pub struct App {
    pub name: String,
    pub description: String,
    pub version: String,
    home: Box<dyn Fn() -> Value>,
    commands: BTreeMap<String, CommandFn>,
}

/// What a parsed invocation resolves to, before any handler runs.
enum Dispatch<'a> {
    Version,
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
            commands: BTreeMap::new(),
        }
    }

    /// Set the home/ambient view printed when no command is given.
    pub fn home(mut self, f: impl Fn() -> Value + 'static) -> Self {
        self.home = Box::new(f);
        self
    }

    /// Register a command handler.
    pub fn command(
        mut self,
        name: impl Into<String>,
        f: impl Fn(&[String]) -> Result<Value, AxiError> + 'static,
    ) -> Self {
        self.commands.insert(name.into(), Box::new(f));
        self
    }

    /// Run against argv (excluding the binary name); returns a process exit code.
    pub fn run(&self, argv: &[String]) -> i32 {
        match self.parse(argv) {
            Dispatch::Version => {
                println!("{} {}", self.name, self.version);
                0
            }
            Dispatch::Home => {
                print_value(&(self.home)());
                0
            }
            Dispatch::Command(name, rest) => match self.commands.get(name) {
                Some(handler) => match handler(&rest) {
                    Ok(value) => {
                        print_value(&value);
                        0
                    }
                    Err(err) => {
                        print_error(&err);
                        exit_code_for(&err)
                    }
                },
                None => {
                    let err = AxiError::not_found(format!("Unknown command: {name}"))
                        .suggest(format!("Run `{} --help` for available commands", self.name));
                    print_error(&err);
                    exit_code_for(&err)
                }
            },
        }
    }

    fn parse<'a>(&self, argv: &'a [String]) -> Dispatch<'a> {
        if argv
            .iter()
            .any(|a| a == "--version" || a == "-v" || a == "-V")
        {
            return Dispatch::Version;
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

fn print_value(value: &Value) {
    if value.is_null() {
        return;
    }
    println!("{}", toon::encode(value));
}

fn print_error(err: &AxiError) {
    eprintln!("{}", toon::encode(&err.to_value()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn app() -> App {
        App::new("axi", "test app", "0.0.1")
            .home(|| json!({"ready": true}))
            .command("echo", |args| Ok(json!({"args": args})))
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
        assert_eq!(app().run(&["nope".to_string()]), 3);
    }

    #[test]
    fn known_command_succeeds() {
        assert_eq!(app().run(&["echo".to_string(), "hi".to_string()]), 0);
    }
}
