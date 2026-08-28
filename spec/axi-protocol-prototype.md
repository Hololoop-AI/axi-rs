# axi-rs protocol crate — prototype
status: in-progress

One prototype milestone (not story-by-story): a working AXI protocol crate —
TOON output conformant to SPEC v4.1, command-first dispatch with a slim help,
and an opt-in session-start hook installer. Built as one batch, then the surface
backend (axum) becomes the next milestone.

## A. TOON encoding (conform to SPEC v4.1)

The encoder exists (`src/toon.rs`) but only implements the core shape. Bring it
to conformance:

- **Encounter order, not sorted.** TOON MUST preserve key insertion order.
  Requires `serde_json` with the `preserve_order` feature (IndexMap). Today keys
  sort alphabetically — a real bug the tests missed by using alphabetical keys.
- **Empty array** renders `key: []` (not the legacy `key[0]:`).
- **List form uses inline `- `**: scalar item `- 1`; object item first field on
  the dash line `- a: 1`, remaining fields aligned beneath.
- **Number canonicalization**: `1.0` → `1`, `1.5000` → `1.5`, `-0` → `0`, no
  exponent for 1e-6 ≤ |n| < 1e21.
- **Quoting** MUST fire on: empty, leading/trailing space/tab, `true|false|null`,
  numeric-like (`/^[+-]?[0-9]+(?:\.[0-9]+)?(?:e[+-]?[0-9]+)?$/i`), any of
  `: " \ [ ] { }`, control chars (→ `\uXXXX`), the active delimiter, and a
  leading `-` or `#`. Escapes: `\\ \" \n \r \t`, control → `\uXXXX`.

Deferred (note, don't build now): nested-object tabular columns
(`{id,customer{name,country}}`), keyed-tabular-root, tab/pipe delimiters.

### Test contract — fixtures (input JSON → expected TOON)

| Case | Input | Expected |
|---|---|---|
| encounter order | `{"name":"ada","id":1}` | `name: ada`⏎`id: 1` |
| nested object | `{"user":{"id":123,"name":"Ada"}}` | `user:`⏎`  id: 123`⏎`  name: Ada` |
| tabular | `{"items":[{"sku":"A1","qty":2},{"sku":"B2","qty":1}]}` | `items[2]{sku,qty}:`⏎`  A1,2`⏎`  B2,1` |
| scalar array | `{"tags":["admin","ops","dev"]}` | `tags[3]: admin,ops,dev` |
| empty array | `{"tags":[]}` | `tags: []` |
| list form | `{"items":[1,{"a":1},"text"]}` | `items[3]:`⏎`  - 1`⏎`  - a: 1`⏎`  - text` |
| number canon | `{"a":1.0,"b":1.5000}` | `a: 1`⏎`b: 1.5` |
| quote numeric-like | `{"v":"12"}` | `v: "12"` |
| quote leading dash | `{"v":"-x"}` | `v: "-x"` |
| quote delimiter | `{"msg":"a, b"}` | `msg: "a, b"` |
| null / bool | `{"a":null,"b":true}` | `a: null`⏎`b: true` |
| root scalar | `42` | `42` |

## B. CLI dispatch + slim help

Dispatch exists (`src/cli.rs`). Add help; keep the ambient/home payload slim.

- No command → `home()` output as TOON, exit 0.
- `--version`/`-v`/`-V` → `<name> <version>`, exit 0, before dispatch.
- `--help`/`-h` → the slim index (name, one-line description, command list with
  one-line summaries) — NOT the fat AXI ambient blob.
- Known command → handler; Ok→TOON on stdout exit 0; Err→structured error on
  stderr, exit = per-class code (validation 2, not-found 3, server 1).
- Unknown command → `NOT_FOUND` + a `--help` suggestion, exit 3.

### Test contract

- no-args prints `home()` as TOON on stdout, exit 0.
- `--version` prints `<name> <version>`, exit 0, and does not run a command.
- `--help` prints the name, description, and each registered command with its
  summary; asserts the fat/ambient content is absent.
- a registered `echo` command round-trips its args to TOON on stdout, exit 0.
- unknown command → exit 3, and stderr contains a TOON error with
  `code: NOT_FOUND` and a suggestion.

## C. Opt-in session-start hook installer

`setup hooks` registers a session-start hook so harnesses inject the CLI's slim
home output at session start. We own it: never implicit, auditable, marker-managed.

- Runs only on explicit `setup hooks`.
- Writes managed entries to (prototype target set — confirm on the surface):
  Claude Code `~/.claude/settings.json` and Codex `~/.codex/hooks.json` +
  `config.toml`. OpenCode plugin: deferred unless flagged.
- Marker-managed + idempotent: only touches its own entries; refuses to
  overwrite an unmanaged file; a second run is a no-op; a stale path is repaired.

### Test contract (against a temp `$HOME`)

- `setup hooks` on a clean temp HOME creates `~/.claude/settings.json` with one
  managed `SessionStart` command hook running this binary; exit 0.
- second `setup hooks` run is a no-op (no diff, exit 0).
- an existing `settings.json` with unmanaged hooks is preserved (managed entry
  added alongside, unmanaged untouched).
- a file whose managed marker is absent but that the installer would write is
  NOT clobbered — reported, not overwritten.
- no other command touches `$HOME`.

## Notes

- Prototype pace: build A+B+C as one batch, one build session, full `cargo test`
  green gate. Surface backend (axum) is the next milestone, separate spec.
- `preserve_order` on `serde_json` is a required Cargo change for A.
- OpenCode hook target and the deferred TOON forms are open — flag on the surface
  if they should be in-scope now.

## Tests

_(pending test-writer — this spec's contract is the source)_
