# spec/ — axi-rs behavioral specs

System intent: a Rust library for building an agent-facing CLI on the AXI
protocol — command-first dispatch, a slim home/ambient discovery view, structured
errors, and TOON output. This repo is the protocol crate; the surface backend
that consumes it is the next milestone (`crates/axi-surface`).

## Slice index

| Spec | Status | What it proves |
|---|---|---|
| [axi-protocol-prototype](axi-protocol-prototype.md) | in-progress | TOON v4.1 conformance + CLI dispatch/help + opt-in hook installer, built as one prototype batch |

Next milestone (no spec yet): `axi-surface` — axum server + state store (the Rust
port of Review Surface's node backend).
