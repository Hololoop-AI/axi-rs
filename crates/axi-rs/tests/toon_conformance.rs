//! TOON encoding conformance to SPEC v4.1 — locked integration contract.
//!
//! Source of truth: `spec/axi-protocol-prototype.md` § A "Test contract —
//! fixtures". Every row of that 12-row table is asserted verbatim here against
//! the public `axi::toon::encode(&serde_json::Value) -> String`.
//!
//! This file compiles against the encoder as it stands today, so it runs RED on
//! assertions now: the rows the current encoder gets wrong (encounter order,
//! empty-array form, inline-dash list form, number canonicalization, and the
//! tightened quoting for a leading `-`) diverge from the spec's expected output.
//! Build brings the encoder to conformance (including enabling `serde_json`'s
//! `preserve_order` feature so key insertion order survives) until every row
//! matches.

use axi::toon::encode;
use serde_json::{json, Value};

/// One row of the spec's fixture table: a label, the input JSON, and the exact
/// TOON the encoder must produce (no trailing newline).
struct Fixture {
    label: &'static str,
    input: Value,
    expected: &'static str,
}

fn fixtures() -> Vec<Fixture> {
    vec![
        Fixture {
            label: "encounter order (not sorted)",
            input: json!({"name": "ada", "id": 1}),
            expected: "name: ada\nid: 1",
        },
        Fixture {
            label: "nested object",
            input: json!({"user": {"id": 123, "name": "Ada"}}),
            expected: "user:\n  id: 123\n  name: Ada",
        },
        Fixture {
            label: "tabular block",
            input: json!({"items": [{"sku": "A1", "qty": 2}, {"sku": "B2", "qty": 1}]}),
            expected: "items[2]{sku,qty}:\n  A1,2\n  B2,1",
        },
        Fixture {
            label: "scalar array inline",
            input: json!({"tags": ["admin", "ops", "dev"]}),
            expected: "tags[3]: admin,ops,dev",
        },
        Fixture {
            label: "empty array renders `key: []`",
            input: json!({"tags": []}),
            expected: "tags: []",
        },
        Fixture {
            label: "mixed array uses inline `- ` list form",
            input: json!({"items": [1, {"a": 1}, "text"]}),
            expected: "items[3]:\n  - 1\n  - a: 1\n  - text",
        },
        Fixture {
            label: "number canonicalization (1.0 -> 1, 1.5000 -> 1.5)",
            input: json!({"a": 1.0, "b": 1.5000}),
            expected: "a: 1\nb: 1.5",
        },
        Fixture {
            label: "quote a numeric-like string",
            input: json!({"v": "12"}),
            expected: "v: \"12\"",
        },
        Fixture {
            label: "quote a leading-dash string",
            input: json!({"v": "-x"}),
            expected: "v: \"-x\"",
        },
        Fixture {
            label: "quote a value containing the delimiter",
            input: json!({"msg": "a, b"}),
            expected: "msg: \"a, b\"",
        },
        Fixture {
            label: "null and bool literals",
            input: json!({"a": null, "b": true}),
            expected: "a: null\nb: true",
        },
        Fixture {
            label: "root scalar",
            input: json!(42),
            expected: "42",
        },
    ]
}

/// Data-driven over the full fixture table. Collects every mismatching row and
/// reports them together, so the red state names exactly which rows diverge from
/// SPEC v4.1 rather than stopping at the first failure.
#[test]
fn toon_encodes_spec_v41_fixture_table() {
    // Arrange
    let cases = fixtures();

    // Act + Assert (accumulated so all divergences surface in one run)
    let mut failures: Vec<String> = Vec::new();
    for case in &cases {
        let actual = encode(&case.input);
        if actual != case.expected {
            failures.push(format!(
                "\n[{}]\n  input:    {}\n  expected: {:?}\n  actual:   {:?}",
                case.label, case.input, case.expected, actual
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "TOON output diverges from SPEC v4.1 on {} row(s):{}",
        failures.len(),
        failures.join("")
    );
}
