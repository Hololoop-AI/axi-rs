# axi-rs

A Rust implementation of the **AXI** agent-facing CLI protocol: command-first
dispatch, a home/ambient view for discovery, structured errors with actionable
suggestions, and **TOON** output the agent reads.

This is a clean-room reimplementation of the AXI convention (originally
[`axi-sdk-js`](https://github.com/kunchenguid/axi) by Kun Chen). We keep the
protocol and drop the JS dependency along with its built-in self-update command.
The TOON output format is [Token-Oriented Object Notation](https://github.com/toon-format/spec).

## Status

`0.1` — the protocol crate, pre-1.0 (the API may still change between minor
versions):

- `toon` — encode-only TOON (`serde_json::Value` → TOON string), conformant to
  the TOON SPEC v4.1 core forms.
- `error` — `AxiError` with actionable suggestions and stable per-class exit codes.
- `cli` — `App`: command-first dispatch, home view, `--help`/`--version`, no
  reserved self-update command.
- `hooks` — opt-in installer (`setup hooks`) that registers a session-start
  hook for Claude Code, Codex and OpenCode; marker-managed and idempotent.

## Use

```toml
[dependencies]
axi-rs = "0.1"
```

The library is imported as `axi` (`use axi::cli::App;`).

## Design record

Design decisions that bind this crate are summarised in
[`context/decisions.md`](context/decisions.md).

## Layout

```
crates/axi-rs/     the protocol crate (lib name: axi)
  src/toon.rs      encode-only TOON
  src/error.rs     AxiError + exit codes
  src/cli.rs       App / command-first dispatch
  src/hooks.rs     opt-in session-start hook installer
```

## Develop

```sh
cargo test
```

## License

MIT. The AXI protocol is credited to Kun Chen; TOON to the toon-format project.
