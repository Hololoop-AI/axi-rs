# Session-start hook installer
status: planned

An opt-in `setup hooks` command that registers a session-start hook so agent
harnesses inject the CLI's home/ambient output at session start (tool discovery
without being told). We own the installer deliberately: it never runs implicitly,
and its `$HOME` writes are auditable.

## Behavior (intended)

- Runs only on explicit `setup hooks` — never on first run or any other command.
- Idempotent and marker-managed: only touches its own entries, refuses to
  overwrite unmanaged files, repairs a stale path in place.
- Targets (from the JS behavior, to confirm during slicing): Claude Code
  (`~/.claude/settings.json`), Codex (`~/.codex/hooks.json` + `config.toml`),
  OpenCode (a generated plugin under `~/.config/opencode/plugins/`).
- The injected payload is the **slim** ambient index (see cli-dispatch), not the
  fat blob.

## Notes

- Not built. This is the surface the security audits flagged as high-leverage;
  owning it (vs. inheriting the JS framework's freebie) is the point.
- Open: do we support all three harnesses in the first cut, or start with Claude
  Code only and add the others behind evidence? Decide during slicing.

## Integration Test Contract

_(pending test-planning)_

## Tests

_(pending test-writer)_
