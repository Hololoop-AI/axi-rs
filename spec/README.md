# spec/ — axi-rs behavioral specs

System intent: a Rust library that lets you build an agent-facing CLI on the AXI
protocol — command-first dispatch, a home/ambient discovery view, structured
errors, and TOON output. This repo is the protocol crate; the surface backend
that consumes it is a future epic (`crates/axi-surface`, not yet specced).

## Slice index

| Spec | Status | What it proves |
|---|---|---|
| [toon-encoding](toon-encoding.md) | in-progress | JSON value → TOON string, conformant to SPEC v4.1 |
| [cli-dispatch](cli-dispatch.md) | in-progress | command-first routing, home/help/version, error output |
| [hook-installer](hook-installer.md) | planned | opt-in `setup hooks` install into agent configs |

Future (no spec yet): `axi-surface` — axum server + state store (the Rust port
of Review Surface's node backend).
