# axi-rs

A Rust implementation of the **AXI** agent-facing CLI protocol: command-first
dispatch, a home/ambient view for discovery, structured errors with actionable
suggestions, and **TOON** output the agent reads.

This is a clean-room reimplementation of the AXI convention (originally
[`axi-sdk-js`](https://github.com/kunchenguid/axi) by Kun Chen). We keep the
protocol and drop the JS dependency along with its built-in self-update command.
The TOON output format is [Token-Oriented Object Notation](https://github.com/toon-format/spec).

## Status

Early scaffold. First slice landed:

- `toon` — encode-only TOON (`serde_json::Value` → TOON string): objects, nested
  objects, tabular uniform-object arrays, scalar arrays, list fallback.
- `error` — `AxiError` with actionable suggestions and stable per-class exit codes.
- `cli` — `App`: command-first dispatch, home view, `--help`/`--version`, no
  reserved self-update command.

Not yet built: the surface backend (axum server + state), full command surface,
the opt-in session-start hook installer, and a TOON conformance pass against
SPEC.md v4.1.

## Design record

Decisions live in `cadre-context` (Hololoop-AI), not here:

- `decisions/2026-08-29-review-surface-rust-backend-keep-axi-protocol.md`
- `decisions/2026-08-29-axi-rs-design-calls.md` — hooks, crate name, TOON
  everywhere (no fallback), slim ambient blob.
- `research/2026-08-29-toon-output-format.md`
- `discussions/2026-08-29-axi-as-first-party-tool-standard.md`

## Layout

```
crates/axi-rs/     the protocol crate (lib name: axi)
  src/toon.rs      encode-only TOON
  src/error.rs     AxiError + exit codes
  src/cli.rs       App / command-first dispatch
```

## Develop

```sh
cargo test
```

## License

MIT. The AXI protocol is credited to Kun Chen; TOON to the toon-format project.
