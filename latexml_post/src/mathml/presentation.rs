//! Presentation MathML rendering rules.
//!
//! Port of `LaTeXML::Post::MathML::Presentation` (146 lines) +
//! the presentation portion of `LaTeXML::Post::MathML` (main module, ~1000 lines).
//! Converts XMath nodes to Presentation MathML elements (mi, mo, mn, mrow, etc.).
//!
//! Key concepts:
//! - `pmml(node)` dispatches conversion by tag (XMTok, XMApp, XMDual, etc.)
//! - `stylizeContent(item, tag)` determines text, mathvariant, size, spacing
//! - Scripts (sub/sup/under/over) handle pre/mid/post positioning
//! - Style context tracks display/text/script/scriptscript levels

use std::cell::Cell;

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;

use super::operator_dictionary;
use crate::document::{NodeData, PostDocument, element_children, element_children_iter};

// Thread-local flag for invisible times emission.
// When false, U+2062 is replaced with U+200B (zero-width space).
thread_local! {
  static INVISIBLE_TIMES: Cell<bool> = const { Cell::new(true) };
  /// Current math style context, tracking Perl's `$LaTeXML::MathML::STYLE` /
  /// `$LaTeXML::MathML::SIZE`. Stepped down inside sub/superscripts (`pmml_scriptsize`)
  /// and fraction parts (`pmml_smaller`); its `size_percent()` is the contextual size a
  /// token's `fontsize` is compared against, so a token only emits an explicit
  /// `mathsize` when it *differs* from what the surrounding script/fraction structure
  /// already implies (Perl `stylizeContent` L777). Reset to `Display` at each
  /// `convert_to_pmml` entry.
  static CURRENT_STYLE: Cell<MathStyle> = const { Cell::new(MathStyle::Display) };
}

/// Set whether to emit invisible times (called by MathML processor before rendering).
pub fn set_invisible_times(emit: bool) { INVISIBLE_TIMES.with(|f| f.set(emit)); }

fn get_invisible_times() -> bool { INVISIBLE_TIMES.with(|f| f.get()) }

/// The contextual font size (e.g. "100%", "70%", "50%") implied by the current math style.
/// A token whose own `fontsize` equals this needs no explicit `mathsize` attribute.
fn current_context_size() -> &'static str { CURRENT_STYLE.with(|s| s.get().size_percent()) }

/// Math style levels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MathStyle {
  Display,
  Text,
  Script,
  ScriptScript,
}

impl MathStyle {
  /// Step down one level (for fractions, etc.).
  pub fn step_down(self) -> Self {
    match self {
      MathStyle::Display => MathStyle::Text,
      MathStyle::Text => MathStyle::Script,
      MathStyle::Script => MathStyle::ScriptScript,
      MathStyle::ScriptScript => MathStyle::ScriptScript,
    }
  }

  /// Step to script size (for sub/superscripts).
  pub fn script_step(self) -> Self {
    match self {
      MathStyle::Display | MathStyle::Text => MathStyle::Script,
      MathStyle::Script => MathStyle::ScriptScript,
      MathStyle::ScriptScript => MathStyle::ScriptScript,
    }
  }

  /// CSS-style size percentage.
  pub fn size_percent(self) -> &'static str {
    match self {
      MathStyle::Display | MathStyle::Text => "100%",
      MathStyle::Script => "70%",
      MathStyle::ScriptScript => "50%",
    }
  }

  /// Parse a source `mathstyle` attribute (Perl gates on `$stylestep{$style}`,
  /// so only the four canonical names count).
  pub fn from_attr(s: &str) -> Option<Self> {
    match s {
      "display" => Some(MathStyle::Display),
      "text" => Some(MathStyle::Text),
      "script" => Some(MathStyle::Script),
      "scriptscript" => Some(MathStyle::ScriptScript),
      _ => None,
    }
  }
}

/// m:mstyle attributes for a mathstyle transition.
///
/// Port of Perl `%stylemap` (`needs`=true: something below cares about
/// displaystyle) and `%stylemap2` (`needs`=false: only a fontsize context is
/// required), MathML.pm L240-268. Same-style transitions yield nothing.
fn stylemap_attrs(
  ostyle: MathStyle,
  nstyle: MathStyle,
  needs: bool,
) -> &'static [(&'static str, &'static str)] {
  use MathStyle::*;
  match (ostyle, nstyle, needs) {
    (Display, Text, true) => &[("displaystyle", "false")],
    (Display, Script, true) => &[("displaystyle", "false"), ("scriptlevel", "+1")],
    (Display, ScriptScript, true) => &[("displaystyle", "false"), ("scriptlevel", "+2")],
    (Text, Display, true) => &[("displaystyle", "true")],
    (Display, Script, false) | (Text, Script, _) => &[("scriptlevel", "+1")],
    (Display, ScriptScript, false) | (Text, ScriptScript, _) => &[("scriptlevel", "+2")],
    (Script, Display, _) => &[("displaystyle", "true"), ("scriptlevel", "-1")],
    (Script, Text, _) => &[("scriptlevel", "-1")],
    (Script, ScriptScript, _) => &[("scriptlevel", "+1")],
    (ScriptScript, Display, _) => &[("displaystyle", "true"), ("scriptlevel", "-2")],
    (ScriptScript, Text, _) => &[("scriptlevel", "-2")],
    (ScriptScript, Script, _) => &[("scriptlevel", "-1")],
    _ => &[],
  }
}

/// Does this subtree contain something whose rendering depends on
/// displaystyle? Port of Perl `needsMathstyle` (MathML.pm L512-523):
/// m:mfrac → yes; `_largeop` → yes; an m:mstyle that already pins
/// displaystyle shields its subtree.
fn needs_mathstyle(node: &NodeData) -> bool {
  if let NodeData::Element { tag, attributes, children } = node {
    if tag == "m:mfrac" {
      return true;
    }
    if let Some(attrs) = attributes {
      if attrs.contains_key("_largeop") {
        return true;
      }
      if tag == "m:mstyle" && attrs.contains_key("displaystyle") {
        return false;
      }
    }
    return children.iter().any(needs_mathstyle);
  }
  false
}

/// Wrap `result` in m:mstyle for an ostyle→nstyle transition, when the
/// transition table says the wrap carries information (Perl MathML.pm
/// L421-427 / L487-491).
fn maybe_style_wrap(result: NodeData, ostyle: MathStyle, nstyle: Option<MathStyle>) -> NodeData {
  let Some(nstyle) = nstyle else { return result };
  let style_attrs = stylemap_attrs(ostyle, nstyle, needs_mathstyle(&result));
  if style_attrs.is_empty() {
    return result;
  }
  NodeData::Element {
    tag:        "m:mstyle".to_string(),
    attributes: Some(HashMap::from_iter(
      style_attrs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string())),
    )),
    children:   vec![result],
  }
}

/// Wrap `result` in m:mpadded / frame it, when the source node (or its
/// containing XMDual) carries sizing attributes.
///
/// Port of Perl `pmml_maybe_resize` (MathML.pm L525-575): stretchy-ARROW
/// width → m:mover + m:mspace; width/height/depth/xoffset/yoffset →
/// m:mpadded (reusing an existing mpadded/mrow); framed/framecolor →
/// ltx_framed_* class + border-color style.
fn pmml_maybe_resize(doc: &PostDocument, node: &Node, result: NodeData) -> NodeData {
  // Relevant attributes MAY sit on a containing XMDual (Perl L529-531).
  let parent = node.get_parent().filter(|p| doc.is_qname(p, "ltx:XMDual"));
  let getattr = |name: &str| {
    node
      .get_attribute(name)
      .or_else(|| parent.as_ref().and_then(|p| p.get_attribute(name)))
  };
  let width = getattr("width");
  let height = getattr("height");
  let depth = getattr("depth");
  let xoff = getattr("xoffset");
  let yoff = getattr("yoffset");
  let role = getattr("role");
  let class = getattr("class");

  let mut result = result;
  if let Some(ref w) = width
    && role.as_deref() == Some("ARROW")
    && class.as_deref().is_some_and(|c| {
      c.split_ascii_whitespace()
        .any(|w| w == "ltx_horizontally_stretchy")
    })
  {
    // Special-case hack for stretchy arrows with a specified width;
    // stretchiness (currently) only has effect within munder/mover (Perl L543-545).
    result = NodeData::Element {
      tag:        "m:mover".to_string(),
      attributes: None,
      children:   vec![result, NodeData::Element {
        tag:        "m:mspace".to_string(),
        attributes: Some(HashMap::from_iter([("width".to_string(), w.clone())])),
        children:   vec![],
      }],
    };
  } else if width.is_some()
    || height.is_some()
    || depth.is_some()
    || xoff.is_some()
    || yoff.is_some()
  {
    // Reuse an m:mpadded, convert an m:mrow, else wrap (Perl L547-552).
    let needs_wrap = !matches!(&result,
      NodeData::Element { tag, .. } if tag == "m:mpadded" || tag == "m:mrow");
    if needs_wrap {
      result = NodeData::Element {
        tag:        "m:mpadded".to_string(),
        attributes: None,
        children:   vec![result],
      };
    }
    if let NodeData::Element { tag, attributes, .. } = &mut result {
      if tag == "m:mrow" {
        *tag = "m:mpadded".to_string();
      }
      let attrs = attributes.get_or_insert_with(Default::default);
      for (key, val) in [
        ("width", width),
        ("height", height),
        ("depth", depth),
        ("lspace", xoff),
        ("voffset", yoff),
      ] {
        if let Some(v) = val {
          attrs.insert(key.to_string(), v);
        }
      }
    }
  }

  // framed/framecolor come from the node itself only (Perl L566-574).
  if let Some(frame) = node.get_attribute("framed")
    && let NodeData::Element { attributes, .. } = &mut result
  {
    let attrs = attributes.get_or_insert_with(Default::default);
    let frame_class = format!("ltx_framed_{frame}");
    let merged = match attrs.get("class") {
      Some(c) if !c.is_empty() => format!("{c} {frame_class}"),
      _ => frame_class,
    };
    attrs.insert("class".to_string(), merged);
    if let Some(color) = node.get_attribute("framecolor") {
      let style = format!("border-color: {color}");
      let merged = match attrs.get("style") {
        Some(s) if !s.is_empty() => format!("{s}; {style}"),
        _ => style,
      };
      attrs.insert("style".to_string(), merged);
    }
  }
  result
}

/// Embellishing roles (scripts/accents applied to operators).
fn is_embellishing_role(role: &str) -> bool {
  matches!(
    role,
    "SUPERSCRIPTOP" | "SUBSCRIPTOP" | "OVERACCENT" | "UNDERACCENT" | "MODIFIER" | "MODIFIEROP"
  )
}

/// Default token content for invisible operators.
fn default_token_content(role: &str) -> Option<&'static str> {
  match role {
    "MULOP" => Some("\u{2062}"), // INVISIBLE TIMES
    "ADDOP" => Some("\u{2064}"), // INVISIBLE PLUS
    "PUNCT" => Some("\u{2063}"), // INVISIBLE SEPARATOR
    _ => None,
  }
}

/// Get the operator role, following embellished operators.
fn get_operator_role(doc: &PostDocument, node: &Node) -> Option<String> {
  if let Some(role) = node.get_attribute("role") {
    return Some(role);
  }
  if doc.is_qname(node, "ltx:XMApp") {
    let children = element_children(node);
    if children.len() >= 2 {
      let op_role = children[0].get_attribute("role").unwrap_or_default();
      if is_embellishing_role(&op_role) {
        return get_operator_role(doc, &children[1]);
      }
    }
  }
  None
}

/// Convert an XMath tree to Presentation MathML.
///
/// Entry point for Presentation MathML conversion.
/// Port of `MathML::Presentation::convertNode` + `pmml_top`.
pub fn convert_to_pmml(doc: &PostDocument, xmath: &Node) -> NodeData {
  // Perl Presentation.pm `convertNode` L20-21 + `pmml_top`: display-mode math
  // starts in displaystyle, everything else in textstyle. Both are 100% size,
  // but the mathstyle transitions (m:mstyle wraps for \tfrac/\dfrac/
  // \displaystyle, audit F7) key off this baseline.
  let mode_is_display = xmath
    .get_parent()
    .and_then(|p| p.get_attribute("mode"))
    .is_some_and(|m| m == "display");
  CURRENT_STYLE.with(|s| {
    s.set(if mode_is_display {
      MathStyle::Display
    } else {
      MathStyle::Text
    })
  });
  let children = element_children(xmath);
  let results: Vec<NodeData> = children.iter().map(|c| pmml(doc, c)).collect();
  let mut result = if results.len() == 1 {
    results.into_iter().next().unwrap()
  } else {
    pmml_row(results)
  };
  // Adjust spacing to match TeX rules (Perl's adjust_spacing)
  adjust_spacing(&mut result);
  // Clean up internal _role/_lspace/_rspace before serialization
  clean_internal_attrs(&mut result);
  result
}

/// Presentation conversion of a node for use as ci CONTENT by the content
/// side (Perl `cmml_decoratedSymbol` L1403 calls pmml($item)).
pub(super) fn pmml_for_ci(doc: &PostDocument, node: &Node) -> NodeData { pmml(doc, node) }

/// Core dispatch: convert a single XMath node to Presentation MathML.
///
/// Port of `pmml` + `pmml_internal`.
fn pmml(doc: &PostDocument, node: &Node) -> NodeData {
  let mut result = pmml_inner(doc, node);
  // Perl MathML.pm L339-341: wrap in m:menclose if the source node carries an
  // `enclose` attribute (e.g. \boxed puts enclose="box" on the whole XMApp).
  if let Some(enclose) = node.get_attribute("enclose") {
    let mut attrs = HashMap::default();
    attrs.insert("notation".to_string(), enclose);
    result = NodeData::Element {
      tag:        "m:menclose".to_string(),
      attributes: Some(attrs),
      children:   vec![result],
    };
  }
  // Port of Perl MathML.pm L344-348: attach author spacing (lpadding/rpadding,
  // e.g. from `~` ties or collapsed XMHints — `{\rm number~of~…}`) as the
  // internal `_lpadding`/`_rpadding` attributes the space_walk consumes.
  // Perl's `_getspace($refr, $node, …)` SUMS the referring XMRef's padding
  // with the target's; here the XMRef branch of `pmml_inner` recurses through
  // this wrapper for the target, and the XMRef's own padding is then added on
  // top — same sum. Without this attachment the spacewalk sees zero author
  // padding and `~`-separated words render jammed together (witness
  // astro-ph0001001 S9.Ex4.m1). Same recursion argument covers enclose /
  // class / _role below, with refr-preference falling out of the outer level
  // overwriting (_role) or appending (class) the inner one; sole corner
  // divergence: an XMRef AND its target both carrying `enclose` would nest
  // two m:menclose where Perl picks the XMRef's one.
  attach_source_padding(node, &mut result);
  if let NodeData::Element { ref mut attributes, .. } = result {
    // Perl L350-352: merge the source node's class onto the result.
    if let Some(cl) = node.get_attribute("class")
      && !cl.is_empty()
    {
      let attrs = attributes.get_or_insert_with(Default::default);
      match attrs.get("class") {
        Some(ocl) if !ocl.is_empty() && *ocl != cl => {
          let merged = format!("{ocl} {cl}");
          attrs.insert("class".to_string(), merged);
        },
        _ => {
          attrs.insert("class".to_string(), cl);
        },
      }
    }
    // Perl L354-355: record the source role so the spacewalk can atom-type
    // composite results (XMApp/XMDual with role=RELOP etc., not just tokens).
    if let Some(role) = node.get_attribute("role") {
      let attrs = attributes.get_or_insert_with(Default::default);
      attrs.insert("_role".to_string(), role);
    }
  }
  result
}

/// Add `node`'s lpadding/rpadding (converted to em) onto `result`'s internal
/// `_lpadding`/`_rpadding`, summing with any value already present.
fn attach_source_padding(node: &Node, result: &mut NodeData) {
  for (src, dst) in [("lpadding", "_lpadding"), ("rpadding", "_rpadding")] {
    if let Some(v) = node.get_attribute(src) {
      let em = super::get_xm_hint_spacing(&v);
      if em != 0.0
        && let NodeData::Element { ref mut attributes, .. } = *result
      {
        let attrs = attributes.get_or_insert_with(Default::default);
        let prior = attrs
          .get(dst)
          .and_then(|s| s.trim_end_matches("em").parse::<f64>().ok())
          .unwrap_or(0.0);
        attrs.insert(dst.to_string(), fmt_em(prior + em));
      }
    }
  }
}

fn pmml_inner(doc: &PostDocument, node: &Node) -> NodeData {
  // Fast-path dispatch: all tags we recognize live in the ltx namespace.
  // Compare localname directly and check namespace prefix separately to
  // avoid the `format!("{}:{}", prefix, localname)` allocation inside
  // `get_qname`. On non-ltx nodes, fall through to the generic m:mtext
  // wrapping (same as the original catchall).
  let is_ltx = doc.qname_prefix(node).as_deref() == Some("ltx");
  let localname = if is_ltx {
    node.get_name()
  } else {
    String::new()
  };

  // Follow XMRef
  if is_ltx && localname == "XMRef" {
    if let Some(idref) = node.get_attribute("idref") {
      if let Some(target) = doc.find_node_by_id(&idref) {
        return pmml(doc, target);
      }
    }
    return pmml_error("Unresolved XMRef");
  }

  if is_ltx {
    match localname.as_str() {
      "XMath" => {
        let results: Vec<NodeData> = element_children_iter(node).map(|c| pmml(doc, &c)).collect();
        return pmml_row(results);
      },
      "XMDual" => {
        let children = element_children(node);
        return if children.len() >= 2 {
          pmml(doc, &children[1]) // Presentation branch
        } else {
          pmml_error("Empty XMDual")
        };
      },
      "XMWrap" | "XMArg" => {
        // Perl L400-401: only present when parsing failed; resizable.
        let results: Vec<NodeData> = element_children_iter(node).map(|c| pmml(doc, &c)).collect();
        return pmml_maybe_resize(doc, node, pmml_row(results));
      },
      "XMApp" => return pmml_apply(doc, node),
      "XMTok" => return pmml_token(doc, node),
      "XMHint" => return pmml_hint(doc, node),
      "XMArray" => return pmml_array(doc, node),
      "XMText" => {
        // Perl L494-501: iterate over child nodes, not just text content.
        // This preserves ltx:picture (SVG) elements inside XMText.
        let mut children = Vec::new();
        if let Some(child) = node.get_first_child() {
          let mut current = Some(child);
          while let Some(ref c) = current {
            children.extend(super::pmml_text_aux(doc, c));
            current = c.get_next_sibling();
          }
        }
        return pmml_maybe_resize(doc, node, pmml_row(children));
      },
      _ => {},
    }
  }

  // Catchall: wrap content in m:mtext.
  NodeData::Element {
    tag:        "m:mtext".to_string(),
    attributes: None,
    children:   vec![NodeData::Text(node.get_content())],
  }
}

/// Convert an XMApp to Presentation MathML.
///
/// Port of `pmml_internal` XMApp branch.
fn pmml_apply(doc: &PostDocument, node: &Node) -> NodeData {
  let children = element_children(node);
  if children.is_empty() {
    return pmml_error("Missing Operator");
  }

  let role = node.get_attribute("role").unwrap_or_default();

  // Handle floating/post scripts.
  //
  // `<msub>` / `<msup>` require a base; for "floating" scripts (e.g.
  // `{}^c`, `_d`) the base is structurally absent. We materialize
  // the missing base as `<m:mrow></m:mrow>` — not `<m:mi></m:mi>`.
  // Same rationale as the `absent` case below: `<mi>` is a semantic
  // claim ("here is an identifier") that's false when empty; `<mrow>`
  // is presentational scaffolding with no semantic content. Task #264.
  if role.contains("SUBSCRIPT") || role.contains("SUPERSCRIPT") {
    let is_sub = role.contains("SUB");
    let tag = if is_sub { "m:msub" } else { "m:msup" };
    return NodeData::Element {
      tag:        tag.to_string(),
      attributes: None,
      children:   vec![
        NodeData::Element {
          tag:        "m:mrow".to_string(),
          attributes: None,
          children:   vec![],
        },
        pmml_scriptsize(doc, &children[0]),
      ],
    };
  }

  let op = &children[0];
  let args = &children[1..];

  // Realize the operator
  let rop = if doc.is_qname(op, "ltx:XMRef") {
    op.get_attribute("idref")
      .and_then(|id| doc.find_node_by_id(&id).cloned())
      .unwrap_or_else(|| op.clone())
  } else {
    op.clone()
  };

  let op_role = get_operator_role(doc, &rop).unwrap_or_default();
  let meaning = rop.get_attribute("meaning").unwrap_or_default();

  // Perl MathML.pm L413-427: the operator's `mathstyle` switches the current
  // style for the conversion of this application, and the result is wrapped
  // in m:mstyle per the transition tables (\tfrac in display math →
  // <mstyle displaystyle="false">, \dfrac in text → displaystyle="true").
  let style_attr = rop
    .get_attribute("mathstyle")
    .or_else(|| op.get_attribute("mathstyle"));
  let ostyle = CURRENT_STYLE.with(|s| s.get());
  let nstyle = style_attr.as_deref().and_then(MathStyle::from_attr);
  if let Some(n) = nstyle {
    CURRENT_STYLE.with(|s| s.set(n));
  }
  let result = pmml_apply_dispatch(doc, op, &rop, args, &op_role, &meaning);
  CURRENT_STYLE.with(|s| s.set(ostyle));
  // Perl L421: resize BEFORE the mstyle wrap.
  let result = pmml_maybe_resize(doc, node, result);
  maybe_style_wrap(result, ostyle, nstyle)
}

/// Role/meaning dispatch for an XMApp (the body of Perl's
/// `lookupPresenter('Apply',…)` call in `pmml_internal`).
fn pmml_apply_dispatch(
  doc: &PostDocument,
  op: &Node,
  rop: &Node,
  args: &[Node],
  op_role: &str,
  meaning: &str,
) -> NodeData {
  // Dispatch by role
  match op_role {
    "SUPERSCRIPTOP" | "SUBSCRIPTOP" if args.len() >= 2 => {
      pmml_script_full(doc, op, &args[0], &args[1])
    },
    "FRACOP" if args.len() >= 2 => {
      // Perl MathML.pm L1597-1605 `Apply:FRACOP:?`: linethickness passes
      // through VERBATIM whenever defined (\binom → "0pt", \genfrac 2pt →
      // "2.0pt"); mathcolor from the op's color; bevelled fractions carry
      // class="ltx_bevelled" → bevelled="true". (Perl's context $COLOR
      // fallback is part of the unported pmml_top bindings — F8.)
      let mut attrs = HashMap::default();
      if let Some(t) = rop.get_attribute("thickness") {
        attrs.insert("linethickness".to_string(), t);
      }
      if let Some(c) = rop.get_attribute("color") {
        attrs.insert("mathcolor".to_string(), c);
      }
      if let Some(cl) = rop.get_attribute("class")
        && cl.split_ascii_whitespace().any(|c| c == "ltx_bevelled")
      {
        attrs.insert("bevelled".to_string(), "true".to_string());
      }
      NodeData::Element {
        tag:        "m:mfrac".to_string(),
        attributes: if attrs.is_empty() { None } else { Some(attrs) },
        children:   vec![pmml_smaller(doc, &args[0]), pmml_smaller(doc, &args[1])],
      }
    },
    "OVERACCENT" if !args.is_empty() => {
      // Perl MathML.pm L1492-1504: check if base is XMApp with UNDERACCENT → m:munderover
      let base = &args[0];
      let base_children = element_children(base);
      if doc.is_qname(base, "ltx:XMApp") && base_children.len() == 2 {
        let inner_role = base_children[0].get_attribute("role").unwrap_or_default();
        if inner_role == "UNDERACCENT" {
          // Combine into m:munderover: base_of_inner, under_accent, over_accent
          return NodeData::Element {
            tag:        "m:munderover".to_string(),
            attributes: Some(HashMap::from_iter([
              ("accent".to_string(), "true".to_string()),
              ("accentunder".to_string(), "true".to_string()),
            ])),
            children:   vec![
              pmml(doc, &base_children[1]), // the actual base
              pmml(doc, &base_children[0]), // the under-accent
              pmml(doc, op),                // the over-accent
            ],
          };
        }
      }
      NodeData::Element {
        tag:        "m:mover".to_string(),
        attributes: Some(HashMap::from_iter([(
          "accent".to_string(),
          "true".to_string(),
        )])),
        children:   vec![pmml(doc, base), pmml(doc, op)],
      }
    },
    "UNDERACCENT" if !args.is_empty() => {
      // Perl MathML.pm L1507-1519: check if base is XMApp with OVERACCENT → m:munderover
      let base = &args[0];
      let base_children = element_children(base);
      if doc.is_qname(base, "ltx:XMApp") && base_children.len() == 2 {
        let inner_role = base_children[0].get_attribute("role").unwrap_or_default();
        if inner_role == "OVERACCENT" {
          return NodeData::Element {
            tag:        "m:munderover".to_string(),
            attributes: Some(HashMap::from_iter([
              ("accent".to_string(), "true".to_string()),
              ("accentunder".to_string(), "true".to_string()),
            ])),
            children:   vec![
              pmml(doc, &base_children[1]), // the actual base
              pmml(doc, op),                // the under-accent
              pmml(doc, &base_children[0]), // the over-accent
            ],
          };
        }
      }
      NodeData::Element {
        tag:        "m:munder".to_string(),
        attributes: Some(HashMap::from_iter([(
          "accentunder".to_string(),
          "true".to_string(),
        )])),
        children:   vec![pmml(doc, base), pmml(doc, op)],
      }
    },
    "POSTFIX" if !args.is_empty() => {
      let mut items: Vec<NodeData> = args.iter().map(|a| pmml(doc, a)).collect();
      items.push(pmml(doc, op));
      pmml_row(items)
    },
    "ADDOP" | "RELOP" | "MULOP" | "BINOP" | "ARROW" | "METARELOP" | "COMPOSEOP" | "MODIFIEROP"
    | "MIDDLE" => {
      // Infix: arg1 op arg2 op arg3 ...
      pmml_infix(doc, op, args)
    },
    "SUMOP" | "INTOP" | "BIGOP" | "LIMITOP" => {
      // Big operator: Σ/∫ applied to args (gets FUNCTION APPLICATION ⁡).
      // DIFFOP (∂, d, ∇-as-diff) is deliberately EXCLUDED — Perl MathML.pm:702
      // `$ismoveop = … (SUMOP|INTOP|BIGOP|LIMITOP)$/  # Not DIFFOP`; a DIFFOP
      // falls to the generic apply below, which juxtaposes (no ⁡) because its
      // base renders as <m:mo>. (Witness: `\partial f` → ∂f, not ∂⁡f.)
      pmml_summation(doc, op, args)
    },
    "OPEN" | "CLOSE" if !args.is_empty() => {
      // Fenced: (args)
      pmml_parenthesize(doc, op, args)
    },
    "ENCLOSE" if !args.is_empty() => {
      // Perl MathML.pm L1507-1513 `Apply:ENCLOSE:?`: m:menclose with the
      // operator's `enclose` attribute as notation (e.g. \cancel →
      // updiagonalstrike); if the op carries a color, the enclosure gets it
      // as mathcolor and the base is reset via m:mstyle (Perl's context
      // $COLOR fallback is part of the unported pmml_top bindings — F8).
      let mut attrs = HashMap::default();
      if let Some(notation) = rop.get_attribute("enclose") {
        attrs.insert("notation".to_string(), notation);
      }
      let color = rop.get_attribute("color");
      let base = pmml(doc, &args[0]);
      let inner = if let Some(ref c) = color {
        attrs.insert("mathcolor".to_string(), c.clone());
        NodeData::Element {
          tag:        "m:mstyle".to_string(),
          attributes: Some(HashMap::from_iter([(
            "mathcolor".to_string(),
            "black".to_string(),
          )])),
          children:   vec![base],
        }
      } else {
        base
      };
      NodeData::Element {
        tag:        "m:menclose".to_string(),
        attributes: if attrs.is_empty() { None } else { Some(attrs) },
        children:   vec![inner],
      }
    },
    _ if meaning == "multirelation" => {
      // Multirelation: a = b = c (interleaved args and operators)
      // Port of `Apply:?:multirelation` handler.
      let mut items = Vec::new();
      for (i, arg) in args.iter().enumerate() {
        if i > 0 && i % 2 == 1 {
          // Odd positions are operators in multirelation
          items.push(pmml(doc, arg));
        } else {
          items.push(pmml(doc, arg));
        }
      }
      pmml_row(items)
    },
    _ => {
      // Default: function application
      if meaning == "limit-from" && !args.is_empty() {
        // limit-from: base followed by direction
        let items: Vec<NodeData> = args.iter().map(|a| pmml(doc, a)).collect();
        pmml_row(items)
      } else if meaning == "annotated" && args.len() >= 2 {
        // annotated: variable with annotation (e.g. "x modulo p")
        pmml_row(vec![
          pmml(doc, &args[0]),
          NodeData::Element {
            tag:        "m:mspace".to_string(),
            attributes: Some(HashMap::from_iter([(
              "width".to_string(),
              "0.389em".to_string(),
            )])),
            children:   vec![],
          },
          pmml(doc, &args[1]),
        ])
      } else if meaning == "square-root" && !args.is_empty() {
        // Perl L1639-1642: mathcolor from the op's color (context $COLOR — F8)
        NodeData::Element {
          tag:        "m:msqrt".to_string(),
          attributes: rop
            .get_attribute("color")
            .map(|c| HashMap::from_iter([("mathcolor".to_string(), c)])),
          children:   vec![pmml(doc, &args[0])],
        }
      } else if meaning == "continued-fraction" && args.len() >= 2 {
        pmml_cfrac(doc, op, &args[0], &args[1])
      } else if meaning == "nth-root" && args.len() >= 2 {
        // Perl L1644-1647: `['m:mroot', …, pmml($_[2]), pmml_scriptsize($_[1])]`
        // — args are (degree, radicand) in BOTH engines' XMath; m:mroot takes
        // the base first, then the scriptsized degree. (Previously swapped:
        // degree rendered as the base, radicand shrunk.)
        NodeData::Element {
          tag:        "m:mroot".to_string(),
          attributes: rop
            .get_attribute("color")
            .map(|c| HashMap::from_iter([("mathcolor".to_string(), c)])),
          children:   vec![pmml(doc, &args[1]), pmml_scriptsize(doc, &args[0])],
        }
      } else {
        // Generic application: op(arg1, arg2, ...). Insert FUNCTION APPLICATION
        // (⁡, U+2061) ONLY when the operator's base is NOT an <m:mo> — Perl
        // MathML.pm `Apply:?:?` (`$is_mo ? () : pmml_mo("\x{2061}")`). So an
        // OPERATOR/DIFFOP like ∇ juxtaposes (∇ϕ, spacing via the mo's rspace),
        // while a function identifier f gets f⁡(x).
        let pop = pmml(doc, op);
        let needs_apply = !op_base_is_mo(&pop);
        let mut items = vec![pop];
        if needs_apply {
          items.push(pmml_mo_str("\u{2061}")); // FUNCTION APPLICATION
        }
        for arg in args {
          items.push(pmml(doc, arg));
        }
        pmml_row(items)
      }
    },
  }
}

/// Convert an XMTok to the appropriate Presentation MathML token.
///
/// Port of `stylizeContent` + token converter.
fn pmml_token(doc: &PostDocument, node: &Node) -> NodeData {
  // Perl `pmml_bigop` (MathML.pm L847-856): a SUMOP/INTOP/BIGOP token whose
  // recorded `mathstyle` differs from the current style converts under the
  // switched style and wraps in m:mstyle via %stylemap (the displaystyle-
  // carrying table, unconditionally) — e.g. `\displaystyle\sum` in inline
  // math keeps its large rendering. Token:LIMITOP is plain pmml_mo in Perl.
  let nstyle = match node.get_attribute("role").as_deref() {
    Some("SUMOP" | "INTOP" | "BIGOP") => node
      .get_attribute("mathstyle")
      .as_deref()
      .and_then(MathStyle::from_attr),
    _ => None,
  };
  let ostyle = CURRENT_STYLE.with(|s| s.get());
  if let Some(n) = nstyle {
    CURRENT_STYLE.with(|s| s.set(n));
  }
  let result = pmml_token_inner(doc, node);
  CURRENT_STYLE.with(|s| s.set(ostyle));
  match nstyle {
    Some(n) if n != ostyle => {
      let style_attrs = stylemap_attrs(ostyle, n, true);
      if style_attrs.is_empty() {
        result
      } else {
        NodeData::Element {
          tag:        "m:mstyle".to_string(),
          attributes: Some(HashMap::from_iter(
            style_attrs
              .iter()
              .map(|(k, v)| (k.to_string(), v.to_string())),
          )),
          children:   vec![result],
        }
      }
    },
    _ => result,
  }
}

/// Resolve a token's explicit fontsize for emission (Perl `stylizeContent`
/// L782-792): a %-size in script context is re-expressed relative to the
/// script style's nominal size, then any %-size is converted to em ("safari
/// apparently ignores %").
fn resolve_token_size(mut s: String) -> String {
  if let Some(req) = s.strip_suffix('%') {
    let ctx = current_context_size().trim_end_matches('%');
    if matches!(
      CURRENT_STYLE.with(|c| c.get()),
      MathStyle::Script | MathStyle::ScriptScript
    ) && let (Ok(req), Ok(ex)) = (req.parse::<f64>(), ctx.parse::<f64>())
      && ex != 0.0
    {
      s = format!("{}%", (100.0 * req / ex) as i32);
    }
    if let Some(pct) = s.strip_suffix('%')
      && let Ok(pct) = pct.parse::<f64>()
    {
      s = fmt_em(pct / 100.0);
    }
  }
  s
}

fn pmml_token_inner(doc: &PostDocument, node: &Node) -> NodeData {
  let role = node
    .get_attribute("role")
    .unwrap_or_else(|| "UNKNOWN".to_string());
  let font = node.get_attribute("font");
  let mut text = node.get_content();
  let meaning = node.get_attribute("meaning");

  // Handle special meanings
  if meaning.as_deref() == Some("absent") {
    // "absent" is an XMath placeholder for a structurally-missing operand
    // (e.g. the LHS of a continuation row `& = ...` in `align*` whose
    // LHS is inherited from the previous row, or a prefix operator
    // applied with no left argument). At the MathML Presentation
    // layer we materialize this as an EMPTY `<m:mrow></m:mrow>` —
    // not `<m:mi></m:mi>`. `<m:mi>` is a semantic assertion ("here
    // is a mathematical identifier") with no defined meaning when
    // empty; renderers vary, screen readers announce "blank" or
    // skip awkwardly, and search/indexing tools pollute their
    // index with content-free identifier tokens. `<m:mrow>` is
    // presentational grouping without a semantic claim — well-
    // defined for empty content (zero-width, no announcement).
    // Task #264.
    return NodeData::Element {
      tag:        "m:mrow".to_string(),
      attributes: None,
      children:   vec![],
    };
  }

  // Determine tag based on role
  let tag = match role.as_str() {
    "NUMBER" => "m:mn",
    "ID" | "UNKNOWN" => "m:mi",
    "FUNCTION" | "OPFUNCTION" | "TRIGFUNCTION" => "m:mi",
    _ => "m:mo",
  };

  // Handle empty tokens
  if text.is_empty() {
    if let Some(default) = default_token_content(&role) {
      text = default.to_string();
    } else {
      text = meaning
        .or_else(|| node.get_attribute("name"))
        .unwrap_or_else(|| role.clone());
    }
  }

  // Minus sign normalization
  if text == "-" && matches!(role.as_str(), "ADDOP" | "OPERATOR") {
    text = "\u{2212}".to_string(); // MINUS SIGN
  }

  // Perl L772-775: when invisibletimes is false, replace U+2062 with U+200B
  let is_replaced_invisible_times = text == "\u{2062}" && !get_invisible_times();
  if is_replaced_invisible_times {
    text = "\u{200B}".to_string(); // ZERO WIDTH SPACE
  }

  let mut attrs = HashMap::default();

  // Perl: zero-width space <mo> needs lspace/rspace="0em" to prevent browser
  // default operator spacing that creates visible gaps between letters.
  if is_replaced_invisible_times && tag == "m:mo" {
    attrs.insert("lspace".to_string(), "0em".to_string());
    attrs.insert("rspace".to_string(), "0em".to_string());
  }

  // Math variant from font — with Plane 1 Unicode conversion.
  // Port of Perl stylizeContent lines 689-756.
  {
    use crate::unicode;
    let mut variant: Option<&str> = font.as_deref().map(unicode::unicode_mathvariant);

    // Single char mi: italic is default
    if tag == "m:mi" && text.chars().count() == 1 {
      if variant == Some("italic") {
        variant = None;
      } else if variant.is_none() && font.is_none() {
        // Check if it's a named symbol (not a variable) → use "normal"
        if node.get_attribute("name").is_some() {
          variant = Some("normal");
        }
      } else if variant.is_none() {
        variant = Some("normal");
      }
    } else if font.is_some() && variant == Some("normal") {
      variant = None; // normal is default for non-single-char-mi tokens
    } else if tag == "m:mi" && text.chars().count() > 1 && font.is_none() {
      variant = Some("normal"); // multi-char mi without font → normal
    }

    // Plane 1 Unicode conversion
    if let Some(v) = variant {
      if tag != "m:mtext" {
        if let Some(u_text) = unicode::unicode_convert(&text, v) {
          if !u_text.is_empty() || text.is_empty() {
            text = u_text;
            variant = None; // character carries style
          }
        }
      }
    }

    // Emit remaining variant attribute
    if let Some(v) = variant {
      if tag == "m:mi" && text.chars().count() == 1 {
        if v != "italic" {
          attrs.insert("mathvariant".to_string(), v.to_string());
        }
      } else if v != "normal" {
        attrs.insert("mathvariant".to_string(), v.to_string());
      }
    }

    // Font-based CSS class fallbacks.
    // Port of Perl L746-756: added regardless of plane1 conversion.
    let is_format_only = text.chars().all(|c| {
      matches!(c,
        '\u{200B}'..='\u{200F}' | '\u{2028}'..='\u{202F}'
        | '\u{2060}'..='\u{2064}' | '\u{FEFF}' | '\u{00AD}')
    }) && !text.is_empty();
    if let Some(ref f) = font {
      if !is_format_only {
        if f.contains("caligraphic") {
          let prev = attrs.get("class").cloned().unwrap_or_default();
          let new = if prev.is_empty() {
            "ltx_font_mathcaligraphic".to_string()
          } else {
            format!("{} ltx_font_mathcaligraphic", prev)
          };
          attrs.insert("class".to_string(), new);
        } else if f.contains("script") {
          let prev = attrs.get("class").cloned().unwrap_or_default();
          let new = if prev.is_empty() {
            "ltx_font_mathscript".to_string()
          } else {
            format!("{} ltx_font_mathscript", prev)
          };
          attrs.insert("class".to_string(), new);
        } else if f.contains("fraktur") && text.chars().all(|c| "+-0123456789.".contains(c)) {
          let prev = attrs.get("class").cloned().unwrap_or_default();
          let new = if prev.is_empty() {
            "ltx_font_oldstyle".to_string()
          } else {
            format!("{} ltx_font_oldstyle", prev)
          };
          attrs.insert("class".to_string(), new);
        } else if f.contains("smallcaps") {
          let prev = attrs.get("class").cloned().unwrap_or_default();
          let new = if prev.is_empty() {
            "ltx_font_smallcaps".to_string()
          } else {
            format!("{} ltx_font_smallcaps", prev)
          };
          attrs.insert("class".to_string(), new);
        } else if let Some(v) = variant {
          if v != "normal" {
            let prev = attrs.get("class").cloned().unwrap_or_default();
            let new = if prev.is_empty() {
              format!("ltx_mathvariant_{}", v)
            } else {
              format!("{} ltx_mathvariant_{}", prev, v)
            };
            attrs.insert("class".to_string(), new);
          }
        }
      }
    }
  }

  // Perl emits mathsize for ALL token types, not just m:mo (witness:
  // smallmatrix cells get mathsize="0.700em"). The context gate compensates
  // for our engine stamping absolute fontsize="70%" on script tokens where
  // Perl's leaves them bare — a matching size must NOT be re-emitted.
  if tag != "m:mo"
    && let Some(size) = node.get_attribute("fontsize")
    && size != current_context_size()
  {
    attrs.insert("mathsize".to_string(), resolve_token_size(size));
  }

  // Operator-specific attributes: the mo half of Perl `stylizeContent`
  // (L697-827) — operator-dictionary xor-emission, size/stretchy interplay,
  // largeop/movablelimits/symmetric. (audit F8)
  if tag == "m:mo" {
    let props = operator_dictionary::opdict_lookup(&text, &role);
    // Perl L697-704: implied attributes.
    let mut stretchy = node.get_attribute("stretchy").as_deref() == Some("true");
    let is_fence = matches!(role.as_str(), "OPEN" | "CLOSE" | "MIDDLE");
    let is_sep = role == "PUNCT";
    let is_largeop = matches!(role.as_str(), "SUMOP" | "INTOP");
    let is_moveop = matches!(role.as_str(), "SUMOP" | "INTOP" | "BIGOP" | "LIMITOP"); // Not DIFFOP
    let is_symm = is_largeop || text == "/"; // WANTS to be symmetric
    let pos = node
      .get_attribute("scriptpos")
      .unwrap_or_else(|| "post".to_string());

    // Perl L774-778: ignore size when stretching; invisible operators get
    // neither size nor stretchiness.
    let mut size = node.get_attribute("fontsize");
    if stretchy {
      size = None;
    }
    let is_invisible =
      !text.is_empty() && text.chars().all(|c| matches!(c, '\u{2061}'..='\u{2063}'));
    if is_invisible {
      stretchy = false;
      size = None;
    }

    // Perl L779-798: size resolution. Emit only when the token's size
    // differs from the current contextual size (inside a script the msup/
    // msub structure already shrinks it — a matching "70%" must NOT be
    // re-emitted); a differing %-size in script context is re-expressed
    // relative to the script's nominal size, then converted to em ("safari
    // apparently ignores %"). Symmetric-wanting delimiters at explicit
    // sizes use the minsize/maxsize stretchyhack ("Thanks Peter
    // Krautzberger") instead of mathsize.
    let mut props_stretchy = props.stretchy;
    let mut stretchyhack = false;
    let resolved_size = size
      .filter(|s| s != current_context_size())
      .map(resolve_token_size);
    if let Some(size) = resolved_size {
      if is_symm || props.symmetric {
        stretchyhack = true;
        // Force the attribute to avoid browser bugs (esp "|").
        if !matches!(text.as_str(), "(" | ")" | "[" | "]" | "{" | "}") {
          props_stretchy = false;
        }
        stretchy = true; // pretend we asked for stretchy
        attrs.insert("minsize".to_string(), size.clone());
        attrs.insert("maxsize".to_string(), size);
      } else {
        stretchy = false; // size specifically set → don't stretch it
        attrs.insert("mathsize".to_string(), size);
      }
    }
    let _ = stretchyhack;

    // Perl L811-826: emit operator-dictionary attributes only where the
    // wanted value differs from what the dictionary already implies (xor).
    if stretchy != props_stretchy {
      attrs.insert(
        "stretchy".to_string(),
        (if stretchy { "true" } else { "false" }).to_string(),
      );
    }
    if is_fence != props.fence {
      attrs.insert(
        "fence".to_string(),
        (if is_fence { "true" } else { "false" }).to_string(),
      );
    }
    if is_sep != props.separator {
      attrs.insert(
        "separator".to_string(),
        (if is_sep { "true" } else { "false" }).to_string(),
      );
    }
    if is_largeop != props.largeop {
      attrs.insert(
        "largeop".to_string(),
        (if is_largeop { "true" } else { "false" }).to_string(),
      );
    }
    if is_largeop {
      attrs.insert("_largeop".to_string(), "1".to_string()); // For needsMathstyle
    }
    if is_symm && !props.symmetric && (stretchy || props_stretchy) {
      attrs.insert("symmetric".to_string(), "true".to_string());
    }
    // If an operator has specifically located its scripts, don't let MathML
    // move them. (Perl also honors $NOMOVABLELIMITS from script layout —
    // unported, audit F17.)
    if is_moveop && pos.contains("mid") {
      attrs.insert("movablelimits".to_string(), "false".to_string());
    }

    // Store internal spacing attributes for adjust_spacing (Perl L821-824).
    attrs.insert("_role".to_string(), role.clone());
    if props.lspace > 0.0 {
      attrs.insert("_lspace".to_string(), fmt_em(props.lspace));
    }
    if props.rspace > 0.0 {
      attrs.insert("_rspace".to_string(), fmt_em(props.rspace));
    }
  }

  // Color
  if let Some(color) = node.get_attribute("color") {
    attrs.insert("mathcolor".to_string(), color);
  }

  // Href
  if let Some(href) = node.get_attribute("href") {
    attrs.insert("href".to_string(), href);
  }

  // Class
  if let Some(class) = node.get_attribute("class") {
    attrs.insert("class".to_string(), class);
  }

  // Source locator (token-locators): carry the XMTok's source position onto the
  // MathML token element so the editor can map a rendered symbol back to its
  // source (per-token in-equation provenance, §7 A.3). The math XSLT copies the
  // generated MathML verbatim, so emit the final HTML5 `data-sourcepos` name.
  // `data:sourcepos` is namespaced (the LaTeXML `data:` namespace), so read it
  // by local name + namespace URI (like `xml:id` is read elsewhere).
  if let Some(sp) = node.get_attribute_ns("sourcepos", "http://dlmf.nist.gov/LaTeXML/data") {
    attrs.insert("data-sourcepos".to_string(), sp);
  }

  // Perl pmml_mi/pmml_mn/pmml_mo (L830-845) all pass the token through
  // pmml_maybe_resize (raised/framed/phantom-sized tokens).
  pmml_maybe_resize(doc, node, NodeData::Element {
    tag:        tag.to_string(),
    attributes: if attrs.is_empty() { None } else { Some(attrs) },
    children:   vec![NodeData::Text(text)],
  })
}

/// Convert an XMHint to MathML.
///
/// Port of Hint handler.
fn pmml_hint(_doc: &PostDocument, node: &Node) -> NodeData {
  // Perl `Hint:?:?` (MathML.pm L1479-1483): the width is normalized through
  // getXMHintSpacing to em (so `\qquad` → width="2em", not the raw "20pt");
  // zero-width hints still MUST return a node, marked `_ignorable` so
  // filter_row drops them from rows.
  let w = node
    .get_attribute("width")
    .map(|w| super::get_xm_hint_spacing(&w))
    .unwrap_or(0.0);
  let attrs = if w != 0.0 {
    // Perl appends 'em' to the raw number ($w . 'em'), NOT fmt_em — emulate
    // Perl's default %.15g stringification.
    HashMap::from_iter([("width".to_string(), format!("{}em", perl_num(w)))])
  } else {
    HashMap::from_iter([("_ignorable".to_string(), "1".to_string())])
  };
  NodeData::Element {
    tag:        "m:mspace".to_string(),
    attributes: Some(attrs),
    children:   vec![],
  }
}

/// Format a float the way Perl stringifies numbers (%.15g): up to 15
/// significant digits, no trailing zeros.
fn perl_num(v: f64) -> String {
  let s = format!("{v:.15}");
  if s.contains('.') {
    s.trim_end_matches('0').trim_end_matches('.').to_string()
  } else {
    s
  }
}

/// Convert an XMArray to an mtable.
///
/// Port of `pmml_internal` XMArray branch (`MathML.pm` L432-486).
fn pmml_array(doc: &PostDocument, node: &Node) -> NodeData {
  // Perl `pmml_internal` XMArray branch (L432-506): the array's `mathstyle`
  // switches the current style for the cell conversions; the mtable gets
  // displaystyle="true" when that style is display ("Mozilla seems to need
  // some encouragement?"), and the whole result wraps in m:mstyle per the
  // transition tables.
  let ostyle = CURRENT_STYLE.with(|s| s.get());
  let nstyle = node
    .get_attribute("mathstyle")
    .as_deref()
    .and_then(MathStyle::from_attr);
  if let Some(n) = nstyle {
    CURRENT_STYLE.with(|s| s.set(n));
  }
  let result = pmml_array_inner(doc, node);
  CURRENT_STYLE.with(|s| s.set(ostyle));
  // Perl L492-506: XMArray resizes AFTER the mstyle wrap (XMApp: before).
  let result = maybe_style_wrap(result, ostyle, nstyle);
  pmml_maybe_resize(doc, node, result)
}

fn pmml_array_inner(doc: &PostDocument, node: &Node) -> NodeData {
  let mut rows = Vec::new();
  let width = node.get_attribute("width");
  let vattach = node
    .get_attribute("vattach")
    .unwrap_or_else(|| "middle".to_string());
  let align = match vattach.as_str() {
    "top" => "bottom1",
    "middle" | "" => "axis",
    _ => vattach.as_str(),
  };
  let rowsep = node
    .get_attribute("rowsep")
    .unwrap_or_else(|| "0pt".to_string());
  let colsep = node
    .get_attribute("colsep")
    .unwrap_or_else(|| "5pt".to_string());

  let mut nrows = 0;
  let mut ncols = 0;
  for row_node in element_children(node) {
    let mut cols = Vec::new();
    let mut nc = 0;
    for cell_node in element_children(&row_node) {
      nc += 1;
      let cell_align = cell_node.get_attribute("align");
      let colspan = cell_node.get_attribute("colspan");
      let rowspan = cell_node.get_attribute("rowspan");
      let mut td_attrs = HashMap::default();
      if let Some(a) = &cell_align {
        if a != "center" {
          td_attrs.insert("columnalign".to_string(), a.clone());
          td_attrs.insert("class".to_string(), format!("ltx_align_{}", a));
        }
      }
      if let Some(cs) = colspan {
        td_attrs.insert("columnspan".to_string(), cs);
      }
      if let Some(rs) = rowspan {
        td_attrs.insert("rowspan".to_string(), rs);
      }

      let cell_children = element_children(&cell_node);
      let cell_content = if cell_children.is_empty() {
        vec![]
      } else {
        // Perl L468: cells filter _ignorable items too.
        filter_row(cell_children.iter().map(|c| pmml(doc, c)).collect())
      };

      cols.push(NodeData::Element {
        tag:        "m:mtd".to_string(),
        attributes: if td_attrs.is_empty() {
          None
        } else {
          Some(td_attrs)
        },
        children:   cell_content,
      });
    }
    if nc > ncols {
      ncols = nc;
    }
    nrows += 1;
    rows.push(NodeData::Element {
      tag:        "m:mtr".to_string(),
      attributes: None,
      children:   cols,
    });
  }

  // Perl L478-479: drop separators if there's only one row/column.
  let emit_rowsep = nrows >= 2;
  let emit_colsep = ncols >= 2;

  let mut table_attrs = HashMap::default();
  if align != "axis" {
    table_attrs.insert("align".to_string(), align.to_string());
  }
  if emit_rowsep {
    table_attrs.insert("rowspacing".to_string(), rowsep);
  }
  if emit_colsep {
    table_attrs.insert("columnspacing".to_string(), colsep);
  }
  if let Some(w) = width {
    table_attrs.insert("width".to_string(), w);
  }
  // Perl L484-485: "Mozilla seems to need some encouragement?"
  if CURRENT_STYLE.with(|s| s.get()) == MathStyle::Display {
    table_attrs.insert("displaystyle".to_string(), "true".to_string());
  }

  NodeData::Element {
    tag:        "m:mtable".to_string(),
    attributes: if table_attrs.is_empty() {
      None
    } else {
      Some(table_attrs)
    },
    children:   rows,
  }
}

// ======================================================================
// Layout helpers

/// Simple sub/superscript.
fn pmml_script_simple(doc: &PostDocument, tag: &str, base: &Node, script: &Node) -> NodeData {
  NodeData::Element {
    tag:        tag.to_string(),
    attributes: None,
    children:   vec![pmml(doc, base), pmml_scriptsize(doc, script)],
  }
}

/// Convert node at script size (sub/superscripts). Port of Perl `pmml_scriptsize`:
/// steps the style to scriptstyle (→ scriptscript when already in a script) for the
/// duration of the recursion, so contained tokens compare against the smaller size.
fn pmml_scriptsize(doc: &PostDocument, node: &Node) -> NodeData {
  let old = CURRENT_STYLE.with(|s| {
    let o = s.get();
    s.set(o.script_step());
    o
  });
  let r = pmml(doc, node);
  CURRENT_STYLE.with(|s| s.set(old));
  r
}

/// Convert node at smaller size (fraction numerator/denominator). Port of Perl
/// `pmml_smaller`: steps the style down one level for the duration of the recursion.
fn pmml_smaller(doc: &PostDocument, node: &Node) -> NodeData {
  let old = CURRENT_STYLE.with(|s| {
    let o = s.get();
    s.set(o.step_down());
    o
  });
  let r = pmml(doc, node);
  CURRENT_STYLE.with(|s| s.set(old));
  r
}

/// Infix operator: arg1 op arg2 op arg3 ...
///
/// Port of `pmml_infix`.
fn pmml_infix(doc: &PostDocument, op: &Node, args: &[Node]) -> NodeData {
  let op_mml = pmml(doc, op);
  if args.is_empty() {
    return op_mml;
  }
  // For Presentation MathML we suppress XMath's `absent` placeholders
  // entirely — they exist to satisfy the content-arm structural
  // contract (every binary application has 2 operands), but materializing
  // them as visible MathML degrades accessibility (screen readers
  // announce a blank/empty group, indexers see a spurious atom).
  //
  // The shape decision depends on WHICH operand is absent:
  //   - absent left, real right (`Apply(=, absent, RHS)`) → prefix: <mrow><mo>=</mo><RHS></mrow>
  //     (continuation row `& = RHS` whose LHS is inherited from the previous row — see
  //     prefix_relop_apply in semantics.rs)
  //   - real left, absent right (`Apply(=, LHS, absent)`) → postfix: <mrow><LHS><mo>=</mo></mrow>
  //     (trailing relop — see postfix_relop in semantics.rs)
  //   - real left, real right (normal case) → infix: <mrow><LHS><mo>=</mo><RHS></mrow>
  //   - both absent → just the operator.
  //
  // For the chained case (n≥3 args), drop only absents that are
  // strictly at the boundary (leading or trailing). Interior absents
  // — if any — keep their slot since omitting would change the
  // operand-count interpretation of the chain.
  // Task #264 step 2.
  let leading_absent = is_absent_operand(&args[0]);
  let trailing_absent = args.len() >= 2 && is_absent_operand(&args[args.len() - 1]);
  let slice_start = if leading_absent { 1 } else { 0 };
  let slice_end = if trailing_absent {
    args.len() - 1
  } else {
    args.len()
  };
  let live_args = &args[slice_start..slice_end];

  if live_args.is_empty() {
    return op_mml;
  }
  if live_args.len() == 1 {
    let arg_mml = pmml(doc, &live_args[0]);
    return if trailing_absent {
      // Trailing-relop postfix: `<arg> <mo>op</mo>` (real left, absent right — e.g. a
      // trailing relop `LHS =` continued on the next alignment row).
      pmml_row(vec![arg_mml, op_mml])
    } else {
      // Single operand is rendered PREFIX. Port of Perl `pmml_infix` L632:
      // "Infix with 1 arg is presumably Prefix! (aka Operator)". Covers genuine unary
      // operators (`-21`, `+x`) AND the leading-absent continuation row (`& = RHS`,
      // whose LHS is inherited from the previous row): `<mo>op</mo> <arg>`.
      pmml_row(vec![op_mml, arg_mml])
    };
  }
  let mut items = vec![pmml(doc, &live_args[0])];
  for arg in &live_args[1..] {
    items.push(op_mml.clone());
    items.push(pmml(doc, arg));
  }
  pmml_row(items)
}

/// True iff `node` is the XMath placeholder for a structurally-absent
/// operand — an `<ltx:XMTok>` with `meaning="absent"`. The math
/// parser inserts these as the left operand for prefix-relop rules
/// (`Apply(=, absent, RHS)` for `& = ...` continuation rows) and as
/// the right operand for postfix-relop rules. Used by `pmml_infix`
/// to suppress materialization in Presentation MathML. Task #264.
fn is_absent_operand(node: &Node) -> bool {
  if node.get_name() != "XMTok" {
    return false;
  }
  node.get_attribute("meaning").as_deref() == Some("absent")
}

/// Big operator with possible limits.
///
/// Port of `pmml_summation`.
fn pmml_summation(doc: &PostDocument, op: &Node, args: &[Node]) -> NodeData {
  let op_mml = pmml(doc, op);
  // FUNCTION APPLICATION (⁡) only if the operator base is NOT an <m:mo> — Perl's
  // universal is_mo rule (MathML.pm Apply:?:?). Big operators ∑/∫/⋃/∏/lim all
  // render as <m:mo> (incl. scripted forms like ∑_i via munder), so they
  // juxtapose their body (∑a_i, ∫f) rather than emit ∑⁡a_i — matching Perl.
  let needs_apply = !op_base_is_mo(&op_mml);
  let mut items = vec![op_mml];
  if needs_apply {
    items.push(pmml_mo_str("\u{2061}")); // FUNCTION APPLICATION
  }
  for arg in args {
    items.push(pmml(doc, arg));
  }
  pmml_row(items)
}

/// Parenthesized/fenced expression.
///
/// Port of `pmml_parenthesize`.
fn pmml_parenthesize(doc: &PostDocument, op: &Node, args: &[Node]) -> NodeData {
  let mut items = vec![pmml(doc, op)];
  for arg in args {
    items.push(pmml(doc, arg));
  }
  pmml_row(items)
}

// ======================================================================
// Script handling
//
// Port of `pmml_script` + `pmml_script_decipher` + `pmml_script_multi_layout`.
// Handles complex sub/superscript positioning with pre/mid/post scripts.

/// Script pair: (sub, sup) where either can be None.
type ScriptPair = (Option<Node>, Option<Node>);

/// Full script handler: disentangles pre/mid/post scripts.
///
/// Port of `pmml_script`.
fn pmml_script_full(doc: &PostDocument, op: &Node, base: &Node, script: &Node) -> NodeData {
  let (inner_base, pre_scripts, mid_scripts, post_scripts) =
    pmml_script_decipher(doc, op, base, script);

  // Perl `pmml_script` (L876-891) + `pmml_script_mid_layout` (L899-906):
  // the inner base converts under ITS recorded mathstyle (blocking a nested
  // m:mstyle from the token/apply paths), and when that style differs from
  // the context the whole script layout gets one m:mstyle displaystyle wrap
  // — mstyle doesn't nest well inside scripts.
  let ostyle = CURRENT_STYLE.with(|s| s.get());
  let bstyle = inner_base
    .get_attribute("mathstyle")
    .as_deref()
    .and_then(MathStyle::from_attr);
  if let Some(b) = bstyle {
    CURRENT_STYLE.with(|s| s.set(b));
  }
  let base_mml = pmml(doc, &inner_base);
  CURRENT_STYLE.with(|s| s.set(ostyle));

  // Apply mid scripts (under/over)
  let base_mml = apply_mid_scripts(doc, base_mml, &mid_scripts);

  // Apply pre/post scripts
  let layout = apply_multi_scripts(doc, base_mml, &pre_scripts, &post_scripts);
  match bstyle {
    Some(b) if b != ostyle => NodeData::Element {
      tag:        "m:mstyle".to_string(),
      attributes: Some(HashMap::from_iter([(
        "displaystyle".to_string(),
        (if b == MathStyle::Display {
          "true"
        } else {
          "false"
        })
        .to_string(),
      )])),
      children:   vec![layout],
    },
    _ => layout,
  }
}

/// Decipher nested script applications into pre/mid/post groups.
///
/// Port of `pmml_script_decipher`.
fn pmml_script_decipher(
  doc: &PostDocument,
  op: &Node,
  base: &Node,
  script: &Node,
) -> (Node, Vec<ScriptPair>, Vec<ScriptPair>, Vec<ScriptPair>) {
  let mut pre_scripts: Vec<ScriptPair> = Vec::new();
  let mut mid_scripts: Vec<ScriptPair> = Vec::new();
  let mut post_scripts: Vec<ScriptPair> = Vec::new();

  let role = op.get_attribute("role").unwrap_or_default();
  let pos_str = op
    .get_attribute("scriptpos")
    .unwrap_or_else(|| "post0".to_string());
  let pos = if pos_str.starts_with("pre") {
    "pre"
  } else if pos_str.starts_with("mid") {
    "mid"
  } else {
    "post"
  };
  let is_sub = role.contains("SUB");

  // Place first script
  let pair = if is_sub {
    (Some(script.clone()), None)
  } else {
    (None, Some(script.clone()))
  };

  match pos {
    "pre" => pre_scripts.push(pair),
    "mid" => mid_scripts.push(pair),
    _ => post_scripts.push(pair),
  }

  // Walk down through nested scripts on the base
  let mut current_base = base.clone();
  loop {
    // Realize XMRef
    let realized = if doc.is_qname(&current_base, "ltx:XMRef") {
      current_base
        .get_attribute("idref")
        .and_then(|id| doc.find_node_by_id(&id).cloned())
        .unwrap_or_else(|| current_base.clone())
    } else {
      current_base.clone()
    };

    if !doc.is_qname(&realized, "ltx:XMApp") {
      break;
    }

    let children = element_children(&realized);
    if children.len() < 3 {
      break;
    }

    let xop = &children[0];
    if !doc.is_qname(xop, "ltx:XMTok") {
      break;
    }

    let xrole = xop.get_attribute("role").unwrap_or_default();
    let is_script_op = xrole.contains("SUPERSCRIPTOP") || xrole.contains("SUBSCRIPTOP");
    if !is_script_op {
      break;
    }

    let xbase = &children[1];
    let xscript = &children[2];
    let xpos_str = xop
      .get_attribute("scriptpos")
      .unwrap_or_else(|| "post0".to_string());
    let xpos = if xpos_str.starts_with("pre") {
      "pre"
    } else if xpos_str.starts_with("mid") {
      "mid"
    } else {
      "post"
    };
    let x_is_sub = xrole.contains("SUB");
    let xpair = if x_is_sub {
      (Some(xscript.clone()), None)
    } else {
      (None, Some(xscript.clone()))
    };

    match xpos {
      "pre" => {
        // Try to merge with existing pre pair
        if let Some(last) = pre_scripts.last_mut() {
          let slot = if x_is_sub { &mut last.0 } else { &mut last.1 };
          if slot.is_none() {
            *slot = if x_is_sub { xpair.0 } else { xpair.1 };
          } else {
            pre_scripts.push(xpair);
          }
        } else {
          pre_scripts.push(xpair);
        }
      },
      "mid" => {
        if let Some(first) = mid_scripts.first_mut() {
          let slot = if x_is_sub { &mut first.0 } else { &mut first.1 };
          if slot.is_none() {
            *slot = if x_is_sub { xpair.0 } else { xpair.1 };
          } else {
            mid_scripts.insert(0, xpair);
          }
        } else {
          mid_scripts.push(xpair);
        }
      },
      _ => {
        if let Some(first) = post_scripts.first_mut() {
          let slot = if x_is_sub { &mut first.0 } else { &mut first.1 };
          if slot.is_none() {
            *slot = if x_is_sub { xpair.0 } else { xpair.1 };
          } else {
            post_scripts.insert(0, xpair);
          }
        } else {
          post_scripts.push(xpair);
        }
      },
    }

    current_base = xbase.clone();
  }

  (current_base, pre_scripts, mid_scripts, post_scripts)
}

/// Apply mid scripts (under/over) to a base.
fn apply_mid_scripts(
  doc: &PostDocument,
  mut base: NodeData,
  mid_scripts: &[ScriptPair],
) -> NodeData {
  for (sub_opt, sup_opt) in mid_scripts {
    let under = sub_opt.as_ref().map(|s| pmml_scriptsize(doc, s));
    let over = sup_opt.as_ref().map(|s| pmml_scriptsize(doc, s));

    base = match (under, over) {
      (Some(u), None) => NodeData::Element {
        tag:        "m:munder".to_string(),
        attributes: None,
        children:   vec![base, u],
      },
      (None, Some(o)) => NodeData::Element {
        tag:        "m:mover".to_string(),
        attributes: None,
        children:   vec![base, o],
      },
      (Some(u), Some(o)) => NodeData::Element {
        tag:        "m:munderover".to_string(),
        attributes: None,
        children:   vec![base, u, o],
      },
      (None, None) => base,
    };
  }
  base
}

/// Apply pre/post scripts to a base.
///
/// Port of `pmml_script_multi_layout`.
fn apply_multi_scripts(
  doc: &PostDocument,
  base: NodeData,
  pre_scripts: &[ScriptPair],
  post_scripts: &[ScriptPair],
) -> NodeData {
  let none_mml = || NodeData::Element {
    tag:        "m:none".to_string(),
    attributes: None,
    children:   vec![],
  };

  if !pre_scripts.is_empty() {
    // mmultiscripts with prescripts
    let mut children = vec![base];
    for (sub_opt, sup_opt) in post_scripts {
      children.push(
        sub_opt
          .as_ref()
          .map(|s| pmml_scriptsize(doc, s))
          .unwrap_or_else(none_mml),
      );
      children.push(
        sup_opt
          .as_ref()
          .map(|s| pmml_scriptsize(doc, s))
          .unwrap_or_else(none_mml),
      );
    }
    children.push(NodeData::Element {
      tag:        "m:mprescripts".to_string(),
      attributes: None,
      children:   vec![],
    });
    for (sub_opt, sup_opt) in pre_scripts {
      children.push(
        sub_opt
          .as_ref()
          .map(|s| pmml_scriptsize(doc, s))
          .unwrap_or_else(none_mml),
      );
      children.push(
        sup_opt
          .as_ref()
          .map(|s| pmml_scriptsize(doc, s))
          .unwrap_or_else(none_mml),
      );
    }
    NodeData::Element {
      tag: "m:mmultiscripts".to_string(),
      attributes: None,
      children,
    }
  } else if post_scripts.len() > 1 {
    // mmultiscripts with multiple postscripts
    let mut children = vec![base];
    for (sub_opt, sup_opt) in post_scripts {
      children.push(
        sub_opt
          .as_ref()
          .map(|s| pmml_scriptsize(doc, s))
          .unwrap_or_else(none_mml),
      );
      children.push(
        sup_opt
          .as_ref()
          .map(|s| pmml_scriptsize(doc, s))
          .unwrap_or_else(none_mml),
      );
    }
    NodeData::Element {
      tag: "m:mmultiscripts".to_string(),
      attributes: None,
      children,
    }
  } else if post_scripts.is_empty() {
    base
  } else {
    // Single post script pair
    let (sub_opt, sup_opt) = &post_scripts[0];
    match (sub_opt, sup_opt) {
      (Some(sub_node), None) => NodeData::Element {
        tag:        "m:msub".to_string(),
        attributes: None,
        children:   vec![base, pmml_scriptsize(doc, sub_node)],
      },
      (None, Some(sup_node)) => NodeData::Element {
        tag:        "m:msup".to_string(),
        attributes: None,
        children:   vec![base, pmml_scriptsize(doc, sup_node)],
      },
      (Some(sub_node), Some(sup_node)) => NodeData::Element {
        tag:        "m:msubsup".to_string(),
        attributes: None,
        children:   vec![
          base,
          pmml_scriptsize(doc, sub_node),
          pmml_scriptsize(doc, sup_node),
        ],
      },
      (None, None) => base,
    }
  }
}

// ======================================================================
// Continued fractions
//
// Port of `do_cfrac`.

/// Handle continued fraction rendering.
///
/// Port of `Apply:?:continued-fraction` + `do_cfrac`.
fn pmml_cfrac(doc: &PostDocument, _op: &Node, numer: &Node, denom: &Node) -> NodeData {
  // Simplified: just produce mfrac
  // Full version unrolls nested continued fractions and pulls \cdots up
  NodeData::Element {
    tag:        "m:mfrac".to_string(),
    attributes: None,
    children:   vec![pmml_smaller(doc, numer), pmml_smaller(doc, denom)],
  }
}

// ======================================================================
// Utility functions

/// Wrap nodes in an mrow (or return single node unwrapped).
/// Perl MathML.pm `Apply:?:?`: descend through script wrappers (msub/msup/…) to
/// the operator's base and report whether it renders as `<m:mo>`. A generic
/// application inserts FUNCTION APPLICATION (⁡) only when the base is NOT an
/// `<m:mo>`, so an OPERATOR/DIFFOP (∇, ∂) juxtaposes its argument while a
/// function identifier (`f`, `\sin`) gets the invisible apply char.
fn op_base_is_mo(node: &NodeData) -> bool {
  let mut cur = node;
  loop {
    let NodeData::Element { tag, children, .. } = cur else {
      return false;
    };
    if tag == "m:mo" {
      return true;
    }
    // Perl regex `^m:(?:msub|msup|munder|mover|mprescripts)` — a prefix match,
    // so it also covers msubsup/munderover; descend to the base (first child).
    // m:mstyle: the F7 mathstyle wrap (e.g. `\displaystyle\sum`) is transparent
    // embellishment too — Perl's summation never re-examines it (it never
    // emits ⁡ at all, L1796-1798).
    if matches!(
      tag.as_str(),
      "m:msub"
        | "m:msup"
        | "m:msubsup"
        | "m:munder"
        | "m:mover"
        | "m:munderover"
        | "m:mprescripts"
        | "m:mstyle"
    ) {
      match children.first() {
        Some(child) => cur = child,
        None => return false,
      }
    } else {
      return false;
    }
  }
}

/// Port of Perl `filter_row` (L577-579): drop `_ignorable` items.
fn filter_row(items: Vec<NodeData>) -> Vec<NodeData> {
  items
    .into_iter()
    .filter(|i| {
      !matches!(i, NodeData::Element { attributes: Some(a), .. } if a.contains_key("_ignorable"))
    })
    .collect()
}

fn pmml_row(children: Vec<NodeData>) -> NodeData {
  // Perl `pmml_row` (L581-584) filters `_ignorable` items (zero-width hints).
  let children = filter_row(children);
  if children.len() == 1 {
    children.into_iter().next().unwrap()
  } else {
    NodeData::Element {
      tag: "m:mrow".to_string(),
      attributes: None,
      children,
    }
  }
}

/// Create an mo element from a string.
fn pmml_mo_str(text: &str) -> NodeData {
  NodeData::Element {
    tag:        "m:mo".to_string(),
    attributes: None,
    children:   vec![NodeData::Text(text.to_string())],
  }
}

/// Create a MathML error element.
fn pmml_error(msg: &str) -> NodeData {
  NodeData::Element {
    tag:        "m:merror".to_string(),
    attributes: None,
    children:   vec![NodeData::Element {
      tag:        "m:mtext".to_string(),
      attributes: None,
      children:   vec![NodeData::Text(msg.to_string())],
    }],
  }
}

/// Map a LaTeXML font name to a MathML mathvariant value.
///
/// Wrapper around `unicode::unicode_mathvariant` (full Perl parity).
/// Returns `Some(variant)` for recognized fonts, `None` only for empty input.
pub fn font_to_mathvariant(font: &str) -> Option<&'static str> {
  if font.is_empty() {
    return None;
  }
  Some(crate::unicode::unicode_mathvariant(font))
}

// ======================================================================
// TeX spacing adjustment
//
// Port of Perl's `adjust_spacing` / `space_walk` / `adjust_pair`.
// Walks adjacent pairs in mrow and adjusts lspace/rspace to match TeX spacing.

/// TeX spacing values: thin=3mu, med=4mu, thick=5mu (in em = mu/18)
const TEX_SPACING: [f64; 4] = [0.0, 0.167, 0.222, 0.2778];

/// Spacing epsilon — ignore differences below this (em)
const SPACING_EPSILON: f64 = 0.01;

/// Don't complain if we can't adjust less than this (em) — Perl `$fudge`.
const SPACING_FUDGE: f64 = 0.3;

/// Map LaTeXML role to TeX atom type.
fn role_to_atom_type(role: &str) -> &'static str {
  match role {
    "ATOM" | "UNKNOWN" | "ID" | "NUMBER" | "POSTFIX" | "FUNCTION" | "DIFFOP" | "SUPOP"
    | "ELIDEOP" => "Ord",
    "OPFUNCTION" | "TRIGFUNCTION" | "BIGOP" | "SUMOP" | "INTOP" | "LIMITOP" | "OPERATOR" => "Op",
    "ADDOP" | "MULOP" | "BINOP" | "APPLYOP" | "COMPOSEOP" => "Bin",
    "RELOP" | "METARELOP" | "MODIFIEROP" | "MODIFIER" | "ARROW" => "Rel",
    "OPEN" => "Open",
    "CLOSE" => "Close",
    "PUNCT" | "VERTBAR" | "PERIOD" => "Punct",
    "ARRAY" | "POSTSUBSCRIPT" | "POSTSUPERSCRIPT" | "FLOATSUPERSCRIPT" | "FLOATSUBSCRIPT" => {
      "Inner"
    },
    "MIDDLE" => "Ord",
    _ => "Ord",
  }
}

/// Get TeX spacing code for a pair of atom types.
fn atompair_spacing(left: &str, right: &str) -> i32 {
  match (left, right) {
    ("Ord", "Op") | ("Op", "Ord") | ("Op", "Op") | ("Close", "Op") => 1,
    ("Ord", "Bin")
    | ("Bin", "Ord")
    | ("Bin", "Open")
    | ("Bin", "Inner")
    | ("Close", "Bin")
    | ("Inner", "Bin")
    | ("Bin", "Op") => -2,
    ("Ord", "Rel")
    | ("Rel", "Ord")
    | ("Op", "Rel")
    | ("Rel", "Open")
    | ("Rel", "Inner")
    | ("Close", "Rel")
    | ("Inner", "Rel")
    | ("Rel", "Op") => -3,
    ("Ord", "Inner")
    | ("Op", "Inner")
    | ("Close", "Inner")
    | ("Inner", "Inner")
    | ("Inner", "Open")
    | ("Punct", "Ord")
    | ("Punct", "Op")
    | ("Punct", "Rel")
    | ("Punct", "Open")
    | ("Punct", "Close")
    | ("Punct", "Punct")
    | ("Punct", "Inner")
    | ("Inner", "Ord") => -1,
    ("Inner", "Op") => 1,
    _ => 0,
  }
}

/// Map MathML tag to TeX atom type.
fn m_atom_type(tag: &str) -> Option<&'static str> {
  match tag {
    "m:mfrac" => Some("Ord"),
    "m:marray" => Some("Inner"),
    "m:mspace" => Some("Ord"),
    _ => None,
  }
}

/// Check if a MathML tag is an embellished operator container.
fn is_embellisher_tag(tag: &str) -> bool {
  matches!(
    tag,
    "m:msub" | "m:msup" | "m:msubsup" | "m:munder" | "m:mover" | "m:munderover"
  )
}

/// Check if a MathML tag is mrow-like.
fn is_mrow_like(tag: &str) -> bool {
  matches!(
    tag,
    "m:mrow" | "m:mpadded" | "m:msqrt" | "m:mstyle" | "m:merror" | "m:mphantom" | "m:mtd"
  )
}

/// Check if text is an invisible operator.
fn is_invisible_op(text: &str) -> bool {
  !text.is_empty()
    && text
      .chars()
      .all(|c| matches!(c, '\u{200B}' | '\u{2061}' | '\u{2062}' | '\u{2063}'))
}

/// Format em value. Port of Perl `fmt_em` (MathML.pm L1285):
/// `sprintf("%.3fem")` — trailing zeros are KEPT ("0.330em", "1.200em"),
/// matching Perl byte-for-byte; zero (Perl false-y) → "0em". (audit F4)
fn fmt_em(val: f64) -> String {
  if val == 0.0 {
    "0em".to_string()
  } else {
    format!("{val:.3}em")
  }
}

/// Get role from a NodeData, following embellished operators.
fn get_node_role(node: &NodeData) -> String {
  match node {
    NodeData::Element { tag, attributes, children } => {
      if is_embellisher_tag(tag) {
        if let Some(base) = children.first() {
          return get_node_role(base);
        }
      }
      attributes
        .as_ref()
        .and_then(|a| a.get("_role"))
        .cloned()
        .unwrap_or_default()
    },
    _ => String::new(),
  }
}

fn get_node_tag(node: &NodeData) -> &str {
  match node {
    NodeData::Element { tag, .. } => tag,
    _ => "",
  }
}

fn get_node_text(node: &NodeData) -> String {
  match node {
    NodeData::Text(t) => t.clone(),
    NodeData::Element { children, .. } => children.iter().map(get_node_text).collect(),
    _ => String::new(),
  }
}

/// Check if all text in `node` consists only of invisible-op characters,
/// without allocating a concatenated String like `get_node_text` does.
/// Used by `adjust_spacing` where we only need the boolean answer.
fn is_node_text_invisible_op(node: &NodeData) -> bool {
  fn check(node: &NodeData, seen_any: &mut bool) -> bool {
    match node {
      NodeData::Text(t) => {
        if !t.is_empty() {
          *seen_any = true;
        }
        t.chars()
          .all(|c| matches!(c, '\u{200B}' | '\u{2061}' | '\u{2062}' | '\u{2063}'))
      },
      NodeData::Element { children, .. } => children.iter().all(|c| check(c, seen_any)),
      _ => true,
    }
  }
  let mut seen_any = false;
  let ok = check(node, &mut seen_any);
  ok && seen_any
}

fn set_node_attr(node: &mut NodeData, key: &str, value: &str) {
  if let NodeData::Element { attributes, .. } = node {
    let attrs = attributes.get_or_insert_with(HashMap::default);
    attrs.insert(key.to_string(), value.to_string());
  }
}

fn get_node_attr(node: &NodeData, key: &str) -> Option<String> {
  match node {
    NodeData::Element { attributes, .. } => attributes.as_ref().and_then(|a| a.get(key)).cloned(),
    _ => None,
  }
}

fn get_node_attr_f64(node: &NodeData, key: &str) -> f64 {
  match node {
    NodeData::Element { attributes, .. } => attributes
      .as_ref()
      .and_then(|a| a.get(key))
      .and_then(|v| v.strip_suffix("em"))
      .and_then(|v| v.parse::<f64>().ok())
      .unwrap_or(0.0),
    _ => 0.0,
  }
}

/// Perl `%tag_arg_pattern` (MathML.pm L1084-1088): how a tag participates in
/// the spacing walk.
#[derive(PartialEq, Clone, Copy)]
enum WalkType {
  Atom,
  Mrow,
  Other,
}
fn walk_type(tag: &str) -> WalkType {
  match tag {
    "m:mi" | "m:mo" | "m:mn" | "m:ms" | "m:mtext" => WalkType::Atom,
    "m:mrow" | "m:mpadded" | "m:msqrt" | "m:mstyle" | "m:merror" | "m:mphantom" | "m:mtd" => {
      WalkType::Mrow
    },
    _ => WalkType::Other,
  }
}

/// Resolve a child-index path from the walk root. The walk mutates only
/// attributes and rewraps nodes in place (sibling indices stay stable), so
/// queued paths never dangle.
fn node_at<'a>(root: &'a NodeData, path: &[usize]) -> &'a NodeData {
  let mut cur = root;
  for &i in path {
    match cur {
      NodeData::Element { children, .. } => cur = &children[i],
      _ => unreachable!("spacewalk path into non-element"),
    }
  }
  cur
}

fn node_at_mut<'a>(root: &'a mut NodeData, path: &[usize]) -> &'a mut NodeData {
  let mut cur = root;
  for &i in path {
    match cur {
      NodeData::Element { children, .. } => cur = &mut children[i],
      _ => unreachable!("spacewalk path into non-element"),
    }
  }
  cur
}

fn child_path(path: &[usize], i: usize) -> Vec<usize> {
  let mut p = path.to_vec();
  p.push(i);
  p
}

/// Descend through embellished operators (scripts) to the inner operator,
/// for role/opdict-spacing reads (Perl adjust_pair L1225-1227).
fn descend_embellishers(root: &NodeData, mut path: Vec<usize>) -> Vec<usize> {
  loop {
    match node_at(root, &path) {
      NodeData::Element { tag, children, .. }
        if is_embellisher_tag(tag) && !children.is_empty() =>
      {
        path.push(0);
      },
      _ => return path,
    }
  }
}

/// Walk the MathML tree and adjust spacing. Port of Perl `adjust_spacing` /
/// `space_walk` (MathML.pm L1079-1133): resolves the difference between
/// TeX's inter-atom spacing and MathML's operator-dictionary spacing.
pub fn adjust_spacing(node: &mut NodeData) { space_walk(node, Vec::new()); }

/// Port of Perl `space_walk`: pairs VISUALLY adjacent items by unwinding
/// nested mrows into the pair stream, streaming script bases (TeX attaches
/// scripts without affecting inter-atom spacing) while recursing on the
/// scripts themselves, and carrying invisible operators between a pair as
/// the preferred place to materialize an adjustment.
fn space_walk(root: &mut NodeData, path: Vec<usize>) {
  use std::collections::VecDeque;
  let (wt, nch) = match node_at(root, &path) {
    NodeData::Element { tag, children, .. } => (walk_type(tag), children.len()),
    _ => return,
  };
  match wt {
    WalkType::Atom => {},
    WalkType::Other => {
      for i in 0..nch {
        space_walk(root, child_path(&path, i));
      }
    },
    WalkType::Mrow => {
      let mut queue: VecDeque<Vec<usize>> = (0..nch).map(|i| child_path(&path, i)).collect();
      // First prev: unwrap leading nested mrows (Perl L1105-1108).
      let mut first = None;
      while let Some(p) = queue.pop_front() {
        let unwrap = match node_at(root, &p) {
          NodeData::Element { tag, children, .. } if tag == "m:mrow" => Some(children.len()),
          _ => None,
        };
        match unwrap {
          Some(n) => {
            for i in (0..n).rev() {
              queue.push_front(child_path(&p, i));
            }
          },
          None => {
            first = Some(p);
            break;
          },
        }
      }
      let Some(mut prev) = first else { return };
      space_walk(root, prev.clone());
      while let Some(popped) = queue.pop_front() {
        let mut next = popped;
        // Save an invisible operator as the potential target for lspace
        // (Perl L1111-1114).
        let mut invisop: Option<Vec<usize>> = None;
        {
          let n = node_at(root, &next);
          if get_node_tag(n) == "m:mo" && is_node_text_invisible_op(n) {
            invisop = Some(next);
            match queue.pop_front() {
              Some(p) => next = p,
              None => break,
            }
          }
        }
        enum Kind {
          Mrow(usize),
          Script(usize),
          Plain,
        }
        let kind = match node_at(root, &next) {
          NodeData::Element { tag, children, .. } => {
            if tag == "m:mrow" {
              Kind::Mrow(children.len())
            } else if tag.starts_with("m:msup")
              || tag.starts_with("m:msub")
              || tag.starts_with("m:munder")
              || tag.starts_with("m:mover")
              || tag.starts_with("m:mmultiscripts")
            {
              // Prefix match like Perl's regex — covers msubsup/munderover.
              Kind::Script(children.len())
            } else {
              Kind::Plain
            }
          },
          _ => Kind::Plain,
        };
        match kind {
          Kind::Mrow(n) => {
            // Unwrap into the stream; the invisible op goes back in front.
            for i in (0..n).rev() {
              queue.push_front(child_path(&next, i));
            }
            if let Some(iv) = invisop {
              queue.push_front(iv);
            }
            continue;
          },
          Kind::Script(n) => {
            // Stream the base; recurse on the scripts (Perl L1121-1128).
            for i in 1..n {
              space_walk(root, child_path(&next, i));
            }
            queue.push_front(child_path(&next, 0));
            if let Some(iv) = invisop {
              queue.push_front(iv);
            }
            continue;
          },
          Kind::Plain => {},
        }
        space_walk(root, next.clone());
        adjust_pair(root, &prev, &next, invisop.as_deref());
        prev = next;
      }
    },
  }
}

/// Adjust the spacing between a visually adjacent pair. Port of Perl
/// `adjust_pair` (MathML.pm L1220-1284), all branches.
fn adjust_pair(root: &mut NodeData, prev: &[usize], next: &[usize], invisop: Option<&[usize]>) {
  let iprev = descend_embellishers(root, prev.to_vec());
  let inext = descend_embellishers(root, next.to_vec());

  // Author spacing (in em) reads from the OUTER pair; role/opdict spacing
  // from the inner (possibly embellished) operator.
  let prev_req_right = get_node_attr_f64(node_at(root, prev), "_rpadding");
  let next_req_left = get_node_attr_f64(node_at(root, next), "_lpadding");
  let (iprev_tag, prev_role, prev_dict_right) = {
    let n = node_at(root, &iprev);
    (
      get_node_tag(n).to_string(),
      get_node_attr(n, "_role").unwrap_or_else(|| "ATOM".to_string()),
      get_node_attr_f64(n, "_rspace"),
    )
  };
  let (inext_tag, next_role, next_dict_left) = {
    let n = node_at(root, &inext);
    (
      get_node_tag(n).to_string(),
      get_node_attr(n, "_role").unwrap_or_else(|| "ATOM".to_string()),
      get_node_attr_f64(n, "_lspace"),
    )
  };
  let prev_type = m_atom_type(&iprev_tag).unwrap_or_else(|| role_to_atom_type(&prev_role));
  let next_type = m_atom_type(&inext_tag).unwrap_or_else(|| role_to_atom_type(&next_role));
  let tex_code = atompair_spacing(prev_type, next_type);
  let tex_space = TEX_SPACING[tex_code.unsigned_abs() as usize];
  let target = prev_req_right + next_req_left + tex_space;
  let default = prev_dict_right + next_dict_left;
  if (target - default).abs() <= SPACING_EPSILON {
    return;
  }

  let prev_tag = get_node_tag(node_at(root, prev)).to_string();
  let next_tag = get_node_tag(node_at(root, next)).to_string();
  // In MathML Core neither mspace nor mpadded may have negative width, and
  // relative +/- widths are unsupported — so a NEGATIVE target rewraps prev
  // in an m:mpadded with an ADJUSTED absolute width (Perl L1252-1260,
  // compute_size L1135-1145: atoms only, string metrics of the default math
  // font, with the ridiculous-but-Perl minimum-10pt hack for mathscript).
  if target < 0.0 {
    let sizeable = match node_at(root, prev) {
      NodeData::Element { tag, attributes, .. } if walk_type(tag) == WalkType::Atom => Some(
        attributes
          .as_ref()
          .and_then(|a| a.get("class"))
          .cloned()
          .unwrap_or_default(),
      ),
      _ => None,
    };
    if let Some(class) = sizeable {
      let text = get_node_text(node_at(root, prev));
      let font = latexml_core::common::font::Font::math_default();
      let (w, _h, _d) = font.compute_string_size(&text, Default::default());
      let mut w_sp = w.0;
      if w_sp != 0 && class.contains("mathscript") {
        w_sp = w_sp.max(10 * 65536);
      }
      let mut reqw = (w_sp as f64 / 65536.0) / 10.0 + target;
      if reqw < 0.0 {
        reqw = 0.0;
      }
      let slot = node_at_mut(root, prev);
      let old = std::mem::replace(slot, NodeData::Text(String::new()));
      *slot = NodeData::Element {
        tag:        "m:mpadded".to_string(),
        attributes: Some(HashMap::from_iter([("width".to_string(), fmt_em(reqw))])),
        children:   vec![old],
      };
    }
  } else if prev_tag == "m:mspace" || next_tag == "m:mspace" {
    // Merge into the mspace's existing width (Perl L1261-1262).
    let target_path = if prev_tag == "m:mspace" { prev } else { next };
    let n = node_at_mut(root, target_path);
    let old_w = match n {
      NodeData::Element { attributes, .. } => attributes
        .as_ref()
        .and_then(|a| a.get("width"))
        .map(|w| super::get_xm_hint_spacing(w))
        .unwrap_or(0.0),
      _ => 0.0,
    };
    set_node_attr(n, "width", &fmt_em(target + old_w));
  } else if let Some(iv) = invisop {
    set_node_attr(node_at_mut(root, iv), "lspace", &fmt_em(target));
  } else if prev_tag == "m:mo" && next_tag == "m:mo" {
    // BOTH are mo: account for each one's dictionary spacing (Perl L1264-1275).
    let p = prev_dict_right;
    let n = next_dict_left;
    let rem = target - n;
    if rem >= 0.0 {
      let v = if rem > SPACING_EPSILON { rem } else { 0.0 };
      set_node_attr(node_at_mut(root, prev), "rspace", &fmt_em(v));
    } else {
      let rem = target - p;
      if rem >= 0.0 {
        let v = if rem > SPACING_EPSILON { rem } else { 0.0 };
        set_node_attr(node_at_mut(root, next), "lspace", &fmt_em(v));
      } else {
        // Split the difference; Perl concatenates the raw number here
        // (`$rem . 'em'`), NOT fmt_em.
        let rem = target / 2.0;
        if rem != p {
          set_node_attr(
            node_at_mut(root, prev),
            "rspace",
            &format!("{}em", perl_num(rem)),
          );
        }
        if rem != n {
          set_node_attr(
            node_at_mut(root, next),
            "lspace",
            &format!("{}em", perl_num(rem)),
          );
        }
      }
    }
  } else if prev_tag == "m:mo" {
    set_node_attr(node_at_mut(root, prev), "rspace", &fmt_em(target));
  } else if next_tag == "m:mo" {
    set_node_attr(node_at_mut(root, next), "lspace", &fmt_em(target));
  } else if (target - default).abs() > SPACING_FUDGE {
    Info!(
      "ignored",
      "spacing",
      "No place to set spacing to {target} (default {default})"
    );
  }
}

/// Clean up internal _role/_lspace/_rspace attributes before serialization.
pub fn clean_internal_attrs(node: &mut NodeData) {
  if let NodeData::Element { attributes, children, .. } = node {
    if let Some(attrs) = attributes {
      attrs.remove("_role");
      attrs.remove("_lspace");
      attrs.remove("_rspace");
      attrs.remove("_largeop");
      attrs.remove("_lpadding");
      attrs.remove("_rpadding");
      attrs.remove("_ignorable");
      if attrs.is_empty() {
        *attributes = None;
      }
    }
    for child in children {
      clean_internal_attrs(child);
    }
  }
}

#[cfg(test)]
mod tests {
  use rustc_hash::FxHashMap as HashMap;

  use super::*;

  #[test]
  fn math_style_step_down_monotone_saturates_at_scriptscript() {
    assert_eq!(MathStyle::Display.step_down(), MathStyle::Text);
    assert_eq!(MathStyle::Text.step_down(), MathStyle::Script);
    assert_eq!(MathStyle::Script.step_down(), MathStyle::ScriptScript);
    assert_eq!(MathStyle::ScriptScript.step_down(), MathStyle::ScriptScript);
  }

  #[test]
  fn math_style_script_step_collapses_display_and_text() {
    assert_eq!(MathStyle::Display.script_step(), MathStyle::Script);
    assert_eq!(MathStyle::Text.script_step(), MathStyle::Script);
    assert_eq!(MathStyle::Script.script_step(), MathStyle::ScriptScript);
    assert_eq!(
      MathStyle::ScriptScript.script_step(),
      MathStyle::ScriptScript
    );
  }

  #[test]
  fn math_style_size_percent_matches_tex_tradition() {
    assert_eq!(MathStyle::Display.size_percent(), "100%");
    assert_eq!(MathStyle::Text.size_percent(), "100%");
    assert_eq!(MathStyle::Script.size_percent(), "70%");
    assert_eq!(MathStyle::ScriptScript.size_percent(), "50%");
  }

  #[test]
  fn invisible_times_roundtrip() {
    set_invisible_times(false);
    assert!(!get_invisible_times());
    set_invisible_times(true);
    assert!(get_invisible_times());
  }

  #[test]
  fn embellishing_role_matches_canonical_set() {
    for r in [
      "SUPERSCRIPTOP",
      "SUBSCRIPTOP",
      "OVERACCENT",
      "UNDERACCENT",
      "MODIFIER",
      "MODIFIEROP",
    ] {
      assert!(is_embellishing_role(r), "{} should embellish", r);
    }
  }

  #[test]
  fn embellishing_role_rejects_others() {
    for r in ["ADDOP", "MULOP", "ATOM", "UNKNOWN", ""] {
      assert!(!is_embellishing_role(r), "{} should not embellish", r);
    }
  }

  #[test]
  fn default_token_content_maps_invisible_chars() {
    assert_eq!(default_token_content("MULOP"), Some("\u{2062}"));
    assert_eq!(default_token_content("ADDOP"), Some("\u{2064}"));
    assert_eq!(default_token_content("PUNCT"), Some("\u{2063}"));
  }

  #[test]
  fn default_token_content_none_for_other_roles() {
    assert_eq!(default_token_content("ATOM"), None);
    assert_eq!(default_token_content(""), None);
    assert_eq!(default_token_content("RELOP"), None);
  }

  #[test]
  fn clean_internal_attrs_removes_underscore_attrs() {
    let mut node = NodeData::Element {
      tag:        "mrow".to_string(),
      attributes: Some(HashMap::from_iter([
        ("_role".to_string(), "MULOP".to_string()),
        ("_lspace".to_string(), "4".to_string()),
        ("keep".to_string(), "yes".to_string()),
      ])),
      children:   vec![],
    };
    clean_internal_attrs(&mut node);
    if let NodeData::Element { attributes, .. } = &node {
      let attrs = attributes
        .as_ref()
        .expect("still has the non-internal attr");
      assert_eq!(attrs.len(), 1);
      assert_eq!(attrs.get("keep").map(String::as_str), Some("yes"));
    } else {
      panic!("expected element");
    }
  }

  #[test]
  fn clean_internal_attrs_unsets_attributes_when_empty() {
    let mut node = NodeData::Element {
      tag:        "mrow".to_string(),
      attributes: Some(HashMap::from_iter([
        ("_role".to_string(), "MULOP".to_string()),
        ("_largeop".to_string(), "true".to_string()),
      ])),
      children:   vec![],
    };
    clean_internal_attrs(&mut node);
    if let NodeData::Element { attributes, .. } = &node {
      // All attrs were internal → attributes becomes None.
      assert!(attributes.is_none());
    } else {
      panic!("expected element");
    }
  }

  #[test]
  fn clean_internal_attrs_recurses_into_children() {
    let mut node = NodeData::Element {
      tag:        "mrow".to_string(),
      attributes: None,
      children:   vec![NodeData::Element {
        tag:        "mi".to_string(),
        attributes: Some(HashMap::from_iter([(
          "_rspace".to_string(),
          "1".to_string(),
        )])),
        children:   vec![],
      }],
    };
    clean_internal_attrs(&mut node);
    if let NodeData::Element { children, .. } = &node {
      if let NodeData::Element { attributes, .. } = &children[0] {
        assert!(attributes.is_none(), "recursion cleared child's only attr");
      } else {
        panic!("expected element child");
      }
    } else {
      panic!("expected element root");
    }
  }

  #[test]
  fn clean_internal_attrs_ignores_text_nodes() {
    let mut node = NodeData::Text("x".to_string());
    clean_internal_attrs(&mut node);
    match &node {
      NodeData::Text(s) => assert_eq!(s, "x"),
      _ => panic!("expected text untouched"),
    }
  }
  #[test]
  fn test_role_to_atom_type() {
    // Perl MathML.pm $role_atomtype (L1150)
    assert_eq!(role_to_atom_type("ID"), "Ord");
    assert_eq!(role_to_atom_type("NUMBER"), "Ord");
    assert_eq!(role_to_atom_type("ADDOP"), "Bin");
    assert_eq!(role_to_atom_type("RELOP"), "Rel");
    assert_eq!(role_to_atom_type("OPEN"), "Open");
    assert_eq!(role_to_atom_type("CLOSE"), "Close");
    assert_eq!(role_to_atom_type("SUMOP"), "Op");
    assert_eq!(role_to_atom_type("PUNCT"), "Punct");
    assert_eq!(role_to_atom_type("ARRAY"), "Inner");
    assert_eq!(role_to_atom_type("no-such-role"), "Ord");
  }

  #[test]
  fn test_atompair_spacing() {
    // Perl MathML.pm $atompair_spacing (L1196): negative = display/text-style only
    assert_eq!(atompair_spacing("Ord", "Op"), 1);
    assert_eq!(atompair_spacing("Ord", "Bin"), -2);
    assert_eq!(atompair_spacing("Rel", "Ord"), -3);
    assert_eq!(atompair_spacing("Open", "Ord"), 0);
    assert_eq!(atompair_spacing("Open", "Open"), 0);
    assert_eq!(atompair_spacing("Punct", "Bin"), 0);
    assert_eq!(atompair_spacing("Inner", "Op"), 1);
    assert_eq!(atompair_spacing("Inner", "Close"), 0);
  }

  #[test]
  fn test_fmt_em() {
    // Perl fmt_em (L1285) byte-parity: %.3f keeps trailing zeros (audit F4).
    assert_eq!(fmt_em(0.0), "0em");
    assert_eq!(fmt_em(1.0), "1.000em");
    assert_eq!(fmt_em(0.167), "0.167em");
    assert_eq!(fmt_em(0.33), "0.330em");
    assert_eq!(fmt_em(1.2), "1.200em");
  }
}
