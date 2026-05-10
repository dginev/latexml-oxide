//! RelaxNG XML → AST scanner.
//!
//! Port of `RelaxNG.pm` lines 100–390. The Perl original is a recursive
//! visitor over a `LibXML::Document`; this is a recursive visitor over
//! `libxml::tree::Node`.
//!
//! The scanner is **side-effect-free with respect to the AST**: it
//! returns a `Vec<Pattern>` per node and never mutates `Relaxng`'s
//! definition tables. The only pieces it touches on `Relaxng` are
//! `internal_grammars` (a fresh counter for embedded `<grammar>`
//! blocks) and `document_namespaces` (driven by `xmlns:` declarations
//! on RelaxNG nodes — analogous to `Model::registerDocumentNamespace`
//! in Perl). Definition recording, "Used by" graph, element tables —
//! all happen during [`super::simplify`].
//!
//! The matching style intentionally mirrors `getRelaxOp` + the
//! `$relaxop eq 'rng:foo'` cascade in Perl, so a reader who knows the
//! original code finds the same shape here.

use libxml::parser::Parser as XmlParser;
use libxml::readonly::RoNode;
use libxml::tree::{Document as XmlDocument, NodeType};
use std::path::{Path, PathBuf};

use super::{CombineOp, DefCombiner, Pattern, Relaxng};

/// RelaxNG namespace URI.
const RNG_NS: &str = "http://relaxng.org/ns/structure/1.0";
/// Compatibility-annotations namespace URI (carries `<a:documentation>`).
const RNGA_NS: &str = "http://relaxng.org/ns/compatibility/annotations/1.0";

/// Errors a scan can produce.
#[derive(Debug)]
pub enum ScanError {
  /// The named .rng file could not be located on `search_paths`.
  FileNotFound(String),
  /// libxml could not parse the .rng file.
  Parse(String),
  /// A non-fatal "unrecognised RelaxNG construct" warning escalated to
  /// an error (we collect these and continue, but the caller may want
  /// to inspect the list afterward).
  UnknownOp { op: String, file: PathBuf },
}

impl std::fmt::Display for ScanError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ScanError::FileNotFound(name) => write!(f, "RelaxNG file not found: {}", name),
      ScanError::Parse(msg) => write!(f, "RelaxNG parse error: {}", msg),
      ScanError::UnknownOp { op, file } => {
        write!(f, "Unknown RelaxNG op '{}' in {}", op, file.display())
      },
    }
  }
}

impl std::error::Error for ScanError {}

// ----- public entry points ------------------------------------------------

/// Locate, parse, and scan a single RelaxNG schema file.
///
/// Wraps the result in `Pattern::Module` whose `name` is the file's
/// basename without extension — matches `scanExternal` in Perl.
pub fn scan_external(
  rng: &mut Relaxng,
  name: &str,
  inherit_ns: Option<&str>,
  search_paths: &[&Path],
) -> Result<Vec<Pattern>, ScanError> {
  let path = find_file(name, search_paths)
    .ok_or_else(|| ScanError::FileNotFound(name.to_string()))?;
  let parser = XmlParser::default();
  let xml_doc: XmlDocument = parser
    .parse_file(path.to_str().unwrap_or(""))
    .map_err(|e| ScanError::Parse(format!("{:?}", e)))?;
  let root = xml_doc
    .get_root_readonly()
    .ok_or_else(|| ScanError::Parse("empty document".into()))?;

  // Collect namespace declarations on the root for downstream qname
  // resolution. (LaTeXML's encodeQName-equivalent — but simpler: we
  // just record the prefix→URI map.)
  collect_namespaces(rng, root);

  // First-call-wins: capture the master grammar's `ns="…"` URI as
  // the schema's primary namespace. Recursive scan_external calls
  // (for `<externalRef>` etc.) don't overwrite this — they're
  // satellite modules whose ns may differ from the entry-point.
  if rng.primary_namespace.is_none() {
    if let Some(uri) = root.get_attribute("ns") {
      if !uri.is_empty() {
        rng.primary_namespace = Some(uri);
      }
    }
  }

  let modname = strip_rng_ext(name);
  let mut new_paths: Vec<&Path> = Vec::with_capacity(search_paths.len() + 1);
  let dir = path.parent().unwrap_or(Path::new("."));
  new_paths.push(dir);
  new_paths.extend(search_paths);

  // The scanner takes the search-paths slice via context so includes
  // resolve relative to *this* file's directory first.
  let mut ctx = ScanContext { search_paths: new_paths };
  let body = scan_pattern(rng, root, inherit_ns, &mut ctx)?;
  Ok(vec![Pattern::Module { name: modname, body }])
}

/// Scan an already-parsed RelaxNG XML root in-memory. Useful for unit
/// tests that pass small RNG fragments inline.
pub fn scan_string(rng: &mut Relaxng, xml: &str) -> Result<Vec<Pattern>, ScanError> {
  let parser = XmlParser::default();
  let xml_doc = parser
    .parse_string(xml)
    .map_err(|e| ScanError::Parse(format!("{:?}", e)))?;
  let root = xml_doc
    .get_root_readonly()
    .ok_or_else(|| ScanError::Parse("empty document".into()))?;
  collect_namespaces(rng, root);
  let mut ctx = ScanContext { search_paths: Vec::new() };
  scan_pattern(rng, root, None, &mut ctx)
}

// ----- internal recursion -------------------------------------------------

struct ScanContext<'p> {
  search_paths: Vec<&'p Path>,
}

/// Compute the RelaxNG op identifier for `node`, e.g. `"rng:element"`.
/// Returns `None` for non-element nodes or for elements outside the
/// RelaxNG / compatibility-annotations namespaces.
fn get_relax_op(node: RoNode) -> Option<String> {
  if node.get_type() != Some(NodeType::ElementNode) {
    return None;
  }
  let local = node.get_name();
  let ns_uri = node
    .get_namespace()
    .map(|ns| ns.get_href())
    .unwrap_or_default();
  let prefix = match ns_uri.as_str() {
    RNG_NS => "rng",
    RNGA_NS => "rnga",
    "" => return None,
    other => return Some(format!("{{{}}}:{}", other, local)),
  };
  Some(format!("{}:{}", prefix, local))
}

/// Element-only children of `node` (filters out text nodes and
/// comments). Mirrors `getElements` in Perl.
fn get_elements(node: RoNode) -> Vec<RoNode> {
  let mut out = Vec::new();
  let mut child = node.get_first_child();
  while let Some(c) = child {
    if c.get_type() == Some(NodeType::ElementNode) {
      out.push(c);
    }
    child = c.get_next_sibling();
  }
  out
}

/// Map a RelaxNG combiner localname (`group`/`interleave`/…) to
/// [`CombineOp`].
fn combine_op_from_localname(name: &str) -> Option<CombineOp> {
  Some(match name {
    "group" => CombineOp::Group,
    "interleave" => CombineOp::Interleave,
    "choice" => CombineOp::Choice,
    "optional" => CombineOp::Optional,
    "zeroOrMore" => CombineOp::ZeroOrMore,
    "oneOrMore" => CombineOp::OneOrMore,
    "list" => CombineOp::List,
    _ => return None,
  })
}

/// Encode `(ns?, local)` as the `prefix:local` qname Perl
/// `Model::encodeQName` produces. With no namespace, returns `local`
/// unchanged. For URIs without a non-empty prefix mapping yet,
/// synthesises a fresh `namespace<N>` prefix and registers it (mirrors
/// LaTeXML's `getDocumentNamespacePrefix(...)` auto-assignment).
fn encode_qname(rng: &mut Relaxng, ns: Option<&str>, local: &str) -> String {
  match ns {
    None | Some("") => local.to_string(),
    Some(uri) => format!("{}:{}", ensure_prefix(rng, uri), local),
  }
}

fn ensure_prefix(rng: &mut Relaxng, uri: &str) -> String {
  if let Some((prefix, _)) = rng
    .document_namespaces
    .iter()
    .find(|(p, u)| !p.is_empty() && u.as_str() == uri)
  {
    return prefix.clone();
  }
  let n = rng
    .document_namespaces
    .keys()
    .filter(|p| p.starts_with("namespace"))
    .count()
    + 1;
  let new_prefix = format!("namespace{}", n);
  rng
    .document_namespaces
    .insert(new_prefix.clone(), uri.to_string());
  new_prefix
}

/// Walk a single RelaxNG pattern node. `inherit_ns` is the namespace
/// that scope around `node` would assign to unqualified names (the
/// nearest enclosing `ns="..."` attribute).
fn scan_pattern(
  rng: &mut Relaxng,
  node: RoNode,
  inherit_ns: Option<&str>,
  ctx: &mut ScanContext<'_>,
) -> Result<Vec<Pattern>, ScanError> {
  let Some(op) = get_relax_op(node) else { return Ok(Vec::new()); };
  let ns = node
    .get_attribute("ns")
    .or_else(|| inherit_ns.map(String::from));
  let ns_ref = ns.as_deref();

  match op.as_str() {
    "rng:element" => scan_pattern_element(rng, ns_ref, node, ctx),
    "rng:attribute" => scan_pattern_attribute(rng, ns_ref, node, ctx),
    "rng:mixed" => {
      let mut body = vec![Pattern::Text];
      body.extend(scan_children(rng, ns_ref, get_elements(node), ctx)?);
      Ok(vec![Pattern::Combination { op: CombineOp::Interleave, body }])
    },
    "rng:ref" => Ok(vec![Pattern::Ref {
      qname: node.get_attribute("name").unwrap_or_default(),
    }]),
    "rng:parentRef" => Ok(vec![Pattern::ParentRef {
      qname: node.get_attribute("name").unwrap_or_default(),
    }]),
    "rng:empty" | "rng:notAllowed" => Ok(Vec::new()),
    "rng:text" => Ok(vec![Pattern::Text]),
    "rng:value" => Ok(vec![Pattern::Value(node.get_content())]),
    "rng:data" => Ok(vec![Pattern::Data(node.get_attribute("type").unwrap_or_default())]),
    "rng:externalRef" => {
      let href = node.get_attribute("href").unwrap_or_default();
      let paths: Vec<&Path> = ctx.search_paths.iter().copied().collect();
      scan_external(rng, &href, ns_ref, &paths)
    },
    "rng:grammar" => {
      rng.internal_grammars += 1;
      let name = format!("grammar{}", rng.internal_grammars);
      let body = scan_grammar_content(rng, ns_ref, get_elements(node), ctx)?;
      Ok(vec![Pattern::Grammar { name, body }])
    },
    "rnga:documentation" => {
      let text = node.get_content();
      Ok(vec![Pattern::Doc(text)])
    },
    other => {
      // Combiners (group/interleave/choice/optional/zeroOrMore/
      // oneOrMore/list).
      if let Some(stripped) = other.strip_prefix("rng:") {
        if let Some(cop) = combine_op_from_localname(stripped) {
          let body = scan_children(rng, ns_ref, get_elements(node), ctx)?;
          return Ok(vec![Pattern::Combination { op: cop, body }]);
        }
      }
      // Unknown — Perl warns and returns empty; we do the same.
      Ok(Vec::new())
    },
  }
}

fn scan_pattern_element(
  rng: &mut Relaxng,
  ns: Option<&str>,
  node: RoNode,
  ctx: &mut ScanContext<'_>,
) -> Result<Vec<Pattern>, ScanError> {
  let mut children = get_elements(node);
  if let Some(name) = node.get_attribute("name") {
    let body = scan_children(rng, ns, children, ctx)?;
    Ok(vec![Pattern::Element { name: encode_qname(rng, ns, &name), body }])
  } else if !children.is_empty() {
    let name_node = children.remove(0);
    let names = scan_name_class(rng, name_node, false, ns);
    let body_proto = scan_children(rng, ns, children, ctx)?;
    Ok(
      names
        .into_iter()
        .map(|n| Pattern::Element { name: n, body: body_proto.clone() })
        .collect(),
    )
  } else {
    Ok(Vec::new())
  }
}

fn scan_pattern_attribute(
  rng: &mut Relaxng,
  ns: Option<&str>,
  node: RoNode,
  ctx: &mut ScanContext<'_>,
) -> Result<Vec<Pattern>, ScanError> {
  let xns = node.get_attribute("ns"); // EXPLICIT only (no inherit)
  let xns_ref = xns.as_deref();
  let mut children = get_elements(node);
  if let Some(name) = node.get_attribute("name") {
    let body = scan_children(rng, ns, children, ctx)?;
    Ok(vec![Pattern::Attribute { name: encode_qname(rng, xns_ref, &name), body }])
  } else if !children.is_empty() {
    let name_node = children.remove(0);
    let names = scan_name_class(rng, name_node, true, ns);
    let body_proto = scan_children(rng, ns, children, ctx)?;
    Ok(
      names
        .into_iter()
        .map(|n| Pattern::Attribute { name: n, body: body_proto.clone() })
        .collect(),
    )
  } else {
    Ok(Vec::new())
  }
}

fn scan_children(
  rng: &mut Relaxng,
  ns: Option<&str>,
  children: Vec<RoNode>,
  ctx: &mut ScanContext<'_>,
) -> Result<Vec<Pattern>, ScanError> {
  let mut out = Vec::new();
  for child in children {
    out.extend(scan_pattern(rng, child, ns, ctx)?);
  }
  Ok(out)
}

fn scan_grammar_content(
  rng: &mut Relaxng,
  ns: Option<&str>,
  content: Vec<RoNode>,
  ctx: &mut ScanContext<'_>,
) -> Result<Vec<Pattern>, ScanError> {
  let mut out = Vec::new();
  for node in content {
    out.extend(scan_grammar_item(rng, node, ns, ctx)?);
  }
  Ok(out)
}

fn scan_grammar_item(
  rng: &mut Relaxng,
  node: RoNode,
  inherit_ns: Option<&str>,
  ctx: &mut ScanContext<'_>,
) -> Result<Vec<Pattern>, ScanError> {
  let Some(op) = get_relax_op(node) else { return Ok(Vec::new()); };
  let children = get_elements(node);
  let ns = node
    .get_attribute("ns")
    .or_else(|| inherit_ns.map(String::from));
  let ns_ref = ns.as_deref();

  match op.as_str() {
    "rng:start" => {
      let body = scan_children(rng, ns_ref, children, ctx)?;
      Ok(vec![Pattern::Start { body }])
    },
    "rng:define" => {
      let name = node.get_attribute("name").unwrap_or_default();
      let combiner = match node.get_attribute("combine").as_deref() {
        Some("choice") => DefCombiner::Choice,
        Some("interleave") => DefCombiner::Interleave,
        _ => DefCombiner::Group,
      };
      let body = scan_children(rng, ns_ref, children, ctx)?;
      Ok(vec![Pattern::Def { combiner, name, body }])
    },
    "rng:div" => scan_grammar_content(rng, ns_ref, children, ctx),
    "rng:include" => {
      let href = node.get_attribute("href").unwrap_or_default();
      let paths: Vec<&Path> = ctx.search_paths.iter().copied().collect();
      // Find + parse the included file.
      let path = find_file(&href, &paths)
        .ok_or_else(|| ScanError::FileNotFound(href.clone()))?;
      let parser = XmlParser::default();
      let xml_doc = parser
        .parse_file(path.to_str().unwrap_or(""))
        .map_err(|e| ScanError::Parse(format!("{:?}", e)))?;
      let inner_root = xml_doc
        .get_root_readonly()
        .ok_or_else(|| ScanError::Parse("empty include".into()))?;
      collect_namespaces(rng, inner_root);
      // Push the included file's directory to the search path so its
      // own includes resolve correctly.
      let dir = path.parent().unwrap_or(Path::new("."));
      let mut nested_paths: Vec<&Path> = Vec::with_capacity(ctx.search_paths.len() + 1);
      nested_paths.push(dir);
      nested_paths.extend(&ctx.search_paths);
      let mut nested_ctx = ScanContext { search_paths: nested_paths };

      // Ignore the outer <grammar>, if any (`<include>` doesn't establish
      // a binding in RelaxNG).
      let patterns = if get_relax_op(inner_root).as_deref() == Some("rng:grammar") {
        let nns = inner_root
          .get_attribute("ns")
          .or_else(|| inherit_ns.map(String::from));
        scan_grammar_content(
          rng,
          nns.as_deref(),
          get_elements(inner_root),
          &mut nested_ctx,
        )?
      } else {
        scan_pattern(rng, inner_root, None, &mut nested_ctx)?
      };

      let modname = strip_rng_ext(&href);
      let module = Pattern::Module { name: modname, body: patterns };
      let replacements = scan_grammar_content(rng, ns_ref, children, ctx)?;
      if replacements.is_empty() {
        Ok(vec![module])
      } else {
        Ok(vec![Pattern::Override {
          module: Box::new(module),
          replacements,
        }])
      }
    },
    _ => Ok(Vec::new()),
  }
}

/// Walk a `<name>`/`<anyName>`/`<nsName>`/`<choice>`/`<except>` name-class.
/// Returns the qnames covered, exclusions appearing as `!qname` per the
/// Perl convention.
fn scan_name_class(
  rng: &mut Relaxng,
  node: RoNode,
  for_attr: bool,
  ns: Option<&str>,
) -> Vec<String> {
  let Some(op) = get_relax_op(node) else { return Vec::new(); };
  match op.as_str() {
    "rng:name" => {
      let raw = node.get_content();
      let (decns, local) = decode_qname(rng, &raw);
      let effective_ns = decns.as_deref().or(ns);
      let resolved_ns = if for_attr { None } else { effective_ns };
      vec![encode_qname(rng, resolved_ns, &local)]
    },
    "rng:anyName" => {
      let except: Vec<String> = get_elements(node)
        .into_iter()
        .flat_map(|c| scan_name_class(rng, c, for_attr, ns))
        .collect();
      let mut all = vec!["*".to_string(), "*:*".to_string()];
      all.extend(except);
      filter_names(all)
    },
    "rng:nsName" => {
      let xns = node.get_attribute("ns").or_else(|| ns.map(String::from));
      let star = encode_qname(rng, xns.as_deref(), "*");
      let except: Vec<String> = get_elements(node)
        .into_iter()
        .flat_map(|c| scan_name_class(rng, c, for_attr, ns))
        .collect();
      let mut all = vec![star];
      all.extend(except);
      filter_names(all)
    },
    "rng:choice" => {
      let mut names = std::collections::BTreeSet::new();
      let mut child = node.get_first_child();
      while let Some(c) = child {
        for n in scan_name_class(rng, c, for_attr, ns) {
          names.insert(n);
        }
        child = c.get_next_sibling();
      }
      names.into_iter().collect()
    },
    "rng:except" => {
      let mut names = std::collections::BTreeSet::new();
      for c in get_elements(node) {
        for n in scan_name_class(rng, c, for_attr, ns) {
          names.insert(n);
        }
      }
      names.into_iter().map(|n| format!("!{}", n)).collect()
    },
    _ => Vec::new(),
  }
}

/// Collapse `(*:*, !*:*)` etc. — drops exclusions that cancel an
/// inclusion. Perl `filterNames`.
fn filter_names(names: Vec<String>) -> Vec<String> {
  use std::collections::BTreeMap;
  let mut include: BTreeMap<String, String> = BTreeMap::new();
  let mut exclude: BTreeMap<String, String> = BTreeMap::new();
  for n in names {
    if let Some(rest) = n.strip_prefix('!') {
      exclude.insert(n.clone(), rest.to_string());
    } else {
      include.insert(n.clone(), n);
    }
  }
  let drop_keys: Vec<String> = exclude
    .iter()
    .filter(|(_, target)| include.contains_key(target.as_str()))
    .map(|(k, _)| k.clone())
    .collect();
  for k in drop_keys {
    if let Some(target) = exclude.remove(&k) {
      include.remove(&target);
    }
  }
  include.keys().cloned().chain(exclude.keys().cloned()).collect()
}

/// Split a `prefix:local` token into `(ns_uri?, local)` using the
/// schema's namespace map. Mirrors Perl's `decodeQName`.
fn decode_qname(rng: &Relaxng, raw: &str) -> (Option<String>, String) {
  match raw.split_once(':') {
    Some((prefix, local)) => match rng.document_namespaces.get(prefix) {
      Some(uri) => (Some(uri.clone()), local.to_string()),
      None => (None, raw.to_string()),
    },
    None => (None, raw.to_string()),
  }
}

// ----- helpers ------------------------------------------------------------

fn collect_namespaces(rng: &mut Relaxng, root: RoNode) {
  for ns in root.get_namespace_declarations() {
    let prefix = ns.get_prefix();
    let href = ns.get_href();
    if href.starts_with("http://relaxng.org") {
      continue;
    }
    // If the schema author bound a different prefix to this URI, drop
    // any pre-seeded conventional binding so the schema's choice wins.
    if !prefix.is_empty() {
      rng
        .document_namespaces
        .retain(|p, u| p == &prefix || u != &href);
    }
    rng.document_namespaces.insert(prefix, href);
  }
}

fn strip_rng_ext(name: &str) -> String {
  name
    .strip_suffix(".rng")
    .or_else(|| name.strip_suffix(".rnc"))
    .unwrap_or(name)
    .to_string()
}

/// Resolve a schema reference. Honours LaTeXML's `urn:x-LaTeXML:RelaxNG:`
/// URN scheme: strips the prefix, then translates remaining `:`
/// separators into path separators so e.g.
/// `urn:x-LaTeXML:RelaxNG:svg:svg11.rng` → `svg/svg11.rng` lookup.
fn find_file(name: &str, search_paths: &[&Path]) -> Option<PathBuf> {
  let bare = match name.strip_prefix("urn:x-LaTeXML:RelaxNG:") {
    Some(rest) => rest.replace(':', "/"),
    None => name.to_string(),
  };
  let asis = Path::new(&bare);
  if asis.is_file() {
    return Some(asis.to_path_buf());
  }
  for dir in search_paths {
    let candidate = dir.join(&bare);
    if candidate.is_file() {
      return Some(candidate);
    }
  }
  None
}

// ----- unit tests ---------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;
  use crate::common::relaxng::Pattern;

  fn matches_combination(pat: &Pattern, op: CombineOp) -> bool {
    matches!(pat, Pattern::Combination { op: o, .. } if *o == op)
  }

  #[test]
  fn scan_empty_grammar() {
    let xml = r#"<grammar xmlns="http://relaxng.org/ns/structure/1.0"></grammar>"#;
    let mut rng = Relaxng::default();
    let patterns = scan_string(&mut rng, xml).expect("scan");
    assert_eq!(patterns.len(), 1);
    match &patterns[0] {
      Pattern::Grammar { name, body } => {
        assert_eq!(name, "grammar1");
        assert!(body.is_empty());
      },
      other => panic!("expected Grammar, got {:?}", other),
    }
  }

  #[test]
  fn scan_simple_element() {
    let xml = r#"
      <grammar xmlns="http://relaxng.org/ns/structure/1.0">
        <start><element name="root"><empty/></element></start>
      </grammar>
    "#;
    let mut rng = Relaxng::default();
    let patterns = scan_string(&mut rng, xml).expect("scan");
    let body = match &patterns[0] {
      Pattern::Grammar { body, .. } => body,
      other => panic!("expected Grammar, got {:?}", other),
    };
    let start_body = match &body[0] {
      Pattern::Start { body } => body,
      other => panic!("expected Start, got {:?}", other),
    };
    match &start_body[0] {
      Pattern::Element { name, body: _ } => assert_eq!(name, "root"),
      other => panic!("expected Element, got {:?}", other),
    }
  }

  #[test]
  fn scan_choice_combinator() {
    let xml = r#"
      <grammar xmlns="http://relaxng.org/ns/structure/1.0">
        <start>
          <choice>
            <element name="a"><empty/></element>
            <element name="b"><empty/></element>
          </choice>
        </start>
      </grammar>
    "#;
    let mut rng = Relaxng::default();
    let patterns = scan_string(&mut rng, xml).expect("scan");
    let body = match &patterns[0] {
      Pattern::Grammar { body, .. } => body,
      _ => unreachable!(),
    };
    let start_body = match &body[0] {
      Pattern::Start { body } => body,
      _ => unreachable!(),
    };
    assert!(matches_combination(&start_body[0], CombineOp::Choice));
  }

  #[test]
  fn scan_define_with_combine_choice() {
    let xml = r#"
      <grammar xmlns="http://relaxng.org/ns/structure/1.0">
        <define name="X"><element name="x1"><empty/></element></define>
        <define name="X" combine="choice"><element name="x2"><empty/></element></define>
      </grammar>
    "#;
    let mut rng = Relaxng::default();
    let patterns = scan_string(&mut rng, xml).expect("scan");
    let body = match &patterns[0] {
      Pattern::Grammar { body, .. } => body,
      _ => unreachable!(),
    };
    assert_eq!(body.len(), 2);
    match &body[0] {
      Pattern::Def { combiner: DefCombiner::Group, name, .. } => assert_eq!(name, "X"),
      other => panic!("expected Def(Group), got {:?}", other),
    }
    match &body[1] {
      Pattern::Def { combiner: DefCombiner::Choice, name, .. } => assert_eq!(name, "X"),
      other => panic!("expected Def(Choice), got {:?}", other),
    }
  }

  #[test]
  fn scan_attribute_with_value() {
    let xml = r#"
      <grammar xmlns="http://relaxng.org/ns/structure/1.0">
        <start>
          <element name="root">
            <attribute name="kind"><value>sample</value></attribute>
          </element>
        </start>
      </grammar>
    "#;
    let mut rng = Relaxng::default();
    let patterns = scan_string(&mut rng, xml).expect("scan");
    // descend Grammar → Start → Element → Attribute → Value
    let attr_body = match &patterns[0] {
      Pattern::Grammar { body, .. } => match &body[0] {
        Pattern::Start { body: sb } => match &sb[0] {
          Pattern::Element { body: eb, .. } => match &eb[0] {
            Pattern::Attribute { body: ab, .. } => ab.clone(),
            _ => unreachable!(),
          },
          _ => unreachable!(),
        },
        _ => unreachable!(),
      },
      _ => unreachable!(),
    };
    match &attr_body[0] {
      Pattern::Value(v) => assert_eq!(v, "sample"),
      other => panic!("expected Value, got {:?}", other),
    }
  }

  #[test]
  fn scan_documentation_annotation() {
    let xml = r#"
      <grammar
        xmlns="http://relaxng.org/ns/structure/1.0"
        xmlns:a="http://relaxng.org/ns/compatibility/annotations/1.0">
        <define name="X">
          <a:documentation>An example pattern</a:documentation>
          <element name="x"><empty/></element>
        </define>
      </grammar>
    "#;
    let mut rng = Relaxng::default();
    let patterns = scan_string(&mut rng, xml).expect("scan");
    let body = match &patterns[0] {
      Pattern::Grammar { body, .. } => body,
      _ => unreachable!(),
    };
    let def_body = match &body[0] {
      Pattern::Def { body, .. } => body,
      _ => unreachable!(),
    };
    match &def_body[0] {
      Pattern::Doc(s) => assert_eq!(s, "An example pattern"),
      other => panic!("expected Doc, got {:?}", other),
    }
  }

  #[test]
  fn scan_mixed_normalises_to_interleave_with_text() {
    let xml = r#"
      <grammar xmlns="http://relaxng.org/ns/structure/1.0">
        <start>
          <mixed><element name="b"><empty/></element></mixed>
        </start>
      </grammar>
    "#;
    let mut rng = Relaxng::default();
    let patterns = scan_string(&mut rng, xml).expect("scan");
    let inner = match &patterns[0] {
      Pattern::Grammar { body, .. } => match &body[0] {
        Pattern::Start { body } => match &body[0] {
          Pattern::Combination { op, body } => (op, body.clone()),
          _ => unreachable!(),
        },
        _ => unreachable!(),
      },
      _ => unreachable!(),
    };
    assert_eq!(*inner.0, CombineOp::Interleave);
    assert!(matches!(inner.1[0], Pattern::Text));
  }

  #[test]
  fn filter_names_drops_canceling_exclusion() {
    let names = vec!["x".into(), "y".into(), "!y".into()];
    let result = filter_names(names);
    assert_eq!(result, vec!["x".to_string()]);
  }

  #[test]
  fn filter_names_preserves_uncanceled_exclusion() {
    let names = vec!["x".into(), "!z".into()];
    let result = filter_names(names);
    assert_eq!(result, vec!["x".to_string(), "!z".to_string()]);
  }
}
