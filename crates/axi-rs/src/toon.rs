//! Encode-only TOON (Token-Oriented Object Notation) output.
//!
//! AXI emits command results as TOON: the agent *reads* TOON, never writes it,
//! so this crate needs only the encode direction (a JSON value -> a TOON string).
//! The source is a `serde_json::Value` so the JSON model stays in memory for free
//! (cadre-context decisions/2026-08-29-axi-rs-design-calls: TOON everywhere, no
//! output fallback, but keeping the internal model is not a fallback).
//!
//! Covered here: objects, nested objects, uniform-object arrays folded into a
//! tabular block, scalar arrays inline, and a list fallback for mixed/nested
//! arrays.
//!
//! TODO(spec-conformance): this is the core TOON shape, not yet checked line for
//! line against SPEC.md v4.1. The quoting rules and the mixed-array list form in
//! particular need a conformance pass before we depend on byte-exact output.

use serde_json::Value;

const INDENT: &str = "  ";

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

    if items.is_empty() {
        push_line(out, indent, &format!("{}[0]:", prefix));
        return;
    }

    // Uniform array of scalar-only objects -> tabular block.
    if let Some(fields) = tabular_fields(items) {
        let header = format!("{}[{}]{{{}}}:", prefix, n, fields.join(","));
        push_line(out, indent, &header);
        for item in items {
            let obj = item.as_object().expect("tabular_fields verified objects");
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

    // Mixed / nested -> list fallback.
    push_line(out, indent, &format!("{}[{}]:", prefix, n));
    for item in items {
        match item {
            Value::Object(map) => {
                push_line(out, indent + 1, "-");
                emit_object(map, indent + 2, out);
            }
            Value::Array(inner) => emit_array(None, inner, indent + 1, out),
            scalar => push_line(out, indent + 1, &format!("- {}", scalar_to_toon(scalar))),
        }
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
        Value::Number(n) => n.to_string(),
        Value::String(s) => quote_scalar(s),
        // Not reachable through the scalar paths above.
        _ => String::new(),
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
            '\t' => q.push_str("\\t"),
            '\r' => q.push_str("\\r"),
            _ => q.push(c),
        }
    }
    q.push('"');
    q
}

fn needs_quoting(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    if s != s.trim() {
        return true;
    }
    if s.chars().any(|c| {
        matches!(
            c,
            ',' | ':' | '"' | '\\' | '{' | '}' | '[' | ']' | '\n' | '\t' | '\r'
        )
    }) {
        return true;
    }
    // Disambiguate a string from a literal or number it would otherwise look like.
    if matches!(s, "true" | "false" | "null") {
        return true;
    }
    s.parse::<f64>().is_ok()
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
    fn flat_object() {
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
    fn empty_array() {
        let v = json!({"tags": []});
        assert_eq!(encode(&v), "tags[0]:");
    }

    #[test]
    fn quotes_when_delimiter_or_ambiguous() {
        assert_eq!(encode(&json!({"msg": "a, b: c"})), "msg: \"a, b: c\"");
        // A numeric-looking string must be quoted so it doesn't read as a number.
        assert_eq!(encode(&json!({"v": "12"})), "v: \"12\"");
        // A literal-looking string likewise.
        assert_eq!(encode(&json!({"v": "true"})), "v: \"true\"");
    }

    #[test]
    fn mixed_array_falls_back_to_list() {
        // Non-uniform (different key sets) -> list form, not a tabular block.
        let v = json!({"items": [{"a": 1}, {"a": 2, "b": 3}]});
        assert_eq!(encode(&v), "items[2]:\n  -\n    a: 1\n  -\n    a: 2\n    b: 3");
    }

    #[test]
    fn root_scalar() {
        assert_eq!(encode(&json!("hello")), "hello");
    }
}
