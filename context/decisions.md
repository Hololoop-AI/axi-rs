# Decisions

Decisions are made and locked on review pages; copies a codebase obeys live in
the nearest `context/` folder. The entries below
were first written in the retired `cadre-context` repo (Hololoop-AI on GitHub; history only, not authority); the summaries here are what
still binds axi-rs.

- `decisions/2026-08-29-review-surface-rust-backend-keep-axi-protocol.md` —
  backend goes Rust; keep the AXI protocol; drop `axi-sdk-js` and its
  self-update command.
- `decisions/2026-08-29-axi-rs-design-calls.md` — hooks kept + opt-in via
  `setup hooks`; crate name `axi-rs`; **TOON everywhere, no fallback**; slim the
  ambient/home blob while command results stay rich.
- `research/2026-08-29-toon-output-format.md` — TOON evidence, the multi-turn
  cascade risk the driver accepted, and the "write our own encoder" call.
- `discussions/2026-08-29-axi-as-first-party-tool-standard.md` — AXI as the shape
  of every agent-tool we ship (thesis, scoped: interop with existing MCP/CLI
  stays).

Decisions that are purely local to this crate live in the relevant
`context/*.md` topic file or the owning spec's `## Notes`.
