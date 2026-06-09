//! End-to-end validation of runtime Rhai script bindings through a real
//! conversion (docs/script_bindings_plan.md, milestone M4).
//!
//! A sample binding (one `DefMacro`, one `DefConstructor`) is authored in Rhai
//! and loaded at runtime via the *extra* binding dispatcher when the document
//! `\usepackage{lxrhaitest}`s it — exactly the path real contrib packages use.
//! We then assert on the produced XML: the macro must expand and the constructor
//! must emit its element.
#![cfg(feature = "script-bindings")]

use std::rc::Rc;

use latexml::core_interface::DigestionAPI;
use latexml_core::common::error::Result;
use latexml_core::state;
use latexml_core::{Core, CoreOptions};

/// A sample contrib binding, authored in Rhai (no Rust toolchain, no recompile).
const SAMPLE: &str = r#"
  // Expandable macro: \twicex{X} -> XX
  DefMacro("\\twicex{}", |x| x + x);

  // Constructor (imperative, proxy syntax close to Perl's $document->method):
  // \myemph{X} -> <ltx:emph>X</ltx:emph>.
  DefConstructor("\\myemph{}", |document, x| {
    document.openElement("ltx:emph");
    document.absorb(x);
    document.closeElement("ltx:emph");
  });

  // The documentation example, translated 1:1 from Perl. A no-arg constructor
  // that just (maybe-)closes elements; safe no-op where they aren't open.
  DefConstructor("\\endreferences", |document| {
    document.maybeCloseElement("ltx:biblist");
    document.maybeCloseElement("ltx:bibliography");
  });

  // Constructor (template form, the dominant dialect): \mytext{X} ->
  // <ltx:text class="rhai">X</ltx:text>, executed by the runtime template
  // interpreter (no Rhai per invocation).
  DefConstructor("\\mytext{}", "<ltx:text class=\"rhai\">#1</ltx:text>");

  // Re-entrancy: \wrap absorbs its (already-digested) argument, so
  // \wrap{\myemph{..}} makes one script constructor's body trigger another
  // script constructor's construction while \wrap's active-context is live.
  DefConstructor("\\wrap{}", |document, x| {
    document.openElement("ltx:text");
    document.absorb(x);
    document.closeElement("ltx:text");
  });

  // Imperative constructor exercising attributes + literal text:
  // \note{N} -> <ltx:text class="note">[N]</ltx:text>
  DefConstructor("\\note{}", |document, x| {
    document.openElement("ltx:text");
    document.setAttribute("class", "note");
    document.absorbString("[");
    document.absorb(x);
    document.absorbString("]");
    document.closeElement("ltx:text");
  });

  // Variable argument order + omission: 3 required args, template uses #3 then
  // #1 (reordered) and never references #2 (omitted) — exactly as the Rust
  // DefConstructor! macro / Perl template would.
  DefConstructor("\\rot{}{}{}", "<ltx:text>#3#1</ltx:text>");

  // Primitive: digestion-time side-effect into State (global, so it survives the
  // document group for the post-conversion assertion).
  DefPrimitive("\\setx{}", |v| { assign_global("script:x", v); });
"#;

/// Extra dispatcher: load the sample script when `lxrhaitest` is requested.
fn script_dispatch(request: &str) -> Option<Result<()>> {
  let base = request.split('.').next().unwrap_or(request);
  if base == "lxrhaitest" {
    Some(latexml_contrib::script_bindings::load_script(SAMPLE).map(|_| ()))
  } else {
    None
  }
}

#[test]
fn script_binding_macro_and_constructor_convert() {
  let mut latexml = Core::new(CoreOptions {
    verbosity: Some(-2),
    include_comments: Some(false),
    ..CoreOptions::default()
  });
  state::set_bindings_dispatch(Rc::new(latexml_package::dispatch));
  state::add_binding_names(latexml_package::binding_names());
  state::set_extra_bindings_dispatch(Rc::new(script_dispatch));

  let tex = concat!(
    "literal:\\documentclass{article}\\usepackage{lxrhaitest}",
    "\\begin{document}\\twicex{ab} \\myemph{hi} \\mytext{zz} \\wrap{\\myemph{deep}} \\note{N} \\rot{xx}{yy}{zz2} \\endreferences \\setx{hello}\\end{document}"
  );
  let doc = latexml
    .convert_file(tex.to_string())
    .expect("conversion with a script binding should succeed");
  let xml = doc.serialize_to_string();

  // NB: the serializer emits the LaTeXML namespace as the default (no `ltx:`
  // prefix), so elements appear unprefixed.
  assert!(xml.contains("abab"), "macro \\twicex did not expand; xml=\n{xml}");
  assert!(
    xml.contains("<emph>hi</emph>"),
    "imperative constructor \\myemph did not emit its element; xml=\n{xml}"
  );
  assert!(
    xml.contains("class=\"rhai\"") && xml.contains("zz"),
    "template constructor \\mytext did not emit; xml=\n{xml}"
  );
  // Re-entrancy (GATE-1): the nested script constructor ran inside another
  // script constructor's body without a borrow panic.
  assert!(
    xml.contains("<emph>deep</emph>"),
    "re-entrant nested script constructor failed; xml=\n{xml}"
  );
  // Imperative attributes + text (el_attr/el_text).
  assert!(
    xml.contains("class=\"note\""),
    "imperative el_attr/el_text constructor \\note failed; xml=\n{xml}"
  );
  // Variable argument order + omission: \rot{xx}{yy}{zz2} with template
  // "#3#1" must yield "zz2xx" (order #3 then #1) and never "yy" (arg #2 omitted).
  assert!(
    xml.contains("zz2xx") && !xml.contains("yy"),
    "variable-order/omitted-arg handling failed; xml=\n{xml}"
  );

  // Primitive seam: the digestion-time side-effect persisted into State.
  let stored = latexml_core::state::lookup_value("script:x");
  let val = match stored {
    Some(latexml_core::common::store::Stored::String(s)) => Some(latexml_core::common::arena::to_string(s)),
    _ => None,
  };
  assert_eq!(
    val.as_deref(),
    Some("hello"),
    "primitive \\setx side-effect not observed in State"
  );

  drop(latexml);
  latexml_core::reset_thread_engine();
}
