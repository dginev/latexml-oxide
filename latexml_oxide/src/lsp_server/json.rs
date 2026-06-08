//! JSON value helpers (thin shims over `serde_json`).

// ======================================================================
// JSON — backed by serde_json. The server uses only `Value` parse/
// serialize (no derive), which costs ~16 KiB in the LTO'd binary — well
// within the distribution size budget. `Value::get`/`as_str` map 1:1 onto
// the call sites the previous hand-rolled `Value` exposed.
// ======================================================================

use serde_json::Value;

/// Parse a JSON document. Keeps the `Result<_, String>` signature so call
/// sites are unchanged from the previous hand-rolled parser.
pub(crate) fn parse_json(s: &str) -> Result<Value, String> {
  serde_json::from_str(s).map_err(|e| e.to_string())
}

/// Build a `Value::String`.
pub(crate) fn jstr(s: impl Into<String>) -> Value { Value::String(s.into()) }

/// Build a JSON number from an `f64` (non-finite → `null`).
pub(crate) fn jnum(n: f64) -> Value {
  serde_json::Number::from_f64(n).map(Value::Number).unwrap_or(Value::Null)
}

/// Build a JSON object from `(key, value)` pairs. `serde_json::Map` is a
/// `BTreeMap` by default, so serialized key order is deterministic.
pub(crate) fn jobj(pairs: Vec<(&str, Value)>) -> Value {
  let mut map = serde_json::Map::new();
  for (k, v) in pairs {
    map.insert(k.to_string(), v);
  }
  Value::Object(map)
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn json_roundtrip_object() {
    let v = parse_json(r#"{"a":1,"b":[true,null,"x"],"c":{"d":-2.5}}"#).unwrap();
    // serde_json parses `1` as an integer Number (distinct repr from a float),
    // so compare via the accessor rather than against `jnum`.
    assert_eq!(v.get("a").and_then(Value::as_i64), Some(1));
    assert_eq!(
      v.get("b"),
      Some(&Value::Array(vec![
        Value::Bool(true),
        Value::Null,
        Value::String("x".to_string())
      ]))
    );
    // BTreeMap → deterministic, sorted key order on serialization.
    assert_eq!(v.to_string(), r#"{"a":1,"b":[true,null,"x"],"c":{"d":-2.5}}"#);
  }

  #[test]
  fn json_escapes_all_control_chars() {
    // The old serializer only escaped \n \r \t; a form-feed / NUL produced
    // invalid JSON. Verify every control char is \u00xx-escaped and the
    // result re-parses (round-trip).
    let s = "x\u{0}\u{1}\u{8}\u{b}\u{c}\u{1f}y\t\n\r\"\\";
    let serialized = jstr(s).to_string();
    assert!(serialized.contains("\\u0000"));
    assert!(serialized.contains("\\u0001"));
    assert!(serialized.contains("\\u000b")); // vertical tab
    assert!(serialized.contains("\\b"));
    assert!(serialized.contains("\\f"));
    assert!(serialized.contains("\\u001f"));
    assert!(serialized.contains("\\t") && serialized.contains("\\n") && serialized.contains("\\r"));
    assert!(serialized.contains("\\\"") && serialized.contains("\\\\"));
    let reparsed = parse_json(&serialized).unwrap();
    assert_eq!(reparsed, Value::String(s.to_string()));
  }
}
