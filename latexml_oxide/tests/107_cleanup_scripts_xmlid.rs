//! Guard for the re-enabled `cleanup_scripts` pass — the port of Perl
//! `MathParser.pm:cleanupScripts` (L106-126).
//!
//! The pass was silently dead: its bare `get_attribute("xml:id")` reads always
//! returned `None` (xml:id is stored namespaced — local `id` in the XML
//! namespace), so every iteration bailed at the `appid` read and no XMRef was
//! ever redirected. This test drives it against a crafted XMath fragment in a
//! fully-initialized session (the schema model gates both `generate_id` and
//! the attribute copy of the replacement build, so a bare `Document` is not a
//! faithful environment — role= would be dropped and namespaces misresolved).
//!
//! Covers the `createXMRefs` branches the pass exercises (Perl `Package.pm`
//! L1544-1575): a script child that already has an id ("refer to it"), a
//! script child with NO id (gets one via `generate_id`), and an XMRef child
//! (its idref is cloned — never a ref-to-a-ref).

mod cluster;

use latexml_core::{common::xml::XML_NS, document::Document};
use latexml_math_parser::MathParser;

#[test]
fn cleanup_scripts_redirects_script_xmapp_refs() {
  // A tiny conversion first: initializes the session singletons, the schema
  // model, and namespace registration that the pass's replacement build
  // depends on. The output itself is irrelevant here. (The cluster helpers
  // take a file path, so stage the doc in a tempdir.)
  let boot = tempfile::tempdir().expect("tempdir");
  let boot_tex = boot.path().join("boot.tex");
  std::fs::write(
    &boot_tex,
    "\\documentclass{article}\n\\begin{document}\n$x$\n\\end{document}\n",
  )
  .expect("write boot.tex");
  cluster::convert_clean(boot_tex.to_str().expect("utf8 path"));

  // NOTE: no whitespace inside the XMApps — the pass takes `firstChild`
  // verbatim (as Perl does; XMath trees carry no indentation text nodes),
  // so a pretty-printed fixture would hand it a text node.
  let xml_doc = libxml::parser::Parser::default()
    .parse_string(
      r#"<?xml version="1.0"?>
<XMath xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="m1"><XMDual><XMRef idref="m1.app1"/><XMApp xml:id="m1.app1" role="POSTSUBSCRIPT"><XMTok xml:id="m1.x1">b</XMTok></XMApp></XMDual><XMDual><XMRef idref="m1.app2"/><XMApp xml:id="m1.app2" role="POSTSUPERSCRIPT"><XMRef idref="m1.tok9"/></XMApp></XMDual><XMDual><XMRef idref="m1.app3"/><XMApp xml:id="m1.app3" role="FLOATSUPERSCRIPT"><XMTok>c</XMTok></XMApp></XMDual><XMTok xml:id="m1.tok9">d</XMTok></XMath>"#,
    )
    .expect("parse fixture");
  let mut document = Document::from_xml_document(xml_doc, Default::default()).expect("wrap");
  let mut parser = MathParser::default();
  parser.cleanup_scripts(&mut document).expect("cleanup");

  // Every script XMApp lost its id (NS-aware remove + unrecord)…
  for app_id in ["m1.app1", "m1.app2", "m1.app3"] {
    assert!(
      document
        .findnodes(&format!("//*[@xml:id='{app_id}']"), None)
        .is_empty(),
      "{app_id} must lose its xml:id"
    );
    assert!(document.lookup_id(app_id).is_none());
    // …so nothing may still reference it.
    assert!(
      document
        .findnodes(&format!("//*[@idref='{app_id}']"), None)
        .is_empty(),
      "no ref to the stripped {app_id} may survive"
    );
  }

  // Branch 1 (script child already carrying an id — Perl createXMRefs
  // L1563-1565 "already has id, so refer to it"): the dual's ref slot became
  // an XMApp (attrs copied from app1) wrapping an XMRef to the XMTok itself.
  assert_eq!(
    document
      .findnodes(
        "//*[local-name()='XMApp' and @role='POSTSUBSCRIPT']/*[local-name()='XMRef' and @idref='m1.x1']",
        None
      )
      .len(),
    1,
    "exactly one replacement XMApp[XMRef -> already-id'd script tok] expected"
  );

  // Branch 2 (XMRef script child): the replacement clones the idref —
  // original + replacement both point at m1.tok9 directly, and no XMRef
  // acquired an xml:id of its own (no ref-to-a-ref).
  assert_eq!(
    document
      .findnodes(
        "//*[local-name()='XMApp' and @role='POSTSUPERSCRIPT']/*[local-name()='XMRef' and @idref='m1.tok9']",
        None
      )
      .len(),
    2,
    "original app2 and its replacement must both ref m1.tok9 directly"
  );
  assert!(
    document
      .findnodes("//*[local-name()='XMRef' and @xml:id]", None)
      .is_empty(),
    "no XMRef may be given an xml:id (ref-to-a-ref)"
  );

  // Branch 3 (id-less script child): generate_id gave the XMTok an id, the
  // replacement refs it, and the id is recorded in the idstore.
  let tok3 = document
    .findnodes(
      "//*[local-name()='XMApp' and @role='FLOATSUPERSCRIPT']/*[local-name()='XMTok']",
      None,
    )
    .into_iter()
    .next()
    .expect("original app3 still holds its XMTok");
  let tok3_id = tok3
    .get_attribute_ns("id", XML_NS)
    .expect("id-less script content must receive a generated xml:id");
  assert!(
    document.lookup_id(&tok3_id).is_some(),
    "generated script id must be recorded in the idstore"
  );
  assert_eq!(
    document
      .findnodes(
        &format!(
          "//*[local-name()='XMApp' and @role='FLOATSUPERSCRIPT']/*[local-name()='XMRef' and @idref='{tok3_id}']"
        ),
        None
      )
      .len(),
    1,
    "replacement must ref the generated script id"
  );

  // The replacement XMApps live in the LaTeXML namespace (the build sets the
  // namespace from the original app; a bare environment misresolved this).
  for repl in document.findnodes("//*[local-name()='XMApp' and @role]", None) {
    assert_eq!(
      repl.get_namespace().map(|ns| ns.get_href()).as_deref(),
      Some("http://dlmf.nist.gov/LaTeXML"),
      "replacement XMApp must stay in the ltx namespace"
    );
  }

  // Duplicate-id guard: the attrs copy must not mint a second xml:id
  // (get_attributes() reports xml:id under its LOCAL name "id" — copying it
  // onto every replacement would duplicate ids the moment the pass fires).
  let all_ids: Vec<String> = document
    .findnodes("//*[@xml:id]", None)
    .into_iter()
    .filter_map(|n| n.get_attribute_ns("id", XML_NS))
    .collect();
  let unique: std::collections::HashSet<&String> = all_ids.iter().collect();
  assert_eq!(
    all_ids.len(),
    unique.len(),
    "duplicate xml:id minted: {all_ids:?}"
  );
}
