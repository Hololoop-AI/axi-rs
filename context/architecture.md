# Architecture

axi-rs is the Rust reimplementation of the **AXI** agent-facing CLI protocol.
It replaces the JS `axi-sdk-js` dependency; the surface backend that consumes it
(the Rust rewrite of Review Surface's server/state layers) is a later, separate
crate in this same workspace.

## Layers

- **`crates/axi-rs`** — the protocol crate (lib name `axi`). Command-first
  dispatch, home/ambient view, structured errors, TOON output. Reusable by any
  agent-facing CLI we build.
- **`crates/axi-surface`** (future) — the axum server + state store: the Rust
  port of Review Surface's node backend (layers 1–3). Depends on `axi-rs`.
- **The web UI is out of scope for this repo.** Review Surface's layer 4 stays
  JavaScript (it reaches into the browser DOM); it is served/embedded, not
  ported. (Originally argued in the retired `cadre-context` repo (Hololoop-AI on GitHub; history only, not authority), questions/2026-08-27-surface-embedding-across-hosts.)

## Invariants (do not break without a decision)

- **TOON everywhere, no output fallback.** Command results serialize to TOON.
  The encoder's *source* is a `serde_json::Value` (internal model kept for free),
  but JSON is never emitted as the agent-facing format.
- **No reserved self-update command.** The AXI JS framework shipped a free
  `update` self-updater; we deliberately do not. Nothing writes to `$HOME`
  implicitly.
- **Encode-only TOON.** The agent reads TOON, never writes it, so no decoder.
- **Structured errors carry actionable suggestions + stable per-class exit
  codes** (validation=2, not-found=3, server=1).

## Why (single source)

The cross-project rationale — why Rust, why keep AXI, why TOON everywhere — is
summarised in [decisions.md](decisions.md).
