//! Encode-only TOON (Token-Oriented Object Notation) output.
//!
//! AXI emits command results as TOON: the agent *reads* TOON, never writes it,
//! so this crate needs only the encode direction (a JSON value -> a TOON string).
//! The source is a `serde_json::Value` so the JSON model stays in memory for free
//! (cadre-context decisions/2026-08-29-axi-rs-design-calls: TOON everywhere, no
//! output fallback, but keeping the internal model is not a fallback).
//!
//! Conformance target: SPEC v4.1. Covered here — encounter-order keys (requires
//! `serde_json`'s `preserve_order` feature), nested objects, uniform-object
//! arrays folded into a tabular block, scalar arrays inline, the inline-dash list
//! form for mixed/nested arrays, canonical numbers, and the tightened quoting
//! rules.
//!
//! Deferred (SPEC v4.1 features not built for the prototype): nested-object
//! tabular columns (`{id,customer{name,country}}`), keyed-tabular-root, and
//! tab/pipe delimiters. The active delimiter here is always the comma.

use serde_json::{Number, Value};

const INDENT: &str = "  ";
/// The active field delimiter. Fixed to comma for the prototype.
const DELIMITER: char = ',';

/// Encode a JSON value as a TOON document (no trailing newline).
pub fn encode(value: &Value) -> String {
    let mut out = String::new();
    match value {
        Value::Object(map) => emit_object(map, 0, &mut out),
        Value::Array(items) => emit_array(None, items, 0, &mut out),
        scalar => {
            out.push_str(&scalar_to_toon(scalar));
            out.push('\n');
        }
    }
    while out.ends_with('\n') {
        out.pop();
    }
    out
}

fn emit_object(map: &serde_json::Map<String, Value>, indent: usize, out: &mut String) {
    for (key, value) in map {
        emit_entry(key, value, indent, out);
    }
}

fn emit_entry(key: &str, value: &Value, indent: usize, out: &mut String) {
    match value {
        Value::Object(map) if map.is_empty() => {
            // An empty object still declares its key.
            push_line(out, indent, &format!("{}:", quote_scalar(key)));
        }
        Value::Object(map) => {
            push_line(out, indent, &format!("{}:", quote_scalar(key)));
            emit_object(map, indent + 1, out);
        }
        Value::Array(items) => emit_array(Some(key), items, indent, out),
        scalar => push_line(
            out,
            indent,
            &format!("{}: {}", quote_scalar(key), scalar_to_toon(scalar)),
        ),
    }
}

fn emit_array(key: Option<&str>, items: &[Value], indent: usize, out: &mut String) {
    let prefix = key.map(quote_scalar).unwrap_or_default();
    let n = items.len();

    // Empty array renders `key: []` (SPEC v4.1), not the legacy `key[0]:`.
    if items.is_empty() {
        if prefix.is_empty() {
            push_line(out, indent, "[]");
        } else {
            push_line(out, indent, &format!("{}: []", prefix));
        }
        return;
    }

    // Uniform array of scalar-only objects -> tabular block.
    if let Some(fields) = tabular_fields(items) {
        let header = format!("{}[{}]{{{}}}:", prefix, n, fields.join(","));
        push_line(out, indent, &header);
        for item in items {
            let obj = match item.as_object() {
                Some(obj) => obj,
                None => continue,
            };
            let row: Vec<String> = fields
                .iter()
                .map(|f| scalar_to_toon(obj.get(f).unwrap_or(&Value::Null)))
                .collect();
            push_line(out, indent + 1, &row.join(","));
        }
        return;
    }

    // Array of scalars -> inline.
    if items.iter().all(is_scalar) {
        let vals: Vec<String> = items.iter().map(scalar_to_toon).collect();
        push_line(out, indent, &format!("{}[{}]: {}", prefix, n, vals.join(",")));
        return;
    }

    // Mixed / nested -> inline-dash list form.
    push_line(out, indent, &format!("{}[{}]:", prefix, n));
    for item in items {
        emit_list_item(item, indent + 1, out);
    }
}

/// Emit one array element as a list item at `indent` (the dash column).
///
/// Scalars render as `- value`. Objects put their first field on the dash line
/// (`- key: value`) with remaining fields aligned beneath it.
fn emit_list_item(item: &Value, indent: usize, out: &mut String) {
    match item {
        Value::Object(map) if !map.is_empty() => {
            let mut entries = map.iter();
            if let Some((first_key, first_value)) = entries.next() {
                emit_dash_entry(first_key, first_value, indent, out);
            }
            for (key, value) in entries {
                emit_entry(key, value, indent + 1, out);
            }
        }
        Value::Object(_) => {
            // Empty object element.
            push_line(out, indent, "-");
        }
        Value::Array(inner) => {
            // Nested array element: dash header, then the array beneath it.
            push_line(out, indent, "-");
            emit_array(None, inner, indent + 1, out);
        }
        scalar => push_line(out, indent, &format!("- {}", scalar_to_toon(scalar))),
    }
}

/// Emit an object's first field on a dash line: `- key: value`.
fn emit_dash_entry(key: &str, value: &Value, indent: usize, out: &mut String) {
    match value {
        Value::Object(map) if map.is_empty() => {
            push_line(out, indent, &format!("- {}:", quote_scalar(key)));
        }
        Value::Object(map) => {
            push_line(out, indent, &format!("- {}:", quote_scalar(key)));
            emit_object(map, indent + 2, out);
        }
        Value::Array(items) => {
            // Array-valued first field: keep it simple — dash header, key array
            // aligned beneath. (Not exercised by the fixtures; best-effort.)
            push_line(out, indent, "-");
            emit_array(Some(key), items, indent + 1, out);
        }
        scalar => push_line(
            out,
            indent,
            &format!("- {}: {}", quote_scalar(key), scalar_to_toon(scalar)),
        ),
    }
}

/// Returns the shared field order if `items` is a non-empty array of objects that
/// all carry the same keys with scalar-only values; otherwise `None`.
fn tabular_fields(items: &[Value]) -> Option<Vec<String>> {
    let first = items.first()?.as_object()?;
    if first.is_empty() {
        return None;
    }
    let fields: Vec<String> = first.keys().cloned().collect();
    for item in items {
        let obj = item.as_object()?;
        if obj.len() != fields.len() {
            return None;
        }
        for f in &fields {
            match obj.get(f) {
                Some(v) if is_scalar(v) => {}
                _ => return None,
            }
        }
    }
    Some(fields)
}

fn is_scalar(v: &Value) -> bool {
    matches!(
        v,
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
    )
}

fn scalar_to_toon(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => number_to_toon(n),
        Value::String(s) => quote_scalar(s),
        // Not reachable through the scalar paths above.
        _ => String::new(),
    }
}

/// Canonical TOON number: integers print bare; floats drop trailing zeros
/// (`1.0` -> `1`, `1.5000` -> `1.5`), `-0` normalizes to `0`, and no exponent is
/// used for `1e-6 <= |n| < 1e21` (Rust's `f64` Display never emits an exponent).
fn number_to_toon(n: &Number) -> String {
    if let Some(i) = n.as_i64() {
        return i.to_string();
    }
    if let Some(u) = n.as_u64() {
        return u.to_string();
    }
    match n.as_f64() {
        Some(f) if f == 0.0 => "0".to_string(),
        Some(f) => f.to_string(),
        // A Number that is neither an integer nor representable as f64 should not
        // occur through serde_json; fall back to its own rendering.
        None => n.to_string(),
    }
}

fn quote_scalar(s: &str) -> String {
    if !needs_quoting(s) {
        return s.to_string();
    }
    let mut q = String::with_capacity(s.len() + 2);
    q.push('"');
    for c in s.chars() {
        match c {
            '"' => q.push_str("\\\""),
            '\\' => q.push_str("\\\\"),
            '\n' => q.push_str("\\n"),
            '\r' => q.push_str("\\r"),
            '\t' => q.push_str("\\t"),
            c if c.is_control() => q.push_str(&format!("\\u{:04x}", c as u32)),
            c => q.push(c),
        }
    }
    q.push('"');
    q
}

fn needs_quoting(s: &str) -> bool {
    // Empty.
    if s.is_empty() {
        return true;
    }
    // Leading/trailing space or tab.
    if starts_or_ends_with_ws(s) {
        return true;
    }
    // Literal-looking strings.
    if matches!(s, "true" | "false" | "null") {
        return true;
    }
    // Numeric-like (so a string never reads as a number).
    if is_numeric_like(s) {
        return true;
    }
    // Leading `-` or `#`.
    if let Some(first) = s.chars().next() {
        if first == '-' || first == '#' {
            return true;
        }
    }
    // Structural characters, the active delimiter, and control characters.
    s.chars().any(|c| {
        c == DELIMITER
            || matches!(c, ':' | '"' | '\\' | '[' | ']' | '{' | '}')
            || c.is_control()
    })
}

fn starts_or_ends_with_ws(s: &str) -> bool {
    let first = s.starts_with(' ') || s.starts_with('\t');
    let last = s.ends_with(' ') || s.ends_with('\t');
    first || last
}

/// Matches `/^[+-]?[0-9]+(?:\.[0-9]+)?(?:e[+-]?[0-9]+)?$/i` (SPEC v4.1).
fn is_numeric_like(s: &str) -> bool {
    let b = s.as_bytes();
    let mut i = 0;

    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let int_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i == int_start {
        return false; // require at least one integer digit
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let frac_start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == frac_start {
            return false;
        }
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        let exp_start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == exp_start {
            return false;
        }
    }
    i == b.len()
}

fn push_line(out: &mut String, indent: usize, content: &str) {
    for _ in 0..indent {
        out.push_str(INDENT);
    }
    out.push_str(content);
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::encode;
    use serde_json::json;

    #[test]
    fn flat_object_keeps_encounter_order() {
        let v = json!({"name": "review", "open": true});
        assert_eq!(encode(&v), "name: review\nopen: true");
    }

    #[test]
    fn nested_object() {
        let v = json!({"session": {"file": "a.html", "status": "open"}});
        assert_eq!(encode(&v), "session:\n  file: a.html\n  status: open");
    }

    #[test]
    fn tabular_uniform_array() {
        let v = json!({"runs": [{"id": 1, "ok": true}, {"id": 2, "ok": false}]});
        assert_eq!(encode(&v), "runs[2]{id,ok}:\n  1,true\n  2,false");
    }

    #[test]
    fn scalar_array_inline() {
        let v = json!({"tags": ["a", "b", "c"]});
        assert_eq!(encode(&v), "tags[3]: a,b,c");
    }

    #[test]
    fn empty_array_renders_bracket_pair() {
        let v = json!({"tags": []});
        assert_eq!(encode(&v), "tags: []");
    }

    #[test]
    fn number_canonicalization() {
        assert_eq!(encode(&json!({"a": 1.0, "b": 1.5000})), "a: 1\nb: 1.5");
        assert_eq!(encode(&json!({"z": -0.0})), "z: 0");
    }

    #[test]
    fn quotes_when_delimiter_or_ambiguous() {
        assert_eq!(encode(&json!({"msg": "a, b: c"})), "msg: \"a, b: c\"");
        // A numeric-looking string must be quoted so it doesn't read as a number.
        assert_eq!(encode(&json!({"v": "12"})), "v: \"12\"");
        // A literal-looking string likewise.
        assert_eq!(encode(&json!({"v": "true"})), "v: \"true\"");
        // A leading dash is ambiguous with the list form.
        assert_eq!(encode(&json!({"v": "-x"})), "v: \"-x\"");
    }

    #[test]
    fn mixed_array_uses_inline_dash_list() {
        let v = json!({"items": [1, {"a": 1}, "text"]});
        assert_eq!(encode(&v), "items[3]:\n  - 1\n  - a: 1\n  - text");
    }

    #[test]
    fn mixed_array_multi_field_object_item() {
        // Non-uniform (different key sets) -> list form, not a tabular block.
        let v = json!({"items": [{"a": 1}, {"a": 2, "b": 3}]});
        assert_eq!(encode(&v), "items[2]:\n  - a: 1\n  - a: 2\n    b: 3");
    }

    #[test]
    fn root_scalar() {
        assert_eq!(encode(&json!("hello")), "hello");
        assert_eq!(encode(&json!(42)), "42");
    }
}
