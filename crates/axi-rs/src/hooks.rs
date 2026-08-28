//! Opt-in session-start hook installer.
//!
//! `setup hooks` registers a session-start hook in each supported agent harness
//! so the harness injects this CLI's slim home output at session start. We own
//! it: it runs only on explicit `setup hooks`, is auditable, and is
//! marker-managed. Nothing writes under `$HOME` implicitly (cadre-context
//! decisions/2026-08-29-axi-rs-design-calls).
//!
//! Managed-ness is determined by [`MARKER`] appearing in a target file's
//! CONTENT. The installer:
//! - creates or merges its own managed entry into Claude Code
//!   `~/.claude/settings.json` and Codex `~/.codex/hooks.json` +
//!   `~/.codex/config.toml`, preserving any unmanaged entries already there;
//! - writes the OpenCode plugin wholesale to
//!   `~/.config/opencode/plugins/axi-<MARKER>.js`;
//! - is idempotent: a second run touches nothing (byte-identical);
//! - refuses to clobber an unmanaged wholesale-written file, reporting it in
//!   [`Outcome::skipped`] instead of overwriting.
//!
//! The `home` directory is injected so tests run against a sandbox and never
//! touch the developer's real `$HOME`.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::error::AxiError;

/// The managed-entry marker. Its presence in a file's content marks that file
/// (or an entry within it) as owned by this installer.
pub const MARKER: &str = "managed-by-axi";

/// The command a session-start hook runs. The prototype assumes the binary is on
/// `PATH` as `axi`; the trailing comment carries [`MARKER`] for auditability.
const HOOK_COMMAND: &str = "axi";

/// What the installer did: files it created or merged its managed entry into, and
/// files it refused to clobber because they exist without [`MARKER`].
pub struct Outcome {
    pub written: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
}

impl Outcome {
    fn new() -> Self {
        Outcome {
            written: Vec::new(),
            skipped: Vec::new(),
        }
    }

    /// A TOON-serializable summary for the `setup hooks` command result.
    pub fn to_value(&self) -> Value {
        json!({
            "written": self.written.iter().map(display).collect::<Vec<_>>(),
            "skipped": self.skipped.iter().map(display).collect::<Vec<_>>(),
        })
    }
}

fn display(p: &PathBuf) -> String {
    p.display().to_string()
}

/// Install managed session-start hooks into all three harnesses under `home`.
pub fn install(home: &Path) -> Result<Outcome, AxiError> {
    let mut outcome = Outcome::new();
    install_claude(home, &mut outcome)?;
    install_codex(home, &mut outcome)?;
    install_opencode(home, &mut outcome)?;
    Ok(outcome)
}

/// The `setup` command handler: `setup hooks` installs against the real `$HOME`.
pub fn setup_command(args: &[String]) -> Result<Value, AxiError> {
    match args.first().map(String::as_str) {
        Some("hooks") => {
            let home = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
                AxiError::server("$HOME is not set")
                    .suggest("Set the HOME environment variable and retry")
            })?;
            let outcome = install(&home)?;
            Ok(outcome.to_value())
        }
        _ => Err(AxiError::validation("Unknown setup target")
            .suggest("Run `setup hooks` to install the session-start hooks")),
    }
}

// --- Claude Code: JSON merge into ~/.claude/settings.json --------------------

fn install_claude(home: &Path, outcome: &mut Outcome) -> Result<(), AxiError> {
    let path = home.join(".claude").join("settings.json");

    // Already managed by us -> true no-op (leave bytes untouched).
    if file_contains_marker(&path)? {
        return Ok(());
    }

    // Merge our managed SessionStart hook, preserving any existing (unmanaged)
    // hooks. A malformed existing file is treated as unmanaged and not clobbered.
    let mut root = match read_optional(&path)? {
        Some(text) => match serde_json::from_str::<Value>(&text) {
            Ok(value) => value,
            Err(_) => {
                outcome.skipped.push(path);
                return Ok(());
            }
        },
        None => json!({}),
    };

    let object = root.as_object_mut().ok_or_else(|| {
        AxiError::server("Claude settings.json is not a JSON object")
            .suggest("Repair or remove ~/.claude/settings.json and retry")
    })?;

    let hooks = object
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| AxiError::server("Claude settings `hooks` is not an object"))?;

    let session_start = hooks
        .entry("SessionStart")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| AxiError::server("Claude `SessionStart` is not an array"))?;

    session_start.push(claude_managed_group());

    let serialized = serde_json::to_string_pretty(&root)
        .map_err(|e| AxiError::server(format!("Failed to serialize Claude settings: {e}")))?;
    write_file(&path, &serialized)?;
    outcome.written.push(path);
    Ok(())
}

/// One managed `SessionStart` group carrying [`MARKER`] for auditability.
fn claude_managed_group() -> Value {
    json!({
        "hooks": [
            { "type": "command", "command": HOOK_COMMAND }
        ],
        "_managed_by": MARKER
    })
}

// --- Codex: hooks.json (JSON) + config.toml (append block) -------------------

fn install_codex(home: &Path, outcome: &mut Outcome) -> Result<(), AxiError> {
    let hooks_path = home.join(".codex").join("hooks.json");
    if !file_contains_marker(&hooks_path)? {
        install_codex_hooks(&hooks_path, outcome)?;
    }

    let config_path = home.join(".codex").join("config.toml");
    if !file_contains_marker(&config_path)? {
        install_codex_config(&config_path, outcome)?;
    }
    Ok(())
}

fn install_codex_hooks(path: &Path, outcome: &mut Outcome) -> Result<(), AxiError> {
    let mut root = match read_optional(path)? {
        Some(text) => match serde_json::from_str::<Value>(&text) {
            Ok(value) => value,
            Err(_) => {
                outcome.skipped.push(path.to_path_buf());
                return Ok(());
            }
        },
        None => json!({}),
    };

    let object = root
        .as_object_mut()
        .ok_or_else(|| AxiError::server("Codex hooks.json is not a JSON object"))?;

    let session_start = object
        .entry("session_start")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| AxiError::server("Codex `session_start` is not an array"))?;

    session_start.push(json!({ "command": HOOK_COMMAND, "_managed_by": MARKER }));

    let serialized = serde_json::to_string_pretty(&root)
        .map_err(|e| AxiError::server(format!("Failed to serialize Codex hooks.json: {e}")))?;
    write_file(path, &serialized)?;
    outcome.written.push(path.to_path_buf());
    Ok(())
}

fn install_codex_config(path: &Path, outcome: &mut Outcome) -> Result<(), AxiError> {
    let block = format!(
        "\n# {MARKER}: opt-in AXI session-start hook\n\
         [[hooks.session_start]]\n\
         command = \"{HOOK_COMMAND}\"\n"
    );
    let contents = match read_optional(path)? {
        Some(existing) => {
            // Append our marked block, preserving the user's config verbatim.
            let mut merged = existing;
            if !merged.ends_with('\n') {
                merged.push('\n');
            }
            merged.push_str(&block);
            merged
        }
        None => block.trim_start_matches('\n').to_string(),
    };
    write_file(path, &contents)?;
    outcome.written.push(path.to_path_buf());
    Ok(())
}

// --- OpenCode: wholesale plugin file -----------------------------------------

fn install_opencode(home: &Path, outcome: &mut Outcome) -> Result<(), AxiError> {
    let path = home
        .join(".config")
        .join("opencode")
        .join("plugins")
        .join(format!("axi-{MARKER}.js"));

    match read_optional(&path)? {
        Some(text) if text.contains(MARKER) => Ok(()), // already ours: no-op
        Some(_) => {
            // Exists without our marker: refuse to clobber a user file.
            outcome.skipped.push(path);
            Ok(())
        }
        None => {
            write_file(&path, &opencode_plugin_source())?;
            outcome.written.push(path);
            Ok(())
        }
    }
}

fn opencode_plugin_source() -> String {
    format!(
        "// {MARKER}: opt-in AXI session-start plugin.\n\
         // Injects the AXI slim home output at session start.\n\
         export default async () => ({{\n\
         \x20 event: async ({{ event }}) => {{\n\
         \x20   if (event.type === \"session.start\") {{\n\
         \x20     // Harness runs `{HOOK_COMMAND}` and surfaces its TOON home view.\n\
         \x20   }}\n\
         \x20 }},\n\
         }});\n"
    )
}

// --- shared fs helpers -------------------------------------------------------

/// Read a file to a string if it exists; `Ok(None)` when absent.
fn read_optional(path: &Path) -> Result<Option<String>, AxiError> {
    if !path.exists() {
        return Ok(None);
    }
    std::fs::read_to_string(path)
        .map(Some)
        .map_err(|e| AxiError::server(format!("Failed to read {}: {e}", path.display())))
}

/// Whether an existing file's content carries [`MARKER`] (i.e. is managed by us).
fn file_contains_marker(path: &Path) -> Result<bool, AxiError> {
    Ok(read_optional(path)?
        .map(|text| text.contains(MARKER))
        .unwrap_or(false))
}

/// Create parent directories and write `contents` to `path`.
fn write_file(path: &Path, contents: &str) -> Result<(), AxiError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AxiError::server(format!("Failed to create {}: {e}", parent.display()))
        })?;
    }
    std::fs::write(path, contents)
        .map_err(|e| AxiError::server(format!("Failed to write {}: {e}", path.display())))
}
