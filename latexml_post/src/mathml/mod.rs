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
  /// Whether to remap styled alphanumerics to Unicode's Plane-1 Mathematical
  /// Alphanumeric Symbols. Perl `$$MATHPROCESSOR{plane1}`, default on
  /// (`preprocess` L70); `--noplane1` keeps ASCII + a `mathvariant` attribute.
  plane1:          bool,
  /// Perl `$$MATHPROCESSOR{hackplane1}` (`--hackplane1`): remap only the
  /// variants in `plane1_hackable`, and to the simpler variant named there.
  /// Implies `plane1` (Perl L71).
  hack_plane1:     bool,
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
  /// Whether to add intent=":literal" on all `<math>` elements.
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
      hack_plane1:     false,
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
      hack_plane1:     false,
      linebreaking:    false,
      line_width:      80,
      keep_xmath:      false,
      invisible_times: true,
      mathtex:         false,
      intent_literal:  false,
      secondaries:     Vec::new(),
    }
  }

  /// Enable intent=":literal" on all `<math>` elements.
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

  /// Set the Plane-1 remapping mode. `plane1` off keeps ASCII text plus a
  /// `mathvariant` attribute; `hack_plane1` remaps only the poorly-supported
  /// variants and implies `plane1`. Perl `--plane1` / `--hackplane1`.
  pub fn with_plane1(mut self, plane1: bool, hack_plane1: bool) -> Self {
    self.plane1 = plane1;
    self.hack_plane1 = hack_plane1;
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
  /// conversions are merged into one `<m:semantics>` by
  /// [`combine_parallel`](MathProcessor::combine_parallel)
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
    presentation::set_plane1(self.plane1, self.hack_plane1);

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

      // Image fallback (Perl L81-87): when the `--mathimages` post-processor has
      // rendered this formula, advertise the bitmap so a renderer without MathML
      // support can show it. `altimg-valign` carries the baseline offset and Perl
      // NEGATES the depth ("Note the sign!"): `imagedepth="5"` → `-5px`.
      if let Some(src) = math.get_attribute("imagesrc").filter(|s| !s.is_empty()) {
        attrs.insert("altimg".to_string(), src);
        // Perl appends 'px' unconditionally once `imagesrc` is present, so a
        // missing `imagewidth` yields the literal `"px"` rather than omitting the
        // attribute. Mirrored: `--mathimages` always sets both dimensions, so the
        // quirk is unreachable in practice, and diverging here would cost
        // byte-parity for no gain.
        attrs.insert(
          "altimg-width".to_string(),
          format!("{}px", math.get_attribute("imagewidth").unwrap_or_default()),
        );
        attrs.insert(
          "altimg-height".to_string(),
          format!(
            "{}px",
            math.get_attribute("imageheight").unwrap_or_default()
          ),
        );
        // Perl-falsy depth (absent, empty, or "0") omits the attribute entirely
        // rather than emitting a bare "-px" or "-0px".
        if let Some(depth) = math
          .get_attribute("imagedepth")
          .filter(|d| !d.is_empty() && d != "0")
        {
          attrs.insert("altimg-valign".to_string(), format!("-{depth}px"));
        }
      }

      // RDFa (Perl L88-90): the Math element's own value, else the XMath's.
      // Perl's `$math->getAttribute($_) || $xmath->getAttribute($_)` is a TRUTH
      // test, so an EMPTY value on the Math falls through to the XMath rather
      // than shadowing it; the trailing `$val ? … : ()` then drops the pair if
      // neither had one.
      for key in [
        "about", "resource", "property", "rel", "rev", "typeof", "datatype", "content",
      ] {
        let non_empty = |n: &Node| n.get_attribute(key).filter(|v| !v.is_empty());
        if let Some(val) = non_empty(&math).or_else(|| non_empty(xmath)) {
          attrs.insert(key.to_string(), val);
        }
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

/// Perl's `%attr` for `pmml_text_aux` — the presentation attributes an
/// enclosing `ltx:*` element contributes to the `m:mtext` elements below it.
///
/// Perl threads an open hash (`MathML.pm` L1029, L1041-1045), but only these
/// five keys are ever written on this path, so they are named fields. The other
/// keys `stylizeContent` consults (`role`, `class`, `cssstyle`, `href`, `title`,
/// `stretchy`) are never set by this caller — they come off the node itself.
#[derive(Clone, Default, Debug)]
pub struct TextAttrs {
  font:            Option<String>,
  fontsize:        Option<String>,
  color:           Option<String>,
  backgroundcolor: Option<String>,
  opacity:         Option<String>,
}

impl TextAttrs {
  /// Overlay `node`'s own presentation attributes, as Perl `pmml_text_aux`
  /// L1041-1045 does: an attribute PRESENT on the element wins over what was
  /// inherited; an absent one leaves the inherited value in place.
  fn overlay(&self, node: &Node) -> Self {
    let pick = |cur: &Option<String>, name: &str| -> Option<String> {
      node
        .get_attribute(name)
        .filter(|v| !v.is_empty())
        .or_else(|| cur.clone())
    };
    Self {
      font:            pick(&self.font, "font"),
      fontsize:        pick(&self.fontsize, "fontsize"),
      color:           pick(&self.color, "color"),
      backgroundcolor: pick(&self.backgroundcolor, "backgroundcolor"),
      opacity:         pick(&self.opacity, "opacity"),
    }
  }
}

/// The first direct element child of `node` with the given namespace URI and
/// local name — Perl's `findnode('<prefix>:<name>', $node)` without depending on
/// the prefix being registered in the document's XPath context.
fn element_child_named(node: &Node, ns_uri: &str, local: &str) -> Option<Node> {
  let mut current = node.get_first_child();
  while let Some(c) = current {
    if c.get_type() == Some(libxml::tree::NodeType::ElementNode)
      && c.get_name() == local
      && c.get_namespace().map(|ns| ns.get_href()).as_deref() == Some(ns_uri)
    {
      return Some(c);
    }
    current = c.get_next_sibling();
  }
  None
}

/// Perl's `\p{Format}` over the codepoints that occur in math content — the
/// same approximation `pmml_token_inner` uses, so the two arms agree.
fn is_format_char(c: char) -> bool {
  matches!(c,
    '\u{00AD}' | '\u{200B}'..='\u{200F}' | '\u{2060}'..='\u{2064}' | '\u{FEFF}')
}

/// `stylizeContent` (`MathML.pm` L672-828) for the `$tag eq 'm:mtext'` case —
/// the styling half of the `pmml_text_aux` path.
///
/// **The token half of the same Perl function lives in
/// `presentation::pmml_token_inner`**, the golden-guarded faithful copy of the
/// `m:mi`/`m:mo`/`m:mn` branches (operator dictionary, plane-1 remapping,
/// stretch/size interplay). Perl is one function; the Rust split is by target
/// tag, and neither half should grow the other's branches. (This function used
/// to be a whole second copy of `stylizeContent`, tag-generic and dead — nothing
/// called it, so its `m:mo` arm had drifted out of parity unnoticed.)
///
/// What `m:mtext` reaches, and hence emits:
/// - **no `mathvariant`** — Perl clears it unconditionally for `m:mtext`
///   (L756-757); a font survives only as an `ltx_font_*` / `ltx_mathvariant_*`
///   CSS class.
/// - **no plane-1 remapping** — guarded off by `($tag ne 'm:mtext')` (L737).
/// - **no `href`/`title`** — gated on `$istoken` (L691-692).
/// - **no operator-dictionary attributes** — `%props` is filled only for `m:mo`
///   (L764), and `$stretchy` is cleared for every other tag (L767). So Perl's
///   `delete $mmlattr{stretchy}` in `pmml_text_aux` (L1069) is belt-and-braces
///   over something already absent, and has nothing to port.
///
/// `node` is `None` for a text node — Perl's non-`XML_ELEMENT_NODE` `$item`,
/// which contributes no attributes of its own (`$iselement` is false).
///
/// Returns Perl's `($text, %mmlattr)` pair. The text can differ from the input
/// only via the empty-item failsafe below; the caller that discards it (the
/// raw-markup arm, Perl's `my ($ignore, %mmlattr)`) is doing what Perl does.
fn stylize_text_content(
  node: Option<&Node>,
  attrs: &TextAttrs,
  text: &str,
) -> (String, HashMap<String, String>) {
  let attr_of = |name: &str| -> Option<String> {
    node
      .and_then(|n| n.get_attribute(name))
      .filter(|v| !v.is_empty())
  };
  // Perl L677-686: the passed-in %attr wins, then the item's own attribute,
  // then the inherited context.
  let font = attrs
    .font
    .clone()
    .or_else(|| attr_of("font"))
    .or_else(presentation::ctx_font);
  let size = attrs.fontsize.clone().or_else(|| attr_of("fontsize"));
  let color = attrs
    .color
    .clone()
    .or_else(|| attr_of("color"))
    .or_else(presentation::ctx_color);
  // NB Perl L683-684 reads `$attr{backgroundcolor} && ($iselement &&
  // $item->getAttribute('backgroundcolor')) || $BGCOLOR`, so the item's own
  // attribute counts only when an inherited one is ALSO set, and the result is
  // then the item's. Mirrored, quirk included; `pmml_token_inner` carries the
  // same note for the token arm.
  let bgcolor = attrs
    .backgroundcolor
    .as_ref()
    .and_then(|_| attr_of("backgroundcolor"))
    .or_else(presentation::ctx_bgcolor);
  let opacity = attrs
    .opacity
    .clone()
    .or_else(|| attr_of("opacity"))
    .or_else(presentation::ctx_opacity);
  let mut class = attr_of("class");
  // NB Perl reads `$attr{ccsstyle}` here (L686) — three c's, a typo for
  // `cssstyle`, so the passed-in half never contributes and only the item's own
  // attribute is ever seen. `%attr` carries no cssstyle on this path either way.
  let mut cssstyle = attr_of("cssstyle");

  // Perl L707-713: the failsafe for an item with nothing to show. An invisible
  // operator supplies its own character; anything else falls back to the item's
  // name, meaning or role — or a literal `?` for a bare text node — and is
  // painted red, since arriving here means something upstream emitted an empty
  // token. The red usually does NOT survive: an all-empty fallback is caught by
  // the Format test below, which clears the color again.
  let role = attr_of("role");
  let mut color = color;
  let text = if text.is_empty() {
    match role
      .as_deref()
      .and_then(presentation::default_token_content)
    {
      Some(default) => default.to_string(),
      None => {
        color = Some("red".to_string());
        if node.is_some() {
          attr_of("name")
            .or_else(|| attr_of("meaning"))
            .or(role)
            .unwrap_or_default()
        } else {
          "?".to_string()
        }
      },
    }
  } else {
    text.to_string()
  };

  // Perl L744-745: purely-Format content (invisible times/apply/separator, …)
  // needs no visual styling attributes at all.
  let (font, color, bgcolor, opacity) = if text.chars().all(is_format_char) {
    (None, None, None, None)
  } else {
    (font, color, bgcolor, opacity)
  };

  // Perl L746-756: patch up weak font translations with a CSS class. For
  // `m:mtext` this is the ONLY channel a font has, since the mathvariant is
  // dropped below.
  if let Some(ref f) = font {
    let extra = if f.contains("caligraphic") {
      Some("ltx_font_mathcaligraphic".to_string())
    } else if f.contains("script") {
      Some("ltx_font_mathscript".to_string())
    } else if f.contains("fraktur") && text.chars().all(|c| "+-0123456789.".contains(c)) {
      Some("ltx_font_oldstyle".to_string())
    } else if f.contains("smallcaps") {
      Some("ltx_font_smallcaps".to_string())
    } else {
      Some(crate::unicode::unicode_mathvariant(f))
        .filter(|v| *v != "normal")
        .map(|v| format!("ltx_mathvariant_{v}"))
    };
    if let Some(extra) = extra {
      class = Some(match class {
        Some(c) if !c.is_empty() => format!("{c} {extra}"),
        _ => extra,
      });
    }
  }

  // Perl L758-759: opacity folds into the css style.
  if let Some(op) = opacity {
    cssstyle = Some(match cssstyle {
      Some(c) if !c.is_empty() => format!("{c};opacity:{op}"),
      _ => format!("opacity:{op}"),
    });
  }

  let mut out: HashMap<String, String> = HashMap::default();
  // Perl L770-771: text that is empty or purely invisible operators gets no
  // size (nor stretchiness, which an `m:mtext` could not carry anyway). Note
  // this is a NARROWER class than the `\p{Format}` test above — only the three
  // invisible operators — so the two cannot be folded together.
  let size = size.filter(|_| !text.chars().all(|c| matches!(c, '\u{2061}'..='\u{2063}')));

  // Perl L779-797: emit a size only when it differs from the style's nominal
  // size, re-expressed relative to a script context and converted to em. The
  // `stretchyhack` minsize/maxsize arm needs `$issymm`, which for a non-`m:mo`
  // tag reduces to `$text eq '/'` (L703) — `$islargeop` needs a SUMOP/INTOP
  // role and `$props{symmetric}` is `m:mo`-only.
  if let Some(s) = size.filter(|s| s != presentation::context_size()) {
    let s = presentation::resolve_size(s);
    if text == "/" {
      out.insert("minsize".to_string(), s.clone());
      out.insert("maxsize".to_string(), s);
    } else {
      out.insert("mathsize".to_string(), s);
    }
  }
  if let Some(c) = color {
    out.insert("mathcolor".to_string(), c);
  }
  if let Some(bg) = bgcolor {
    out.insert("mathbackground".to_string(), bg);
  }
  if let Some(style) = cssstyle.filter(|s| !s.is_empty()) {
    out.insert("style".to_string(), style);
  }
  if let Some(c) = class.filter(|c| !c.is_empty()) {
    out.insert("class".to_string(), c);
  }
  (text, out)
}

/// Convert an XMHint spacing attribute to em value.
///
/// Port of `getXMHintSpacing`.
pub fn get_xm_hint_spacing(width: &str) -> f64 {
  // Perl (MathML.pm L380-385): /^([\d\.\+\-]+)(pt|mu|em)(\s+plus\s+…)?(\s+minus\s+…)?$/
  // — a GLUE width ("3.0pt plus 2.0pt minus 1.0pt") contributes its natural
  // part; the stretch/shrink tails are ignored.
  let trimmed = width.trim();
  let base = trimmed
    .split(" plus ")
    .next()
    .unwrap_or(trimmed)
    .split(" minus ")
    .next()
    .unwrap_or(trimmed)
    .trim();
  if let Some((num_str, unit)) = base
    .rfind(|c: char| c.is_ascii_digit() || c == '.')
    .map(|i| (&base[..=i], base[i + 1..].trim()))
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
    // Perl getQName returns undef for non-elements → stop. Also guards the
    // FFI: reading the ns field of a Document node is a misaligned deref.
    if n.get_type() != Some(libxml::tree::NodeType::ElementNode) {
      break;
    }
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
/// Port of `pmml_text_aux` (`MathML.pm` L1029-1077). `attrs` is Perl's `%attr`:
/// the presentation attributes accumulated from the enclosing `ltx:*` elements,
/// which `stylize_text_content` then puts on the `m:mtext`. The top-level caller
/// (the `ltx:XMText` arm of `pmml_internal`, Perl L494-498) passes an empty set,
/// exactly as Perl's bare `pmml_text_aux($_)` does.
pub fn pmml_text_aux(doc: &PostDocument, node: &Node, attrs: &TextAttrs) -> Vec<NodeData> {
  use libxml::tree::NodeType;

  match node.get_type() {
    Some(NodeType::TextNode) => {
      // Perl stylizes the RAW content first (L1034) and only then rewrites the
      // whitespace (L1035), so an empty text node reaches the failsafe as empty.
      let (text, attributes) = stylize_text_content(None, attrs, &node.get_content());
      // Perl L1035: `s/^\s+/NBSP/` then `s/\s+$/NBSP/` — a leading or trailing
      // whitespace RUN is REPLACED by a single NBSP, not trimmed away. (This arm
      // used to `trim_start()` unconditionally and only then test the
      // already-trimmed string with `starts_with(is_whitespace)`, which can never
      // be true — so leading space was silently dropped instead of becoming the
      // NBSP that keeps `$a \text{ and } b$` from closing up.)
      let head_trimmed = text.trim_start();
      let mut text = if head_trimmed.len() == text.len() {
        text.clone()
      } else {
        format!("\u{00A0}{head_trimmed}")
      };
      let tail_trimmed = text.trim_end();
      if tail_trimmed.len() != text.len() {
        text = format!("{tail_trimmed}\u{00A0}");
      }
      vec![NodeData::Element {
        tag:        "m:mtext".to_string(),
        attributes: (!attributes.is_empty()).then_some(attributes),
        children:   vec![NodeData::Text(text)],
      }]
    },
    Some(NodeType::ElementNode) => {
      // Perl L1041-1045: the element's own font/fontsize/color/backgroundcolor/
      // opacity join the inherited set for everything below it.
      let attrs = attrs.overlay(node);
      let tag = doc.get_qname(node).unwrap_or_default();
      match tag.as_str() {
        "ltx:Math" => {
          // Nested math: convert XMath if present
          match doc.findnode_at("ltx:XMath", node) {
            Some(xmath) => {
              vec![presentation::convert_to_pmml(doc, &xmath)]
            },
            // Perl L1051-1052: no XMath left means this Math was already
            // converted on an earlier pass — hand back the existing
            // `m:math`'s children rather than dropping the formula. Perl finds
            // it with `findnode('m:math', …)`, a DIRECT child; we scan the
            // children by namespace URI because the `m` prefix is not in the
            // document's XPath context at this point — this very processor is
            // what introduces MathML, so `m:` would fail to resolve and the
            // formula would go on being dropped silently.
            _ => match element_child_named(node, MML_URI, "math") {
              Some(mml) => {
                let mut out = Vec::new();
                let mut current = mml.get_first_child();
                while let Some(ref c) = current {
                  if let Some(nd) = rebuild_text_subtree_with_doc(c, true, Some(doc)) {
                    out.push(nd);
                  }
                  current = c.get_next_sibling();
                }
                out
              },
              _ => vec![],
            },
          }
        },
        // Perl L1057-1059: an `ltx:text` is transparent — recurse and let the
        // attributes ride down — but ONLY when it is not framed. `m:mtext`
        // cannot express a frame, so a framed one falls through to the
        // raw-markup arm below, where the XSLT can still render the box.
        "ltx:text" if !node.has_attribute("framed") && !node.has_attribute("framecolor") => {
          // Recurse on children
          let mut results = Vec::new();
          if let Some(child) = node.get_first_child() {
            let mut current = Some(child);
            while let Some(ref c) = current {
              results.extend(pmml_text_aux(doc, c, &attrs));
              current = c.get_next_sibling();
            }
          }
          vec![presentation::maybe_resize(doc, node, pmml_row(results))]
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
          // Perl L1061-1063 passes no %attr through this arm — the picture is
          // its own rendering, so the surrounding math font/color does not
          // restyle it.
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
          //
          // Perl L1067-1072 stylizes this `m:mtext` from the accumulated %attr
          // and — when the raw subtree still holds an `ltx:Math` — warns
          // `unexpected:nested-math` and leaves the content-MathML unconverted,
          // which renders operator-first (garbled) in the browser. We instead
          // convert any nested `ltx:Math` in `rebuild_text_subtree_with_doc`
          // below (to a self-contained inline `<m:math>` — see
          // `nested_ltx_math_to_inline_mathml`), so a `\parbox`/`\mbox`-with-math
          // in math renders correctly (arXiv html_feedback #6847). Surpass-Perl;
          // see OXIDIZED_DESIGN #101.
          // Perl `my ($ignore, %mmlattr) = …` — the raw subtree is carried over
          // verbatim below, so only the attributes are wanted here.
          let (_, attributes) = stylize_text_content(Some(node), &attrs, &node.get_content());
          let cloned = rebuild_text_subtree_with_doc(node, true, Some(doc))
            .unwrap_or_else(|| NodeData::Text("\u{00A0}".to_string()));
          vec![NodeData::Element {
            tag:        "m:mtext".to_string(),
            attributes: (!attributes.is_empty()).then_some(attributes),
            children:   vec![cloned],
          }]
        },
      }
    },
    _ => vec![],
  }
}

/// Convert a nested `ltx:Math` — a `$...$` inside a text box (`\parbox`/`\mbox`/
/// `\text`) that itself sits in math — into a self-contained INLINE `<m:math>`
/// element, or `None` if the Math is empty.
///
/// Why a full `<m:math>` rather than the bare presentation `convert_to_pmml`
/// returns: this node lands inside the text box's HTML (`ltx:inline-block` /
/// `ltx:text` → `<span>`), and per HTML5's MathML text-integration-point rules a
/// bare `<mrow>` inside that HTML is parsed as HTML and renders as flat text —
/// not math (subscripts/superscripts/calligraphic lost). The `<math>` re-enters
/// MathML context. Nested math is always inline (a `$...$`), so `display=inline`;
/// `alttext`/`class` ride from the `ltx:Math`. The top-level pass gets the same
/// wrapper from `MathProcessor::outer_wrapper`; this is its nested analogue
/// (arXiv html_feedback #6847 / OXIDIZED_DESIGN #101).
fn nested_ltx_math_to_inline_mathml(doc: &PostDocument, math_node: &Node) -> Option<NodeData> {
  let inner = match doc.findnode_at("ltx:XMath", math_node) {
    Some(xmath) => presentation::convert_to_pmml(doc, &xmath),
    // No XMath left => already converted on an earlier pass; its `<m:math>` is
    // already a full element, so reuse it as-is.
    None => {
      return element_child_named(math_node, MML_URI, "math")
        .and_then(|mml| rebuild_text_subtree_with_doc(&mml, true, Some(doc)));
    },
  };
  let mut attrs: HashMap<String, String> = HashMap::default();
  attrs.insert("display".to_string(), "inline".to_string());
  if let Some(tex) = math_node.get_attribute("tex") {
    attrs.insert("alttext".to_string(), tex);
  }
  if let Some(class) = math_node.get_attribute("class") {
    attrs.insert("class".to_string(), class);
  }
  Some(NodeData::Element {
    tag:        "m:math".to_string(),
    attributes: Some(attrs),
    children:   vec![inner],
  })
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
/// the space. A nested `ltx:Math` (a `$...$` inside a text box that itself
/// sits in math) is CONVERTED to presentation MathML by
/// `rebuild_text_subtree_with_doc` rather than cloned raw — see the
/// reentrancy note there (arXiv html_feedback #6847); Perl leaves it raw and
/// warns `unexpected:nested-math`, which renders operator-first (garbled).
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
      // A nested `ltx:Math` — a `$...$` inside a \parbox/\mbox/inline-block that
      // itself sits in math. The top-level pass skipped it
      // (`//ltx:Math[not(ancestor::ltx:Math)]`), so cloning it verbatim leaks
      // unconverted `<ltx:XMath>` content-MathML into the HTML, which the browser
      // renders in operator-first document order (garbled, arXiv html_feedback
      // #6847 / arXiv:2608.05024). Convert it here instead. Needs `doc` for
      // URI→prefix + the ancestor style/font context; the doc-less
      // `rebuild_text_subtree` callers keep the verbatim clone (they never carry
      // nested math). See OXIDIZED_DESIGN #101.
      if let Some(d) = doc
        && d.get_qname(node).as_deref() == Some("ltx:Math")
        && let Some(mml) = nested_ltx_math_to_inline_mathml(d, node)
      {
        return Some(mml);
      }
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
      // than forge a wrong id. NOTE: get_attributes() reports xml:id
      // under its LOCAL name "id", so that spelling must be skipped
      // too — it used to leak through as a plain id= duplicate. The
      // namespace probe keeps a GENUINE plain `id` (the model grants
      // one to `ltx:bib-identifier`/`ltx:bib-review`, carrying a
      // DOI/ISSN) from being dropped along with it.
      let has_xml_id = node
        .get_attribute_ns("id", latexml_core::common::xml::XML_NS)
        .is_some();
      let mut attrs: HashMap<String, String> = HashMap::default();
      for (k, v) in node.get_attributes() {
        if k.starts_with('_') || k == "xml:id" || k == "fragid" || (k == "id" && has_xml_id) {
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
