# Conventions

## Workflow

- **TDD.** Test contract in the slice spec → red test (test-writer) → implement
  (build). `cargo test` is the gate; a slice isn't built until the full suite is
  green.
- **Spec-as-source.** Behavior lives in `spec/`, architectural truth in
  `context/`, git is the changelog. A durable fact lives in exactly one layer.
- **No AI co-authorship** on commits or PRs.

## Rust

- Edition 2021, workspace-pinned. Dependencies stay minimal (`serde_json` for the
  value model; `axum`/`tokio` arrive with the surface crate).
- **clippy / rustfmt are not installed on this Fedora system-rust.** No lint gate
  yet — adding a rustup toolchain + CI (fmt + clippy + test) is owed. Until then,
  hand-format to standard rustfmt style.

## Owed / known gaps

- **TOON conformance to SPEC.md v4.1** is not yet verified — the encoder
  implements the core shape only (`TODO(spec-conformance)` in `toon.rs`). The
  mixed-array list form and quoting rules need a spec pass before byte-exact
  output is trusted.
- A small **real-model read test** of TOON output (the driver's "trial and
  error") before hardening the format decision.
