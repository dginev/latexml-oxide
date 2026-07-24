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

/// Where the in-depth documentation for one `.rhai` call lives.
enum Doc {
  /// A thin wrapper over a documented Rust item — link straight to it, so the
  /// reader lands on the real semantics instead of a paraphrase that can rot.
  /// Emitted as a rustdoc INTRA-DOC link, which means rustdoc itself validates
  /// the path: a renamed or deleted target is a `cargo doc` warning naming the
  /// line in `API.md`, so a dead link cannot ship unnoticed.
  Rust(&'static str),
  /// No single Rust item to point at — a Rhai-only helper, or a shim whose
  /// whole behaviour lives in the registration. Written out here, because "no
  /// counterpart" must not mean "no documentation". Rendered as Markdown, so a
  /// note may still LINK the parts that do have a Rust home (`parent` below
  /// describes its guard and links the traversal it wraps); rustdoc validates
  /// those links exactly as it does a [`Doc::Rust`] one.
  Note(&'static str),
}

/// Every registered call, by name. Overloads share one entry: the signatures
/// differ, what the call MEANS does not. Names registered on more than one
/// handle are the exception — they live in [`HANDLE_DOCS`] instead.
///
/// Three tests keep this honest — [`undocumented_names`] fails when a
/// `register_fn` has no entry (so a new helper cannot ship undocumented),
/// [`stale_names`] fails when an entry names a call that no longer exists, and
/// [`ambiguous_names`] fails when one entry would be shared by two handles.
const DOCS: &[(&str, Doc)] = &[
  (
    "AddToCounter",
    Doc::Rust("latexml_core::binding::counter::dialect::add_to_counter"),
  ),
  (
    "AssignCatcode",
    Doc::Rust("latexml_core::state::assign_catcode"),
  ),
  (
    "AssignMapping",
    Doc::Rust("latexml_core::state::assign_mapping"),
  ),
  (
    "AssignMeaning",
    Doc::Rust("latexml_core::state::assign_meaning"),
  ),
  (
    "AssignValue",
    Doc::Rust("latexml_core::state::assign_value"),
  ),
  (
    "Command",
    Doc::Note(
      "Start a `std::process::Command` builder. A Rhai-only shim; nothing runs until `output()`.",
    ),
  ),
  (
    "CounterValue",
    Doc::Rust("latexml_core::binding::counter::dialect::counter_value"),
  ),
  (
    "DeclareOption",
    Doc::Note(
      "Declare a class/package option: named with a body, or the bare-closure form for the default handler (Perl `DeclareOption(undef, sub {...})`).",
    ),
  ),
  (
    "DefAccent",
    Doc::Note("Define an accent command in both its combining and standalone forms."),
  ),
  (
    "DefColumnType",
    Doc::Rust("latexml_core::binding::def::dialect::def_macro"),
  ),
  (
    "DefConditional",
    Doc::Rust("latexml_core::binding::def::dialect::def_conditional"),
  ),
  (
    "DefConstructor",
    Doc::Rust("latexml_core::binding::def::dialect::def_constructor"),
  ),
  (
    "DefEnvironment",
    Doc::Rust("latexml_core::binding::def::dialect::def_environment"),
  ),
  (
    "DefKeyVal",
    Doc::Note("Declare one key in a keyval family."),
  ),
  (
    "DefLigature",
    Doc::Note("Register a text ligature pattern."),
  ),
  (
    "DefMacro",
    Doc::Rust("latexml_core::binding::def::dialect::def_macro"),
  ),
  (
    "DefMath",
    Doc::Rust("latexml_core::binding::def::dialect::def_math"),
  ),
  (
    "DefMathLigature",
    Doc::Note("Register a math ligature, as a pattern or a matcher closure."),
  ),
  (
    "DefMathRewrite",
    Doc::Note("As `DefRewrite`, scoped to math."),
  ),
  (
    "DefPrimitive",
    Doc::Rust("latexml_core::binding::def::dialect::def_primitive"),
  ),
  (
    "DefRegister",
    Doc::Rust("latexml_core::binding::def::dialect::def_register"),
  ),
  (
    "DefRewrite",
    Doc::Note("Register a document rewrite rule (data form, or a replace closure)."),
  ),
  ("Digest", Doc::Rust("latexml_core::stomach::digest")),
  (
    "DigestText",
    Doc::Rust("latexml_core::binding::content::digest_text"),
  ),
  (
    "Error",
    Doc::Note(
      "Log an `Error:`. Past `MAX_ERRORS` this escalates to `Fatal` and ends the conversion.",
    ),
  ),
  (
    "ExecuteOptions",
    Doc::Rust("latexml_core::binding::content::execute_options"),
  ),
  ("Expand", Doc::Rust("latexml_core::gullet::do_expand")),
  (
    "ExpandPartially",
    Doc::Rust("latexml_core::gullet::do_expand_partially"),
  ),
  ("Fatal", Doc::Note("End the conversion with a `Fatal:`.")),
  (
    "GetKeyVal",
    Doc::Note("One value out of a parsed keyval set."),
  ),
  (
    "GetKeyVals",
    Doc::Note("A whole keyval string parsed into a map."),
  ),
  ("Info", Doc::Note("Log an `Info:` line.")),
  (
    "InputDefinitions",
    Doc::Rust("latexml_core::binding::content::input_definitions"),
  ),
  (
    "IsDefined",
    Doc::Rust("latexml_core::binding::def::dialect::is_defined_token"),
  ),
  ("LaTeXMLVersion", Doc::Note("The engine version string.")),
  ("Let", Doc::Rust("latexml_core::state::let_i")),
  (
    "LoadClass",
    Doc::Rust("latexml_core::binding::content::load_class"),
  ),
  ("LookupBool", Doc::Rust("latexml_core::state::lookup_bool")),
  (
    "LookupCatcode",
    Doc::Rust("latexml_core::state::lookup_catcode"),
  ),
  (
    "LookupDefinition",
    Doc::Note("Fetch an installed definition so hooks can be pushed onto it; `()` when undefined."),
  ),
  (
    "LookupMapping",
    Doc::Rust("latexml_core::state::with_mapping"),
  ),
  (
    "LookupMeaning",
    Doc::Rust("latexml_core::state::lookup_meaning"),
  ),
  (
    "LookupNumber",
    Doc::Rust("latexml_core::state::lookup_number"),
  ),
  (
    "LookupString",
    Doc::Rust("latexml_core::state::lookup_string"),
  ),
  (
    "LookupTokens",
    Doc::Rust("latexml_core::state::lookup_tokens"),
  ),
  (
    "LookupValue",
    Doc::Rust("latexml_core::state::lookup_value"),
  ),
  (
    "MergeFont",
    Doc::Rust("latexml_core::binding::content::merge_font"),
  ),
  (
    "NewCounter",
    Doc::Rust("latexml_core::binding::counter::dialect::new_counter"),
  ),
  (
    "NoteLog",
    Doc::Note("Write a progress note to the conversion log only."),
  ),
  (
    "NoteSTDERR",
    Doc::Note("Write a progress note to stderr as well as the log."),
  ),
  (
    "ParseXML",
    Doc::Rust("latexml_core::common::xml::parse_fragment"),
  ),
  (
    "PassOptions",
    Doc::Rust("latexml_core::binding::content::pass_options"),
  ),
  (
    "ProcessOptions",
    Doc::Rust("latexml_core::binding::content::process_options"),
  ),
  (
    "ProgressSpindown",
    Doc::Rust("latexml_core::common::error::note_end"),
  ),
  (
    "ProgressSpinup",
    Doc::Rust("latexml_core::common::error::note_begin"),
  ),
  (
    "ProgressStep",
    Doc::Rust("latexml_core::common::error::progress_step"),
  ),
  ("RawTeX", Doc::Rust("latexml_core::stomach::raw_tex")),
  ("ReadArg", Doc::Rust("latexml_core::gullet::read_arg")),
  (
    "ReadOptional",
    Doc::Rust("latexml_core::gullet::read_optional"),
  ),
  ("ReadUntil", Doc::Rust("latexml_core::gullet::read_until")),
  (
    "RefCurrentID",
    Doc::Rust("latexml_core::binding::counter::dialect::ref_current_id"),
  ),
  (
    "RefStepCounter",
    Doc::Rust("latexml_core::binding::counter::dialect::ref_step_counter"),
  ),
  (
    "RefStepID",
    Doc::Rust("latexml_core::binding::counter::dialect::ref_step_id"),
  ),
  (
    "RegisterDocumentNamespace",
    Doc::Note("Bind an OUTPUT-document prefix to a namespace URI."),
  ),
  (
    "RegisterNamespace",
    Doc::Note("Bind a CODE prefix to a namespace URI (Perl `Package.pm` `RegisterNamespace`)."),
  ),
  (
    "RelaxNGSchema",
    Doc::Rust("latexml_core::binding::content::select_relaxng_schema"),
  ),
  (
    "RequirePackage",
    Doc::Rust("latexml_core::binding::content::require_package"),
  ),
  (
    "RequireResource",
    Doc::Rust("latexml_core::binding::content::require_resource"),
  ),
  (
    "ResetCounter",
    Doc::Rust("latexml_core::binding::counter::dialect::reset_counter"),
  ),
  (
    "Revert",
    Doc::Rust("latexml_core::digested::Digested::revert"),
  ),
  ("SkipSpaces", Doc::Rust("latexml_core::gullet::skip_spaces")),
  (
    "StepCounter",
    Doc::Rust("latexml_core::binding::counter::dialect::step_counter"),
  ),
  (
    "T_CS",
    Doc::Note(
      "One control-sequence token, wrapped as `Tokens` so it composes with `Digest`/`Expand`.",
    ),
  ),
  (
    "Tag",
    Doc::Rust("latexml_core::binding::content::install_tag"),
  ),
  ("TeX", Doc::Rust("latexml_core::stomach::digest")),
  (
    "ToAttribute",
    Doc::Rust("latexml_core::digested::Digested::to_attribute"),
  ),
  (
    "ToString",
    Doc::Note("The digested value as plain text (its `Display`)."),
  ),
  ("Today", Doc::Rust("latexml_engine::base_utilities::today")),
  ("Tokenize", Doc::Rust("latexml_core::mouth::tokenize")),
  (
    "TokenizeInternal",
    Doc::Rust("latexml_core::mouth::tokenize_internal"),
  ),
  ("UnTeX", Doc::Rust("latexml_core::tokens::Tokens::untex")),
  (
    "Warn",
    Doc::Note("Log a `Warning:` with the given category and object."),
  ),
  ("XEquals", Doc::Rust("latexml_core::state::x_equals")),
  (
    "absorb",
    Doc::Rust("latexml_core::document::Document::absorb"),
  ),
  (
    "absorbProperty",
    Doc::Rust("latexml_core::document::Document::absorb"),
  ),
  (
    "absorbString",
    Doc::Rust("latexml_core::document::Document::absorb_string"),
  ),
  (
    "addClass",
    Doc::Rust("latexml_core::document::Document::add_class"),
  ),
  (
    "appendClone",
    Doc::Rust("latexml_core::document::Document::append_clone"),
  ),
  ("arg", Doc::Note("Append one argument to the command.")),
  (
    "argString",
    Doc::Note("The nth digested argument of the current whatsit, reverted to source text."),
  ),
  (
    "args",
    Doc::Note("Append several arguments at once, from an array."),
  ),
  (
    "assign_global",
    Doc::Note("`AssignValue` with global scope: the binding survives the enclosing TeX group."),
  ),
  (
    "assign_value",
    Doc::Note("`AssignValue` with the default (group-local) scope."),
  ),
  ("children", Doc::Rust("libxml::tree::Node::get_child_nodes")),
  (
    "closeElement",
    Doc::Rust("latexml_core::document::Document::close_element"),
  ),
  (
    "closeElementAt",
    Doc::Rust("latexml_core::document::Document::close_element_at"),
  ),
  ("content", Doc::Rust("libxml::tree::Node::get_content")),
  (
    "current_dir",
    Doc::Note("Run the command in this working directory."),
  ),
  (
    "env",
    Doc::Note("Set one environment variable for the command."),
  ),
  (
    "findnode",
    Doc::Rust("latexml_core::document::Document::findnode"),
  ),
  (
    "findnodes",
    Doc::Rust("latexml_core::document::Document::findnodes"),
  ),
  (
    "firstChild",
    Doc::Rust("libxml::tree::Node::get_first_child"),
  ),
  (
    "generateID",
    Doc::Rust("latexml_core::document::Document::generate_id"),
  ),
  (
    "getAttribute",
    Doc::Rust("latexml_core::common::model::get_node_attribute"),
  ),
  (
    "getElement",
    Doc::Rust("latexml_core::document::Document::get_element"),
  ),
  (
    "getNode",
    Doc::Rust("latexml_core::document::Document::get_node"),
  ),
  (
    "hasAttribute",
    Doc::Rust("latexml_core::common::model::get_node_attribute"),
  ),
  (
    "insertElement",
    Doc::Rust("latexml_core::document::Document::insert_element"),
  ),
  (
    "insertXML",
    Doc::Rust("latexml_core::document::Document::insert_nodes"),
  ),
  (
    "lookup_value",
    Doc::Note("`LookupValue` coerced to a string; empty when unset."),
  ),
  (
    "maybeCloseElement",
    Doc::Rust("latexml_core::document::Document::maybe_close_element"),
  ),
  ("name", Doc::Rust("libxml::tree::Node::get_name")),
  (
    "neutralize_font",
    Doc::Rust("latexml_engine::base_utilities::neutralize_font"),
  ),
  (
    "nextSibling",
    Doc::Rust("libxml::tree::Node::get_next_sibling"),
  ),
  (
    "openElement",
    Doc::Rust("latexml_core::document::Document::open_element"),
  ),
  (
    "openElementAt",
    Doc::Rust("latexml_core::document::Document::open_element_at"),
  ),
  (
    "output",
    Doc::Note(
      "Run the command and return `#{ status, stdout, stderr }`. Refused when `LATEXML_DISABLE_SHELL_ESCAPE` is set (see SAFETY.md).",
    ),
  ),
  (
    "parent",
    Doc::Note(
      "The parent node ([`get_parent`](libxml::tree::Node::get_parent)), or `()` when there is none. A node from `ParseXML` is a TOP-LEVEL node of its chunk, so it reports NO parent: everything above it is a parse artifact ([`is_parse_artifact`](latexml_core::common::xml::is_parse_artifact)), never markup the script wrote.",
    ),
  ),
  (
    "prevSibling",
    Doc::Rust("libxml::tree::Node::get_prev_sibling"),
  ),
  (
    "propertyString",
    Doc::Note("One property of the current whatsit, as a string."),
  ),
  (
    "pushAfterConstruct",
    Doc::Note("Append a hook to that definition's `afterConstruct` list."),
  ),
  (
    "pushAfterDigest",
    Doc::Note("Append a hook to that definition's `afterDigest` list."),
  ),
  (
    "pushAfterDigestBody",
    Doc::Note("Append a hook to that definition's `afterDigestBody` list."),
  ),
  (
    "pushBeforeConstruct",
    Doc::Note("Append a hook to that definition's `beforeConstruct` list."),
  ),
  (
    "pushBeforeDigest",
    Doc::Note("Append a hook to that definition's `beforeDigest` list."),
  ),
  (
    "qname",
    Doc::Rust("latexml_core::common::model::with_node_qname"),
  ),
  (
    "removeAttribute",
    Doc::Rust("latexml_core::common::model::remove_node_attribute"),
  ),
  (
    "removeNode",
    Doc::Rust("latexml_core::document::Document::remove_node"),
  ),
  (
    "renameNode",
    Doc::Rust("latexml_core::document::Document::rename_node"),
  ),
  (
    "replaceNode",
    Doc::Rust("latexml_core::document::Document::replace_node"),
  ),
  // `setAttribute` is deliberately absent: it is registered on two handles and
  // means something different on each, so it lives in HANDLE_DOCS.
  ("setContent", Doc::Rust("libxml::tree::Node::set_content")),
  (
    "setNode",
    Doc::Rust("latexml_core::document::Document::set_node"),
  ),
  (
    "setProperty",
    Doc::Note(
      "Set a property on the current whatsit. Only a DIGEST hook may: in a construction hook the whatsit is already read-only, and this is a clean script error.",
    ),
  ),
  (
    "toString",
    Doc::Note("The node and its subtree serialized back to markup — the inverse of `ParseXML`."),
  ),
  ("unlink", Doc::Rust("libxml::tree::Node::unlink")),
  (
    "unshiftAfterConstruct",
    Doc::Note("Prepend a hook to that definition's `afterConstruct` list."),
  ),
  (
    "unshiftAfterDigest",
    Doc::Note("Prepend a hook to that definition's `afterDigest` list."),
  ),
  (
    "unshiftAfterDigestBody",
    Doc::Note("Prepend a hook to that definition's `afterDigestBody` list."),
  ),
  (
    "unshiftBeforeConstruct",
    Doc::Note("Prepend a hook to that definition's `beforeConstruct` list."),
  ),
  (
    "unshiftBeforeDigest",
    Doc::Note("Prepend a hook to that definition's `beforeDigest` list."),
  ),
  (
    "unwrapNodes",
    Doc::Rust("latexml_core::document::Document::unwrap_nodes"),
  ),
  (
    "whatsit",
    Doc::Note(
      "A handle on the whatsit under construction. Meaningful only inside a digest hook — the handle itself is always returned, but USING it outside one is a clean script error, not a crash.",
    ),
  ),
  (
    "wrapNodes",
    Doc::Rust("latexml_core::document::Document::wrap_nodes"),
  ),
];

/// Calls whose meaning depends on the HANDLE they are called on. One name
/// registered on two receivers is two different functions, and a single
/// name-keyed [`DOCS`] entry would document one of them wrongly:
/// `document.setAttribute` writes to wherever the document is currently
/// positioned and goes through the model (prefix decoding, id bookkeeping),
/// while `node.setAttribute` writes to the node the script is holding.
///
/// [`ambiguous_names`] enforces the rule: a name that appears under more than
/// one handle must have an entry here for EACH of them.
const HANDLE_DOCS: &[(&str, &str, Doc)] = &[
  (
    "Document",
    "setAttribute",
    Doc::Rust("latexml_core::document::Document::set_attribute"),
  ),
  (
    "Node",
    "setAttribute",
    Doc::Rust("libxml::tree::Node::set_attribute"),
  ),
];

/// Registered calls with neither a [`DOCS`] nor a [`HANDLE_DOCS`] entry — a new
/// `register_fn` nobody documented. Reported as `Handle::name` so the failure
/// message says which table to add to.
pub(super) fn undocumented_names(engine: &Engine) -> Vec<String> {
  let mut missing: Vec<String> = registered_calls(engine)
    .into_iter()
    .filter(|(receiver, name)| lookup(receiver.as_deref(), name).is_none())
    .map(|(receiver, name)| qualified(receiver.as_deref(), &name))
    .collect();
  missing.sort();
  missing.dedup();
  missing
}

/// Entries naming a call the engine no longer registers — from either table, so
/// a rename cannot leave a stale row behind in whichever one holds it.
pub(super) fn stale_names(engine: &Engine) -> Vec<String> {
  let live = registered_calls(engine);
  let docs = DOCS
    .iter()
    .map(|(name, _)| (None, *name))
    .filter(|(_, name)| !live.iter().any(|(_, n)| n == name));
  let handle_docs = HANDLE_DOCS
    .iter()
    .map(|(receiver, name, _)| (Some(*receiver), *name))
    .filter(|(receiver, name)| {
      !live
        .iter()
        .any(|(r, n)| r.as_deref() == *receiver && n == name)
    });
  docs
    .chain(handle_docs)
    .map(|(receiver, name)| qualified(receiver, name))
    .collect()
}

/// Names registered on more than one handle that do not have a [`HANDLE_DOCS`]
/// entry for each — they would otherwise share one [`DOCS`] cell, which can only
/// be right for one of them (`setAttribute` shipped documented as
/// `Document::get_node` until this check existed).
pub(super) fn ambiguous_names(engine: &Engine) -> Vec<String> {
  let calls = registered_calls(engine);
  let mut ambiguous: Vec<String> = calls
    .iter()
    .filter(|(receiver, name)| {
      calls.iter().any(|(r, n)| n == name && r != receiver)
        && !HANDLE_DOCS
          .iter()
          .any(|(r, n, _)| Some(*r) == receiver.as_deref() && n == name)
    })
    .map(|(receiver, name)| qualified(receiver.as_deref(), name))
    .collect();
  ambiguous.sort();
  ambiguous.dedup();
  ambiguous
}

/// Every registered call as `(handle, name)` — the pair a doc entry is keyed
/// by, since the same name on two handles is two different calls.
fn registered_calls(engine: &Engine) -> Vec<(Option<String>, String)> {
  engine
    .gen_fn_signatures(false)
    .iter()
    .map(|s| {
      let sig = Signature::parse(s);
      (sig.receiver, sig.name)
    })
    .collect()
}

fn qualified(receiver: Option<&str>, name: &str) -> String {
  match receiver {
    Some(handle) => format!("{handle}::{name}"),
    None => name.to_string(),
  }
}

/// The doc entry for one call: the handle-specific one when there is one,
/// otherwise the name-keyed one.
fn lookup(receiver: Option<&str>, name: &str) -> Option<&'static Doc> {
  HANDLE_DOCS
    .iter()
    .find(|(r, n, _)| Some(*r) == receiver && *n == name)
    .map(|(_, _, doc)| doc)
    .or_else(|| DOCS.iter().find(|(n, _)| *n == name).map(|(_, doc)| doc))
}

/// The documentation cell for one call: a rustdoc intra-doc link, or the note.
fn doc_cell(receiver: Option<&str>, name: &str) -> String {
  match lookup(receiver, name) {
    Some(Doc::Rust(path)) => format!("[`{}`]({path})", short_path(path)),
    Some(Doc::Note(note)) => (*note).to_string(),
    None => String::new(),
  }
}

/// Show `Document::insert_xml`, not the whole module path — the type qualifier
/// is what disambiguates, the module chain is noise in a table cell.
fn short_path(path: &str) -> String {
  let Some((head, tail)) = path.rsplit_once("::") else {
    return path.to_string();
  };
  match head.rsplit_once("::") {
    Some((_, ty)) if ty.starts_with(char::is_uppercase) => format!("{ty}::{tail}"),
    _ => tail.to_string(),
  }
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
    // Overloads collapse into one row: they share a name, and a name is what
    // has documentation.
    let mut by_name: Vec<(&str, Vec<String>)> = Vec::new();
    for sig in &group {
      match by_name.last_mut() {
        Some((n, sigs)) if *n == sig.name => sigs.push(sig.render()),
        _ => by_name.push((&sig.name, vec![sig.render()])),
      }
    }
    out.push_str(&format!(
      "\n## {title}\n\n{blurb}\n\n{} function{}, {} call{}.\n\n\
       | call | documentation |\n|---|---|\n",
      by_name.len(),
      if by_name.len() == 1 { "" } else { "s" },
      group.len(),
      if group.len() == 1 { "" } else { "s" }
    ));
    for (name, sigs) in by_name {
      out.push_str(&format!(
        "| {} | {} |\n",
        sigs
          .iter()
          .map(|s| format!("`{s}`"))
          .collect::<Vec<_>>()
          .join("<br>"),
        doc_cell(receiver.as_deref(), name)
      ));
    }
  }
  out
}

const HEADER: &str = r#"# The `.rhai` binding interface

Every function a runtime (Rhai) binding can call, grouped by the handle it is
called on. **Generated** from the live engine via Rhai's reflections API
(`Engine::gen_fn_signatures`) and checked on every test run, so it always
matches what is actually registered — see `api_doc.rs`.

Overloads share a row — the accepted argument shapes differ, what the call means
does not.

The **documentation** column links the Rust item that implements the call: that
is where its exact semantics live, and linking rather than paraphrasing is what
keeps this reference from drifting out of step with the engine. Where a call has
no single counterpart — a Rhai-only helper, or a shim whose whole behaviour is in
the registration — the column describes it instead. A name registered on two
different handles is two different calls, and is documented separately under
each.

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
