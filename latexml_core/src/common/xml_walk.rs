//! Read-only walks for the XPath location paths LaTeXML asks on every document and every formula
//! (`//ltx:Math[not(ancestor::ltx:Math)]`, `descendant-or-self::*[@_pvis or @_cvis]`, `//ltx:ref//*[@fragid]`, …).
//!
//! libxml2 answers such a query by building a node set over every element it passes and rust-libxml then wraps each
//! result; on a large document the node set also hits libxml2's 10M ceiling, where the evaluation returns NULL and
//! "no matches" cannot be told from "could not answer" (a 614 MB core XML: six queries failed, post wrote 0 bytes).
//! A [`WalkQuery`] tests read-only nodes and makes `Node` handles only for its matches. It answers only shapes it
//! can prove equal to XPath (the same nodes, document order, each once) and declines everything else, so the caller
//! evaluates those as XPath unchanged. It keeps no state between queries: each one reads the live tree.

use libxml::{
  readonly::RoNode,
  tree::{Document, Node, NodeType},
};

use crate::common::xml::XML_NS;

const LTX_NS: &str = "http://dlmf.nist.gov/LaTeXML";

/// Where a query starts: `//…` the whole document; `descendant::` (also `.//`) and `descendant-or-self::` the context
/// node's subtree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
  Document,
  Descendant,
  DescendantOrSelf,
}

/// `*`, or an `ltx:` element name.
#[derive(Debug, Clone)]
enum NameTest {
  Any,
  Ltx(String),
}

/// `@name` (no namespace) or `@xml:name`.
#[derive(Debug)]
enum AttrName {
  Plain(String),
  Xml(String),
}

/// One argument of `contains(concat(…), '…')`.
#[derive(Debug)]
enum StrPart {
  Literal(String),
  Attr(AttrName),
}

/// The predicate atoms the queries use: a closed set, so an unrecognised shape stays XPath.
#[derive(Debug)]
enum Atom {
  Attr(AttrName),
  NoAttr(AttrName),
  /// `not(ancestor::ltx:NAME)`
  NoAncestor(NameTest),
  /// `not(ltx:NAME)`: no such child element
  NoChild(String),
  /// The leading step of `//A//B`: an ancestor element `A`.
  Ancestor(NameTest),
  /// The leading step of `//A/B`: the parent element is `A`.
  Parent(NameTest),
  /// `contains(STRING, 'needle')`, the string an attribute value or a `concat` of them and literals.
  Contains(Vec<StrPart>, String),
}

/// One union arm: a name test and its predicate in disjunctive normal form (an OR of AND-groups; empty = none).
#[derive(Debug)]
struct Arm {
  name:  NameTest,
  preds: Vec<Vec<Atom>>,
}

/// A parsed query inside the walk's grammar (see the module doc).
#[derive(Debug)]
pub struct WalkQuery {
  axis:       Axis,
  arms:       Vec<Arm>,
  /// Some arm tests an element name (read once per element, not per arm).
  needs_name: bool,
}

impl WalkQuery {
  /// The query `xpath` as a walk, or `None` when it is outside the grammar: a union (`|`) of `//STEP`, `//A//STEP` or
  /// `//A/STEP` arms (`A` a bare name test), or of `descendant::STEP` / `.//STEP` / `descendant-or-self::STEP` arms,
  /// where `STEP` is `*` or `ltx:NAME` with at most one predicate of `or`/`and`-joined atoms (no parentheses).
  pub fn parse(xpath: &str) -> Option<WalkQuery> {
    let mut axis = None;
    let mut arms = Vec::new();
    for raw in split_top(xpath, "|") {
      let raw = raw.trim();
      let (arm_axis, path) = if let Some(rest) = raw.strip_prefix("//") {
        (Axis::Document, rest)
      } else if let Some(rest) = raw.strip_prefix(".//") {
        (Axis::Descendant, rest)
      } else if let Some(rest) = raw.strip_prefix("descendant::") {
        (Axis::Descendant, rest)
      } else if let Some(rest) = raw.strip_prefix("descendant-or-self::") {
        (Axis::DescendantOrSelf, rest)
      } else {
        return None;
      };
      if axis.is_some_and(|a| a != arm_axis) {
        return None;
      }
      axis = Some(arm_axis);
      // A leading step only on a whole-document arm; it is a bare name test.
      let steps = split_top(path, "/");
      let (lead, step) = match steps.as_slice() {
        [step] => (None, *step),
        [lead, step] if arm_axis == Axis::Document => (Some((*lead, false)), *step),
        [lead, "", step] if arm_axis == Axis::Document => (Some((*lead, true)), *step),
        _ => return None,
      };
      let (name, pred) = match step.split_once('[') {
        Some((name, pred)) => (name, Some(pred.strip_suffix(']')?)),
        None => (step, None),
      };
      let mut preds = match pred {
        Some(pred) => parse_predicate(pred)?,
        None => Vec::new(),
      };
      if let Some((lead, descendant)) = lead {
        let lead = parse_name_test(lead)?;
        let atom = || {
          if descendant {
            Atom::Ancestor(lead.clone())
          } else {
            Atom::Parent(lead.clone())
          }
        };
        if preds.is_empty() {
          preds.push(vec![atom()]);
        } else {
          for group in &mut preds {
            group.insert(0, atom());
          }
        }
      }
      arms.push(Arm {
        name: parse_name_test(name)?,
        preds,
      });
    }
    let needs_name = arms.iter().any(|arm| matches!(arm.name, NameTest::Ltx(_)));
    Some(WalkQuery { axis: axis?, arms, needs_name })
  }

  /// The query's matches in `doc`, in document order, each once: XPath's answer, evaluated at `context` (or at the
  /// root element when `None`, as LaTeXML's `findnodes` binds it). `None` when the walk cannot vouch for that: no root,
  /// a null context or one outside `doc`'s tree (a detached or foreign node: XPath decides how it fails), or a DTD
  /// (its default attributes and entity references are seen differently by XPath than by a tree walk).
  pub fn find(&self, doc: &Document, context: Option<&Node>) -> Option<Vec<Node>> {
    self.find_up_to(doc, context, usize::MAX)
  }

  /// The first of [`find`](Self::find)'s matches (`Some(None)`: none), the walk stopping there.
  pub fn find_first(&self, doc: &Document, context: Option<&Node>) -> Option<Option<Node>> {
    Some(self.find_up_to(doc, context, 1)?.pop())
  }

  fn find_up_to(&self, doc: &Document, context: Option<&Node>, limit: usize) -> Option<Vec<Node>> {
    let root = doc.get_root_readonly()?;
    let doc_node = root.get_parent()?;
    let mut top = doc_node.get_first_child();
    while let Some(child) = top {
      if child.get_type() == Some(NodeType::DTDNode) {
        return None;
      }
      top = child.get_next_sibling();
    }
    let context = match context {
      Some(node) => {
        let ro = node.to_readonly();
        if ro.is_null() {
          return None;
        }
        let mut top = ro;
        while let Some(parent) = top.get_parent() {
          top = parent;
        }
        if top != doc_node {
          return None;
        }
        ro
      },
      None => root,
    };
    let mut found = Vec::new();
    match self.axis {
      Axis::Document => self.collect(root, true, limit, &mut found),
      Axis::Descendant => self.collect(context, false, limit, &mut found),
      Axis::DescendantOrSelf => self.collect(context, true, limit, &mut found),
    }
    Some(
      found
        .into_iter()
        .map(|node| doc.get_node_from_readonly(node))
        .collect(),
    )
  }

  /// The first `limit` matching elements of `start`'s subtree in document order (`start` itself when
  /// `include_self`). Like XPath's descendant axis it enters only elements, so never an entity reference's content.
  fn collect(&self, start: RoNode, include_self: bool, limit: usize, out: &mut Vec<RoNode>) {
    if include_self && self.matches(start) {
      out.push(start);
    }
    if out.len() >= limit
      || !matches!(
        start.get_type(),
        Some(NodeType::ElementNode | NodeType::DocumentNode)
      )
    {
      return;
    }
    let mut cur = start.get_first_child();
    while let Some(node) = cur {
      let is_element = node.get_type() == Some(NodeType::ElementNode);
      if is_element && self.matches(node) {
        out.push(node);
        if out.len() >= limit {
          return;
        }
      }
      if is_element && let Some(child) = node.get_first_child() {
        cur = Some(child);
        continue;
      }
      // The next node in document order, never past `start`.
      let mut last = node;
      cur = loop {
        if let Some(next) = last.get_next_sibling() {
          break Some(next);
        }
        match last.get_parent() {
          Some(parent) if parent != start => last = parent,
          _ => break None,
        }
      };
    }
  }

  fn matches(&self, node: RoNode) -> bool {
    if node.get_type() != Some(NodeType::ElementNode) {
      return false;
    }
    let name = if self.needs_name {
      node.get_name()
    } else {
      String::new()
    };
    let mut in_ltx = None;
    self.arms.iter().any(|arm| {
      if let NameTest::Ltx(want) = &arm.name
        && (name != *want || !*in_ltx.get_or_insert_with(|| is_ltx(node)))
      {
        return false;
      }
      arm.preds.is_empty()
        || arm
          .preds
          .iter()
          .any(|group| group.iter().all(|atom| atom.holds(node)))
    })
  }
}

impl Atom {
  fn holds(&self, node: RoNode) -> bool {
    match self {
      Atom::Attr(attr) => attr.present(node),
      Atom::NoAttr(attr) => !attr.present(node),
      Atom::NoAncestor(test) => !ancestors(node).any(|a| test.accepts(a)),
      Atom::NoChild(name) => {
        let mut child = node.get_first_child();
        while let Some(c) = child {
          if c.get_type() == Some(NodeType::ElementNode) && c.get_name() == *name && is_ltx(c) {
            return false;
          }
          child = c.get_next_sibling();
        }
        true
      },
      Atom::Ancestor(test) => ancestors(node).any(|a| test.accepts(a)),
      Atom::Parent(test) => ancestors(node).next().is_some_and(|p| test.accepts(p)),
      Atom::Contains(parts, needle) => {
        let mut haystack = String::new();
        for part in parts {
          match part {
            StrPart::Literal(s) => haystack.push_str(s),
            StrPart::Attr(attr) => haystack.push_str(&attr.value(node).unwrap_or_default()),
          }
        }
        haystack.contains(needle.as_str())
      },
    }
  }
}

impl AttrName {
  fn present(&self, node: RoNode) -> bool {
    match self {
      AttrName::Plain(name) => node.has_attribute_no_ns(name),
      AttrName::Xml(name) => node.has_attribute_ns(name, XML_NS),
    }
  }

  fn value(&self, node: RoNode) -> Option<String> {
    match self {
      AttrName::Plain(name) => node.get_attribute_no_ns(name),
      AttrName::Xml(name) => node.get_attribute_ns(name, XML_NS),
    }
  }
}

impl NameTest {
  /// `node` (an element) passes this test.
  fn accepts(&self, node: RoNode) -> bool {
    match self {
      NameTest::Any => true,
      NameTest::Ltx(name) => node.get_name() == *name && is_ltx(node),
    }
  }
}

/// The element ancestors of `node`, nearest first.
fn ancestors(node: RoNode) -> impl Iterator<Item = RoNode> {
  std::iter::successors(node.get_parent(), |p| p.get_parent())
    .take_while(|p| p.get_type() == Some(NodeType::ElementNode))
}

fn is_ltx(node: RoNode) -> bool {
  node
    .get_namespace()
    .is_some_and(|ns| ns.get_href() == LTX_NS)
}

/// `s` split at each `sep` outside quotes, parentheses and brackets.
fn split_top<'a>(s: &'a str, sep: &str) -> Vec<&'a str> {
  let bytes = s.as_bytes();
  let mut parts = Vec::new();
  let (mut depth, mut quote, mut start, mut i) = (0usize, None, 0, 0);
  while i < bytes.len() {
    let b = bytes[i];
    match quote {
      Some(q) if b == q => quote = None,
      Some(_) => {},
      None => match b {
        b'\'' | b'"' => quote = Some(b),
        b'(' | b'[' => depth += 1,
        b')' | b']' => depth = depth.saturating_sub(1),
        _ if depth == 0 && s[i..].starts_with(sep) => {
          parts.push(&s[start..i]);
          i += sep.len();
          start = i;
          continue;
        },
        _ => {},
      },
    }
    i += 1;
  }
  parts.push(&s[start..]);
  parts
}

fn is_ncname(name: &str) -> bool {
  name
    .chars()
    .next()
    .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
    && name
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn parse_name_test(test: &str) -> Option<NameTest> {
  match test {
    "*" => Some(NameTest::Any),
    _ => {
      let name = test.strip_prefix("ltx:")?;
      is_ncname(name).then(|| NameTest::Ltx(name.to_string()))
    },
  }
}

fn parse_attr(attr: &str) -> Option<AttrName> {
  match attr.strip_prefix("xml:") {
    Some(name) => is_ncname(name).then(|| AttrName::Xml(name.to_string())),
    None => is_ncname(attr).then(|| AttrName::Plain(attr.to_string())),
  }
}

/// A quoted XPath literal's text.
fn parse_literal(lit: &str) -> Option<String> {
  let quote = lit.chars().next().filter(|q| *q == '\'' || *q == '"')?;
  let inner = lit[1..].strip_suffix(quote)?;
  (!inner.contains(quote)).then(|| inner.to_string())
}

/// `pred` as an OR of AND-groups, split at the top level only (a literal may hold ` or `).
fn parse_predicate(pred: &str) -> Option<Vec<Vec<Atom>>> {
  split_top(pred, " or ")
    .into_iter()
    .map(|disjunct| {
      split_top(disjunct, " and ")
        .into_iter()
        .map(|atom| parse_atom(atom.trim()))
        .collect()
    })
    .collect()
}

fn parse_atom(atom: &str) -> Option<Atom> {
  if let Some(attr) = atom.strip_prefix('@') {
    return parse_attr(attr).map(Atom::Attr);
  }
  if let Some(inner) = atom.strip_prefix("not(").and_then(|a| a.strip_suffix(')')) {
    let inner = inner.trim();
    return if let Some(attr) = inner.strip_prefix('@') {
      parse_attr(attr).map(Atom::NoAttr)
    } else if let Some(name) = inner.strip_prefix("ancestor::ltx:") {
      is_ncname(name).then(|| Atom::NoAncestor(NameTest::Ltx(name.to_string())))
    } else if let Some(name) = inner.strip_prefix("ltx:") {
      is_ncname(name).then(|| Atom::NoChild(name.to_string()))
    } else {
      None
    };
  }
  let args = atom.strip_prefix("contains(")?.strip_suffix(')')?;
  let [haystack, needle] = split_top(args, ",")[..] else {
    return None;
  };
  let haystack = haystack.trim();
  let parts = match haystack
    .strip_prefix("concat(")
    .and_then(|h| h.strip_suffix(')'))
  {
    Some(list) => split_top(list, ",")
      .into_iter()
      .map(|part| parse_str_part(part.trim()))
      .collect::<Option<Vec<_>>>()?,
    None => vec![parse_str_part(haystack)?],
  };
  Some(Atom::Contains(parts, parse_literal(needle.trim())?))
}

fn parse_str_part(part: &str) -> Option<StrPart> {
  match part.strip_prefix('@') {
    Some(attr) => parse_attr(attr).map(StrPart::Attr),
    None => parse_literal(part).map(StrPart::Literal),
  }
}

#[cfg(test)]
mod tests {
  use libxml::{parser::Parser, xpath::Context};

  use super::*;

  fn parse(xml: &str) -> Document { Parser::default().parse_string(xml).expect("well-formed") }

  fn label(node: &Node) -> String {
    format!(
      "{}#{}",
      node.get_name(),
      node
        .get_attribute_ns("id", XML_NS)
        .or_else(|| node.get_attribute("id"))
        .unwrap_or_default()
    )
  }

  /// libxml2's own answer for `xpath` at `context` (the root element when `None`).
  fn xpath_answer(doc: &Document, xpath: &str, context: Option<&Node>) -> Vec<String> {
    let ctx = Context::new(doc).expect("ctx");
    ctx.register_namespace("ltx", LTX_NS).expect("ns");
    let root = doc.get_root_element().unwrap();
    ctx
      .node_evaluate(xpath, context.unwrap_or(&root))
      .expect("xpath evaluates on a small doc")
      .get_nodes_as_vec()
      .iter()
      .map(label)
      .collect()
  }

  const DOC: &str = "<document xmlns='http://dlmf.nist.gov/LaTeXML' xmlns:o='urn:other'>\
       <ref href='u1'/>\
       <ref href='u2' idref='i1'/>\
       <ref labelref='L1'/>\
       <ref id='r1' href='u3'><text fragid='f1'><text id='t1' fragid='f2'/></text></ref>\
       <ref id='r2'><ref id='r3' fragid='f3'/></ref>\
       <graphics/>\
       <graphics imagesrc='a.png'/>\
       <Math id='m1'><XMath><Math id='inner'/></XMath></Math>\
       <index/>\
       <index><indexlist/></index>\
       <glossary/>\
       <p idref='i2' o:idref='ns'/>\
       <o:p id='foreign' idref='i3'/>\
       <XMDual _cvis='1'/>\
       <XMDual _pvis='1'/>\
       <XMDual _cvis='1' _pvis='1'/>\
       <XMDual/>\
       <text class='ltx_cref_conjunction' id='c1'/>\
       <text class='a ltx_cref_conjunction b' id='c2'/>\
       <text class='ltx_cref_conjunctions' id='c3'/>\
       <text class='x or y' id='c4'/>\
       <text id='c5'/>\
       <XMTok role='SUPERSCRIPT' xml:id='s1'/>\
       <XMTok role='SUPERSCRIPT' id='s2'/>\
       <XMTok role='ADDOP' xml:id='s3'/>\
       <section id='s1'><para><p><ref idref='deep'/><text>tail</text></p></para>\
         <Math id='m2'><XMath><XMDual _cvis='1' id='outer'><XMDual _pvis='1' id='nested'/></XMDual>\
           <XMText><Math id='m3'><XMath><XMTok id='tok' _pvis='1'/></XMath></Math></XMText></XMath></Math>\
       </section>\
       <p id='after'>no match follows</p>\
     </document>";

  /// Every shape the grammar admits gives libxml2's nodes in libxml2's order, each once: the root, nested and
  /// repeated matches, namespaced look-alikes (`o:p`, `o:idref`), unions whose arms overlap, and every predicate atom.
  #[test]
  fn walk_agrees_with_xpath_on_every_supported_shape() {
    let doc = parse(DOC);
    let root = doc.get_root_element().unwrap();
    let section = root.get_last_child().unwrap().get_prev_sibling().unwrap();
    let math = section.get_last_child().unwrap();
    let doc_node = doc.as_node();
    for xpath in [
      "//*",
      "//ltx:document | //ltx:ref",
      "//ltx:ref | //ltx:document",
      "//*[@idref]",
      "//*[@labelref]",
      "//*[@idref] | //ltx:ref",
      "//ltx:ref[@href and not(@idref) and not(@labelref)]",
      "//ltx:graphics[not(@imagesrc)]",
      "//ltx:Math[not(ancestor::ltx:Math)]",
      "//ltx:index[not(ltx:indexlist)] | //ltx:glossary[not(ltx:glossarylist)]",
      "//*[@_cvis or @_pvis]",
      "//ltx:ref[@href or @idref]",
      "//*[@idref or @labelref]",
      "//ltx:ref//*[@fragid]",
      "//ltx:ref//ltx:ref",
      "//ltx:XMath/*",
      "//*/ltx:ref",
      "//ltx:ref/*[@fragid or @id]",
      "//ltx:text[contains(concat(' ', @class, ' '), ' ltx_cref_conjunction ')]",
      "//ltx:text[contains(concat(' ',@class,' '),' or ')]",
      "//*[@xml:id]",
      "//*[@xml:id and contains(@role,'SCRIPT')]",
      "descendant::*[@decl_id or @meaning]",
      "descendant::ltx:Math",
      ".//ltx:Math",
      "descendant-or-self::*[@_pvis or @_cvis]",
      "descendant-or-self::ltx:document",
      "descendant::ltx:document",
    ] {
      let query = WalkQuery::parse(xpath)
        .unwrap_or_else(|| panic!("`{xpath}` must be inside the walk grammar"));
      for context in [None, Some(&section), Some(&math), Some(&doc_node)] {
        let walked: Vec<String> = query
          .find(&doc, context)
          .expect("no DTD, an attached context")
          .iter()
          .map(label)
          .collect();
        let expected = xpath_answer(&doc, xpath, context);
        assert_eq!(
          walked,
          expected,
          "walk disagrees with XPath for `{xpath}` at {:?}",
          context.map(label)
        );
        let first = query.find_first(&doc, context).unwrap();
        assert_eq!(first.as_ref().map(label), expected.first().cloned());
      }
    }
  }

  /// A shape outside the grammar must not be approximated: it stays XPath.
  #[test]
  fn unsupported_shapes_are_not_claimed_by_the_walk() {
    for xpath in [
      "//ltx:section/ltx:title/ltx:text", // three steps
      "//ltx:section[@id]//ltx:ref",      // a predicate on the leading step
      "descendant::ltx:p/ltx:ref",        // a path on a relative axis
      "//ltx:ref[position()=1]",          // function predicate
      "//ltx:ref[@a][@b]",                // two predicates
      "//ltx:ref[@a='x']",                // comparison
      "ltx:XMath",                        // child axis
      "//svg:svg",                        // non-ltx prefix
      "//*[@svg:href]",                   // non-xml attribute prefix
      "//ltx:Math[not(ancestor::svg:svg)]",
      "//ltx:ref[(@a or @b) and @c]", // parenthesised precedence
      "//ltx:text[contains(@class, x)]",
      "//ltx:text[contains(concat(' ', @class), ' a ', 'b')]",
      "//ltx:ref | descendant::ltx:ref", // mixed axes
      "//processing-instruction('latexml')",
    ] {
      assert!(
        WalkQuery::parse(xpath).is_none(),
        "`{xpath}` must fall through to XPath, not be walked"
      );
    }
  }

  /// The walk declines where it cannot vouch for XPath's answer: a document with a DTD (entity references, default
  /// attributes), a context node outside the document's tree, a null one.
  #[test]
  fn walk_declines_where_xpath_must_answer() {
    let with_dtd = parse(
      "<?xml version='1.0'?>\
       <!DOCTYPE document [<!ENTITY ent '<ref xmlns=\"http://dlmf.nist.gov/LaTeXML\" idref=\"in-ent\"/>'>]>\
       <document xmlns='http://dlmf.nist.gov/LaTeXML'><p><ref idref='before'/></p><p>&ent;</p></document>",
    );
    let query = WalkQuery::parse("//*[@idref]").unwrap();
    assert!(query.find(&with_dtd, None).is_none());
    let doc = parse(DOC);
    let other = parse("<document xmlns='http://dlmf.nist.gov/LaTeXML'><ref idref='x'/></document>");
    let foreign = other.get_root_element().unwrap();
    assert!(query.find(&doc, Some(&foreign)).is_none());
    let relative = WalkQuery::parse("descendant::*[@idref]").unwrap();
    assert!(relative.find(&doc, Some(&foreign)).is_none());
    let mut detached = doc.get_root_element().unwrap().get_first_child().unwrap();
    detached.unlink_node();
    assert!(relative.find(&doc, Some(&detached)).is_none());
    assert!(relative.find(&doc, Some(&Node::null())).is_none());
  }

  /// Nothing is kept between queries: a match added or removed since the last one is seen by the next, in order.
  #[test]
  fn walk_answers_from_the_live_tree_after_mutation() {
    let doc = parse(
      "<document xmlns='http://dlmf.nist.gov/LaTeXML'><p><ref idref='a'/></p><p/></document>",
    );
    let query = WalkQuery::parse("//*[@idref]").unwrap();
    let idrefs = |doc: &Document| -> Vec<String> {
      query
        .find(doc, None)
        .unwrap()
        .iter()
        .filter_map(|n| n.get_attribute("idref"))
        .collect()
    };
    assert_eq!(idrefs(&doc), ["a"]);
    let root = doc.get_root_element().unwrap();
    let mut p2 = root.get_last_child().unwrap();
    let mut added = Node::new("ref", p2.get_namespace(), &doc).unwrap();
    added.set_attribute("idref", "b").unwrap();
    p2.add_child(&mut added).unwrap();
    assert_eq!(idrefs(&doc), ["a", "b"]);
    let mut first = root.get_first_child().unwrap().get_first_child().unwrap();
    first.unlink_node();
    assert_eq!(idrefs(&doc), ["b"]);
  }
}
