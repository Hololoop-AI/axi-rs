//! Opt-in session-start hook installer — locked integration contract.
//!
//! Source of truth: `spec/axi-protocol-prototype.md` § C "Test contract
//! (against a temp `$HOME`)".
//!
//! Testable seam: `axi::hooks::install(home: &Path) -> Result<Outcome, AxiError>`,
//! where the home directory is injected (mirroring the JS `homeDir` option) so
//! these tests run entirely against a `tempfile::tempdir()` sandbox and never
//! touch the developer's real `$HOME`. Build wires the `setup hooks` command to
//! call this function with the real HOME.
//!
//! The returned `Outcome` exposes `written: Vec<PathBuf>` (files the installer
//! created or merged its managed entry into) and `skipped: Vec<PathBuf>` (files
//! it refused to clobber because they exist without its managed marker).
//!
//! `axi::hooks::MARKER` is the managed-entry marker string. Managed-ness is
//! determined by the marker being present in a file's CONTENT. The OpenCode
//! plugin is written wholesale to `~/.config/opencode/plugins/axi-<MARKER>.js`.
//!
//! This file will not compile until that seam exists; that is the intended red
//! state.

use std::fs;
use std::path::{Path, PathBuf};

fn claude_settings(home: &Path) -> PathBuf {
    home.join(".claude").join("settings.json")
}

fn codex_hooks(home: &Path) -> PathBuf {
    home.join(".codex").join("hooks.json")
}

fn codex_config(home: &Path) -> PathBuf {
    home.join(".codex").join("config.toml")
}

fn opencode_plugins_dir(home: &Path) -> PathBuf {
    home.join(".config").join("opencode").join("plugins")
}

fn opencode_plugin(home: &Path) -> PathBuf {
    opencode_plugins_dir(home).join(format!("axi-{}.js", axi::hooks::MARKER))
}

/// Every file (not directory) under `dir`, recursively.
fn collect_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !dir.exists() {
        return out;
    }
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(collect_files(&path));
        } else {
            out.push(path);
        }
    }
    out
}

#[test]
fn when_installing_on_clean_home_then_writes_all_three_harnesses_and_succeeds() {
    // Arrange
    let home = tempfile::tempdir().unwrap();
    let home = home.path();

    // Act
    let outcome = axi::hooks::install(home).expect("install on a clean HOME should succeed");

    // Assert — Claude Code: one managed SessionStart command hook.
    let settings = claude_settings(home);
    assert!(settings.exists(), "expected {}", settings.display());
    let settings_text = fs::read_to_string(&settings).unwrap();
    serde_json::from_str::<serde_json::Value>(&settings_text).expect("settings.json must be JSON");
    assert_eq!(
        settings_text.matches("SessionStart").count(),
        1,
        "exactly one managed SessionStart hook"
    );
    assert!(settings_text.contains(axi::hooks::MARKER));

    // Assert — Codex: hooks.json + config.toml entries.
    assert!(codex_hooks(home).exists());
    assert!(!fs::read_to_string(codex_hooks(home)).unwrap().is_empty());
    assert!(codex_config(home).exists());
    assert!(!fs::read_to_string(codex_config(home)).unwrap().is_empty());

    // Assert — OpenCode: plugin file under the plugins dir, carrying the marker.
    let plugin = opencode_plugin(home);
    assert!(plugin.exists(), "expected {}", plugin.display());
    assert!(fs::read_to_string(&plugin).unwrap().contains(axi::hooks::MARKER));

    // Assert — the installer reported the files it wrote.
    assert!(!outcome.written.is_empty());
}

#[test]
fn when_installing_twice_then_second_run_is_a_no_op_with_no_diff() {
    // Arrange — install once, snapshot every created file's bytes.
    let home = tempfile::tempdir().unwrap();
    let home = home.path();
    axi::hooks::install(home).expect("first install should succeed");

    let before: Vec<(PathBuf, Vec<u8>)> = collect_files(home)
        .into_iter()
        .map(|p| {
            let bytes = fs::read(&p).unwrap();
            (p, bytes)
        })
        .collect();

    // Act — run it again.
    axi::hooks::install(home).expect("second install should succeed");

    // Assert — identical file set and identical bytes: a true no-op.
    let after: Vec<(PathBuf, Vec<u8>)> = collect_files(home)
        .into_iter()
        .map(|p| {
            let bytes = fs::read(&p).unwrap();
            (p, bytes)
        })
        .collect();

    let mut before_sorted = before;
    let mut after_sorted = after;
    before_sorted.sort_by(|a, b| a.0.cmp(&b.0));
    after_sorted.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        before_sorted, after_sorted,
        "a second install must not change any file"
    );
}

#[test]
fn when_settings_json_has_unmanaged_hooks_then_they_are_preserved_alongside_managed() {
    // Arrange — a pre-existing settings.json owned by the user (no marker).
    let home = tempfile::tempdir().unwrap();
    let home = home.path();
    fs::create_dir_all(home.join(".claude")).unwrap();
    let unmanaged = r#"{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"USER_OWNED_HOOK"}]}]}}"#;
    fs::write(claude_settings(home), unmanaged).unwrap();

    // Act
    axi::hooks::install(home).expect("install should succeed and merge");

    // Assert — the user's hook survives, and the managed entry is now present too.
    let text = fs::read_to_string(claude_settings(home)).unwrap();
    serde_json::from_str::<serde_json::Value>(&text).expect("settings.json stays valid JSON");
    assert!(
        text.contains("USER_OWNED_HOOK"),
        "unmanaged hook must be preserved"
    );
    assert!(
        text.contains(axi::hooks::MARKER),
        "managed entry must be added alongside"
    );
}

#[test]
fn when_target_file_exists_without_marker_then_it_is_reported_not_overwritten() {
    // Arrange — the OpenCode plugin path already holds a user file with no marker.
    let home = tempfile::tempdir().unwrap();
    let home = home.path();
    fs::create_dir_all(opencode_plugins_dir(home)).unwrap();
    let sentinel = "// USER PLUGIN — NOT OURS\n";
    let plugin = opencode_plugin(home);
    fs::write(&plugin, sentinel).unwrap();

    // Act
    let outcome = axi::hooks::install(home).expect("install should succeed while refusing");

    // Assert — the unmanaged file is untouched and reported as skipped.
    assert_eq!(
        fs::read_to_string(&plugin).unwrap(),
        sentinel,
        "an unmanaged file must not be clobbered"
    );
    assert!(
        outcome.skipped.iter().any(|p| p == &plugin),
        "the refused file must be reported in `skipped`"
    );
}

#[test]
fn when_installing_then_it_writes_only_within_its_three_managed_subtrees() {
    // Arrange
    let home = tempfile::tempdir().unwrap();
    let home = home.path();

    // Act
    axi::hooks::install(home).expect("install should succeed");

    // Assert — nothing is written to HOME outside .claude, .codex, and
    // .config/opencode; the installer touches only what it owns.
    for file in collect_files(home) {
        let rel = file.strip_prefix(home).unwrap();
        let ok = rel.starts_with(".claude")
            || rel.starts_with(".codex")
            || rel.starts_with(Path::new(".config").join("opencode"));
        assert!(ok, "installer wrote outside its managed subtrees: {}", rel.display());
    }
}
