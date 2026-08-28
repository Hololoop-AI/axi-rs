# CLI dispatch
status: in-progress

Command-first routing for an AXI CLI: `<bin> <command> [args] [--flags]`. The
first non-flag token is the command; everything else passes through to its
handler. Results serialize to TOON; errors print the structured error shape.

## Behavior

- No command → the home/ambient view (`home()` output), TOON-serialized.
- `--version` / `-v` / `-V` → `<name> <version>`, exit 0, before any dispatch.
- `--help` → static top-level guidance (the slim ambient index, not the fat
  blob). _(help output not yet built — see Notes.)_
- Known command → run handler; Ok(value) prints TOON, exit 0; Err prints the
  error to stderr, exit = per-class code.
- Unknown command → `NOT_FOUND` error with a `--help` suggestion, exit 3.
- No reserved `update` (or any other) self-command.

## Notes

- Built: `App` with `home`/`command` registration, command-first `parse`,
  version short-circuit, unknown-command error, TOON output (`src/cli.rs`, unit
  tests for parse/dispatch/exit codes).
- **Open (why in-progress):** `--help` output is not implemented; the home/help
  payload must be the *slim, progressive* index, not the fat AXI ambient blob
  (cadre-context decisions/2026-08-29-axi-rs-design-calls). Needs a decision on
  the help shape, then a contract.

## Integration Test Contract

_(pending test-planning — cover: no-args prints home TOON; version short-circuit;
help prints the slim index; known command round-trips a value to TOON on stdout;
unknown command → exit 3 + structured error on stderr.)_

## Tests

_(pending test-writer)_
