# TOON encoding
status: in-progress

Encode a `serde_json::Value` into a TOON document (encode-only — the agent reads
TOON, never writes it). TOON is the agent-facing output format everywhere, with
no JSON fallback.

## Behavior

- Objects → indented `key: value` lines; nested objects → `key:` then indented
  children.
- Uniform arrays of scalar-only objects → tabular block `key[N]{f1,f2}:` + rows.
- Scalar arrays → inline `key[N]: a,b,c`. Empty array → `key[N]:` with N=0.
- Mixed / nested arrays → list fallback.
- Scalars quoted when empty, whitespace-padded, delimiter/structural chars
  present, or ambiguous with a number/bool/null literal.

## Notes

- Built: the core shape (`src/toon.rs`) with unit tests for objects, nested,
  tabular, scalar arrays, quoting, mixed fallback, root scalar.
- **Open (the reason this slice is in-progress):** not yet conformant to
  SPEC.md v4.1. The mixed-array list form and the quoting rules are the two most
  likely to diverge. Closing this needs the spec fetched (investigating) and the
  contract below expressed as spec-example fixtures.
- Decision: we write our own encoder rather than depend on the churny community
  crate (cadre-context research/2026-08-29-toon-output-format).

## Integration Test Contract

_(pending test-planning — express as fixtures: input JSON → expected TOON,
drawn from SPEC.md v4.1 examples, including the cases above and the edge cases
the spec calls out.)_

## Tests

_(pending test-writer)_
