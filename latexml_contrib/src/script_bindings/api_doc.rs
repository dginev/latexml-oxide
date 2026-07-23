//! Generator for [`API.md`](../../../script_bindings/API.md) — the rendered
//! reference of every function a `.rhai` binding can call.
//!
//! The list is not hand-maintained: it is read back OUT of the live engine via
//! Rhai's reflections API (`Engine::gen_fn_signatures`, the mechanism the Rhai
//! book points at for exactly this), so it cannot drift from what is actually
//! registered. `api_reference_is_up_to_date` in `tests.rs` regenerates and
//! compares on every test run; adding a `register_fn` without regenerating fails
//! CI.
//!
//! Test-only. `gen_fn_signatures` needs Rhai's `metadata` feature, which pulls in
//! serde + serde_json — that must never reach the shipped binary, so `metadata`
//! is enabled on a **dev-dependency** only. Under `resolver = "2"` a
//! dev-dependency's features are not unified into a normal build, so
//! `cargo build --release` stays serde-free.

use rhai::Engine;

/// The receiver groups, in presentation order: free functions first (that is
/// what a binding file calls at top level), then the proxies a body reaches
/// through. `None` is the free-function group.
const GROUPS: &[(Option<&str>, &str, &str)] = &[
  (
    None,
    "Global functions",
    "Called at the top level of a binding file, or from inside any body.",
  ),
  (
    Some("Document"),
    "`document` methods",
    "Reached through the `document` handle a constructor body receives as its \
     first argument — Perl's `$document->method`.",
  ),
  (
    Some("Node"),
    "`Node` methods",
    "An XML node: one returned by `ParseXML`, or one handed to a rewrite / \
     ligature-matcher body.",
  ),
  (
    Some("Whatsit"),
    "`whatsit()` methods",
    "The whatsit under construction, inside a digest hook.",
  ),
  (
    Some("Definition"),
    "`LookupDefinition()` methods",
    "An already-installed definition, for pushing hooks onto it.",
  ),
  (
    Some("Command"),
    "`Command` methods",
    "The `std::process::Command` mirror, for a binding that shells out.",
  ),
];

/// Functions whose registration returns `Dynamic` but whose ACTUAL result is a
/// known handle. Rhai can only report `Dynamic`, which renders as `?` and tells a
/// binding author nothing — and these are exactly the node-walking and
/// node-returning calls that are hardest to infer. `Node?` reads "a Node, or `()`
/// if there is none".
///
/// Hand-written, but not drift-prone in the direction that matters:
/// [`overrides_all_exist`] fails if a name here is no longer registered, and any
/// function NOT listed keeps its honest `?`.
const RETURN_OVERRIDES: &[(&str, &str)] = &[
  ("LookupDefinition", "Definition?"),
  ("children", "array"),
  ("findnode", "Node?"),
  ("firstChild", "Node?"),
  ("getElement", "Node?"),
  ("getNode", "Node"),
  ("insertElement", "Node"),
  ("nextSibling", "Node?"),
  ("openElementAt", "Node"),
  ("parent", "Node?"),
  ("prevSibling", "Node?"),
  ("renameNode", "Node"),
  ("wrapNodes", "Node?"),
];

/// Every [`RETURN_OVERRIDES`] key must still be a registered function — so a
/// renamed or deleted call cannot leave a lie behind in the reference.
pub(super) fn overrides_all_exist(engine: &Engine) -> Vec<&'static str> {
  let registered: Vec<String> = engine
    .gen_fn_signatures(false)
    .iter()
    .map(|s| s.split_once('(').map_or(s.clone(), |(n, _)| n.to_string()))
    .collect();
  RETURN_OVERRIDES
    .iter()
    .map(|(name, _)| *name)
    .filter(|name| !registered.iter().any(|r| r == name))
    .collect()
}

/// Render the whole reference as Markdown.
pub(super) fn generate(engine: &Engine) -> String {
  let signatures = engine.gen_fn_signatures(false);
  let mut parsed: Vec<Signature> = signatures.iter().map(|s| Signature::parse(s)).collect();
  // Rhai hands these back in hash order — sort so the file is stable and its
  // diffs are reviewable.
  parsed.sort_by(|a, b| (&a.name, &a.params).cmp(&(&b.name, &b.params)));

  let mut out = String::new();
  out.push_str(HEADER);
  for (receiver, title, blurb) in GROUPS {
    let group: Vec<&Signature> = parsed
      .iter()
      .filter(|s| s.receiver.as_deref() == *receiver)
      .collect();
    if group.is_empty() {
      continue;
    }
    out.push_str(&format!(
      "\n## {title}\n\n{blurb}\n\n{} function{}.\n\n```text\n",
      group.len(),
      if group.len() == 1 { "" } else { "s" }
    ));
    for sig in group {
      out.push_str(&sig.render());
      out.push('\n');
    }
    out.push_str("```\n");
  }
  out
}

const HEADER: &str = r#"# The `.rhai` binding interface

Every function a runtime (Rhai) binding can call, grouped by the handle it is
called on. **Generated** from the live engine via Rhai's reflections API
(`Engine::gen_fn_signatures`) and checked on every test run, so it always
matches what is actually registered — see `api_doc.rs`.

Overloads are listed separately, one line per accepted argument shape.

Types are Rhai's: `string`, `int`, `bool`, `array`, `map`, `Fn` (a closure),
`?` (any value), and the handle types named by the sections below. A trailing `?`
on a handle — `Node?` — means "that handle, or `()` when there is none". A
missing return type means the function returns nothing. Anything that can fail raises an
ordinary script error; what that costs is in
[`script_bindings_plan.md`](https://github.com/dginev/latexml-oxide/blob/main/docs/parity/script_bindings_plan.md)
under "Failure containment" — briefly, a failing body degrades its own binding
and the conversion continues.

Argument NAMES are not shown because Rhai does not record them for
closure-registered functions; only the accepted types are.
"#;

/// One registered function, split into the parts we present.
struct Signature {
  /// The handle it is called on (`Document`, `Node`, …); `None` for a free
  /// function.
  receiver: Option<String>,
  name:     String,
  params:   Vec<String>,
  ret:      Option<String>,
}

impl Signature {
  /// Parse one `gen_fn_signatures` line:
  /// `name(_: &mut Document, _: string) -> core::result::Result<…>`.
  fn parse(raw: &str) -> Self {
    let (name, rest) = raw.split_once('(').unwrap_or((raw, ""));
    let (args, ret) = match rest.rsplit_once(") -> ") {
      Some((a, r)) => (a, Some(simplify_type(r))),
      None => (rest.trim_end_matches(')'), None),
    };
    let mut params: Vec<String> = args
      .split(", ")
      .filter(|p| !p.is_empty())
      .map(|p| simplify_type(p.split_once(": ").map_or(p, |(_, t)| t)))
      .collect();
    // A leading `&mut T` parameter is the receiver, not an argument.
    let receiver = params
      .first()
      .and_then(|p| p.strip_prefix("&mut ").map(str::to_string));
    if receiver.is_some() {
      params.remove(0);
    }
    let name = name.to_string();
    // A known handle beats the opaque `Dynamic` the registration reports.
    let ret = match RETURN_OVERRIDES.iter().find(|(n, _)| *n == name) {
      Some((_, real)) => Some((*real).to_string()),
      None => ret.filter(|r| r != "()"),
    };
    Self { receiver, name, params, ret }
  }

  fn render(&self) -> String {
    let args = self.params.join(", ");
    match &self.ret {
      Some(r) => format!("{}({args}) -> {r}", self.name),
      None => format!("{}({args})", self.name),
    }
  }
}

/// Rhai reports Rust paths verbatim (`alloc::string::String`,
/// `core::result::Result<…, Box<EvalAltResult>>`). Render them as the Rhai types
/// a binding author actually writes.
fn simplify_type(raw: &str) -> String {
  let t = raw.trim();
  // `Result<T, Box<EvalAltResult>>` is "returns T, or raises" — the raising is a
  // property of every fallible call, so present just T (see the header note).
  if let Some(inner) = t
    .strip_prefix("core::result::Result<")
    .and_then(|r| r.strip_suffix(">"))
    && let Some((ok, _err)) = split_top_level_comma(inner)
  {
    return simplify_type(ok);
  }
  match t {
    "alloc::string::String" => "string".into(),
    "alloc::vec::Vec<rhai::types::dynamic::Dynamic>" => "array".into(),
    "i64" => "int".into(),
    "rhai::types::dynamic::Dynamic" | "types::dynamic::Dynamic" => "?".into(),
    _ => {
      // A BTreeMap<SmartString, Dynamic> is Rhai's `map`; the rest just lose
      // their module path (`latexml_core::tokens::Tokens` -> `Tokens`).
      if t.starts_with("alloc::collections::btree::map::BTreeMap<") {
        return "map".into();
      }
      match t.strip_prefix("&mut ") {
        Some(inner) => format!("&mut {}", last_path_segment(inner)),
        None => last_path_segment(t).to_string(),
      }
    },
  }
}

fn last_path_segment(t: &str) -> &str { t.rsplit("::").next().unwrap_or(t) }

/// Split `T, E` at the comma that is not inside angle brackets — the Ok/Err
/// boundary of a `Result<…>` whose Ok type may itself be generic.
fn split_top_level_comma(s: &str) -> Option<(&str, &str)> {
  let mut depth = 0usize;
  for (i, c) in s.char_indices() {
    match c {
      '<' => depth += 1,
      '>' => depth = depth.saturating_sub(1),
      ',' if depth == 0 => return Some((&s[..i], &s[i + 1..])),
      _ => {},
    }
  }
  None
}
