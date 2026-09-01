# spec/ — axi-rs behavioral specs

System intent: a Rust library for building an agent-facing CLI on the AXI
protocol — command-first dispatch, a slim home/ambient discovery view, structured
errors, and TOON output. This repo is the protocol crate. `hitl`
(Hololoop-AI/hitl) — the human-in-the-loop report-up surface package — is a
separate repo that consumes it.

## Slice index

| Spec | Status | What it proves |
|---|---|---|
| [axi-protocol-prototype](axi-protocol-prototype.md) | built | TOON v4.1 conformance + CLI dispatch/help + opt-in hook installer |
