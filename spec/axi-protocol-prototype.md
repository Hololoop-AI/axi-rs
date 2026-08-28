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
- Writes managed entries to all three harnesses (driver pulled OpenCode into
  this batch): Claude Code `~/.claude/settings.json`, Codex `~/.codex/hooks.json`
  + `config.toml`, and the OpenCode plugin under
  `~/.config/opencode/plugins/axi-<marker>.js`.
- Marker-managed + idempotent: only touches its own entries; refuses to
  overwrite an unmanaged file; a second run is a no-op; a stale path is repaired.

### Test contract (against a temp `$HOME`)

- `setup hooks` on a clean temp HOME creates `~/.claude/settings.json` with one
  managed `SessionStart` command hook running this binary; writes the Codex
  `hooks.json` + `config.toml` entries; and writes the OpenCode plugin file under
  `~/.config/opencode/plugins/`; exit 0.
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
- Approved on the Review Surface (2026-08-28): build A+B+C as one batch; OpenCode
  hook target pulled in; advanced TOON forms (nested tabular columns,
  keyed-tabular-root, tab/pipe delimiters) stay deferred.

## Tests

Locked integration tests (test-writer). Build implements against these and must
not modify them.

**A. TOON encoding** — `crates/axi-rs/tests/toon_conformance.rs`
- § `"toon_encodes_spec_v41_fixture_table"` — data-driven over all 12 rows of § A
  "Test contract — fixtures"; asserts each input JSON → expected TOON verbatim
  (encounter order, empty-array `key: []`, inline-dash list form, number
  canonicalization, tightened quoting).

**B. CLI dispatch + slim help** — `crates/axi-rs/tests/cli_dispatch.rs`
- § `"when_no_args_then_prints_home_as_toon_on_stdout_exit_0"` — covers § B "no-args → home() as TOON, exit 0".
- § `"when_version_flag_then_prints_name_and_version_exit_0_and_runs_no_command"` — covers § B "--version".
- § `"when_help_flag_then_prints_slim_index_with_command_summaries_and_no_ambient_blob"` — covers § B "--help slim index; fat blob absent".
- § `"when_registered_command_runs_then_round_trips_args_to_toon_stdout_exit_0"` — covers § B "echo round-trip".
- § `"when_unknown_command_then_exit_3_and_stderr_is_toon_not_found_error_with_suggestion"` — covers § B "unknown command → NOT_FOUND, exit 3".

**C. Hook installer** — `crates/axi-rs/tests/hook_installer.rs`
- § `"when_installing_on_clean_home_then_writes_all_three_harnesses_and_succeeds"` — covers § C clean-install (Claude/Codex/OpenCode).
- § `"when_installing_twice_then_second_run_is_a_no_op_with_no_diff"` — covers § C idempotent second run.
- § `"when_settings_json_has_unmanaged_hooks_then_they_are_preserved_alongside_managed"` — covers § C unmanaged-hooks preserved.
- § `"when_target_file_exists_without_marker_then_it_is_reported_not_overwritten"` — covers § C refuse-to-clobber (reported, not overwritten).
- § `"when_installing_then_it_writes_only_within_its_three_managed_subtrees"` — covers § C "no other/implicit writes to $HOME" (enforced as scoped-writes).

### Seams build must implement
- `App::execute(&[String]) -> Outcome { stdout: String, stderr: String, code: i32 }`
  (public fields); `run()` becomes a thin printer over it.
- `App::command(name, summary, handler)` — commands carry a one-line summary for
  the slim `--help` index.
- `serde_json` `preserve_order` feature enabled on the crate dependency (encounter
  order + tabular column order depend on it).
- `axi::hooks` module: `pub const MARKER: &str`; `pub fn install(home: &Path) ->
  Result<Outcome, AxiError>` with `Outcome { written: Vec<PathBuf>, skipped:
  Vec<PathBuf> }`; content-marker-based managed-ness; OpenCode plugin written to
  `~/.config/opencode/plugins/axi-<MARKER>.js`.
