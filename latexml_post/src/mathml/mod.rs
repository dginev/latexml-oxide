//! MathML conversion processor.
//!
//! Port of `LaTeXML::Post::MathML` (2162 lines) + submodules:
//! - `Presentation.pm` (146 lines) — Presentation MathML rendering rules
//! - `Content.pm` (31 lines) — Content MathML rendering rules
//! - `Linebreaker.pm` (1053 lines) — MathML line-breaking algorithm
//! - `OperatorDictionary.pm` (252 lines) — Operator symbol table
//!
//! This is the primary math conversion format for web output.
//! Converts XMath parsed math into Presentation MathML and/or Content MathML.

pub mod content;
pub mod linebreaker;
pub mod operator_dictionary;
pub mod presentation;

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;

use crate::{
  document::{NodeData, PostDocument},
  math_processor::{MathConversion, MathProcessor, math_is_parsed, process_math},
  processor::{ProcessResult, Processor},
  unicode,
};

const MML_URI: &str = "http://www.w3.org/1998/Math/MathML";
const MML_MIMETYPE: &str = "application/mathml-presentation+xml";
const CMML_MIMETYPE: &str = "application/mathml-content+xml";

/// MathML post-processor.
///
/// Port of `LaTeXML::Post::MathML`.
/// Handles both Presentation and Content MathML conversion.
pub struct MathML {
  name:            String,
  is_secondary:    bool,
  /// Whether to produce Content MathML (vs Presentation).
  content_mathml:  bool,
  /// Whether to use plane1 Unicode characters for styled identifiers.
  plane1:          bool,
  /// Whether to enable line-breaking.
  linebreaking:    bool,
  /// Line width for line-breaking.
  line_width:      u32,
  /// Whether to keep the XMath nodes alongside the generated MathML.
  keep_xmath:      bool,
  /// Whether to emit invisible times (U+2062). When false, replaces with zero-width space.
  /// Perl: $$MATHPROCESSOR{invisibletimes} — defaults to true.
  invisible_times: bool,
  /// Whether to include TeX source annotation in parallel MathML.
  /// Perl: --mathtex adds <m:annotation encoding='application/x-tex'>
  mathtex:         bool,
  /// Whether to add intent=":literal" on all <math> elements.
  /// ar5iv.sty.ltxml monkey-patches outerWrapper to add this.
  intent_literal:  bool,
  /// Parallel-markup secondaries (e.g. a Content-MathML processor under a
  /// Presentation-MathML primary). Held by the primary rather than registered
  /// as independent chain passes: during the primary's `process_math_node`,
  /// each secondary's `convert_node` runs against the still-live XMath and the
  /// results are folded into one `<m:semantics>` via [`combine_parallel`].
  /// Port of Perl `MathProcessor`'s primary→secondary parallel model
  /// (`$$self{parallel}` / `combineParallel`). Empty for a standalone format.
  secondaries:     Vec<Box<dyn MathProcessor>>,
}

impl MathML {
  /// Create a Presentation MathML processor.
  pub fn new_presentation() -> Self {
    MathML {
      name:            "MathML[Presentation]".to_string(),
      is_secondary:    false,
      content_mathml:  false,
      plane1:          true,
      linebreaking:    false,
      line_width:      80,
      keep_xmath:      false,
      invisible_times: true,
      mathtex:         false,
      intent_literal:  false,
      secondaries:     Vec::new(),
    }
  }

  /// Create a Content MathML processor.
  pub fn new_content() -> Self {
    MathML {
      name:            "MathML[Content]".to_string(),
      is_secondary:    false,
      content_mathml:  true,
      plane1:          true,
      linebreaking:    false,
      line_width:      80,
      keep_xmath:      false,
      invisible_times: true,
      mathtex:         false,
      intent_literal:  false,
      secondaries:     Vec::new(),
    }
  }

  /// Enable intent=":literal" on all <math> elements.
  /// Perl: ar5iv.sty.ltxml monkey-patches outerWrapper for this.
  pub fn with_intent_literal(mut self, enable: bool) -> Self {
    self.intent_literal = enable;
    self
  }

  /// Enable line-breaking with the given width.
  pub fn with_linebreaking(mut self, width: u32) -> Self {
    self.linebreaking = true;
    self.line_width = width;
    self
  }

  /// Keep XMath nodes in the output alongside MathML.
  pub fn with_keep_xmath(mut self, keep: bool) -> Self {
    self.keep_xmath = keep;
    self
  }

  /// Set whether to emit invisible times (U+2062) in MathML output.
  /// When false, invisible times is replaced with zero-width space (U+200B).
  /// Perl: --noinvisibletimes
  pub fn with_invisible_times(mut self, emit: bool) -> Self {
    self.invisible_times = emit;
    self
  }

  /// Enable TeX source annotation in MathML output (--mathtex).
  pub fn with_mathtex(mut self, enable: bool) -> Self {
    self.mathtex = enable;
    self
  }

  /// Mark this processor as a parallel-markup secondary (e.g. the Content-MathML
  /// format under a Presentation primary). Secondaries get their format-specific
  /// `id_suffix` and are folded into the primary's `<m:semantics>` rather than
  /// emitted as a standalone `<m:math>`. Port of `MathProcessor`'s secondary role.
  pub fn secondary(mut self) -> Self {
    self.is_secondary = true;
    self
  }

  /// Attach parallel-markup secondaries to this (primary) processor. Their
  /// conversions are merged into one `<m:semantics>` by [`combine_parallel`]
  /// during the primary's pass. Mirrors Perl `MathProcessor`'s primary holding
  /// its parallel secondaries.
  pub fn with_secondaries(mut self, secondaries: Vec<Box<dyn MathProcessor>>) -> Self {
    self.secondaries = secondaries;
    self
  }
}

impl Processor for MathML {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    doc.findnodes("//ltx:Math[not(ancestor::ltx:Math)]")
  }

  fn process(&mut self, mut doc: PostDocument, nodes: Vec<Node>) -> ProcessResult {
    // Register the MathML namespace so add_nodes can create m: elements
    doc.add_namespace("m", MML_URI);

    // Process all math nodes
    process_math(self, &mut doc, nodes, self.keep_xmath)?;
    Ok(vec![doc])
  }
}

impl MathProcessor for MathML {
  fn convert_node(&self, doc: &PostDocument, xmath: &Node) -> Option<MathConversion> {
    // Set invisible_times flag for rendering
    presentation::set_invisible_times(self.invisible_times);

    let xml = if self.content_mathml {
      content::convert_to_cmml(doc, xmath)
    } else {
      presentation::convert_to_pmml(doc, xmath)
    };

    let mimetype = if self.content_mathml {
      CMML_MIMETYPE
    } else {
      MML_MIMETYPE
    };

    // If mathtex is enabled, wrap in <m:semantics> with TeX annotation.
    // Skip when this primary carries parallel secondaries: `combine_parallel`
    // then builds the single `<m:semantics>` (primary + content annotation-xml
    // + the x-tex annotation), so wrapping here would double-nest semantics.
    let final_xml = if self.mathtex && self.secondaries.is_empty() {
      let tex_str = xmath
        .get_parent()
        .and_then(|p| p.get_attribute("tex"))
        .unwrap_or_default();
      if tex_str.is_empty() {
        xml
      } else {
        NodeData::Element {
          tag:        "m:semantics".to_string(),
          attributes: None,
          children:   vec![xml, NodeData::Element {
            tag:        "m:annotation".to_string(),
            attributes: Some(HashMap::from_iter([(
              "encoding".to_string(),
              "application/x-tex".to_string(),
            )])),
            children:   vec![NodeData::Text(tex_str)],
          }],
        }
      }
    } else {
      xml
    };

    Some(MathConversion {
      processor_name: self.name.clone(),
      mimetype:       Some(mimetype.to_string()),
      xml:            Some(final_xml),
      string:         None,
      src:            None,
      width:          None,
      height:         None,
      depth:          None,
    })
  }

  fn combine_parallel(
    &self,
    _doc: &PostDocument,
    xmath: &Node,
    primary: MathConversion,
    secondaries: Vec<MathConversion>,
  ) -> MathConversion {
    if secondaries.is_empty() {
      return primary;
    }

    // Build m:semantics element with primary + annotation-xml for secondaries
    let mut children = Vec::new();
    if let Some(ref xml) = primary.xml {
      children.push(xml.clone());
    }

    for secondary in &secondaries {
      let mimetype = secondary.mimetype.as_deref().unwrap_or("unknown");
      // Parallel markup names the format via the canonical encoding label
      // (e.g. `MathML-Content`), not the raw internal mimetype. Port of
      // `%ENCODINGS` / `encoding_for_mimetype`.
      let encoding = encoding_for_mimetype(mimetype).to_string();
      if let Some(ref xml) = secondary.xml {
        children.push(NodeData::Element {
          tag:        "m:annotation-xml".to_string(),
          attributes: Some(HashMap::from_iter([("encoding".to_string(), encoding)])),
          children:   vec![xml.clone()],
        });
      } else if let Some(ref string) = secondary.string {
        children.push(NodeData::Element {
          tag:        "m:annotation".to_string(),
          attributes: Some(HashMap::from_iter([("encoding".to_string(), encoding)])),
          children:   vec![NodeData::Text(string.clone())],
        });
      }
    }

    // TeX source annotation. In the standalone (no-secondary) path this is added
    // by `convert_node`; in the parallel path that wrap is skipped, so the
    // single combined `<m:semantics>` carries the x-tex annotation here.
    if self.mathtex {
      let tex_str = xmath
        .get_parent()
        .and_then(|p| p.get_attribute("tex"))
        .unwrap_or_default();
      if !tex_str.is_empty() {
        children.push(NodeData::Element {
          tag:        "m:annotation".to_string(),
          attributes: Some(HashMap::from_iter([(
            "encoding".to_string(),
            "application/x-tex".to_string(),
          )])),
          children:   vec![NodeData::Text(tex_str)],
        });
      }
    }

    MathConversion {
      processor_name: self.name.clone(),
      mimetype:       Some(MML_MIMETYPE.to_string()),
      xml:            Some(NodeData::Element {
        tag: "m:semantics".to_string(),
        attributes: None,
        children,
      }),
      string:         None,
      src:            None,
      width:          None,
      height:         None,
      depth:          None,
    }
  }

  fn outer_wrapper(&self, _doc: &PostDocument, xmath: &Node, conversion: NodeData) -> NodeData {
    let mut attrs = HashMap::default();
    // Determine display mode and alttext from parent Math element
    // Port of MathML::outerWrapper (L77-100)
    if let Some(math) = xmath.get_parent() {
      let mode = math
        .get_attribute("mode")
        .unwrap_or_else(|| "inline".to_string());
      attrs.insert(
        "display".to_string(),
        if mode == "display" {
          "block".to_string()
        } else {
          "inline".to_string()
        },
      );
      if let Some(tex) = math.get_attribute("tex") {
        attrs.insert("alttext".to_string(), tex);
      }
      if let Some(class) = math.get_attribute("class") {
        attrs.insert("class".to_string(), class);
      }
    }

    // ar5iv.sty.ltxml: intent=":literal" for all math elements
    if self.intent_literal {
      attrs.insert("intent".to_string(), ":literal".to_string());
    }

    NodeData::Element {
      tag:        "m:math".to_string(),
      attributes: Some(attrs),
      children:   vec![conversion],
    }
  }

  fn raw_id_suffix(&self) -> &str {
    if self.content_mathml {
      ".cmml"
    } else {
      ".pmml"
    }
  }

  fn is_secondary(&self) -> bool { self.is_secondary }

  fn can_convert(&self, _doc: &PostDocument, math: &Node) -> bool {
    // Content MathML requires parsed math
    if self.content_mathml {
      math_is_parsed(math)
    } else {
      true
    }
  }

  fn parallel_secondaries(&self) -> &[Box<dyn MathProcessor>] { &self.secondaries }

  fn preprocess(&self, _doc: &PostDocument, _nodes: &[Node]) {
    // Register MathML namespace
    log::trace!("MathML: would register m namespace for {}", MML_URI);
  }
}

/// MathML encoding names for parallel markup annotation-xml.
///
/// Port of `%ENCODINGS`.
pub fn encoding_for_mimetype(mimetype: &str) -> &str {
  match mimetype {
    "application/mathml-presentation+xml" => "MathML-Presentation",
    "application/mathml-content+xml" => "MathML-Content",
    "image/svg+xml" => "SVG1.1",
    _ => mimetype,
  }
}

/// Math style step-down table.
///
/// Port of `%stylestep`.
pub fn style_step(style: &str) -> &str {
  match style {
    "display" => "text",
    "text" => "script",
    "script" => "scriptscript",
    _ => "scriptscript",
  }
}

/// Size percentage for math styles.
///
/// Port of `%stylesize`.
pub fn style_size(style: &str) -> &str {
  match style {
    "display" | "text" => "100%",
    "script" => "70%",
    _ => "50%",
  }
}

/// Stylize a token's content for MathML.
///
/// Port of `stylizeContent` (~130 lines of Perl).
/// Given an XMTok node and a target MathML tag, determines:
/// - The text content (with empty-token defaults)
/// - The mathvariant (from font, with plane1 mapping)
/// - Operator dictionary properties (fence, stretchy, etc.)
/// - Color, background, class, href attributes
///
/// Returns (text, attributes_map).
pub fn stylize_content(node: &Node, target_tag: &str) -> (String, HashMap<String, String>) {
  let _is_element = true; // Node is always an element in our API
  let role = node.get_attribute("role").unwrap_or_default();
  let font = node.get_attribute("font");
  let size = node.get_attribute("fontsize");
  let color = node.get_attribute("color");
  let bgcolor = node.get_attribute("backgroundcolor");
  let opacity = node.get_attribute("opacity");
  let class = node.get_attribute("class");
  let href = node.get_attribute("href");
  let title = node.get_attribute("title");

  let mut text = node.get_content();
  let is_token = matches!(target_tag, "m:mi" | "m:mo" | "m:mn");

  // Default token content for invisible operators
  static DEFAULT_CONTENT: &[(&str, &str)] = &[
    ("MULOP", "\u{2062}"), // INVISIBLE TIMES
    ("ADDOP", "\u{2064}"), // INVISIBLE PLUS
    ("PUNCT", "\u{2063}"), // INVISIBLE SEPARATOR
  ];

  if text.is_empty() {
    for (r, default) in DEFAULT_CONTENT {
      if role == *r {
        text = default.to_string();
        break;
      }
    }
    if text.is_empty() {
      text = node
        .get_attribute("name")
        .or_else(|| node.get_attribute("meaning"))
        .unwrap_or_else(|| role.clone());
    }
  }

  // Minus sign normalization (MathML Core prefers Unicode minus)
  if text == "-" && matches!(role.as_str(), "ADDOP" | "OPERATOR") {
    text = "\u{2212}".to_string();
  }

  // Determine mathvariant from font.
  // Port of Perl stylizeContent lines 689-756.
  let mut variant: Option<&str> = font.as_deref().map(unicode::unicode_mathvariant);
  let mut attrs = HashMap::default();

  // Single char mi: italic is default, "normal" must be stated explicitly
  // Perl L717-719
  if target_tag == "m:mi" && text.chars().count() == 1 {
    if variant == Some("italic") {
      variant = None; // defaults to italic
    } else if variant.is_none() && font.is_none() {
      // no font at all — italic is MathML default for single-char mi
    } else if variant.is_none() {
      variant = Some("normal"); // font present but no recognized variant → say "normal"
    }
  } else if font.is_some() && variant == Some("normal") {
    // "normal" is default for non-mi tokens; omit
    variant = None;
  }

  // Plane 1 Unicode conversion.
  // Port of Perl stylizeContent lines 729-741.
  // When plane1=true (default), convert text to Mathematical Alphanumeric Symbols
  // and clear the mathvariant (the character itself carries the style).
  if let Some(v) = variant {
    if target_tag != "m:mtext" {
      if let Some(u_text) = unicode::unicode_convert(&text, v) {
        if !u_text.is_empty() || text.is_empty() {
          text = u_text;
          variant = None; // Plane 1 char carries the style; no mathvariant needed
        }
      }
    }
  }

  // Apply remaining variant (not cleared by plane1 conversion)
  if let Some(v) = variant {
    if target_tag == "m:mi" && text.chars().count() == 1 {
      if v != "italic" {
        attrs.insert("mathvariant".to_string(), v.to_string());
      }
    } else if v != "normal" {
      attrs.insert("mathvariant".to_string(), v.to_string());
    }
  } else if target_tag == "m:mi" && text.chars().count() > 1 && font.is_none() {
    // Multi-char mi without font → normal
    attrs.insert("mathvariant".to_string(), "normal".to_string());
  }

  // Invisible/format-only text needs no styling attributes.
  // Port of Perl L743-744: if ($text =~ /^\p{Format}*$/)
  let is_format_only = text.chars().all(|c| {
    use std::char;
    matches!(char::from_u32(c as u32), Some(ch) if ch.is_control())
      || matches!(c,
        '\u{200B}'..='\u{200F}' | '\u{2028}'..='\u{202F}'
        | '\u{2060}'..='\u{2064}' | '\u{FEFF}' | '\u{00AD}')
  });
  if is_format_only {
    variant = None;
    // Don't clear font/color etc from attrs — they weren't added yet
  }

  // Font-based CSS class fallbacks.
  // Port of Perl L746-756: only when variant was NOT converted to plane1.
  let mut class_val = class;
  if let Some(ref f) = font {
    if !is_format_only {
      if f.contains("caligraphic") {
        let c = class_val.as_deref().unwrap_or("");
        class_val = Some(if c.is_empty() {
          "ltx_font_mathcaligraphic".to_string()
        } else {
          format!("{} ltx_font_mathcaligraphic", c)
        });
      } else if f.contains("script") {
        let c = class_val.as_deref().unwrap_or("");
        class_val = Some(if c.is_empty() {
          "ltx_font_mathscript".to_string()
        } else {
          format!("{} ltx_font_mathscript", c)
        });
      } else if f.contains("fraktur") && text.chars().all(|c| "+-0123456789.".contains(c)) {
        // Perl L751: fraktur number → oldstyle class
        let c = class_val.as_deref().unwrap_or("");
        class_val = Some(if c.is_empty() {
          "ltx_font_oldstyle".to_string()
        } else {
          format!("{} ltx_font_oldstyle", c)
        });
      } else if f.contains("smallcaps") {
        let c = class_val.as_deref().unwrap_or("");
        class_val = Some(if c.is_empty() {
          "ltx_font_smallcaps".to_string()
        } else {
          format!("{} ltx_font_smallcaps", c)
        });
      } else if let Some(v) = variant {
        if v != "normal" {
          // Perl L755-756: leftover mathvariant → CSS fallback
          let c = class_val.as_deref().unwrap_or("");
          class_val = Some(if c.is_empty() {
            format!("ltx_mathvariant_{}", v)
          } else {
            format!("{} ltx_mathvariant_{}", c, v)
          });
        }
      }
    }
  }
  if let Some(cv) = class_val {
    if !cv.is_empty() {
      attrs.insert("class".to_string(), cv);
    }
  }

  // Operator-specific properties (mo only) — from MathML Core operator dictionary
  if target_tag == "m:mo" {
    let dict_props = operator_dictionary::opdict_lookup(&text, &role);

    // Implied role-based properties
    let is_fence = matches!(role.as_str(), "OPEN" | "CLOSE" | "MIDDLE");
    let is_sep = role == "PUNCT";
    let is_large = matches!(role.as_str(), "SUMOP" | "INTOP");
    let is_move = matches!(role.as_str(), "SUMOP" | "INTOP" | "BIGOP" | "LIMITOP");
    let is_symm = is_large || text == "/";

    // Only set attributes that differ from the operator dictionary defaults
    // (matching Perl's xor-based attribute generation)
    let stretchy = node.get_attribute("stretchy").as_deref() == Some("true");
    if stretchy != dict_props.stretchy {
      attrs.insert(
        "stretchy".to_string(),
        if stretchy { "true" } else { "false" }.to_string(),
      );
    }
    if is_fence != dict_props.fence {
      attrs.insert(
        "fence".to_string(),
        if is_fence { "true" } else { "false" }.to_string(),
      );
    }
    if is_sep != dict_props.separator {
      attrs.insert(
        "separator".to_string(),
        if is_sep { "true" } else { "false" }.to_string(),
      );
    }
    if is_large != dict_props.largeop {
      attrs.insert(
        "largeop".to_string(),
        if is_large { "true" } else { "false" }.to_string(),
      );
    }
    if is_large {
      attrs.insert("_largeop".to_string(), "1".to_string()); // For needsMathStyle
    }
    if is_symm && !dict_props.symmetric && (stretchy || dict_props.stretchy) {
      attrs.insert("symmetric".to_string(), "true".to_string());
    }
    if is_move {
      let pos = node.get_attribute("scriptpos").unwrap_or_default();
      if pos.starts_with("mid") {
        attrs.insert("movablelimits".to_string(), "false".to_string());
      }
    }

    // Store dictionary spacing for later spacing resolution
    attrs.insert("_lspace".to_string(), format!("{}em", dict_props.lspace));
    attrs.insert("_rspace".to_string(), format!("{}em", dict_props.rspace));
  }

  // Color
  if let Some(c) = color {
    attrs.insert("mathcolor".to_string(), c);
  }
  if let Some(bg) = bgcolor {
    attrs.insert("mathbackground".to_string(), bg);
  }
  if let Some(op) = opacity {
    let style_val = format!("opacity:{}", op);
    attrs.insert("style".to_string(), style_val);
  }

  // Size
  if let Some(s) = size {
    if s != "100%" {
      attrs.insert("mathsize".to_string(), s);
    }
  }

  // Href and title (tokens only)
  if is_token {
    if let Some(h) = href {
      attrs.insert("href".to_string(), h);
    }
    if let Some(t) = title {
      attrs.insert("title".to_string(), t);
    }
  }

  (text, attrs)
}

/// Convert an XMHint spacing attribute to em value.
///
/// Port of `getXMHintSpacing`.
pub fn get_xm_hint_spacing(width: &str) -> f64 {
  let trimmed = width.trim();
  if let Some((num_str, unit)) = trimmed
    .rfind(|c: char| c.is_ascii_digit() || c == '.')
    .map(|i| (&trimmed[..=i], trimmed[i + 1..].trim()))
  {
    let num: f64 = num_str.parse().unwrap_or(0.0);
    match unit {
      "em" => num,
      "mu" => num / 18.0,
      "pt" => num / 10.0, // Assuming 10pt font
      _ => 0.0,
    }
  } else {
    0.0
  }
}

/// Check if a MathML result needs displaystyle to be set.
///
/// Port of `needsMathstyle`.
/// Checks for large operators (\_largeop attribute) in the tree.
pub fn needs_mathstyle(node: &NodeData) -> bool {
  match node {
    NodeData::Element { attributes, children, .. } => {
      if let Some(attrs) = attributes {
        if attrs.contains_key("_largeop") {
          return true;
        }
      }
      children.iter().any(needs_mathstyle)
    },
    _ => false,
  }
}

/// Find an inherited attribute by walking up the LaTeXML ancestor chain.
///
/// Port of `find_inherited_attribute`.
pub fn find_inherited_attribute(
  _doc: &PostDocument,
  node: &Node,
  attribute: &str,
) -> Option<String> {
  let mut current = Some(node.clone());
  while let Some(ref n) = current {
    if let Some(ns) = n.get_namespace() {
      if ns.get_href() != crate::document::LTX_NSURI {
        break; // Stop at non-LaTeXML elements
      }
    }
    if let Some(val) = n.get_attribute(attribute) {
      return Some(val);
    }
    current = n.get_parent();
  }
  None
}

// ======================================================================
// DefMathML converter dispatch table
//
// Port of the `%MMLTable_P` / `%MMLTable_C` lookup tables and the
// 800+ lines of DefMathML declarations in MathML.pm.
//
// The Perl pattern is:
//   DefMathML("Mode:Role:Meaning", \&pmml_handler, \&cmml_handler);
// Lookup tries: "Mode:Role:Meaning", "Mode:?:Meaning", "Mode:Role:?", "Mode:?:?"
//
// In Rust, we encode this as a static table of known role→tag mappings
// and meaning→element mappings, and the actual dispatch happens in
// presentation.rs::pmml_apply() and content.rs::cmml().

/// Presentation MathML tag for a token role.
///
/// Port of Token:ROLE:? DefMathML declarations.
/// These map roles to their default MathML element type.
pub fn pmml_tag_for_role(role: &str) -> &'static str {
  match role {
    // Operators → m:mo
    "PUNCT" | "PERIOD" | "OPEN" | "CLOSE" | "MIDDLE" | "VERTBAR" | "ARROW" | "OVERACCENT"
    | "UNDERACCENT" | "ADDOP" | "MULOP" | "BINOP" | "RELOP" | "METARELOP" | "MODIFIEROP"
    | "COMPOSEOP" | "APPLYOP" | "OPERATOR" | "SUPOP" | "POSTFIX" | "DIFFOP" => "m:mo",
    // Big operators → m:mo (with largeop)
    "BIGOP" | "SUMOP" | "INTOP" | "LIMITOP" => "m:mo",
    // Functions → m:mi (but rendered as operator names)
    "FUNCTION" | "OPFUNCTION" | "TRIGFUNCTION" => "m:mi",
    // Numbers → m:mn
    "NUMBER" => "m:mn",
    // Identifiers → m:mi (default)
    _ => "m:mi",
  }
}

/// Whether a role should use the "big operator" presentation style.
///
/// Port of `Token:INTOP:?` → `\&pmml_bigop`, `Token:SUMOP:?` → `\&pmml_bigop`, etc.
pub fn is_bigop_role(role: &str) -> bool { matches!(role, "INTOP" | "SUMOP" | "BIGOP" | "LIMITOP") }

/// Presentation handler type for XMApp nodes.
///
/// Port of the `Apply:ROLE:?` entries in DefMathML.
/// Returns the handler category that presentation.rs should use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ApplyHandler {
  /// Infix: op between args (ADDOP, MULOP, RELOP, etc.)
  Infix,
  /// Script: sub/superscript (SUPERSCRIPTOP, SUBSCRIPTOP)
  Script,
  /// Big operator with possible limits (SUMOP, INTOP, BIGOP, LIMITOP)
  Summation,
  /// Prefix: op before args (DIFFOP, default)
  Prefix,
  /// Postfix: args then op (POSTFIX)
  Postfix,
  /// Fraction (FRACOP)
  Fraction,
  /// Over accent (OVERACCENT)
  OverAccent,
  /// Under accent (UNDERACCENT)
  UnderAccent,
  /// Enclose (ENCLOSE)
  Enclose,
  /// Generic application (default)
  Generic,
}

/// Determine the presentation handler for an XMApp based on operator role.
///
/// Port of the `Apply:ROLE:?` DefMathML declarations.
pub fn apply_handler_for_role(role: &str) -> ApplyHandler {
  match role {
    "ADDOP" | "MULOP" | "BINOP" | "RELOP" | "METARELOP" | "ARROW" | "COMPOSEOP" | "MODIFIEROP"
    | "MIDDLE" => ApplyHandler::Infix,
    "SUPERSCRIPTOP" | "SUBSCRIPTOP" => ApplyHandler::Script,
    "SUMOP" | "INTOP" | "BIGOP" | "LIMITOP" => ApplyHandler::Summation,
    "DIFFOP" => ApplyHandler::Prefix,
    "POSTFIX" => ApplyHandler::Postfix,
    "FRACOP" => ApplyHandler::Fraction,
    "OVERACCENT" => ApplyHandler::OverAccent,
    "UNDERACCENT" => ApplyHandler::UnderAccent,
    "ENCLOSE" => ApplyHandler::Enclose,
    _ => ApplyHandler::Generic,
  }
}

/// Determine the presentation handler for a specific meaning.
///
/// Port of the `Apply:?:meaning` DefMathML declarations.
/// Returns Some(handler) if a meaning-specific handler exists, None for role-based fallback.
pub fn apply_handler_for_meaning(meaning: &str) -> Option<ApplyHandler> {
  match meaning {
    "square-root" | "nth-root" => None, // Handled specially in pmml_apply
    "formulae" | "multirelation" => Some(ApplyHandler::Infix),
    "limit-from" | "annotated" => Some(ApplyHandler::Prefix),
    "continued-fraction" => Some(ApplyHandler::Fraction),
    _ => None,
  }
}

/// Known Content MathML elements for specific meanings.
///
/// Port of the `Token:?:meaning` content DefMathML declarations.
/// See also content.rs::meaning_to_cmml_element() for the full list.
pub fn cmml_element_for_meaning(meaning: &str) -> Option<&'static str> {
  content::meaning_to_cmml_element_pub(meaning)
}

/// Whether an XMApp with this meaning has a dedicated Content MathML structure.
///
/// Port of the `Apply:?:meaning` content DefMathML declarations.
pub fn has_dedicated_cmml_structure(meaning: &str) -> bool {
  matches!(
    meaning,
    "square-root"
      | "nth-root"
      | "set"
      | "list"
      | "open-interval"
      | "closed-interval"
      | "closed-open-interval"
      | "open-closed-interval"
      | "formulae"
      | "multirelation"
      | "cases"
  )
}

// ======================================================================
// Presentation MathML helpers
//
// Port of `pmml_maybe_resize`, `pmml_row`, `pmml_parenthesize`,
// `pmml_text_aux`, `filter_row` from MathML.pm.

/// Possibly wrap result in mpadded if width/height/depth are specified.
///
/// Port of `pmml_maybe_resize`.
pub fn pmml_maybe_resize(node: &Node, result: NodeData) -> NodeData {
  let width = node.get_attribute("width");
  let height = node.get_attribute("height");
  let depth = node.get_attribute("depth");
  let xoff = node.get_attribute("xoffset");
  let yoff = node.get_attribute("yoffset");
  let role = node.get_attribute("role");
  let class = node.get_attribute("class");

  let mut result = result;

  // Special case: stretchy arrows with specified width
  if width.is_some()
    && role.as_deref() == Some("ARROW")
    && class
      .as_deref()
      .map(|c| c.contains("ltx_horizontally_stretchy"))
      .unwrap_or(false)
  {
    if let Some(ref w) = width {
      result = NodeData::Element {
        tag:        "m:mover".to_string(),
        attributes: None,
        children:   vec![result, NodeData::Element {
          tag:        "m:mspace".to_string(),
          attributes: Some(HashMap::from_iter([("width".to_string(), w.clone())])),
          children:   vec![],
        }],
      };
      return result;
    }
  }

  // Wrap in mpadded if dimensions specified
  if width.is_some() || height.is_some() || depth.is_some() || xoff.is_some() || yoff.is_some() {
    // Convert result to mpadded (or wrap in one)
    let (tag, attrs, children) = match result {
      NodeData::Element {
        ref tag,
        ref attributes,
        ref children,
      } if tag == "m:mpadded" => (
        tag.clone(),
        attributes.clone().unwrap_or_default(),
        children.clone(),
      ),
      NodeData::Element {
        ref tag,
        ref attributes,
        ref children,
      } if tag == "m:mrow" => (
        "m:mpadded".to_string(),
        attributes.clone().unwrap_or_default(),
        children.clone(),
      ),
      _ => ("m:mpadded".to_string(), HashMap::default(), vec![
        result.clone(),
      ]),
    };

    let mut padded_attrs = attrs;
    if let Some(w) = width {
      padded_attrs.insert("width".to_string(), w);
    }
    if let Some(h) = height {
      padded_attrs.insert("height".to_string(), h);
    }
    if let Some(d) = depth {
      padded_attrs.insert("depth".to_string(), d);
    }
    if let Some(x) = xoff {
      padded_attrs.insert("lspace".to_string(), x);
    }
    if let Some(y) = yoff {
      padded_attrs.insert("voffset".to_string(), y);
    }

    result = NodeData::Element {
      tag,
      attributes: Some(padded_attrs),
      children,
    };
  }

  // Add framing if specified
  if let Some(frame) = node.get_attribute("framed") {
    let frame_class = format!("ltx_framed_{}", frame);
    if let NodeData::Element { attributes, .. } = &mut result {
      let attrs = attributes.get_or_insert_with(HashMap::default);
      let c = attrs.get("class").cloned().unwrap_or_default();
      attrs.insert(
        "class".to_string(),
        if c.is_empty() {
          frame_class
        } else {
          format!("{} {}", c, frame_class)
        },
      );
      if let Some(color) = node.get_attribute("framecolor") {
        let s = attrs.get("style").cloned().unwrap_or_default();
        let style = format!("border-color: {}", color);
        attrs.insert(
          "style".to_string(),
          if s.is_empty() {
            style
          } else {
            format!("{}; {}", s, style)
          },
        );
      }
    }
  }

  result
}

/// Wrap items in an mrow, filtering out ignorable items.
///
/// Port of `pmml_row` + `filter_row`.
pub fn pmml_row(items: Vec<NodeData>) -> NodeData {
  // Filter out ignorable items (those with _ignorable attribute)
  let filtered: Vec<NodeData> = items
    .into_iter()
    .filter(|item| match item {
      NodeData::Element { attributes, .. } => {
        if let Some(attrs) = attributes {
          !attrs.contains_key("_ignorable")
        } else {
          true
        }
      },
      _ => true,
    })
    .collect();

  if filtered.len() == 1 {
    filtered.into_iter().next().unwrap()
  } else {
    NodeData::Element {
      tag:        "m:mrow".to_string(),
      attributes: None,
      children:   filtered,
    }
  }
}

/// Parenthesize an expression with open/close delimiters.
///
/// Port of `pmml_parenthesize`.
pub fn pmml_parenthesize(item: NodeData, open: Option<&str>, close: Option<&str>) -> NodeData {
  if open.is_none() && close.is_none() {
    return item;
  }

  let mut children = Vec::new();
  if let Some(o) = open {
    children.push(NodeData::Element {
      tag:        "m:mo".to_string(),
      attributes: Some(HashMap::from_iter([
        ("fence".to_string(), "true".to_string()),
        ("stretchy".to_string(), "true".to_string()),
      ])),
      children:   vec![NodeData::Text(o.to_string())],
    });
  }
  children.push(item);
  if let Some(c) = close {
    children.push(NodeData::Element {
      tag:        "m:mo".to_string(),
      attributes: Some(HashMap::from_iter([
        ("fence".to_string(), "true".to_string()),
        ("stretchy".to_string(), "true".to_string()),
      ])),
      children:   vec![NodeData::Text(c.to_string())],
    });
  }

  NodeData::Element {
    tag: "m:mrow".to_string(),
    attributes: None,
    children,
  }
}

/// Punctuate a list of items with separators.
///
/// Port of `pmml_punctuate`.
pub fn pmml_punctuate(separators: &str, items: Vec<NodeData>) -> NodeData {
  if items.is_empty() {
    return NodeData::Element {
      tag:        "m:mrow".to_string(),
      attributes: None,
      children:   vec![],
    };
  }

  let mut result = Vec::new();
  let mut sep_chars: Vec<char> = separators.chars().collect();
  let last_sep = if sep_chars.is_empty() {
    ','
  } else {
    *sep_chars.last().unwrap()
  };

  let mut iter = items.into_iter();
  result.push(iter.next().unwrap());

  for item in iter {
    let sep = if sep_chars.is_empty() {
      last_sep
    } else {
      sep_chars.remove(0)
    };
    result.push(NodeData::Element {
      tag:        "m:mo".to_string(),
      attributes: Some(HashMap::from_iter([(
        "separator".to_string(),
        "true".to_string(),
      )])),
      children:   vec![NodeData::Text(sep.to_string())],
    });
    result.push(item);
  }

  pmml_row(result)
}

/// Convert a text node within XMText to Presentation MathML.
///
/// Port of `pmml_text_aux`.
/// Handles text nodes, element nodes (including nested Math),
/// and preserves font/color attributes through the conversion.
pub fn pmml_text_aux(doc: &PostDocument, node: &Node) -> Vec<NodeData> {
  use libxml::tree::NodeType;

  match node.get_type() {
    Some(NodeType::TextNode) => {
      let text = node.get_content();
      // Replace leading/trailing whitespace with NBSP
      let text = text.trim_start().to_string();
      let text = if text.is_empty() {
        "\u{00A0}".to_string()
      } else {
        let mut t = text;
        if t.starts_with(char::is_whitespace) {
          t = format!("\u{00A0}{}", t.trim_start());
        }
        if t.ends_with(char::is_whitespace) {
          t = format!("{}\u{00A0}", t.trim_end());
        }
        t
      };
      vec![NodeData::Element {
        tag:        "m:mtext".to_string(),
        attributes: None,
        children:   vec![NodeData::Text(text)],
      }]
    },
    Some(NodeType::ElementNode) => {
      let tag = doc.get_qname(node).unwrap_or_default();
      match tag.as_str() {
        "ltx:Math" => {
          // Nested math: convert XMath if present
          match doc.findnode_at("ltx:XMath", node) {
            Some(xmath) => {
              vec![presentation::convert_to_pmml(doc, &xmath)]
            },
            _ => {
              vec![]
            },
          }
        },
        "ltx:text" => {
          // Recurse on children
          let mut results = Vec::new();
          if let Some(child) = node.get_first_child() {
            let mut current = Some(child);
            while let Some(ref c) = current {
              results.extend(pmml_text_aux(doc, c));
              current = c.get_next_sibling();
            }
          }
          results
        },
        "ltx:picture" => {
          // Picture in text: wrap in mtext. Eagerly materialize the picture
          // subtree into owned NodeData so the result is not tied to the
          // source node's libxml2 lifetime. Perl: MathProcessor.pm
          // convertXMTextContent (Post.pm L456-489). A lazy
          // `NodeData::XmlNode(node.clone())` here SIGSEGVs in
          // `add_xml_node` once the parent XMath is unlinked (its children
          // are stripped into a detached document fragment and later
          // accesses via the stale rust-libxml wrapper dereference freed
          // memory — reproducible on 0710.1208 / 1110.2158 / 1605.07431).
          vec![NodeData::Element {
            tag:        "m:mtext".to_string(),
            attributes: None,
            children:   convert_xm_text_content(doc, node, true),
          }]
        },
        _ => {
          // Unknown element (e.g. ltx:ref, ltx:bibref, ltx:inline-block,
          // …): preserve the raw subtree inside the mtext so the XSLT
          // can transform it (ltx:ref → HTML <a>, etc.). Perl
          // `pmml_text_aux` (MathML.pm L1063-1073) clones the whole
          // node into the returned mtext; we eagerly materialize an
          // owned subtree, threading `doc` through so URI→prefix
          // resolution recovers the canonical `ltx:` prefix on
          // default-namespace elements.
          let cloned = rebuild_text_subtree_with_doc(node, true, Some(doc))
            .unwrap_or_else(|| NodeData::Text("\u{00A0}".to_string()));
          vec![NodeData::Element {
            tag:        "m:mtext".to_string(),
            attributes: None,
            children:   vec![cloned],
          }]
        },
      }
    },
    _ => vec![],
  }
}

/// Eagerly materialize an XMText-or-picture subtree into owned NodeData.
///
/// Port of `LaTeXML::Post::MathProcessor::convertXMTextContent`
/// (Post.pm L456-489). Walks `node` recursively and rebuilds the subtree
/// as owned NodeData, so downstream consumers do not depend on the
/// source node's libxml2 lifetime. Internal `_*` attributes and stray
/// `xml:id` are dropped (Perl mirrors this); `fragid` would be remapped
/// to a fresh id in Perl but MathML::Presentation does not carry a
/// processor-level id suffix through this path, so we drop it too and
/// let the surrounding MathML ids govern.
///
/// When `convert_spaces` is true, leading/trailing whitespace on text
/// nodes is replaced with NBSP so the rendered MathML does not collapse
/// the space. Nested `ltx:Math` would in Perl re-enter the MathML
/// converter; we preserve it as a plain element subtree here (same
/// limitation as the enclosing pmml_text_aux path: nested math in
/// mtext is uncommon, and the XSLT stage can still render it).
pub fn convert_xm_text_content(
  doc: &PostDocument,
  node: &Node,
  convert_spaces: bool,
) -> Vec<NodeData> {
  node
    .get_child_nodes()
    .iter()
    .filter_map(|c| rebuild_text_subtree_with_doc(c, convert_spaces, Some(doc)))
    .collect()
}

/// Rebuild a libxml2 subtree into owned `NodeData`, dropping internal
/// `_*`, `xml:id`, and `fragid` attributes. Shared between
/// `convert_xm_text_content` (Perl `convertXMTextContent`,
/// Post.pm L456-489) and `pmml_text_aux` for cases where Perl
/// calls `cloneNode($node, 'nest')` (MathML.pm L1073) — i.e. when an
/// unhandled element like `ltx:ref` appears inside a text-mode
/// fragment and must survive into the output so the XSLT can
/// transform it (e.g. `ltx:ref` → `<a>`).
pub fn rebuild_text_subtree(node: &Node, convert_spaces: bool) -> Option<NodeData> {
  rebuild_text_subtree_with_doc(node, convert_spaces, None)
}

/// Same as `rebuild_text_subtree`, but consults the post-document's
/// namespace map to resolve elements whose source `xmlns="…"` carries
/// an empty prefix. `add_nodes` only emits elements whose tag is
/// `prefix:local`; without a prefix the element is dropped with a
/// `malformed:namespace` warning. libxml2 reports an empty
/// `Namespace::get_prefix()` for the default-namespace branch even
/// when the doc has a `ltx:` prefix declared elsewhere, so we
/// reverse-lookup the URI in `PostDocument::namespaces` to recover
/// the canonical prefix.
pub fn rebuild_text_subtree_with_doc(
  node: &Node,
  convert_spaces: bool,
  doc: Option<&PostDocument>,
) -> Option<NodeData> {
  use libxml::tree::NodeType;
  match node.get_type() {
    Some(NodeType::TextNode) => {
      let mut text = node.get_content();
      if convert_spaces {
        if text.starts_with(char::is_whitespace) {
          text = format!("\u{00A0}{}", text.trim_start());
        }
        if text.ends_with(char::is_whitespace) {
          text = format!("{}\u{00A0}", text.trim_end());
        }
      }
      Some(NodeData::Text(text))
    },
    Some(NodeType::ElementNode) => {
      let tag = {
        let local = node.get_name();
        match node.get_namespace() {
          Some(ns) => {
            let prefix = ns.get_prefix();
            if !prefix.is_empty() {
              format!("{prefix}:{local}")
            } else {
              // Default-namespace element. add_nodes won't accept a
              // tag without prefix — reverse-resolve URI → prefix
              // via the post-document's namespace map.
              let uri = ns.get_href();
              match doc.and_then(|d| {
                d.namespaces
                  .iter()
                  .find(|(p, u)| !p.is_empty() && **u == uri)
                  .map(|(p, _)| p.clone())
              }) {
                Some(p) => format!("{p}:{local}"),
                None => local,
              }
            }
          },
          None => local,
        }
      };
      // Copy attributes, skipping internal `_*`, `xml:id`, and `fragid`.
      // Matches Perl convertXMTextContent (Post.pm L479-483); the
      // `fragid → xml:id` remap requires the MathProcessor's IDSuffix
      // which this helper does not receive — drop both here rather
      // than forge a wrong id.
      let mut attrs: HashMap<String, String> = HashMap::default();
      for (k, v) in node.get_attributes() {
        if k.starts_with('_') || k == "xml:id" || k == "fragid" {
          continue;
        }
        attrs.insert(k, v);
      }
      let children: Vec<NodeData> = node
        .get_child_nodes()
        .iter()
        .filter_map(|c| rebuild_text_subtree_with_doc(c, convert_spaces, doc))
        .collect();
      Some(NodeData::Element {
        tag,
        attributes: if attrs.is_empty() { None } else { Some(attrs) },
        children,
      })
    },
    _ => None,
  }
}

/// Unwrap an mrow if it has no attributes.
///
/// Port of `pmml_unrow`.
pub fn pmml_unrow(mml: NodeData) -> Vec<NodeData> {
  match mml {
    NodeData::Element {
      ref tag,
      ref attributes,
      ref children,
    } if tag == "m:mrow" && attributes.as_ref().map(|a| a.is_empty()).unwrap_or(true) => {
      children.clone()
    },
    _ => vec![mml],
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_style_step() {
    assert_eq!(style_step("display"), "text");
    assert_eq!(style_step("text"), "script");
    assert_eq!(style_step("script"), "scriptscript");
    assert_eq!(style_step("scriptscript"), "scriptscript");
  }

  #[test]
  fn test_pmml_tag_for_role() {
    assert_eq!(pmml_tag_for_role("NUMBER"), "m:mn");
    assert_eq!(pmml_tag_for_role("ID"), "m:mi");
    assert_eq!(pmml_tag_for_role("ADDOP"), "m:mo");
    assert_eq!(pmml_tag_for_role("FUNCTION"), "m:mi");
    assert_eq!(pmml_tag_for_role("SUMOP"), "m:mo");
  }

  #[test]
  fn test_apply_handler_for_role() {
    assert_eq!(apply_handler_for_role("ADDOP"), ApplyHandler::Infix);
    assert_eq!(
      apply_handler_for_role("SUPERSCRIPTOP"),
      ApplyHandler::Script
    );
    assert_eq!(apply_handler_for_role("FRACOP"), ApplyHandler::Fraction);
    assert_eq!(
      apply_handler_for_role("OVERACCENT"),
      ApplyHandler::OverAccent
    );
    assert_eq!(apply_handler_for_role("SUMOP"), ApplyHandler::Summation);
    assert_eq!(apply_handler_for_role("FUNCTION"), ApplyHandler::Generic);
  }

  #[test]
  fn test_is_bigop_role() {
    assert!(is_bigop_role("SUMOP"));
    assert!(is_bigop_role("INTOP"));
    assert!(!is_bigop_role("ADDOP"));
    assert!(!is_bigop_role("ID"));
  }

  #[test]
  fn test_encoding_for_mimetype() {
    assert_eq!(
      encoding_for_mimetype("application/mathml-presentation+xml"),
      "MathML-Presentation"
    );
    assert_eq!(
      encoding_for_mimetype("application/mathml-content+xml"),
      "MathML-Content"
    );
    assert_eq!(encoding_for_mimetype("image/svg+xml"), "SVG1.1");
    assert_eq!(encoding_for_mimetype("text/plain"), "text/plain");
  }

  #[test]
  fn test_pmml_row_single() {
    let items = vec![NodeData::Text("x".to_string())];
    let result = pmml_row(items);
    match result {
      NodeData::Text(s) => assert_eq!(s, "x"),
      _ => panic!("Expected Text, got Element"),
    }
  }

  #[test]
  fn test_pmml_row_multiple() {
    let items = vec![
      NodeData::Text("x".to_string()),
      NodeData::Text("+".to_string()),
      NodeData::Text("y".to_string()),
    ];
    let result = pmml_row(items);
    match result {
      NodeData::Element { tag, children, .. } => {
        assert_eq!(tag, "m:mrow");
        assert_eq!(children.len(), 3);
      },
      _ => panic!("Expected Element"),
    }
  }

  #[test]
  fn test_pmml_parenthesize() {
    let item = NodeData::Text("x".to_string());
    let result = pmml_parenthesize(item.clone(), Some("("), Some(")"));
    match result {
      NodeData::Element { tag, children, .. } => {
        assert_eq!(tag, "m:mrow");
        assert_eq!(children.len(), 3); // open, item, close
      },
      _ => panic!("Expected mrow"),
    }

    // No parens → pass through
    let result2 = pmml_parenthesize(item, None, None);
    match result2 {
      NodeData::Text(s) => assert_eq!(s, "x"),
      _ => panic!("Expected passthrough"),
    }
  }
}
