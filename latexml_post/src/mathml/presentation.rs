//! Presentation MathML rendering rules.
//!
//! Port of `LaTeXML::Post::MathML::Presentation` (146 lines)
//! + the presentation portion of `LaTeXML::Post::MathML` (main module, ~1000 lines).
//! Converts XMath nodes to Presentation MathML elements (mi, mo, mn, mrow, etc.).
//!
//! Key concepts:
//! - `pmml(node)` dispatches conversion by tag (XMTok, XMApp, XMDual, etc.)
//! - `stylizeContent(item, tag)` determines text, mathvariant, size, spacing
//! - Scripts (sub/sup/under/over) handle pre/mid/post positioning
//! - Style context tracks display/text/script/scriptscript levels

use libxml::tree::Node;
use std::collections::HashMap;

use crate::document::{element_children, NodeData, PostDocument};

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
    "MULOP" => Some("\u{2062}"),  // INVISIBLE TIMES
    "ADDOP" => Some("\u{2064}"),  // INVISIBLE PLUS
    "PUNCT" => Some("\u{2063}"),  // INVISIBLE SEPARATOR
    _ => None,
  }
}

/// Get the operator role, following embellished operators.
fn get_operator_role(doc: &PostDocument, node: &Node) -> Option<String> {
  if let Some(role) = node.get_attribute("role") {
    return Some(role);
  }
  if doc.get_qname(node).as_deref() == Some("ltx:XMApp") {
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
  let children = element_children(xmath);
  let results: Vec<NodeData> = children.iter().map(|c| pmml(doc, c)).collect();
  if results.len() == 1 {
    results.into_iter().next().unwrap()
  } else {
    pmml_row(results)
  }
}

/// Core dispatch: convert a single XMath node to Presentation MathML.
///
/// Port of `pmml` + `pmml_internal`.
fn pmml(doc: &PostDocument, node: &Node) -> NodeData {
  let tag = doc.get_qname(node).unwrap_or_default();

  // Follow XMRef
  if tag == "ltx:XMRef" {
    if let Some(idref) = node.get_attribute("idref") {
      if let Some(target) = doc.find_node_by_id(&idref) {
        return pmml(doc, target);
      }
    }
    return pmml_error("Unresolved XMRef");
  }

  match tag.as_str() {
    "ltx:XMath" => {
      let results: Vec<NodeData> = element_children(node).iter().map(|c| pmml(doc, c)).collect();
      pmml_row(results)
    }
    "ltx:XMDual" => {
      let children = element_children(node);
      if children.len() >= 2 {
        pmml(doc, &children[1]) // Presentation branch
      } else {
        pmml_error("Empty XMDual")
      }
    }
    "ltx:XMWrap" | "ltx:XMArg" => {
      let results: Vec<NodeData> = element_children(node).iter().map(|c| pmml(doc, c)).collect();
      pmml_row(results)
    }
    "ltx:XMApp" => pmml_apply(doc, node),
    "ltx:XMTok" => pmml_token(doc, node),
    "ltx:XMHint" => pmml_hint(doc, node),
    "ltx:XMArray" => pmml_array(doc, node),
    "ltx:XMText" => {
      let text = node.get_content();
      NodeData::Element {
        tag: "m:mtext".to_string(),
        attributes: None,
        children: vec![NodeData::Text(text)],
      }
    }
    _ => {
      NodeData::Element {
        tag: "m:mtext".to_string(),
        attributes: None,
        children: vec![NodeData::Text(node.get_content())],
      }
    }
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

  // Handle floating/post scripts
  if role.contains("SUBSCRIPT") || role.contains("SUPERSCRIPT") {
    let is_sub = role.contains("SUB");
    let tag = if is_sub { "m:msub" } else { "m:msup" };
    return NodeData::Element {
      tag: tag.to_string(),
      attributes: None,
      children: vec![
        NodeData::Element { tag: "m:mi".to_string(), attributes: None, children: vec![] },
        pmml_scriptsize(doc, &children[0]),
      ],
    };
  }

  let op = &children[0];
  let args = &children[1..];

  // Realize the operator
  let rop = if doc.get_qname(op).as_deref() == Some("ltx:XMRef") {
    op.get_attribute("idref")
      .and_then(|id| doc.find_node_by_id(&id).cloned())
      .unwrap_or_else(|| op.clone())
  } else {
    op.clone()
  };

  let op_role = get_operator_role(doc, &rop).unwrap_or_default();
  let meaning = rop.get_attribute("meaning").unwrap_or_default();

  // Dispatch by role
  match op_role.as_str() {
    "SUPERSCRIPTOP" | "SUBSCRIPTOP" if args.len() >= 2 => {
      pmml_script_full(doc, op, &args[0], &args[1])
    }
    "FRACOP" if args.len() >= 2 => {
      let thickness = rop.get_attribute("thickness");
      let mut attrs = HashMap::new();
      if let Some(ref t) = thickness {
        if t == "0" || t == "0pt" {
          attrs.insert("linethickness".to_string(), "0".to_string());
        }
      }
      NodeData::Element {
        tag: "m:mfrac".to_string(),
        attributes: if attrs.is_empty() { None } else { Some(attrs) },
        children: vec![
          pmml_smaller(doc, &args[0]),
          pmml_smaller(doc, &args[1]),
        ],
      }
    }
    "OVERACCENT" if !args.is_empty() => {
      NodeData::Element {
        tag: "m:mover".to_string(),
        attributes: Some(HashMap::from([("accent".to_string(), "true".to_string())])),
        children: vec![pmml(doc, &args[0]), pmml(doc, op)],
      }
    }
    "UNDERACCENT" if !args.is_empty() => {
      NodeData::Element {
        tag: "m:munder".to_string(),
        attributes: Some(HashMap::from([("accentunder".to_string(), "true".to_string())])),
        children: vec![pmml(doc, &args[0]), pmml(doc, op)],
      }
    }
    "POSTFIX" if !args.is_empty() => {
      let mut items: Vec<NodeData> = args.iter().map(|a| pmml(doc, a)).collect();
      items.push(pmml(doc, op));
      pmml_row(items)
    }
    "ADDOP" | "RELOP" | "MULOP" | "BINOP" | "ARROW" | "METARELOP"
    | "COMPOSEOP" | "MODIFIEROP" | "MIDDLE" => {
      // Infix: arg1 op arg2 op arg3 ...
      pmml_infix(doc, op, args)
    }
    "SUMOP" | "INTOP" | "BIGOP" | "LIMITOP" | "DIFFOP" => {
      // Big operator: Σ/∫ applied to args
      pmml_summation(doc, op, args)
    }
    "OPEN" | "CLOSE" | "ENCLOSE" if !args.is_empty() => {
      // Fenced: (args)
      pmml_parenthesize(doc, op, args)
    }
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
    }
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
            tag: "m:mspace".to_string(),
            attributes: Some(HashMap::from([("width".to_string(), "0.389em".to_string())])),
            children: vec![],
          },
          pmml(doc, &args[1]),
        ])
      } else if meaning == "square-root" && !args.is_empty() {
        NodeData::Element {
          tag: "m:msqrt".to_string(),
          attributes: None,
          children: vec![pmml(doc, &args[0])],
        }
      } else if meaning == "continued-fraction" && args.len() >= 2 {
        pmml_cfrac(doc, op, &args[0], &args[1])
      } else if meaning == "nth-root" && args.len() >= 2 {
        NodeData::Element {
          tag: "m:mroot".to_string(),
          attributes: None,
          children: vec![pmml(doc, &args[0]), pmml_scriptsize(doc, &args[1])],
        }
      } else {
        // Generic application: op(arg1, arg2, ...)
        let mut items = vec![pmml(doc, op)];
        items.push(pmml_mo_str("\u{2061}")); // FUNCTION APPLICATION
        for arg in args {
          items.push(pmml(doc, arg));
        }
        pmml_row(items)
      }
    }
  }
}

/// Convert an XMTok to the appropriate Presentation MathML token.
///
/// Port of `stylizeContent` + token converter.
fn pmml_token(_doc: &PostDocument, node: &Node) -> NodeData {
  let role = node.get_attribute("role").unwrap_or_else(|| "UNKNOWN".to_string());
  let font = node.get_attribute("font");
  let mut text = node.get_content();
  let meaning = node.get_attribute("meaning");

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
      text = meaning.clone()
        .or_else(|| node.get_attribute("name"))
        .unwrap_or_else(|| role.clone());
    }
  }

  // Minus sign normalization
  if text == "-" && matches!(role.as_str(), "ADDOP" | "OPERATOR") {
    text = "\u{2212}".to_string(); // MINUS SIGN
  }

  let mut attrs = HashMap::new();

  // Math variant from font
  if let Some(ref f) = font {
    if let Some(variant) = font_to_mathvariant(f) {
      // Single char in mi with italic → omit (it's the default)
      if tag == "m:mi" && text.chars().count() == 1 {
        if variant != "italic" {
          attrs.insert("mathvariant".to_string(), variant.to_string());
        }
        // else italic is default for single-char mi
      } else if variant != "normal" {
        attrs.insert("mathvariant".to_string(), variant.to_string());
      }
    }
  } else if tag == "m:mi" && text.chars().count() == 1 {
    // Single char mi without font → check if it's a named symbol (not a variable).
    // Named symbols (e.g., \Box → □) should use mathvariant="normal".
    if node.get_attribute("name").is_some() {
      attrs.insert("mathvariant".to_string(), "normal".to_string());
    }
    // Regular letters default to italic (MathML default for single-char mi)
  } else if tag == "m:mi" && text.chars().count() > 1 {
    // Multi-char mi without font → should be normal (function name)
    attrs.insert("mathvariant".to_string(), "normal".to_string());
  }

  // Operator-specific attributes
  if tag == "m:mo" {
    // Check the XMTok's stretchy attribute
    let stretchy_attr = node.get_attribute("stretchy");

    // Handle size attribute
    if let Some(size) = node.get_attribute("fontsize") {
      if size != "100%" {
        attrs.insert("mathsize".to_string(), size);
      }
    }

    // Copy stretchy attribute from source XMTok only when it differs from MathML defaults.
    // Port of Perl's `($stretchy xor $props{stretchy}) ? (stretchy => ...) : ()`.
    // In MathML, OPEN/CLOSE/MIDDLE operators are stretchy by default.
    // We only need to emit stretchy="false" for fences (to override default),
    // or stretchy="true" for non-fence operators.
    let is_fence = matches!(role.as_str(), "OPEN" | "CLOSE" | "MIDDLE");
    if let Some(ref stretchy) = stretchy_attr {
      if is_fence && stretchy == "false" {
        attrs.insert("stretchy".to_string(), "false".to_string());
      } else if !is_fence && stretchy == "true" {
        attrs.insert("stretchy".to_string(), "true".to_string());
      }
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

  NodeData::Element {
    tag: tag.to_string(),
    attributes: if attrs.is_empty() { None } else { Some(attrs) },
    children: vec![NodeData::Text(text)],
  }
}

/// Convert an XMHint to MathML.
///
/// Port of Hint handler.
fn pmml_hint(_doc: &PostDocument, node: &Node) -> NodeData {
  let width = node.get_attribute("width");
  if let Some(w) = width {
    // Convert width to mspace
    NodeData::Element {
      tag: "m:mspace".to_string(),
      attributes: Some(HashMap::from([("width".to_string(), w)])),
      children: vec![],
    }
  } else {
    // Empty hint
    NodeData::Text(String::new())
  }
}

/// Convert an XMArray to an mtable.
///
/// Port of `pmml_internal` XMArray branch.
fn pmml_array(doc: &PostDocument, node: &Node) -> NodeData {
  let mut rows = Vec::new();
  let vattach = node.get_attribute("vattach").unwrap_or_else(|| "middle".to_string());
  let align = match vattach.as_str() {
    "top" => "bottom1",
    "middle" => "axis",
    _ => &vattach,
  };

  for row_node in element_children(node) {
    let mut cols = Vec::new();
    for cell_node in element_children(&row_node) {
      let cell_align = cell_node.get_attribute("align");
      let colspan = cell_node.get_attribute("colspan");
      let mut td_attrs = HashMap::new();
      if let Some(a) = cell_align {
        td_attrs.insert("columnalign".to_string(), a);
      }
      if let Some(cs) = colspan {
        td_attrs.insert("columnspan".to_string(), cs);
      }

      let cell_children = element_children(&cell_node);
      let cell_content = if cell_children.is_empty() {
        vec![]
      } else {
        cell_children.iter().map(|c| pmml(doc, c)).collect()
      };

      cols.push(NodeData::Element {
        tag: "m:mtd".to_string(),
        attributes: if td_attrs.is_empty() { None } else { Some(td_attrs) },
        children: cell_content,
      });
    }
    rows.push(NodeData::Element {
      tag: "m:mtr".to_string(),
      attributes: None,
      children: cols,
    });
  }

  let mut table_attrs = HashMap::new();
  if align != "axis" {
    table_attrs.insert("align".to_string(), align.to_string());
  }

  NodeData::Element {
    tag: "m:mtable".to_string(),
    attributes: if table_attrs.is_empty() { None } else { Some(table_attrs) },
    children: rows,
  }
}

// ======================================================================
// Layout helpers

/// Simple sub/superscript.
fn pmml_script_simple(doc: &PostDocument, tag: &str, base: &Node, script: &Node) -> NodeData {
  NodeData::Element {
    tag: tag.to_string(),
    attributes: None,
    children: vec![pmml(doc, base), pmml_scriptsize(doc, script)],
  }
}

/// Convert node at script size.
fn pmml_scriptsize(doc: &PostDocument, node: &Node) -> NodeData {
  pmml(doc, node)
}

/// Convert node at smaller size (for fractions).
fn pmml_smaller(doc: &PostDocument, node: &Node) -> NodeData {
  pmml(doc, node)
}

/// Infix operator: arg1 op arg2 op arg3 ...
///
/// Port of `pmml_infix`.
fn pmml_infix(doc: &PostDocument, op: &Node, args: &[Node]) -> NodeData {
  let op_mml = pmml(doc, op);
  if args.is_empty() {
    return op_mml;
  }
  if args.len() == 1 {
    // Single arg: prefix
    return pmml_row(vec![op_mml, pmml(doc, &args[0])]);
  }
  let mut items = vec![pmml(doc, &args[0])];
  for arg in &args[1..] {
    items.push(op_mml.clone());
    items.push(pmml(doc, arg));
  }
  pmml_row(items)
}

/// Big operator with possible limits.
///
/// Port of `pmml_summation`.
fn pmml_summation(doc: &PostDocument, op: &Node, args: &[Node]) -> NodeData {
  let op_mml = pmml(doc, op);
  let mut items = vec![op_mml];
  items.push(pmml_mo_str("\u{2061}")); // FUNCTION APPLICATION
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

  // Convert the inner base
  let base_mml = pmml(doc, &inner_base);

  // Apply mid scripts (under/over)
  let base_mml = apply_mid_scripts(doc, base_mml, &mid_scripts);

  // Apply pre/post scripts
  apply_multi_scripts(doc, base_mml, &pre_scripts, &post_scripts)
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
  let pos_str = op.get_attribute("scriptpos").unwrap_or_else(|| "post0".to_string());
  let pos = if pos_str.starts_with("pre") { "pre" }
    else if pos_str.starts_with("mid") { "mid" }
    else { "post" };
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
    let realized = if doc.get_qname(&current_base).as_deref() == Some("ltx:XMRef") {
      current_base.get_attribute("idref")
        .and_then(|id| doc.find_node_by_id(&id).cloned())
        .unwrap_or(current_base.clone())
    } else {
      current_base.clone()
    };

    if doc.get_qname(&realized).as_deref() != Some("ltx:XMApp") {
      break;
    }

    let children = element_children(&realized);
    if children.len() < 3 {
      break;
    }

    let xop = &children[0];
    if doc.get_qname(xop).as_deref() != Some("ltx:XMTok") {
      break;
    }

    let xrole = xop.get_attribute("role").unwrap_or_default();
    let is_script_op = xrole.contains("SUPERSCRIPTOP") || xrole.contains("SUBSCRIPTOP");
    if !is_script_op {
      break;
    }

    let xbase = &children[1];
    let xscript = &children[2];
    let xpos_str = xop.get_attribute("scriptpos").unwrap_or_else(|| "post0".to_string());
    let xpos = if xpos_str.starts_with("pre") { "pre" }
      else if xpos_str.starts_with("mid") { "mid" }
      else { "post" };
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
      }
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
      }
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
      }
    }

    current_base = xbase.clone();
  }

  (current_base, pre_scripts, mid_scripts, post_scripts)
}

/// Apply mid scripts (under/over) to a base.
fn apply_mid_scripts(doc: &PostDocument, mut base: NodeData, mid_scripts: &[ScriptPair]) -> NodeData {
  for (sub_opt, sup_opt) in mid_scripts {
    let under = sub_opt.as_ref().map(|s| pmml_scriptsize(doc, s));
    let over = sup_opt.as_ref().map(|s| pmml_scriptsize(doc, s));

    base = match (under, over) {
      (Some(u), None) => NodeData::Element {
        tag: "m:munder".to_string(), attributes: None,
        children: vec![base, u],
      },
      (None, Some(o)) => NodeData::Element {
        tag: "m:mover".to_string(), attributes: None,
        children: vec![base, o],
      },
      (Some(u), Some(o)) => NodeData::Element {
        tag: "m:munderover".to_string(), attributes: None,
        children: vec![base, u, o],
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
  let none_mml = || NodeData::Element { tag: "m:none".to_string(), attributes: None, children: vec![] };

  if !pre_scripts.is_empty() {
    // mmultiscripts with prescripts
    let mut children = vec![base];
    for (sub_opt, sup_opt) in post_scripts {
      children.push(sub_opt.as_ref().map(|s| pmml_scriptsize(doc, s)).unwrap_or_else(none_mml));
      children.push(sup_opt.as_ref().map(|s| pmml_scriptsize(doc, s)).unwrap_or_else(none_mml));
    }
    children.push(NodeData::Element {
      tag: "m:mprescripts".to_string(), attributes: None, children: vec![],
    });
    for (sub_opt, sup_opt) in pre_scripts {
      children.push(sub_opt.as_ref().map(|s| pmml_scriptsize(doc, s)).unwrap_or_else(none_mml));
      children.push(sup_opt.as_ref().map(|s| pmml_scriptsize(doc, s)).unwrap_or_else(none_mml));
    }
    NodeData::Element {
      tag: "m:mmultiscripts".to_string(), attributes: None, children,
    }
  } else if post_scripts.len() > 1 {
    // mmultiscripts with multiple postscripts
    let mut children = vec![base];
    for (sub_opt, sup_opt) in post_scripts {
      children.push(sub_opt.as_ref().map(|s| pmml_scriptsize(doc, s)).unwrap_or_else(none_mml));
      children.push(sup_opt.as_ref().map(|s| pmml_scriptsize(doc, s)).unwrap_or_else(none_mml));
    }
    NodeData::Element {
      tag: "m:mmultiscripts".to_string(), attributes: None, children,
    }
  } else if post_scripts.is_empty() {
    base
  } else {
    // Single post script pair
    let (sub_opt, sup_opt) = &post_scripts[0];
    match (sub_opt, sup_opt) {
      (Some(sub_node), None) => NodeData::Element {
        tag: "m:msub".to_string(), attributes: None,
        children: vec![base, pmml_scriptsize(doc, sub_node)],
      },
      (None, Some(sup_node)) => NodeData::Element {
        tag: "m:msup".to_string(), attributes: None,
        children: vec![base, pmml_scriptsize(doc, sup_node)],
      },
      (Some(sub_node), Some(sup_node)) => NodeData::Element {
        tag: "m:msubsup".to_string(), attributes: None,
        children: vec![base, pmml_scriptsize(doc, sub_node), pmml_scriptsize(doc, sup_node)],
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
    tag: "m:mfrac".to_string(),
    attributes: None,
    children: vec![pmml_smaller(doc, numer), pmml_smaller(doc, denom)],
  }
}

// ======================================================================
// Utility functions

/// Wrap nodes in an mrow (or return single node unwrapped).
fn pmml_row(children: Vec<NodeData>) -> NodeData {
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
    tag: "m:mo".to_string(),
    attributes: None,
    children: vec![NodeData::Text(text.to_string())],
  }
}

/// Create a MathML error element.
fn pmml_error(msg: &str) -> NodeData {
  NodeData::Element {
    tag: "m:merror".to_string(),
    attributes: None,
    children: vec![NodeData::Element {
      tag: "m:mtext".to_string(),
      attributes: None,
      children: vec![NodeData::Text(msg.to_string())],
    }],
  }
}

/// Map a LaTeXML font name to a MathML mathvariant value.
///
/// Port of `unicode_mathvariant` (partial — full table in LaTeXML::Util::Unicode).
pub fn font_to_mathvariant(font: &str) -> Option<&'static str> {
  // Match Perl's %mathvariants mapping
  match font {
    "italic" | "math" | "CMR" => Some("italic"),
    "bold" => Some("bold"),
    "bold-italic" | "bold italic" => Some("bold-italic"),
    "script" | "caligraphic" => Some("script"),
    "bold-script" | "bold-caligraphic" => Some("bold-script"),
    "fraktur" => Some("fraktur"),
    "bold-fraktur" => Some("bold-fraktur"),
    "double-struck" | "blackboard" => Some("double-struck"),
    "sans-serif" => Some("sans-serif"),
    "bold-sans-serif" => Some("bold-sans-serif"),
    "sans-serif-italic" => Some("sans-serif-italic"),
    "sans-serif-bold-italic" => Some("sans-serif-bold-italic"),
    "monospace" | "typewriter" => Some("monospace"),
    "upright" | "roman" | "normal" => Some("normal"),
    _ => {
      // Check compound font names
      if font.contains("bold") && font.contains("italic") {
        Some("bold-italic")
      } else if font.contains("bold") {
        Some("bold")
      } else if font.contains("italic") {
        Some("italic")
      } else {
        None
      }
    }
  }
}
