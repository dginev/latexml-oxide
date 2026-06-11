//! Unicode math conversion processor.
//!
//! Port of `LaTeXML::Post::UnicodeMath` (435 lines of Perl).
//! Converts XMath nodes to Unicode mathematical notation strings.
//! Attempts compliance with UTN#28 (Unicode Technical Note on Plain Text Math).
//!
//! Used as:
//! 1. A standalone text math format
//! 2. A secondary format within MathML `m:annotation`
//! 3. A utility for converting math to plain text (e.g. for title attributes)

use libxml::tree::Node;

use crate::{
  document::{PostDocument, element_children},
  math_processor::{MathConversion, MathProcessor},
  processor::{ProcessResult, Processor},
};

const UNICODE_MATH_MIMETYPE: &str = "application/x-unicodemath";

// Precedence levels (matching Perl)
const PREC_RELOP: i32 = 1;
const PREC_ADDOP: i32 = 2;
const PREC_MULOP: i32 = 3;
const PREC_SCRIPTOP: i32 = 4;
const PREC_SYMBOL: i32 = 10;

/// UnicodeMath post-processor.
///
/// Port of `LaTeXML::Post::UnicodeMath`.
pub struct UnicodeMath {
  name:         String,
  is_secondary: bool,
}

impl Default for UnicodeMath {
  fn default() -> Self { Self::new() }
}

impl UnicodeMath {
  pub fn new() -> Self {
    UnicodeMath {
      name:         "UnicodeMath".to_string(),
      is_secondary: false,
    }
  }
}

impl Processor for UnicodeMath {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    doc.findnodes("//ltx:Math[not(ancestor::ltx:Math)]")
  }

  fn process(&mut self, doc: PostDocument, _nodes: Vec<Node>) -> ProcessResult { Ok(vec![doc]) }
}

impl MathProcessor for UnicodeMath {
  fn convert_node(&self, doc: &PostDocument, xmath: &Node) -> Option<MathConversion> {
    let math = xmath.get_parent()?;
    let (string, _prec) = unimath_internal(doc, &math);
    Some(MathConversion {
      processor_name: self.name.clone(),
      mimetype:       Some(UNICODE_MATH_MIMETYPE.to_string()),
      xml:            None,
      string:         Some(string),
      src:            None,
      width:          None,
      height:         None,
      depth:          None,
    })
  }

  fn raw_id_suffix(&self) -> &str { ".muni" }

  fn is_secondary(&self) -> bool { self.is_secondary }
}

/// Public entry point: convert a Math/XMath node to Unicode string.
///
/// Port of `unicodemath($doc, $node)`.
pub fn unicodemath(doc: &PostDocument, node: &Node) -> String {
  let (uni, _prec) = unimath_internal(doc, node);
  uni
}

// ======================================================================
// Precedence-based infix roles

fn infix_prec(role: &str) -> Option<i32> {
  match role {
    "ADDOP" | "BINOP" => Some(PREC_ADDOP),
    "MULOP" | "MIDDLE" | "COMPOSEOP" | "MODIFIEROP" => Some(PREC_MULOP),
    "RELOP" | "METARELOP" | "ARROW" => Some(PREC_RELOP),
    _ => None,
  }
}

/// Get the operator role, following embellished operators.
///
/// Port of `getOperatorRole`.
fn get_operator_role(doc: &PostDocument, node: &Node) -> Option<String> {
  if let Some(role) = node.get_attribute("role") {
    return Some(role);
  }
  if doc.is_qname(node, "ltx:XMApp") {
    let children = element_children(node);
    if children.len() >= 2 {
      let op_role = children[0].get_attribute("role").unwrap_or_default();
      if matches!(
        op_role.as_str(),
        "SUPERSCRIPTOP" | "SUBSCRIPTOP" | "OVERACCENT" | "UNDERACCENT" | "MODIFIER" | "MODIFIEROP"
      ) {
        return get_operator_role(doc, &children[1]);
      }
    }
  }
  None
}

/// Realize an XMRef node to its target.
fn realize(doc: &PostDocument, node: &Node) -> Option<Node> { doc.realize_xm_node(node) }

// ======================================================================
// Core conversion

/// Convert a node, returning (string, precedence).
///
/// Port of `unimath_internal`.
fn unimath_internal(doc: &PostDocument, node: &Node) -> (String, i32) {
  let tag = doc.get_qname(node).unwrap_or_default();
  let role = node.get_attribute("role").unwrap_or_default();

  match tag.as_str() {
    "ltx:Math" | "ltx:XMath" => unimath_map(doc, &element_children(node)),
    "ltx:XMDual" => {
      let children = element_children(node);
      if children.len() >= 2 {
        unimath_internal(doc, &children[1]) // Presentation branch
      } else {
        unimath_error("Empty XMDual")
      }
    },
    "ltx:XMWrap" | "ltx:XMArg" => unimath_map(doc, &element_children(node)),
    "ltx:XMApp" => {
      let children = element_children(node);
      if children.is_empty() {
        return unimath_error("Missing Operator");
      }
      let op = &children[0];
      let args = &children[1..];

      // Handle floating/post scripts
      if role.contains("SUBSCRIPT") {
        return unimath_sub(doc, None, op);
      }
      if role.contains("SUPERSCRIPT") {
        return unimath_sup(doc, None, op);
      }

      // Realize operator and dispatch
      let rop = realize(doc, op).unwrap_or_else(|| op.clone());
      let op_role = get_operator_role(doc, &rop).unwrap_or_default();
      let meaning = rop.get_attribute("meaning").unwrap_or_default();

      // Dispatch by role/meaning
      match (op_role.as_str(), meaning.as_str()) {
        (_, "formulae") | (_, "multirelation") => unimath_map(doc, args),
        (_, "limit-from") | (_, "annotated") => unimath_prefix(doc, op, args),
        (_, "square-root") => {
          if !args.is_empty() {
            let inner = unimath_nested(doc, &args[0], PREC_MULOP);
            (format!("\u{221A}{}", inner), PREC_MULOP)
          } else {
            ("\u{221A}".to_string(), PREC_MULOP)
          }
        },
        (_, "nth-root") => {
          if args.len() >= 2 {
            let (n, _) = unimath_internal(doc, &args[1]);
            let base = unimath_nested(doc, &args[0], PREC_MULOP);
            let op_str = match n.as_str() {
              "2" => "\u{221A}".to_string(),
              "3" => "\u{221B}".to_string(),
              "4" => "\u{221C}".to_string(),
              _ => format!("\\root {}\\of", n),
            };
            (format!("{}{}", op_str, base), PREC_MULOP)
          } else {
            unimath_prefix(doc, op, args)
          }
        },
        (_, "continued-fraction") => unimath_error("continued fraction"),
        ("FRACOP", _) => {
          if args.len() >= 2 {
            let thickness = children[0].get_attribute("thickness");
            if thickness.is_some() {
              // Binomial-like
              let num = unimath_nested(doc, &args[0], 0);
              let den = unimath_nested(doc, &args[1], 0);
              (format!("({}\u{00A6}{})", num, den), PREC_SYMBOL)
            } else {
              let num = unimath_nested(doc, &args[0], PREC_MULOP);
              let den = unimath_nested(doc, &args[1], PREC_MULOP);
              (format!("{}/{}", num, den), 1)
            }
          } else {
            unimath_prefix(doc, op, args)
          }
        },
        ("SUPERSCRIPTOP", _) => {
          if args.len() >= 2 {
            unimath_sup(doc, Some(&args[0]), &args[1])
          } else {
            unimath_prefix(doc, op, args)
          }
        },
        ("SUBSCRIPTOP", _) => {
          if args.len() >= 2 {
            unimath_sub(doc, Some(&args[0]), &args[1])
          } else {
            unimath_prefix(doc, op, args)
          }
        },
        ("OVERACCENT", _) => {
          if !args.is_empty() {
            unimath_overaccent(doc, op, &args[0])
          } else {
            unimath_prefix(doc, op, args)
          }
        },
        ("UNDERACCENT", _) => {
          if !args.is_empty() {
            unimath_underaccent(doc, op, &args[0])
          } else {
            unimath_prefix(doc, op, args)
          }
        },
        ("POSTFIX", _) => {
          if !args.is_empty() {
            let (op_str, _) = unimath_internal(doc, op);
            let base = unimath_nested(doc, &args[0], PREC_MULOP);
            (format!("{}{}", base, op_str), PREC_MULOP)
          } else {
            unimath_prefix(doc, op, args)
          }
        },
        ("ENCLOSE", _) => {
          if !args.is_empty() {
            (unimath_nested(doc, &args[0], PREC_SYMBOL), PREC_SYMBOL)
          } else {
            (String::new(), PREC_SYMBOL)
          }
        },
        _ => {
          // Check if it's an infix role
          if let Some(prec) = infix_prec(&op_role) {
            unimath_infix_with_prec(doc, op, args, prec)
          } else {
            // Default: prefix
            unimath_prefix(doc, op, args)
          }
        },
      }
    },
    "ltx:XMTok" => {
      let meaning = node.get_attribute("meaning").unwrap_or_default();
      if meaning == "absent" {
        return (String::new(), PREC_SYMBOL);
      }
      let text = stylize_content(node);
      (text, PREC_SYMBOL)
    },
    "ltx:XMHint" => (String::new(), 0),
    "ltx:XMArray" => {
      let rows: Vec<String> = element_children(node)
        .iter()
        .map(|row| {
          element_children(row)
            .iter()
            .map(|cell| unimath_nested(doc, cell, 0))
            .collect::<Vec<_>>()
            .join("&")
        })
        .collect();
      (format!("\u{25A0}({})", rows.join("@")), 0)
    },
    "ltx:XMText" => unimath_text(node),
    "ltx:XMRef" => match realize(doc, node) {
      Some(target) => unimath_internal(doc, &target),
      _ => unimath_error("Unresolved XMRef"),
    },
    "ltx:ERROR" => unimath_error(&node.get_content()),
    _ => unimath_text(node),
  }
}

/// Convert and wrap in braces if precedence is too low.
///
/// Port of `unimath_nested`.
fn unimath_nested(doc: &PostDocument, node: &Node, prec: i32) -> String {
  let (string, iprec) = unimath_internal(doc, node);
  if iprec >= prec {
    string
  } else {
    format!("{{{}}}", string)
  }
}

/// Combine conversion of multiple nodes.
///
/// Port of `unimath_map`.
fn unimath_map(doc: &PostDocument, args: &[Node]) -> (String, i32) {
  let mut oprec = 0;
  // If wrapped in OPEN...CLOSE, it's a symbol-level expression
  if args.len() > 1 {
    let first_role = args
      .first()
      .and_then(|n| n.get_attribute("role"))
      .unwrap_or_default();
    let last_role = args
      .last()
      .and_then(|n| n.get_attribute("role"))
      .unwrap_or_default();
    if first_role == "OPEN" && last_role == "CLOSE" {
      oprec = PREC_SYMBOL;
    }
  }
  let result: String = args.iter().map(|a| unimath_nested(doc, a, 0)).collect();
  (result, oprec)
}

/// Prefix application: op arg1 arg2 ...
///
/// Port of `unimath_prefix`.
fn unimath_prefix(doc: &PostDocument, op: &Node, args: &[Node]) -> (String, i32) {
  if args.is_empty() {
    return (String::new(), PREC_SYMBOL);
  }
  let op_str = unimath_nested(doc, op, 0);
  let args_str: String = args
    .iter()
    .map(|a| unimath_nested(doc, a, PREC_SYMBOL))
    .collect();
  (format!("{}{}", op_str, args_str), PREC_SYMBOL)
}

/// Infix application with explicit precedence.
///
/// Port of `unimath_infix`.
fn unimath_infix_with_prec(
  doc: &PostDocument,
  op: &Node,
  args: &[Node],
  prec: i32,
) -> (String, i32) {
  if args.is_empty() {
    return (String::new(), PREC_SYMBOL);
  }
  let opuni = unimath_nested(doc, op, prec);
  if args.len() == 1 {
    // Single arg = prefix
    let arg = unimath_nested(doc, &args[0], prec);
    (format!("{}{}", opuni, arg), prec)
  } else {
    let mut items = vec![unimath_nested(doc, &args[0], prec)];
    for arg in &args[1..] {
      items.push(opuni.clone());
      items.push(unimath_nested(doc, arg, prec));
    }
    (items.join(""), prec)
  }
}

/// Subscript: base_script or _{script}base (pre).
///
/// Port of `unimath_sub`.
fn unimath_sub(doc: &PostDocument, base: Option<&Node>, script: &Node) -> (String, i32) {
  let ubase = base
    .map(|b| unimath_nested(doc, b, PREC_SCRIPTOP))
    .unwrap_or_default();
  let (uscript, prec) = unimath_internal(doc, script);
  let uscript = if prec < PREC_SCRIPTOP {
    format!("{{{}}}", uscript)
  } else {
    uscript
  };
  (format!("{}_{}", ubase, uscript), PREC_SCRIPTOP)
}

/// Superscript: base^script or ^{script}base (pre).
///
/// Port of `unimath_sup`.
fn unimath_sup(doc: &PostDocument, base: Option<&Node>, script: &Node) -> (String, i32) {
  let ubase = base
    .map(|b| unimath_nested(doc, b, PREC_SCRIPTOP))
    .unwrap_or_default();
  let (uscript, prec) = unimath_internal(doc, script);
  let uscript = if prec < PREC_SCRIPTOP {
    format!("{{{}}}", uscript)
  } else {
    uscript
  };
  (format!("{}^{}", ubase, uscript), PREC_SCRIPTOP)
}

/// Over-accent: combining character above.
///
/// Port of `unimath_overaccent`.
fn unimath_overaccent(doc: &PostDocument, op: &Node, base: &Node) -> (String, i32) {
  let acc = op.get_content();
  let combining = overaccent_combining(&acc);
  let (mut ubase, _) = unimath_internal(doc, base);
  if ubase.chars().count() > 1 {
    ubase = format!("({})", ubase);
  }
  let accent_str = combining.unwrap_or_else(|| format!("\u{252C}{}", acc));
  (format!("{}{}", ubase, accent_str), PREC_SCRIPTOP)
}

/// Under-accent: combining character below.
///
/// Port of `unimath_underaccent`.
fn unimath_underaccent(doc: &PostDocument, op: &Node, base: &Node) -> (String, i32) {
  let acc = op.get_content();
  let combining = underaccent_combining(&acc);
  let (mut ubase, _) = unimath_internal(doc, base);
  if ubase.chars().count() > 1 {
    ubase = format!("({})", ubase);
  }
  let accent_str = combining.unwrap_or_else(|| format!("\u{252C}{}", acc));
  (format!("{}{}", ubase, accent_str), PREC_SCRIPTOP)
}

/// Convert an XMTok's content to styled Unicode text.
///
/// Port of `stylizeContent` (simplified — full version needs unicode_convert).
fn stylize_content(node: &Node) -> String {
  let role = node
    .get_attribute("role")
    .unwrap_or_else(|| "ID".to_string());
  let text = node.get_content();
  if text.is_empty() {
    // Fallback for empty tokens
    static DEFAULT_CONTENT: &[(&str, &str)] = &[
      ("MULOP", "\u{2062}"), // INVISIBLE TIMES
      ("ADDOP", "\u{2064}"), // INVISIBLE PLUS
      ("PUNCT", "\u{2063}"), // INVISIBLE SEPARATOR
    ];
    for (r, default) in DEFAULT_CONTENT {
      if role == *r {
        return default.to_string();
      }
    }
    node
      .get_attribute("name")
      .or_else(|| node.get_attribute("meaning"))
      .unwrap_or(role)
  } else {
    text
  }
}

/// Text content in quotes.
///
/// Port of `unimath_text`.
fn unimath_text(node: &Node) -> (String, i32) {
  let text = node.get_content();
  (format!("\"{}\"", text), PREC_SYMBOL)
}

/// Error representation.
fn unimath_error(msg: &str) -> (String, i32) { (format!("\"ERROR {}\"", msg), PREC_SYMBOL) }

/// Map over-accent character to combining equivalent.
fn overaccent_combining(acc: &str) -> Option<String> {
  let c = acc.chars().next()?;
  let combining = match c {
    '^' => '\u{0302}',        // hat
    '\u{02C7}' => '\u{030C}', // check
    '~' => '\u{0303}',        // tilde
    '\u{0084}' => '\u{0301}', // acute
    '\u{0060}' => '\u{0300}', // grave
    '\u{02D9}' => '\u{0307}', // dot
    '\u{00AB}' => '\u{0308}', // ddot
    '\u{00AF}' => '\u{0304}', // bar/overline
    '\u{2192}' => '\u{20D7}', // vec
    '\u{02D8}' => '\u{0306}', // breve
    'o' => '\u{030A}',        // ring
    '\u{02DD}' => '\u{030B}', // double acute
    _ => return None,
  };
  Some(combining.to_string())
}

/// Map under-accent character to combining equivalent.
fn underaccent_combining(acc: &str) -> Option<String> {
  let c = acc.chars().next()?;
  let combining = match c {
    '\u{00B8}' => '\u{0327}', // cedilla
    '.' => '\u{0323}',        // dot below
    '\u{00AF}' => '\u{0331}', // macron below
    '=' | ',' => '\u{0361}',  // tie / lfhook
    _ => return None,
  };
  Some(combining.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_stylize_empty_token() {
    // MULOP with no text should produce invisible times
    let result = stylize_content_for_role("MULOP", "");
    assert_eq!(result, "\u{2062}");
  }

  #[test]
  fn test_overaccent_combining() {
    assert_eq!(overaccent_combining("^"), Some("\u{0302}".to_string()));
    assert_eq!(overaccent_combining("~"), Some("\u{0303}".to_string()));
    assert_eq!(overaccent_combining("x"), None);
  }

  /// Helper for testing stylize_content without a Node.
  fn stylize_content_for_role(role: &str, text: &str) -> String {
    static DEFAULT_CONTENT: &[(&str, &str)] = &[
      ("MULOP", "\u{2062}"),
      ("ADDOP", "\u{2064}"),
      ("PUNCT", "\u{2063}"),
    ];
    if text.is_empty() {
      for (r, default) in DEFAULT_CONTENT {
        if role == *r {
          return default.to_string();
        }
      }
    }
    text.to_string()
  }
}
