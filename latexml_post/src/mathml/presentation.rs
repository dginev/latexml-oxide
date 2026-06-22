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
  // Reset the math-style context for each formula (display/inline are both 100% size, so
  // Display is the right baseline; nested scripts/fractions step it down from here).
  CURRENT_STYLE.with(|s| s.set(MathStyle::Display));
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

/// Core dispatch: convert a single XMath node to Presentation MathML.
///
/// Port of `pmml` + `pmml_internal`.
fn pmml(doc: &PostDocument, node: &Node) -> NodeData {
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
        let results: Vec<NodeData> = element_children_iter(node).map(|c| pmml(doc, &c)).collect();
        return pmml_row(results);
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
        return pmml_row(children);
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

  // Dispatch by role
  match op_role.as_str() {
    "SUPERSCRIPTOP" | "SUBSCRIPTOP" if args.len() >= 2 => {
      pmml_script_full(doc, op, &args[0], &args[1])
    },
    "FRACOP" if args.len() >= 2 => {
      let thickness = rop.get_attribute("thickness");
      let mut attrs = HashMap::default();
      if let Some(ref t) = thickness {
        if t == "0" || t == "0pt" {
          attrs.insert("linethickness".to_string(), "0".to_string());
        }
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
    "OPEN" | "CLOSE" | "ENCLOSE" if !args.is_empty() => {
      // Fenced: (args)
      pmml_parenthesize(doc, op, args)
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
        NodeData::Element {
          tag:        "m:msqrt".to_string(),
          attributes: None,
          children:   vec![pmml(doc, &args[0])],
        }
      } else if meaning == "continued-fraction" && args.len() >= 2 {
        pmml_cfrac(doc, op, &args[0], &args[1])
      } else if meaning == "nth-root" && args.len() >= 2 {
        NodeData::Element {
          tag:        "m:mroot".to_string(),
          attributes: None,
          children:   vec![pmml(doc, &args[0]), pmml_scriptsize(doc, &args[1])],
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
fn pmml_token(_doc: &PostDocument, node: &Node) -> NodeData {
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

  // Operator-specific attributes
  if tag == "m:mo" {
    // Check the XMTok's stretchy attribute
    let stretchy_attr = node.get_attribute("stretchy");

    // Handle size attribute. Port of Perl `stylizeContent` L777: emit `mathsize` only when
    // the token's own size differs from the current contextual size. Inside a sub/superscript
    // the context is already scriptsize (70%/50%), so an operator at that size — e.g. the `-`
    // in `10^{-21}` or the `*` in `x^{\ast}` — must NOT get a spurious `mathsize="70%"` (the
    // <msup>/<msub> structure already shrinks it; the redundant attribute double-shrinks it).
    if let Some(size) = node.get_attribute("fontsize") {
      if size != current_context_size() {
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

    // Store internal spacing attributes for adjust_spacing.
    // Port of Perl's `stylizeContent` lines 821-826.
    attrs.insert("_role".to_string(), role.clone());
    let props = operator_dictionary::opdict_lookup(&text, &role);
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

  NodeData::Element {
    tag:        tag.to_string(),
    attributes: if attrs.is_empty() { None } else { Some(attrs) },
    children:   vec![NodeData::Text(text)],
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
      tag:        "m:mspace".to_string(),
      attributes: Some(HashMap::from_iter([("width".to_string(), w)])),
      children:   vec![],
    }
  } else {
    // Empty hint
    NodeData::Text(String::new())
  }
}

/// Convert an XMArray to an mtable.
///
/// Port of `pmml_internal` XMArray branch (`MathML.pm` L432-486).
fn pmml_array(doc: &PostDocument, node: &Node) -> NodeData {
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
        cell_children.iter().map(|c| pmml(doc, c)).collect()
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
    if matches!(
      tag.as_str(),
      "m:msub" | "m:msup" | "m:msubsup" | "m:munder" | "m:mover" | "m:munderover" | "m:mprescripts"
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

/// Format em value.
fn fmt_em(val: f64) -> String {
  if val.abs() < SPACING_EPSILON {
    return "0em".to_string();
  }
  let s = format!("{:.3}", val);
  let s = s.trim_end_matches('0').trim_end_matches('.');
  format!("{}em", s)
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

/// Adjust spacing between two adjacent nodes.
fn adjust_pair(prev: &mut NodeData, next: &mut NodeData) {
  let prev_role_s = get_node_role(prev);
  let next_role_s = get_node_role(next);
  let prev_role = if prev_role_s.is_empty() {
    "ATOM"
  } else {
    &prev_role_s
  };
  let next_role = if next_role_s.is_empty() {
    "ATOM"
  } else {
    &next_role_s
  };

  let prev_type = m_atom_type(get_node_tag(prev)).unwrap_or_else(|| role_to_atom_type(prev_role));
  let next_type = m_atom_type(get_node_tag(next)).unwrap_or_else(|| role_to_atom_type(next_role));

  let tex_code = atompair_spacing(prev_type, next_type);
  let tex_space = TEX_SPACING[tex_code.unsigned_abs() as usize];

  let prev_dict_right = get_node_attr_f64(prev, "_rspace");
  let next_dict_left = get_node_attr_f64(next, "_lspace");
  let default = prev_dict_right + next_dict_left;
  let target = tex_space;

  if (target - default).abs() <= SPACING_EPSILON {
    return;
  }

  let prev_is_mo = get_node_tag(prev) == "m:mo";
  let next_is_mo = get_node_tag(next) == "m:mo";

  if prev_is_mo && next_is_mo {
    let n = next_dict_left;
    let rem = target - n;
    if rem >= 0.0 {
      set_node_attr(
        prev,
        "rspace",
        &fmt_em(if rem > SPACING_EPSILON { rem } else { 0.0 }),
      );
    } else {
      let p = prev_dict_right;
      let rem2 = target - p;
      if rem2 >= 0.0 {
        set_node_attr(
          next,
          "lspace",
          &fmt_em(if rem2 > SPACING_EPSILON { rem2 } else { 0.0 }),
        );
      }
    }
  } else if prev_is_mo {
    set_node_attr(prev, "rspace", &fmt_em(target));
  } else if next_is_mo {
    set_node_attr(next, "lspace", &fmt_em(target));
  }
}

/// Walk the MathML tree and adjust spacing.
pub fn adjust_spacing(node: &mut NodeData) {
  // Fast-path leaf check on a borrowed &str — avoids a per-node String allocation.
  let tag_ref = get_node_tag(node);
  if matches!(tag_ref, "m:mi" | "m:mo" | "m:mn" | "m:ms" | "m:mtext") {
    return;
  }
  let is_mrow = is_mrow_like(tag_ref);

  if is_mrow {
    if let NodeData::Element { children, .. } = node {
      for child in children.iter_mut() {
        adjust_spacing(child);
      }
      let len = children.len();
      if len >= 2 {
        let mut i = 0;
        while i + 1 < len {
          let j = i + 1;
          // Skip invisible operators
          if j + 1 < len
            && get_node_tag(&children[j]) == "m:mo"
            && is_node_text_invisible_op(&children[j])
          {
            if j + 1 < len {
              let (left, right) = children.split_at_mut(j + 1);
              if let Some(next) = right.first_mut() {
                adjust_pair(&mut left[i], next);
              }
            }
            i = j + 1;
          } else {
            let (left, right) = children.split_at_mut(j);
            if let Some(next) = right.first_mut() {
              adjust_pair(&mut left[i], next);
            }
            i = j;
          }
        }
      }
    }
  } else {
    if let NodeData::Element { children, .. } = node {
      for child in children.iter_mut() {
        adjust_spacing(child);
      }
    }
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
}
