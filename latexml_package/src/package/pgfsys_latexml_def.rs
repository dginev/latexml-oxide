//! pgfsys-latexml.def — LaTeXML SVG driver for pgf/tikz
//! Perl: pgfsys-latexml.def.ltxml (1022 lines)
//!
//! Port of the SVG drawing primitives that make pgf/tikz produce SVG output.
//!
//! ## Def*-kind divergence from Perl (audit-flagged, intentional)
//!
//! The DP audit flags ~16 `\pgfsys@moveto`/`@lineto`/`@curveto`/`@rect`/etc.
//! entries as Perl=DefConstructor↔Rust=DefPrimitive. Both are functionally
//! equivalent:
//!
//! - **Perl** uses `DefConstructor('\pgfsys@moveto{Dimension}{Dimension}', '', ...)` with an
//!   **empty XML template** — the `afterDigest` callback pushes path-data state (e.g. `M x y`) onto
//!   an accumulating list. The SVG `<svg:path d="…">` is emitted later by a flush primitive like
//!   `\pgfsys@stroke` / `\pgfsys@fill`.
//! - **Rust** uses `DefPrimitive!(..., sub[(x, y)] { ... })` that does the same state accumulation
//!   imperatively.
//!
//! Same path-buffering architecture, different `Def*` surface. The Rust
//! DefPrimitive shape is more natural for the imperative state-mutation
//! pattern; Perl's DefConstructor-with-empty-template is idiomatic Perl for
//! "this CS doesn't produce local XML, but has post-digest state effects".
//! Do NOT kind-flip these to DefConstructor — the Perl empty-template
//! idiom doesn't map cleanly to Rust's DefConstructor! macro semantics.
use crate::prelude::*;

/// Helper: convert dimension to px value, rounded to 2 decimal places
/// Perl: $dim->pxValue  =>  roundto(sp / UNITY * DPI / 72.27, 2)
fn dim_to_px(d: Dimension) -> f64 {
  let dpi = state::lookup_int("DPI");
  let dpi = if dpi > 0 { dpi as f64 } else { 100.0 };
  let raw = (d.value_of() as f64 / 65536.0) * (dpi / 72.27);
  // Perl roundto(n, 2): round to 2 decimal places
  (raw * 100.0).round() / 100.0
}

/// Perl: sub SVGNextObject — global counter for clip paths etc.
fn svg_next_object() -> i64 {
  let n = state::lookup_int("svg_objcount") + 1;
  state::assign_value("svg_objcount", Stored::Int(n), Scope::Global);
  n
}

/// Perl: sub addToSVGPath — accumulates path data in state
fn add_to_svg_path(operation: &str, points: &[Dimension]) {
  let new_path = if points.is_empty() {
    operation.to_string()
  } else {
    let pts: Vec<String> = points
      .iter()
      .map(|p| format!("{}", dim_to_px(*p)))
      .collect();
    format!("{} {}", operation, pts.join(" "))
  };
  let current = state::lookup_string("pgf_SVGpath");
  let combined = if current.is_empty() {
    new_path
  } else {
    format!("{} {}", current, new_path)
  };
  state::assign_value(
    "pgf_SVGpath",
    Stored::String(arena::pin(&combined)),
    Scope::Global,
  );
}

/// Look up a pgf register as a Dimension
fn pgf_reg_dim(name: &str) -> Dimension {
  match state::lookup_register(name, Vec::new()) {
    Ok(Some(RegisterValue::Dimension(d))) => d,
    Ok(Some(RegisterValue::Number(n))) => Dimension::new(n.value_of()),
    _ => Dimension::new(0),
  }
}

/// Convert a color channel (0.0–1.0) to u8 (0-255), matching Perl's roundto rounding.
/// Uses the same epsilon nudge as color.rs::component_to_u8 to ensure consistency
/// between font color attributes and SVG fill/stroke attributes.
fn channel_to_u8(v: f64) -> u8 {
  let scaled = v.clamp(0.0, 1.0) * 255.0 * (1.0 + 100.0 * f64::EPSILON);
  scaled.round() as u8
}

/// Helper: format color channel (0.0–1.0) to hex
/// Perl: Color('rgb', r, g, b)->toHex → "#RRGGBB"
/// Returns tokens (not a string) because # must be catcode OTHER, not PARAMETER.
fn color_to_hex_tokens(r: f64, g: f64, b: f64) -> Vec<Token> {
  let hex = format!(
    "{:02X}{:02X}{:02X}",
    channel_to_u8(r),
    channel_to_u8(g),
    channel_to_u8(b)
  );
  // Perl uses Explode() which creates catcode-12 (OTHER) tokens.
  // '#' as catcode OTHER, then hex digits as catcode OTHER.
  let mut tokens = vec![T_OTHER!("#")];
  tokens.extend(mouth::tokenize_internal(&hex).unlist());
  tokens
}

// Perl L149-157: DefParameterType('SVGMoveableBox', ...)
// Defined in Perl but never used as a parameter type anywhere in the codebase.
// Omitted from Rust; add if a use site is discovered.

/// Perl L161-169: foreignObjectCheck
/// Check whether an svg:foreignObject is open in the ancestor chain,
/// but don't check beyond an svg:svg node (in case we're nested).
/// Returns Some(node) if foreignObject is found, None otherwise.
#[allow(dead_code)]
fn foreign_object_check(document: &Document) -> Option<Node> {
  let mut node_opt = Some(document.get_node().clone());
  while let Some(node) = node_opt {
    let qname = document::get_node_qname(&node);
    enum Probe {
      Svg,
      ForeignObject,
      Other,
    }
    let probe = arena::with(qname, |s| match s {
      "svg:svg" => Probe::Svg,
      "svg:foreignObject" => Probe::ForeignObject,
      _ => Probe::Other,
    });
    match probe {
      Probe::Svg => return None,
      Probe::ForeignObject => return Some(node),
      Probe::Other => {},
    }
    node_opt = node.get_parent();
  }
  None
}

/// Helper: read a TeX dimension register value by control sequence name
fn read_dim_register(cs: &str) -> Dimension {
  if let RegisterValue::Dimension(d) = LookupRegister!(cs) {
    d
  } else {
    Dimension::default()
  }
}

/// Format a px value: round to 2 decimal places, strip trailing zeros
fn fmt_px(v: f64) -> String {
  let r = (v * 100.0).round() / 100.0;
  if r == 0.0 {
    return "0".to_string();
  }
  if r == r.round() && r.abs() < 1e10 {
    format!("{}", r as i64)
  } else {
    let s = format!("{:.2}", r);
    let s = s.trim_end_matches('0');
    s.trim_end_matches('.').to_string()
  }
}

/// Perl L920-939: tikzAlignmentBindings
/// Sets up an Alignment with SVG-specific callbacks (svg:g elements with transform matrices)
/// instead of the standard tabular/tr/td elements used by alignmentBindings.
fn tikz_alignment_bindings(
  template: latexml_core::alignment::template::Template,
  xml_attributes: rustc_hash::FxHashMap<String, String>,
) {
  use latexml_core::alignment::{Alignment, AlignmentConfig};
  use latexml_core::common::arena::SymHashMap;
  use std::rc::Rc;

  let mode = state::lookup_string_from_sym(pin!("MODE"));
  let is_math = mode.ends_with("math");

  let mut properties = SymHashMap::default();
  properties.insert("preserve_structure", Stored::Bool(true));

  let alignment = Alignment::new(AlignmentConfig {
    template: Some(template),
    // Perl L947-969: openTikzAlignment — creates container <svg:g> with Y-flip transform
    open_container: Rc::new(|document, props| {
      let mut attrs = props;
      attrs.insert("class".to_string(), "ltx_tikzmatrix".to_string());
      // Perl: transform="matrix(1 0 0 -1 x y)" — flips Y-axis (pgf Y=up, SVG Y=down)
      // y = (h + d) - rowdepths[-1]  (baseline of last row)
      let h: f64 = attrs
        .remove("cheight")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
      let d: f64 = attrs
        .remove("cdepth")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
      let _w: f64 = attrs
        .remove("cwidth")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
      let y = h + d;
      attrs
        .entry("transform".to_string())
        .or_insert_with(|| format!("matrix(1 0 0 -1 0 {})", fmt_px(y)));
      attrs.insert("_scopebegin".to_string(), "1".to_string());
      document
        .open_element("svg:g", Some(attrs), None)
        .map(Option::Some)
    }),
    // Perl L971-973: closeTikzAlignmentElement
    close_container: Rc::new(|document| document.close_element("svg:g")),
    // Perl L975-989: openTikzAlignmentRow — creates row <svg:g> with Y-position
    open_row: Rc::new(|document, props| {
      let mut attrs: rustc_hash::FxHashMap<String, String> = rustc_hash::FxHashMap::default();
      let class_base = "ltx_tikzmatrix_row";
      let class_extra = props.get("class").map(|c| c.to_string());
      let class = if let Some(c) = class_extra {
        format!("{} {}", class_base, c)
      } else {
        class_base.to_string()
      };
      attrs.insert("class".to_string(), class);
      attrs.insert("_scopebegin".to_string(), "1".to_string());
      // Perl: transform="matrix(1 0 0 1 0 yy)" where yy = y + cheight
      let y = props
        .get("y")
        .and_then(|v| {
          if let Stored::Dimension(d) = v {
            Some(d.px_value(None))
          } else {
            None
          }
        })
        .unwrap_or(0.0);
      let h = props
        .get("cheight")
        .and_then(|v| {
          if let Stored::Dimension(d) = v {
            Some(d.px_value(None))
          } else {
            None
          }
        })
        .unwrap_or(0.0);
      let yy = y + h;
      attrs.insert(
        "transform".to_string(),
        format!("matrix(1 0 0 1 0 {})", fmt_px(yy)),
      );
      document
        .open_element("svg:g", Some(attrs), None)
        .and(Ok(()))
    }),
    close_row: Rc::new(|document| document.close_element("svg:g")),
    // Perl L991-1009: openTikzAlignmentCol — creates cell <svg:g> with X-position and Y-flip
    open_column: Rc::new(|document, props| {
      let mut attrs = props;
      let class_base = "ltx_tikzmatrix_col";
      let class_extra = attrs.remove("class");
      let class = if let Some(c) = class_extra {
        format!("{} {}", class_base, c)
      } else {
        class_base.to_string()
      };
      attrs.insert("class".to_string(), class);
      attrs.insert("_scopebegin".to_string(), "1".to_string());
      // Perl: transform="matrix(1 0 0 -1 x 0)" — flip Y and position at column x
      let x: f64 = attrs
        .remove("x")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
      let _y: f64 = attrs
        .remove("y")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
      // Remove dimension props that shouldn't become XML attributes
      attrs.remove("cwidth");
      attrs.remove("cheight");
      attrs.remove("cdepth");
      attrs
        .entry("transform".to_string())
        .or_insert_with(|| format!("matrix(1 0 0 -1 {} 0)", fmt_px(x)));
      document
        .open_element("svg:g", Some(attrs), None)
        .map(Option::Some)
    }),
    close_column: Rc::new(|document| document.close_element("svg:g")),
    is_math,
    properties,
    xml_attributes,
  });

  assign_alignment(alignment, None);
  state::let_i(
    &T_MATH!(),
    &if is_math {
      T_CS!("\\lx@dollar@in@mathmode")
    } else {
      T_CS!("\\lx@dollar@in@textmode")
    },
    None,
  );
}

#[rustfmt::skip]
LoadDefinitions!({
  // (protocol fix is applied at \pgfsys@invoke below)

  // Perl L29-31: image suffix list
  DefMacro!("\\pgfsys@imagesuffixlist",
    ".svg:.png:.gif:.jpg:.jpeg:.eps:.ps:.postscript:.ai:.pdf:");

  //===================================================================
  // 0. Environment specific stuff
  //===================================================================

  // Perl L59: \lxSVG@installcommands
  DefPrimitive!("\\lxSVG@installcommands", {
    state::let_i(&T_CS!("\\halign"), &T_CS!("\\lxSVG@halign"), None);
    state::let_i(&T_CS!("\\ignorespaces"), &T_CS!("\\lx@inpgf@ignorespaces"), None);
  });

  // Perl L63: redefine ignorespaces inside PGF context
  DefPrimitive!("\\lx@inpgf@ignorespaces SkipSpaces", None);

  // Perl L892-918: \lxSVG@halign — SVG matrix layout for tikz-cd and pgf matrix
  // Replaces \halign inside pgf/tikz contexts to produce <svg:g> elements
  // with transform matrices instead of <ltx:tabular/tr/td>.
  DefConstructor!("\\lxSVG@halign BoxSpecification",
    "#alignment",
    bounded => true, leave_horizontal => true,
    sizer => sub[whatsit] {
      match whatsit.get_property("alignment").as_deref() { Some(Stored::Digested(alignment_d)) => {
        let (w, h, d, _, _, _) = alignment_d.clone().get_size(None)?;
        Ok((w, h, d))
      } _ => {
        Ok((Dimension::default(), Dimension::default(), Dimension::default()))
      }}
    },
    after_digest => sub[whatsit] {
      use crate::engine::tex_tables::{
        parse_halign_template, digest_alignment_body,
      };
      whatsit.set_property("mode", Stored::from("internal_vertical"));
      begin_mode("restricted_horizontal")?;
      let template = parse_halign_template(whatsit)?;
      // NOTE: a 0-column template (parse couldn't catcode-1-`{`-delimit it,
      // e.g. young.sty's `\halign\bgroup &\setbox…`) is NOT special-cased
      // here — exactly like the standard `\halign` (tex_tables.rs). The body
      // is still digested under the restricted_horizontal frame begun above
      // and the SINGLE `end_mode` at the bottom balances it. A prior
      // half-implemented "bail" ran an early `end_mode` here WITHOUT a
      // `return`, so the body was digested anyway and `end_mode` ran a SECOND
      // time at the bottom — popping past the already-closed
      // restricted_horizontal frame into the enclosing `\vbox`'s
      // `internal_vertical` → "Attempt to end mode restricted_horizontal in
      // horizontal" (driver 1902.11165: `\node {\begin{young}…\end{young}}`,
      // Perl=0). Conversely, `return`ing early left the body's `&`/`\cr`
      // unconsumed → "Stray alignment &". Matching the standard `\halign`
      // (digest body, end once) fixes both.
      // Get width from BoxSpecification 'to' key
      let width_attr: Option<String> = {
        let spec = whatsit.get_arg(1);
        if let Some(ArgWrap::Dimension(w)) = GetKeyVal!(spec, "to") {
          Some(w.to_attribute())
        } else {
          None
        }
      };
      let mut xml_attrs = HashMap::default();
      if let Some(w) = width_attr {
        xml_attrs.insert(String::from("width"), w);
      }
      xml_attrs.insert(String::from("vattach"), String::from("bottom"));
      tikz_alignment_bindings(template, xml_attrs);
      digest_alignment_body(whatsit)?;
      end_mode("restricted_horizontal")?;
      decrement_align_group_count(); // Balance the opening { OUTSIDE of the masking of ALIGN_STATE
    });

  // Perl L65-69: \lxSVG@picture — wraps pgfpicture with SVG setup
  DefMacro!("\\lxSVG@picture", sub[_gullet] {
    vec![
      T_CS!("\\lxSVG@clearpath"),
      T_CS!("\\begingroup"),
      T_CS!("\\lxSVG@installcommands"),
    ]
  });
  DefMacro!("\\endlxSVG@picture", "\\endgroup");

  // Perl L75-147: \lxSVG@insertpicture — creates <ltx:picture> + <svg:svg>
  DefConstructor!("\\lxSVG@insertpicture{}",
    sub[document, _args, props] {
      let current = document.get_node().clone();
      let current_qname = document::get_node_qname(&current);
      let is_in_svg = arena::with(current_qname, |s| s.starts_with("svg:"));
      if is_in_svg {
        // Already in SVG — just open a nested svg:g
        let minx = match props.get("minx") {
          Some(Stored::Float(f)) => f.0, _ => 0.0
        };
        let miny = match props.get("miny") {
          Some(Stored::Float(f)) => f.0, _ => 0.0
        };
        // Avoid "-0" in transform — normalize negative zeros to zero
        let tx = if minx == 0.0 { 0.0 } else { -minx };
        let ty = if miny == 0.0 { 0.0 } else { -miny };
        let transform = format!("matrix(1 0 0 1 {} {})", tx, ty);
        document.open_element("svg:g", Some(string_map!(
          "transform" => transform,
          "_scopebegin" => "1".to_string(),
          "class" => "ltx_nestedsvg".to_string()
        )), None)?;
        if let Some(Stored::Digested(content)) = props.get("content_box") {
          document.absorb(content, None)?;
        }
        document.close_element("svg:g")?;
      } else {
        // Not in SVG — create full picture + svg:svg wrapper
        let pxwidth = match props.get("pxwidth") {
          Some(Stored::Float(f)) => f.0, _ => 1.0
        };
        let pxheight = match props.get("pxheight") {
          Some(Stored::Float(f)) => f.0, _ => 1.0
        };

        // Perl: $document->openElement('ltx:picture');
        // height/width attributes come from pxValue of the picture dimensions
        let pic_attrs = string_map!(
          "height" => format!("{}", pxheight),
          "width" => format!("{}", pxwidth)
        );
        document.open_element("ltx:picture", Some(pic_attrs), None)?;

        let minx = match props.get("minx") {
          Some(Stored::Float(f)) => f.0, _ => 0.0
        };
        let miny = match props.get("miny") {
          Some(Stored::Float(f)) => f.0, _ => 0.0
        };
        // Perl L88: viewBox => "$props{minx} $props{miny} $props{pxwidth} $props{pxheight}"
        let svg_attrs = string_map!(
          "version" => "1.1".to_string(),
          "width" => format!("{}", pxwidth),
          "height" => format!("{}", pxheight),
          "viewBox" => format!("{} {} {} {}", minx, miny, pxwidth, pxheight),
          "overflow" => "visible".to_string()
        );
        document.open_element("svg:svg", Some(svg_attrs), None)?;
        // Perl L89: style => $props{style} (baseline vertical-align)
        if let Some(Stored::String(style)) = props.get("style") {
          arena::with(*style, |s| {
            if !s.is_empty() {
              let svg_node = document.get_node_mut();
              let _ = svg_node.set_attribute("style", s);
            }
          });
        }

        // Perl L91-92: x0=-(0+minx), y0=pxheight+miny
        let x0 = -(0.0 + minx);
        let y0 = pxheight + miny;
        // Avoid -0 in output
        let x0 = if x0 == 0.0 { 0.0 } else { x0 };
        let y0 = if y0 == 0.0 { 0.0 } else { y0 };
        let transform = format!("translate({},{}) matrix(1 0 0 -1 0 0)", x0, y0);
        document.open_element("svg:g", Some(string_map!(
          "transform" => transform,
          "_scopebegin" => "1".to_string()
        )), None)?;

        if let Some(Stored::Digested(content)) = props.get("content_box") {
          document.absorb(content, None)?;
        }

        // Close all svg:g's
        while let Ok(Some(_)) = document.maybe_close_element("svg:g") {}
        document.close_element("svg:svg")?;
        document.close_element("ltx:picture")?;
      }
    },
    reversion => sub[whatsit, _args] {
      // Perl L140-147: reversion produces \hbox to<W>pt{\vbox to<H>pt{...}}
      let w = whatsit.with_properties(|props| {
        match props.get("width_dim") {
          Some(Stored::Dimension(d)) => d.pt_value(None),
          _ => 0.0
        }
      });
      let h = whatsit.with_properties(|props| {
        match props.get("height_dim") {
          Some(Stored::Dimension(d)) => d.pt_value(None),
          _ => 0.0
        }
      });
      let mut toks: Vec<Token> = vec![T_CS!("\\hbox"), T_SPACE!()];
      toks.extend(Explode!(format!("to{}pt", w)));
      toks.push(T_BEGIN!());
      toks.extend(vec![T_CS!("\\vbox"), T_SPACE!()]);
      toks.extend(Explode!(format!("to{}pt", h)));
      toks.push(T_BEGIN!());
      toks.push(T_CS!("\\pgfpicture"));
      toks.push(T_CS!("\\makeatletter"));
      if let Some(arg) = whatsit.get_arg(1) {
        toks.extend(arg.revert()?.unlist());
      }
      toks.push(T_CS!("\\endpgfpicture"));
      toks.push(T_END!());
      toks.push(T_END!());
      Ok(Tokens::new(toks))
    },
    after_digest => sub[whatsit] {
      // Perl L106-138: read pgf registers to compute picture dimensions
      // \pgf@picmaxx,\pgf@picmaxy are now the SIZE of the picture (adjusted by typesetpicturebox)
      let miny = pgf_reg_dim("\\pgf@picminy");
      let width = pgf_reg_dim("\\pgf@picmaxx");
      let height = pgf_reg_dim("\\pgf@picmaxy");
      let w = dim_to_px(width).max(1.0);
      let h = dim_to_px(height).max(1.0);

      // Perl L120-121: baseline shift from \pgf@shift@baseline
      let shift_tokens = Expand!(T_CS!("\\pgf@shift@baseline"));
      let shift_str = shift_tokens.to_string();
      let shift_str = shift_str.trim();
      let base = if !shift_str.is_empty() && shift_str != "0pt" {
        match Dimension::from_str(shift_str) { Ok(shift_dim) => {
          // Perl: $base = ($shift ? $miny->subtract(Dimension($shift))->pxValue : 0)
          let base_dim = Dimension::new(miny.value_of() - shift_dim.value_of());
          dim_to_px(base_dim)
        } _ => { 0.0 }}
      } else { 0.0 };

      whatsit.set_property("minx", Stored::Float(Float::new_f64(0.0)));
      whatsit.set_property("miny", Stored::Float(Float::new_f64(0.0)));
      whatsit.set_property("width", Stored::Dimension(width));
      whatsit.set_property("height", Stored::Dimension(height));
      whatsit.set_property("depth", Stored::Dimension(Dimension::new(0)));
      whatsit.set_property("pxwidth", Stored::Float(Float::new_f64(w)));
      whatsit.set_property("pxheight", Stored::Float(Float::new_f64(h)));
      // Perl L132: style for vertical alignment
      if base != 0.0 {
        let style = format!("vertical-align:{}px", base);
        whatsit.set_property("style", Stored::String(arena::pin(&style)));
      }
      // Store raw dimensions for reversion (tex= attribute)
      whatsit.set_property("width_dim", Stored::Dimension(width));
      whatsit.set_property("height_dim", Stored::Dimension(height));
      // Store the content box for construction time
      let content_opt = whatsit.get_arg(1).cloned();
      if let Some(content) = content_opt {
        whatsit.set_property("content_box", Stored::Digested(content));
      }
    }
  );

  // Perl L171-172: baseline shift macros
  DefMacro!("\\pgf@shift@baseline", "0pt");
  DefMacro!("\\pgf@shift@baseline@smuggle", "0pt");

  //===================================================================
  // 1. Beginning and ending a stream
  //===================================================================

  // Perl L178-192: \pgfsys@typesetpicturebox
  RawTeX!(r"\def\pgfsys@typesetpicturebox#1{\pgf@ya=\pgf@shift@baseline\relax\advance\pgf@ya by-\pgf@picminy\relax\advance\pgf@picmaxy by-\pgf@picminy\relax\advance\pgf@picmaxx by-\pgf@picminx\relax\setbox#1=\hbox{\hskip-\pgf@picminx\lower\pgf@picminy\box#1}\ht#1=\pgf@picmaxy\wd#1=\pgf@picmaxx\dp#1=0pt\leavevmode\lxSVG@insertpicture{\box#1\lxSVG@closescope}}");

  def_macro_noop("\\pgfsys@beginpicture")?;
  def_macro_noop("\\pgfsys@endpicture")?;


  // Perl L197-210: \pgfsys@hbox — inserts a box in SVG context
  // Perl: sizer => sub { my $box = $_[0]->getProperty('thebox');
  //   return ($box ? $box->getSize : (Dimension(0), Dimension(0), Dimension(0))); }
  DefConstructor!("\\pgfsys@hbox{Number}",
    sub[document, _args, props] {
      if let Some(Stored::Digested(thebox)) = props.get("thebox") {
        document.absorb(thebox, None)?;
      }
    },
    after_digest => sub[whatsit] {
      let boxnum = whatsit.get_arg(1)
        .map(|a| a.to_string().parse::<i64>().unwrap_or(0))
        .unwrap_or(0);
      let boxname = format!("box{}", boxnum);
      if let Some(bx) = state::lookup_value(&boxname) {
        // Perl: sizer propagates box dimensions to the whatsit
        if let Stored::Digested(ref digested) = bx {
          if let Some(RegisterValue::Dimension(w)) = digested.get_width(None).unwrap_or(None) {
            whatsit.set_property("cached_width", Stored::Dimension(w));
          }
          if let Some(RegisterValue::Dimension(h)) = digested.get_height() {
            whatsit.set_property("cached_height", Stored::Dimension(h));
          }
          if let Some(RegisterValue::Dimension(d)) = digested.get_depth() {
            whatsit.set_property("cached_depth", Stored::Dimension(d));
          }
        }
        whatsit.set_property("thebox", bx);
      }
    }
  );

  //===================================================================
  // 2. Path construction
  //===================================================================

  DefPrimitive!("\\lxSVG@clearpath", {
    state::assign_value("pgf_SVGpath", Stored::String(arena::pin("")), Scope::Global);
  });
  DefPrimitive!("\\lxSVG@clearclip", {
    state::assign_value("pgf_clipnext", Stored::Int(0), None);
  });

  //============================================================
  // pgfsys low-level drawing primitives — Perl L242-577
  //============================================================
  //
  // Umbrella WISDOM #44 intentional divergence for the whole
  // block below:
  //
  // Perl pgfsys-latexml.def.ltxml defines each of \pgfsys@moveto,
  // \pgfsys@lineto, \pgfsys@curveto, \pgfsys@rect, \pgfsys@closepath,
  // \pgfsys@clipnext, \lxSVG@color@{rgb,cmyk,cmy,gray}@{stroke,fill},
  // \lxSVG@{beginscope,endscope}, and a few more as
  //   DefConstructor('\pgfsys@moveto{Dimension}{Dimension}', '',
  //     afterDigest => sub { addToSVGPath('M', $_[1]->getArgs); },
  //     sizer => 0);
  // — constructors with an EMPTY template and sizer=0, whose only
  // observable work lives in `afterDigest` (mutating the internal
  // SVG-path buffer / color state).
  //
  // Rust ports these as DefPrimitive with the side-effect in the
  // body sub. The direct primitive form does the same state
  // mutation without the zero-sizer whatsit leaking into the
  // digest tree. Kind-wise this is 17 DefConstructor →
  // DefPrimitive flips, all under the same rationale (empty
  // template + sizer=0 → no observable XML, only state).
  //
  // This applies uniformly to every pgfsys / lxSVG@color /
  // lxSVG@scope binding in the block below; individual entries
  // don't re-carry the WISDOM #44 tag to avoid comment noise.

  // Perl L242-276: path operations
  DefPrimitive!("\\pgfsys@moveto{Dimension}{Dimension}", sub[(x, y)] {
    add_to_svg_path("M", &[x, y]);
  });
  DefPrimitive!("\\pgfsys@lineto{Dimension}{Dimension}", sub[(x, y)] {
    add_to_svg_path("L", &[x, y]);
  });
  DefPrimitive!("\\pgfsys@curveto{Dimension}{Dimension}{Dimension}{Dimension}{Dimension}{Dimension}",
    sub[(x1, y1, x2, y2, x3, y3)] {
    add_to_svg_path("C", &[x1, y1, x2, y2, x3, y3]);
  });
  DefPrimitive!("\\pgfsys@rect{Dimension}{Dimension}{Dimension}{Dimension}",
    sub[(x, y, w, h)] {
    add_to_svg_path("M", &[x, y]);
    add_to_svg_path("h", &[w]);
    add_to_svg_path("v", &[h]);
    add_to_svg_path("h", &[w.negate()]);
    add_to_svg_path("Z", &[]);
  });
  DefPrimitive!("\\pgfsys@closepath", {
    add_to_svg_path("Z", &[]);
  });

  //===================================================================
  // 3. Canvas transformation
  //===================================================================

  DefMacro!("\\pgfsys@transformcm{Float}{Float}{Float}{Float}{Dimension}{Dimension}",
    "\\lxSVG@transformcm{#1}{#2}{#3}{#4}{#5}{#6}\\lxSVG@@transformcm{#1}{#2}{#3}{#4}{#5}{#6}");

  DefConstructor!("\\lxSVG@transformcm{Float}{Float}{Float}{Float}{Dimension}{Dimension}", "");

  DefMacro!("\\lxSVG@@transformcm{Float}{Float}{Float}{Float}{Dimension}{Dimension}",
    sub[(a, b, c, d, e, f)] {
    let transform = format!("transform=matrix({} {} {} {} {} {})",
      floatformat(a.0),
      floatformat(b.0),
      floatformat(c.0),
      floatformat(d.0),
      dim_to_px(e),
      dim_to_px(f));
    let tok_str = format!("\\lxSVG@begingroup{{{}}}", transform);
    mouth::tokenize_internal(&tok_str).unlist()
  });

  //===================================================================
  // 4. Stroking, filling, and clipping
  //===================================================================

  // Intentional divergence (WISDOM #44 class: afterDigest-only state
  // toggle): Perl L307 `\pgfsys@clipnext` is a DefConstructor whose
  // constructor body is empty — the only observable effect is
  // `AssignValue(pgf_clipnext=>1)` read back later by the next
  // \pgfsys@drawpath. DefPrimitive is the idiomatic Rust shape for a
  // pure-state-toggle CS with no XML emission. Same class as
  // psfrag_sty's \psfragscanon/off. Audit flags the single L678 entry.
  DefPrimitive!("\\pgfsys@clipnext", {
    state::assign_value("pgf_clipnext", Stored::Int(1), None);
  });

  DefMacro!("\\pgfsys@stroke",     "\\lxSVG@stroke\\lxSVG@drawpath{fill:none}");
  DefMacro!("\\pgfsys@fill",       "\\lxSVG@fill\\lxSVG@drawpath{stroke:none}");
  DefMacro!("\\pgfsys@fillstroke", "\\lxSVG@fillstroke\\lxSVG@drawpath{}");

  DefConstructor!("\\lxSVG@stroke", "");
  DefConstructor!("\\lxSVG@fill", "");
  DefConstructor!("\\lxSVG@fillstroke", "");

  // Perl L321-334: \lxSVG@drawpath
  DefMacro!("\\lxSVG@drawpath{}", sub[(arg)] {
    let arg_str = arg.to_string();
    let path = state::lookup_string("pgf_SVGpath");
    let clip = state::lookup_int("pgf_clipnext") != 0;
    if clip {
      let clip_cmd = format!("\\lxSVG@clearpath\\lxSVG@clearclip\\pgfsysprotocol@literal{{\\lxSVG@drawpath@clipped{{{}}}{{{}}}}}", path, arg_str);
      mouth::tokenize_internal(&clip_cmd).unlist()
    } else {
      let draw_cmd = format!("\\lxSVG@clearpath\\pgfsysprotocol@literal{{\\lxSVG@drawpath@unclipped{{{}}}{{{}}}}}", path, arg_str);
      mouth::tokenize_internal(&draw_cmd).unlist()
    }
  });

  // Perl L337-339: unclipped path — emit svg:path
  DefConstructor!("\\lxSVG@drawpath@unclipped{}{}",
    sub[document, args, props] {
      let d = args.first().and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      if !d.is_empty() {
        let style = args.get(1).and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();

        // Read per-path colors from properties (captured during digestion via properties closure)
        let fill_color = props.get("pgf_fillcolor").map(|v| v.to_string()).unwrap_or_default();
        let stroke_color = props.get("pgf_strokecolor").map(|v| v.to_string()).unwrap_or_default();
        let mut opened_groups = 0;

        // Find the inherited fill/stroke from the nearest ancestor svg:g
        let mut inherited_fill = String::new();
        let mut inherited_stroke = String::new();
        {
          let mut n = document.get_node().clone();
          loop {
            if let Some(f) = n.get_attribute("fill") { if inherited_fill.is_empty() { inherited_fill = f; } }
            if let Some(s) = n.get_attribute("stroke") { if inherited_stroke.is_empty() { inherited_stroke = s; } }
            if !inherited_fill.is_empty() && !inherited_stroke.is_empty() { break; }
            match n.get_parent() {
              Some(p) => n = p,
              None => break,
            }
          }
        }

        // Only wrap path in svg:g if color differs from inherited
        if !stroke_color.is_empty() && stroke_color != inherited_stroke {
          document.open_element("svg:g", Some(string_map!(
            "stroke" => stroke_color,
            "_autoclose" => "1".to_string()
          )), None)?;
          opened_groups += 1;
        }
        if !fill_color.is_empty() && fill_color != inherited_fill {
          document.open_element("svg:g", Some(string_map!(
            "fill" => fill_color,
            "_autoclose" => "1".to_string()
          )), None)?;
          opened_groups += 1;
        }

        let mut attrs = string_map!("d" => d);
        if !style.is_empty() {
          attrs.insert("style".to_string(), style);
        }
        document.insert_element("svg:path", Vec::new(), Some(attrs))?;

        // Close the color groups we opened
        for _ in 0..opened_groups {
          document.close_element("svg:g")?;
        }
      }
    },
    properties => {
      // Capture colors from state during digestion, before scope is popped
      let fc = state::lookup_value("pgf@svg@fillcolor").map(|v| v.to_string()).unwrap_or_default();
      let sc = state::lookup_value("pgf@svg@strokecolor").map(|v| v.to_string()).unwrap_or_default();
      stored_map!(
        "pgf_fillcolor" => Stored::String(arena::pin(&fc)),
        "pgf_strokecolor" => Stored::String(arena::pin(&sc))
      )
    }
  );

  // Perl L341-348: clipped path — obj computed in properties (digestion) to match Perl counter order
  DefConstructor!("\\lxSVG@drawpath@clipped{}{}",
    sub[document, args, props] {
      let d = args.first().and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let style = args.get(1).and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let obj = props.get("obj").map(|v| v.to_string()).unwrap_or_default();
      document.open_element("svg:clipPath", Some(string_map!(
        "id" => format!("pgfcp{}", obj)
      )), None)?;
      document.insert_element("svg:path", Vec::new(), Some(string_map!(
        "id" => format!("pgfpath{}", obj),
        "d" => d
      )))?;
      document.close_element("svg:clipPath")?;
      let mut use_attrs = string_map!(
        "xlink:href" => format!("#pgfpath{}", obj)
      );
      if !style.is_empty() {
        use_attrs.insert("style".to_string(), style);
      }
      document.insert_element("svg:use", Vec::new(), Some(use_attrs))?;
      document.open_element("svg:g", Some(string_map!(
        "clip-path" => format!("url(#pgfcp{})", obj),
        "_autoclose" => "1".to_string()
      )), None)?;
    },
    properties => sub[_args] {
      Ok(stored_map!("obj" => svg_next_object()))
    },
    sizer => 0
  );

  // Perl L351-371: \pgfsys@discardpath
  DefMacro!("\\pgfsys@discardpath", "\\lxSVG@discardpath\\lxSVG@@discardpath");
  DefConstructor!("\\lxSVG@discardpath", "");

  DefMacro!("\\lxSVG@@discardpath", sub[_args] {
    let path = state::lookup_string("pgf_SVGpath");
    let clip = state::lookup_int("pgf_clipnext") != 0;
    if clip {
      let clip_cmd = format!("\\lxSVG@clearpath\\lxSVG@clearclip\\pgfsysprotocol@literal{{\\lxSVG@discardpath@clipped{{{}}}}}", path);
      mouth::tokenize_internal(&clip_cmd).unlist()
    } else {
      vec![]
    }
  });

  DefConstructor!("\\lxSVG@discardpath@clipped{}",
    sub[document, args, props] {
      let d = args.first().and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let obj = props.get("obj").map(|v| v.to_string()).unwrap_or_default();
      document.open_element("svg:clipPath", Some(string_map!(
        "id" => format!("pgfcp{}", obj)
      )), None)?;
      document.insert_element("svg:path", Vec::new(), Some(string_map!("d" => d)))?;
      document.close_element("svg:clipPath")?;
      document.open_element("svg:g", Some(string_map!(
        "clip-path" => format!("url(#pgfcp{})", obj),
        "_autoclose" => "1".to_string()
      )), None)?;
    },
    properties => sub[_args] {
      Ok(stored_map!("obj" => svg_next_object()))
    },
    sizer => 0
  );

  //===================================================================
  // 5. Graphic state options
  //===================================================================

  DefPrimitive!("\\lxSVG@stopexpansion", {});

  DefMacro!("\\lxSVG@begingroup{}",
    "\\ifpgfpicture\\pgfsysprotocol@literal{\\lxSVG@begingroup@{#1}}\\fi");

  // Perl L387-395: \lxSVG@begingroup@ — opens svg:g with RequiredKeyVals
  DefConstructor!("\\lxSVG@begingroup@ RequiredKeyVals",
    sub[document, args, _props] {
      let current = document.get_node().clone();
      let qname = document::get_node_qname(&current);
      let is_ltx = arena::with(qname, |s| s.starts_with("ltx:"));
      if is_ltx {
        document.open_element("svg:svg", Some(string_map!(
          "_autoopened" => "1".to_string(),
          "_autoclose" => "1".to_string()
        )), None)?;
      }
      let mut attrs = string_map!("_autoclose" => "1".to_string());
      if let Some(Some(kv_arg)) = args.first() {
        // Perl: $doc->openElement('svg:g', $kv->getHash, _autoclose => 1);
        if let DigestedData::KeyVals(kv) = kv_arg.data() {
          let hash = kv.get_hash();
          for (k, v) in hash {
            attrs.insert(k, v);
          }
        }
      }
      document.open_element("svg:g", Some(attrs), None)?;
    }
  );

  //===================================================================
  // 5b. Line width, caps, joins, dashes
  //===================================================================

  DefMacro!("\\pgfsys@setlinewidth{}",
    "\\lxSVG@setlinewidth{#1}\\lxSVG@begingroup{stroke-width={#1}}");
  DefMacro!("\\pgfsys@buttcap",
    "\\lxSVG@buttcap\\lxSVG@begingroup{stroke-linecap=butt}");
  DefMacro!("\\pgfsys@roundcap",
    "\\lxSVG@roundcap\\lxSVG@begingroup{stroke-linecap=round}");
  DefMacro!("\\pgfsys@rectcap",
    "\\lxSVG@rectcap\\lxSVG@begingroup{stroke-linecap=rect}");
  DefMacro!("\\pgfsys@miterjoin",
    "\\lxSVG@miterjoin\\lxSVG@begingroup{stroke-linejoin=miter}");
  DefMacro!("\\pgfsys@setmiterlimit{}",
    "\\lxSVG@setmiterlimit{#1}\\lxSVG@begingroup{stroke-miterlimit={#1}}");
  DefMacro!("\\pgfsys@roundjoin",
    "\\lxSVG@roundjoin\\lxSVG@begingroup{stroke-linejoin=round}");
  DefMacro!("\\pgfsys@beveljoin",
    "\\lxSVG@beveljoin\\lxSVG@begingroup{stroke-linejoin=bevel}");
  DefMacro!("\\pgfsys@setdash{}{}",
    "\\lxSVG@setdash{#1}{#2}\\edef\\pgf@test@dashpattern{#1}\\lxSVG@begingroup{stroke-dasharray={\\ifx\\pgf@test@dashpattern\\pgfutil@empty none\\else#1\\fi}, stroke-dashoffset={#2}}");

  // Override \pgfsetdash to bypass the raw TeX \pgf@strip loop.
  // Raw TeX pgfcoregraphicstate.code.tex L96-115:
  //   \def\pgfsetdash#1#2{%
  //     \def\pgf@temp{}%
  //     \pgf@strip#1{pgf@stop}%   ← iterates brace groups, converts to dimensions
  //     \pgfmathsetlength\pgf@x{#2}%
  //     \pgfsys@setdash{\pgf@temp}{\the\pgf@x}%
  //     \ignorespaces}
  // The \pgf@strip loop uses \ifx\pgf@@temp\pgf@stop as a sentinel test.
  // This loop hangs in our engine when newlines between pgfscope commands
  // create space tokens that corrupt the token stream during conditional
  // evaluation. Perl doesn't override \pgfsetdash (raw TeX works there),
  // but we override it following the same pattern as \pgfmathsetlength.
  DefPrimitive!("\\pgfsetdash{}{}", sub[(pattern_toks, offset_toks)] {
    use crate::package::pgfmath_code_tex::pgfmathparse_eval_with_units;
    // Step 1: Parse the dash pattern.
    // #1 is like {{3pt}{1.2pt}{0.6pt}} or {} (empty for solid).
    // Extract brace groups and convert each to a dimension via pgfmathparse.
    let pattern_str = pattern_toks.to_string();
    let pattern_str = pattern_str.trim();
    let mut dash_parts: Vec<String> = Vec::new();

    if !pattern_str.is_empty() {
      // Extract brace-group contents: {3pt}{1.2pt} → ["3pt", "1.2pt"]
      let mut depth = 0;
      let mut current = String::new();
      for ch in pattern_str.chars() {
        match ch {
          '{' => {
            if depth > 0 { current.push(ch); }
            depth += 1;
          },
          '}' => {
            depth -= 1;
            if depth == 0 && !current.is_empty() {
              // Evaluate this dimension via pgfmathparse
              let toks = Tokens::new(Explode!(&current));
              let expanded = gullet::do_expand(toks).unwrap_or_default();
              let input = expanded.to_string();
              let (result_str, _units) = pgfmathparse_eval_with_units(&input);
              let value: f64 = result_str.parse().unwrap_or(0.0);
              // Convert to sp then format as pt dimension (matching \the\pgf@x)
              let dim = Dimension((value * 65536.0).round() as i64);
              dash_parts.push(dim.to_string());
              current.clear();
            } else if depth > 0 {
              current.push(ch);
            }
          },
          _ => {
            if depth > 0 { current.push(ch); }
          },
        }
      }
    }

    // Step 2: Evaluate the offset #2 via pgfmathsetlength → \pgf@x
    let offset_str = offset_toks.to_string();
    let offset_str = offset_str.trim().to_string();
    if !offset_str.is_empty() {
      let toks = Tokens::new(Explode!(&offset_str));
      let expanded = gullet::do_expand(toks).unwrap_or_default();
      let input = expanded.to_string();
      let (result_str, _units) = pgfmathparse_eval_with_units(&input);
      let value: f64 = result_str.parse().unwrap_or(0.0);
      let dim = Dimension((value * 65536.0).round() as i64);
      state::assign_register("\\pgf@x", dim.into(), None, vec![])?;
    } else {
      state::assign_register("\\pgf@x", Dimension(0).into(), None, vec![])?;
    }

    // Step 3: Build the dash pattern string and call \pgfsys@setdash
    let dash_csv = dash_parts.join(",");
    let pgf_x_val = state::lookup_register("\\pgf@x", vec![])?
      .map(|v| v.to_string()).unwrap_or_else(|| "0.0pt".to_string());

    // Emit: \pgfsys@setdash{<dash_csv>}{<offset>}\ignorespaces
    let mut result = vec![T_CS!("\\pgfsys@setdash")];
    result.push(T_BEGIN!());
    result.extend(Explode!(&dash_csv));
    result.push(T_END!());
    result.push(T_BEGIN!());
    result.extend(Explode!(&pgf_x_val));
    result.push(T_END!());
    result.push(T_CS!("\\ignorespaces"));
    gullet::unread(Tokens::new(result));
  }, locked => true);
  DefMacro!("\\pgfsys@eoruletrue",
    "\\lxSVG@eoruletrue\\lxSVG@begingroup{fill-rule=evenodd}");
  DefMacro!("\\pgfsys@eorulefalse",
    "\\lxSVG@eorulefalse\\lxSVG@begingroup{fill-rule=nonzero}");

  // Constructors for reversion
  DefConstructor!("\\lxSVG@setlinewidth Undigested", "");
  DefConstructor!("\\lxSVG@buttcap", "");
  DefConstructor!("\\lxSVG@roundcap", "");
  DefConstructor!("\\lxSVG@rectcap", "");
  DefConstructor!("\\lxSVG@miterjoin", "");
  DefConstructor!("\\lxSVG@setmiterlimit Undigested", "");
  DefConstructor!("\\lxSVG@roundjoin", "");
  DefConstructor!("\\lxSVG@beveljoin", "");
  DefConstructor!("\\lxSVG@setdash Undigested Undigested", "");
  DefConstructor!("\\lxSVG@eoruletrue", "");
  DefConstructor!("\\lxSVG@eorulefalse", "");

  //===================================================================
  // 6. Color
  //===================================================================

  DefMacro!("\\lxSVG@setcolor{}{}", "\\ifpgfpicture\\lxSVG@begingroup{#1={#2}}\\fi");

  // Combined color macros: set both stroke AND fill.
  // These override the TeX definitions from pgfsys-common-svg.def which use
  // SVG-specific \pgf@sys@svg@gs@color that we don't support.
  // Perl only defines the rgb version; we also need gray/cmyk/cmy because
  // in Perl the .ltxml has priority over TeX \def, but in Rust we need locked.
  DefMacro!("\\pgfsys@color@rgb{}{}{}",
    "\\ifpgfpicture\\pgfsys@color@rgb@stroke{#1}{#2}{#3}\\pgfsys@color@rgb@fill{#1}{#2}{#3}\\fi",
    locked => true);
  DefMacro!("\\pgfsys@color@gray{}",
    "\\ifpgfpicture\\pgfsys@color@gray@stroke{#1}\\pgfsys@color@gray@fill{#1}\\fi",
    locked => true);
  DefMacro!("\\pgfsys@color@cmyk{}{}{}{}",
    "\\ifpgfpicture\\pgfsys@color@cmyk@stroke{#1}{#2}{#3}{#4}\\pgfsys@color@cmyk@fill{#1}{#2}{#3}{#4}\\fi",
    locked => true);
  DefMacro!("\\pgfsys@color@cmy{}{}{}",
    "\\ifpgfpicture\\pgfsys@color@cmy@stroke{#1}{#2}{#3}\\pgfsys@color@cmy@fill{#1}{#2}{#3}\\fi",
    locked => true);

  // Perl: DefMacro('\lxSVG@RGB{}{}{}', sub { Explode(Color('rgb', $_[1], $_[2], $_[3])->toHex); });
  // Use plain {} parameters (not {Float}) so these macros expand correctly
  // inside \edef (which pgfsysprotocol@literalbuffered uses for protocol accumulation).
  // {Float} parameters fail during \edef because the Float reader interacts
  // badly with the expansion-mode token stream.
  DefMacro!("\\lxSVG@RGB{}{}{}", sub[args] {
    let r: f64 = args.first().map(|a| a.revert().unwrap_or_default().to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let g: f64 = args.get(1).map(|a| a.revert().unwrap_or_default().to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let b: f64 = args.get(2).map(|a| a.revert().unwrap_or_default().to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
    color_to_hex_tokens(r, g, b)
  });

  DefMacro!("\\lxSVG@GRAY{}", sub[args] {
    let v: f64 = args.first().map(|a| a.revert().unwrap_or_default().to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
    color_to_hex_tokens(v, v, v)
  });

  DefMacro!("\\lxSVG@CMYK{}{}{}{}", sub[args] {
    let c: f64 = args.first().map(|a| a.revert().unwrap_or_default().to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let m: f64 = args.get(1).map(|a| a.revert().unwrap_or_default().to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let y: f64 = args.get(2).map(|a| a.revert().unwrap_or_default().to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let k: f64 = args.get(3).map(|a| a.revert().unwrap_or_default().to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
    color_to_hex_tokens((1.0-c)*(1.0-k), (1.0-m)*(1.0-k), (1.0-y)*(1.0-k))
  });

  DefMacro!("\\lxSVG@CMY{}{}{}", sub[args] {
    let c: f64 = args.first().map(|a| a.revert().unwrap_or_default().to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let m: f64 = args.get(1).map(|a| a.revert().unwrap_or_default().to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let y: f64 = args.get(2).map(|a| a.revert().unwrap_or_default().to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
    color_to_hex_tokens(1.0-c, 1.0-m, 1.0-y)
  });

  // All color fill/stroke macros: compute hex and store in pgf state.
  // The stored colors are applied by \lxSVG@drawpath@unclipped during path construction.
  // Also call the old \lxSVG@begingroup path through the protocol for the initial setup.
  DefMacro!("\\pgfsys@color@rgb@stroke{}{}{}",
    "\\lxSVG@color@rgb@stroke{#1}{#2}{#3}\\lxSVG@begingroup{stroke=\\lxSVG@RGB{#1}{#2}{#3}}", locked => true);
  DefMacro!("\\pgfsys@color@rgb@fill{}{}{}",
    "\\lxSVG@color@rgb@fill{#1}{#2}{#3}\\lxSVG@begingroup{fill=\\lxSVG@RGB{#1}{#2}{#3}}", locked => true);
  DefMacro!("\\pgfsys@color@cmyk@stroke{}{}{}{}",
    "\\lxSVG@color@cmyk@stroke{#1}{#2}{#3}{#4}\\lxSVG@begingroup{stroke=\\lxSVG@CMYK{#1}{#2}{#3}{#4}}", locked => true);
  DefMacro!("\\pgfsys@color@cmyk@fill{}{}{}{}",
    "\\lxSVG@color@cmyk@fill{#1}{#2}{#3}{#4}\\lxSVG@begingroup{fill=\\lxSVG@CMYK{#1}{#2}{#3}{#4}}", locked => true);
  DefMacro!("\\pgfsys@color@cmy@stroke{}{}{}",
    "\\lxSVG@color@cmy@stroke{#1}{#2}{#3}\\lxSVG@begingroup{stroke=\\lxSVG@CMY{#1}{#2}{#3}}", locked => true);
  DefMacro!("\\pgfsys@color@cmy@fill{}{}{}",
    "\\lxSVG@color@cmy@fill{#1}{#2}{#3}\\lxSVG@begingroup{fill=\\lxSVG@CMY{#1}{#2}{#3}}", locked => true);
  DefMacro!("\\pgfsys@color@gray@stroke{}",
    "\\lxSVG@color@gray@stroke{#1}\\lxSVG@begingroup{stroke=\\lxSVG@GRAY{#1}}", locked => true);
  DefMacro!("\\pgfsys@color@gray@fill{}",
    "\\lxSVG@color@gray@fill{#1}\\lxSVG@begingroup{fill=\\lxSVG@GRAY{#1}}", locked => true);

  // (duplicate sub versions removed — the DefMacro versions above handle all color models)

  // The \lxSVG@color@*@fill/stroke constructors store the hex color in state.
  // The color is applied by \lxSVG@drawpath@unclipped during path construction,
  // wrapping paths in svg:g elements with the correct fill/stroke attributes.
  // This approach works around the timing issue where Whatsits created during
  // tikz option processing are not captured in the digested content.

  fn rgb_hex(args: &[&ArgWrap]) -> String {
    let r: f64 = args[0].to_string().trim().parse().unwrap_or(0.0);
    let g: f64 = args[1].to_string().trim().parse().unwrap_or(0.0);
    let b: f64 = args[2].to_string().trim().parse().unwrap_or(0.0);
    format!("#{:02X}{:02X}{:02X}", channel_to_u8(r), channel_to_u8(g), channel_to_u8(b))
  }
  fn gray_hex(args: &[&ArgWrap]) -> String {
    let v: f64 = args[0].to_string().trim().parse().unwrap_or(0.0);
    let c = channel_to_u8(v);
    format!("#{:02X}{:02X}{:02X}", c, c, c)
  }
  fn cmyk_hex(args: &[&ArgWrap]) -> String {
    let c: f64 = args[0].to_string().trim().parse().unwrap_or(0.0);
    let m: f64 = args[1].to_string().trim().parse().unwrap_or(0.0);
    let y: f64 = args[2].to_string().trim().parse().unwrap_or(0.0);
    let k: f64 = args[3].to_string().trim().parse().unwrap_or(0.0);
    format!("#{:02X}{:02X}{:02X}",
      channel_to_u8((1.0-c)*(1.0-k)), channel_to_u8((1.0-m)*(1.0-k)), channel_to_u8((1.0-y)*(1.0-k)))
  }
  fn cmy_hex(args: &[&ArgWrap]) -> String {
    let c: f64 = args[0].to_string().trim().parse().unwrap_or(0.0);
    let m: f64 = args[1].to_string().trim().parse().unwrap_or(0.0);
    let y: f64 = args[2].to_string().trim().parse().unwrap_or(0.0);
    format!("#{:02X}{:02X}{:02X}", channel_to_u8(1.0-c), channel_to_u8(1.0-m), channel_to_u8(1.0-y))
  }

  // Color-channel conversion bindings — same WISDOM #44 intentional
  // divergence as the path-op block at the top of this file: Perl
  // defines each as `DefConstructor('\lxSVG@color@…@…{}{}{}', '',
  // afterDigest => sub { AssignValue('pgf@svg@…color' => …) })`;
  // Rust ports as DefPrimitive with the AssignValue in the body.
  // 8 DefConstructor → DefPrimitive flips (rgb/cmyk/cmy/gray × stroke/fill).
  DefPrimitive!("\\lxSVG@color@rgb@stroke{}{}{}", sub[args] {
    let hex = rgb_hex(&[&args[0], &args[1], &args[2]]);
    assign_value("pgf@svg@strokecolor", Stored::String(arena::pin(hex)), None);
  });
  DefPrimitive!("\\lxSVG@color@rgb@fill{}{}{}", sub[args] {
    let hex = rgb_hex(&[&args[0], &args[1], &args[2]]);
    assign_value("pgf@svg@fillcolor", Stored::String(arena::pin(hex)), None);
  });
  DefPrimitive!("\\lxSVG@color@cmyk@stroke{}{}{}{}", sub[args] {
    let hex = cmyk_hex(&[&args[0], &args[1], &args[2], &args[3]]);
    assign_value("pgf@svg@strokecolor", Stored::String(arena::pin(hex)), None);
  });
  DefPrimitive!("\\lxSVG@color@cmyk@fill{}{}{}{}", sub[args] {
    let hex = cmyk_hex(&[&args[0], &args[1], &args[2], &args[3]]);
    assign_value("pgf@svg@fillcolor", Stored::String(arena::pin(hex)), None);
  });
  DefPrimitive!("\\lxSVG@color@cmy@stroke{}{}{}", sub[args] {
    let hex = cmy_hex(&[&args[0], &args[1], &args[2]]);
    assign_value("pgf@svg@strokecolor", Stored::String(arena::pin(hex)), None);
  });
  DefPrimitive!("\\lxSVG@color@cmy@fill{}{}{}", sub[args] {
    let hex = cmy_hex(&[&args[0], &args[1], &args[2]]);
    assign_value("pgf@svg@fillcolor", Stored::String(arena::pin(hex)), None);
  });
  DefPrimitive!("\\lxSVG@color@gray@stroke{}", sub[args] {
    let hex = gray_hex(&[&args[0]]);
    assign_value("pgf@svg@strokecolor", Stored::String(arena::pin(hex)), None);
  });
  DefPrimitive!("\\lxSVG@color@gray@fill{}", sub[args] {
    let hex = gray_hex(&[&args[0]]);
    assign_value("pgf@svg@fillcolor", Stored::String(arena::pin(hex)), None);
  });

  //===================================================================
  // 7. Pattern
  //===================================================================

  // Perl L487-492: \pgfsys@declarepattern{name}{x1}{y1}{x2}{y2}{x step}{y step}{code}{flag}
  // Expands to \pgfsysprotocol@literal{\lxSVG@setpattern{...}}\lxSVG@(un)coloredpattern{...}
  DefMacro!("\\pgfsys@declarepattern{} {}{}{}{}{}{} {}{Number}", sub[args] {
    let name = args.first().map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let x1 = args.get(1).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let y1 = args.get(2).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let x2 = args.get(3).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let y2 = args.get(4).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let x_step = args.get(5).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let y_step = args.get(6).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let code = args.get(7).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let flag: i64 = args.get(8).map(|a| a.value_of()).unwrap_or(0);
    let op = if flag == 1 {
      T_CS!("\\lxSVG@coloredpattern")
    } else {
      T_CS!("\\lxSVG@uncoloredpattern")
    };
    // Invocation('\pgfsysprotocol@literal',
    //   Invocation('\lxSVG@setpattern', x1..y_step),
    //   Invocation($op, name, x_step, y_step, code))
    let mut toks = vec![T_CS!("\\pgfsysprotocol@literal"), T_BEGIN!()];
    // \lxSVG@setpattern{x1}{y1}{x2}{y2}{x_step}{y_step}
    toks.push(T_CS!("\\lxSVG@setpattern"));
    toks.push(T_BEGIN!()); toks.extend(x1.unlist()); toks.push(T_END!());
    toks.push(T_BEGIN!()); toks.extend(y1.unlist()); toks.push(T_END!());
    toks.push(T_BEGIN!()); toks.extend(x2.unlist()); toks.push(T_END!());
    toks.push(T_BEGIN!()); toks.extend(y2.unlist()); toks.push(T_END!());
    toks.push(T_BEGIN!()); toks.extend_from_slice(x_step.unlist_ref()); toks.push(T_END!());
    toks.push(T_BEGIN!()); toks.extend_from_slice(y_step.unlist_ref()); toks.push(T_END!());
    toks.push(T_END!()); // close \pgfsysprotocol@literal arg
    // \lxSVG@(un)coloredpattern{name}{x_step}{y_step}{code}
    toks.push(op);
    toks.push(T_BEGIN!()); toks.extend(name.unlist()); toks.push(T_END!());
    toks.push(T_BEGIN!()); toks.extend(x_step.unlist()); toks.push(T_END!());
    toks.push(T_BEGIN!()); toks.extend(y_step.unlist()); toks.push(T_END!());
    toks.push(T_BEGIN!()); toks.extend(code.unlist()); toks.push(T_END!());
    toks
  });

  // Perl L494-500: \lxSVG@setpattern — stores pattern bbox/step into pgf registers
  DefPrimitive!("\\lxSVG@setpattern{Dimension}{Dimension}{Dimension}{Dimension}{Dimension}{Dimension}",
    sub[(x1, y1, x2, y2, x_step, y_step)] {
    AssignRegister!("\\pgf@xa", x1.into());
    AssignRegister!("\\pgf@ya", y1.into());
    AssignRegister!("\\pgf@xb", x2.into());
    AssignRegister!("\\pgf@yb", y2.into());
    AssignRegister!("\\pgf@xc", x_step.into());
    AssignRegister!("\\pgf@yc", y_step.into());
  });

  // Perl L502-504: \pgfsys@setpatternuncolored — wraps in pgfsysprotocol@literal
  DefMacro!("\\pgfsys@setpatternuncolored{}{}{}{}", sub[args] {
    let name = args.first().map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let r = args.get(1).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let g = args.get(2).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let b = args.get(3).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    // Invocation('\pgfsysprotocol@literal',
    //   Invocation('\lxSVG@setpatternuncolored@', name, r, g, b))
    let mut toks = vec![T_CS!("\\pgfsysprotocol@literal"), T_BEGIN!()];
    toks.push(T_CS!("\\lxSVG@setpatternuncolored@"));
    toks.push(T_BEGIN!()); toks.extend(name.unlist()); toks.push(T_END!());
    toks.push(T_BEGIN!()); toks.extend(r.unlist()); toks.push(T_END!());
    toks.push(T_BEGIN!()); toks.extend(g.unlist()); toks.push(T_END!());
    toks.push(T_BEGIN!()); toks.extend(b.unlist()); toks.push(T_END!());
    toks.push(T_END!());
    toks
  });

  // Perl L506-507: \pgfsys@setpatterncolored — sets fill to pattern URL
  // \lxSVG@setcolor{fill}{url(\#pgfpat#1)} — \# produces catcode OTHER '#'
  DefMacro!("\\pgfsys@setpatterncolored{}", sub[args] {
    let name = args.first().map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    // Build: \lxSVG@setcolor{fill}{url(#pgfpat<name>)}
    let mut toks = vec![T_CS!("\\lxSVG@setcolor")];
    toks.push(T_BEGIN!()); toks.extend(mouth::tokenize_internal("fill").unlist()); toks.push(T_END!());
    toks.push(T_BEGIN!());
    toks.extend(mouth::tokenize_internal("url(").unlist());
    toks.push(T_OTHER!("#"));
    toks.extend(mouth::tokenize_internal("pgfpat").unlist());
    toks.extend(name.unlist());
    toks.push(T_OTHER!(")"));
    toks.push(T_END!());
    toks
  });

  // Perl L509-519: \lxSVG@coloredpattern — pattern with inherent color
  // <svg:defs><svg:pattern id="pgfpat{name}" patternUnits="userSpaceOnUse"
  //   width="{x_step}" height="{y_step}">{code}</svg:pattern></svg:defs>
  DefConstructor!("\\lxSVG@coloredpattern{}{Dimension}{Dimension}{}",
    sub[document, args, props] {
      let name = args.first().and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let x_step = props.get("x_step").map(|v| v.to_string()).unwrap_or_default();
      let y_step = props.get("y_step").map(|v| v.to_string()).unwrap_or_default();
      document.open_element("svg:defs", None, None)?;
      document.open_element("svg:pattern", Some(string_map!(
        "id" => format!("pgfpat{}", name.trim()),
        "patternUnits" => "userSpaceOnUse".to_string(),
        "width" => x_step,
        "height" => y_step
      )), None)?;
      // #4 — the pattern code content is absorbed by the constructor framework
    },
    properties => sub[args] {
      let x_step = args.get(1).and_then(|a| a.as_ref()).and_then(|a| a.get_dimension())
        .map(dim_to_px).unwrap_or(0.0);
      let y_step = args.get(2).and_then(|a| a.as_ref()).and_then(|a| a.get_dimension())
        .map(dim_to_px).unwrap_or(0.0);
      Ok(stored_map!("x_step" => x_step, "y_step" => y_step))
    },
    sizer => 0
  );

  // Perl L521-532: \lxSVG@uncoloredpattern — pattern without inherent color
  // <svg:defs><svg:pattern id="pgfpat{name}" ...>
  //   <svg:symbol id="pgfsym{name}">{code}</svg:symbol>
  // </svg:pattern></svg:defs>
  DefConstructor!("\\lxSVG@uncoloredpattern{}{Dimension}{Dimension}{}",
    sub[document, args, props] {
      let name = args.first().and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let name = name.trim().to_string();
      let x_step = props.get("x_step").map(|v| v.to_string()).unwrap_or_default();
      let y_step = props.get("y_step").map(|v| v.to_string()).unwrap_or_default();
      document.open_element("svg:defs", None, None)?;
      document.open_element("svg:pattern", Some(string_map!(
        "id" => format!("pgfpat{}", name),
        "patternUnits" => "userSpaceOnUse".to_string(),
        "width" => x_step,
        "height" => y_step
      )), None)?;
      document.open_element("svg:symbol", Some(string_map!(
        "id" => format!("pgfsym{}", name)
      )), None)?;
      // #4 — the pattern code content is absorbed by the constructor framework
      // The svg:symbol, svg:pattern, and svg:defs will be auto-closed
    },
    properties => sub[args] {
      let x_step = args.get(1).and_then(|a| a.as_ref()).and_then(|a| a.get_dimension())
        .map(dim_to_px).unwrap_or(0.0);
      let y_step = args.get(2).and_then(|a| a.as_ref()).and_then(|a| a.get_dimension())
        .map(dim_to_px).unwrap_or(0.0);
      Ok(stored_map!("x_step" => x_step, "y_step" => y_step))
    },
    sizer => 0
  );

  // Perl L534-544: \lxSVG@setpatternuncolored@ — applies uncolored pattern with color
  // Creates a new pattern referencing the symbol, with given stroke/fill color,
  // then opens an svg:g with fill set to the new pattern URL
  DefConstructor!("\\lxSVG@setpatternuncolored@{}{}{}{}",
    sub[document, args, props] {
      let name = args.first().and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let name = name.trim().to_string();
      let obj = props.get("obj").map(|v| v.to_string()).unwrap_or_default();
      let color = props.get("color").map(|v| v.to_string()).unwrap_or_else(|| "#000000".to_string());
      // <svg:defs><svg:pattern id="pgfupat{obj}" xlink:href="#pgfpat{name}">
      document.open_element("svg:defs", None, None)?;
      document.open_element("svg:pattern", Some(string_map!(
        "id" => format!("pgfupat{}", obj),
        "xlink:href" => format!("#pgfpat{}", name)
      )), None)?;
      // <svg:g stroke="{color}" fill="{color}">
      document.open_element("svg:g", Some(string_map!(
        "stroke" => color,
        "fill" => color
      )), None)?;
      // <svg:use xlink:href="#pgfsym{name}"/>
      document.insert_element("svg:use", Vec::new(), Some(string_map!(
        "xlink:href" => format!("#pgfsym{}", name)
      )))?;
      // close svg:g, svg:pattern, svg:defs
      document.close_element("svg:g")?;
      document.close_element("svg:pattern")?;
      document.close_element("svg:defs")?;
      // <svg:g fill="url(#pgfupat{obj})" _autoclose="1">
      document.open_element("svg:g", Some(string_map!(
        "fill" => format!("url(#pgfupat{})", obj),
        "_autoclose" => "1".to_string()
      )), None)?;
    },
    properties => sub[args] {
      let obj = svg_next_object();
      // Color('rgb', r, g, b) — args 2,3,4 are RGB components
      let r: f64 = args.get(1).and_then(|a| a.as_ref())
        .map(|a| a.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let g: f64 = args.get(2).and_then(|a| a.as_ref())
        .map(|a| a.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let b: f64 = args.get(3).and_then(|a| a.as_ref())
        .map(|a| a.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let color = format!("#{:02X}{:02X}{:02X}",
        (r * 255.0).round().clamp(0.0, 255.0) as u8,
        (g * 255.0).round().clamp(0.0, 255.0) as u8,
        (b * 255.0).round().clamp(0.0, 255.0) as u8);
      Ok(stored_map!("obj" => obj, "color" => color))
    },
    sizer => 0
  );

  //===================================================================
  // 8. Scoping
  //===================================================================

  DefMacro!("\\pgfsys@beginscope",
    "\\lxSVG@beginscope\\lxSVG@begingroup{_scopebegin=1}");
  DefMacro!("\\pgfsys@endscope",
    "\\pgfsysprotocol@literal{\\lxSVG@closescope}\\lxSVG@endscope");

  DefConstructor!("\\lxSVG@closescope",
    sub[document, _args, _props] {
      while let Ok(Some(node)) = document.maybe_close_element("svg:g") {
        if node.get_attribute("_scopebegin").is_some() {
          break;
        }
      }
    }
  );

  // Scope bindings — same WISDOM #44 intentional divergence as the
  // path-op / color blocks above: Perl `DefConstructor` with empty
  // template + afterDigest side-effect; Rust `DefPrimitive` with the
  // side-effect in the body.
  DefPrimitive!("\\lxSVG@beginscope", {
    stomach::begingroup();
  });

  DefPrimitive!("\\lxSVG@endscope", {
    let _ = stomach::endgroup();
  });

  //===================================================================
  // 9. Image
  //===================================================================

  RawTeX!("\\newbox\\lxSVG@imgbox");

  DefMacro!("\\pgfsys@defineimage",
    "\\edef\\pgf@image{\\lxSVG@includegraphics{\\pgf@imagewidth}{\\pgf@imageheight}{\\pgf@filename}}");

  // Perl L597-623: \lxSVG@includegraphics — simplified
  // The full Perl version has complex beforeConstruct/afterConstruct for foreignObject
  // wrapping. For now, just emit the graphics element.
  DefConstructor!("\\lxSVG@includegraphics{}{} Semiverbatim",
    "<ltx:graphics graphic='#3' options='#options'/>",
    properties => sub[args] {
      let w = args.first().and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let h = args.get(1).and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let graphic = args.get(2).and_then(|a| a.as_ref()).map(|a| a.to_string().trim().to_string()).unwrap_or_default();
      let mut options_parts = Vec::new();
      if !w.is_empty() { options_parts.push(format!("width={}", w)); }
      if !h.is_empty() { options_parts.push(format!("height={}", h)); }
      let options = options_parts.join(",");
      let candidates = graphic.clone();
      Ok(stored_map!("graphic" => graphic, "candidates" => candidates, "options" => options))
    }
  );

  //===================================================================
  // 11. Transparency
  //===================================================================

  DefMacro!("\\pgfsys@stroke@opacity{}",
    "\\lxSVG@stroke@opacity{#1}\\lxSVG@begingroup{stroke-opacity={#1}}");
  DefMacro!("\\pgfsys@fill@opacity{}",
    "\\lxSVG@fill@opacity{#1}\\lxSVG@begingroup{fill-opacity={#1}}");
  DefMacro!("\\pgfsys@fadingfrombox{}{}", "\\lxSVG@fadingfrombox{#1}{#2}");
  DefMacro!("\\pgfsys@usefading{}{}{}{}{}{}{}", "\\lxSVG@usefading{#1}{#2}{#3}{#4}{#5}{#6}{#7}");
  DefMacro!("\\pgfsys@transparencygroupfrombox{}", "\\lxSVG@transparencygroupfrombox{#1}");
  DefMacro!("\\pgfsys@definemask", "\\lxSVG@definemask");

  DefConstructor!("\\lxSVG@stroke@opacity Undigested", "");
  DefConstructor!("\\lxSVG@fill@opacity Undigested", "");
  DefConstructor!("\\lxSVG@fadingfrombox Undigested{Number}", "");
  DefConstructor!("\\lxSVG@usefading Undigested Undigested Undigested Undigested Undigested Undigested Undigested", "");
  DefConstructor!("\\lxSVG@transparencygroupfrombox{Number}", "");
  DefConstructor!("\\lxSVG@definemask", "");

  //===================================================================
  // 12. Reusable objects
  //===================================================================

  // Perl: DefConstructor('\pgfsys@invoke{}', sub { no warnings 'recursion';
  //   my ($document, $arg) = @_; $document->absorb($arg); return; }, sizer => 0);
  // Perl: DefConstructor('\pgfsys@invoke{}', sub { $document->absorb($arg); }, sizer => 0);
  //
  // The TeX protocol subsystem accumulates content in \pgfsysprotocol@currentprotocol
  // Perl: DefConstructor('\pgfsys@invoke{}', sub {
  //   no warnings 'recursion'; $document->absorb($arg); }, sizer => 0);
  // Use DefPrimitive to read the token argument and re-digest it so that
  // constructors like \lxSVG@begingroup@ fire during digestion.
  // DefMacro #1 causes timing issues (constructors fire too late).
  // DefConstructor with absorb() works BUT only if the argument is properly digested.
  // Perl: DefConstructor('\pgfsys@invoke{}', sub {
  //   no warnings 'recursion'; $document->absorb($arg); }, sizer => 0);
  // Use DefMacro to ensure content flows back through the normal pipeline.
  // The content is already pre-expanded (hex computed in Rust).
  //
  // Intentional divergence (WISDOM #44 class: re-digest-pipeline timing):
  // Perl's DefConstructor+absorb re-enters the document digester; the
  // Rust DefMacro pass-through puts #1 back on the input stream so the
  // expansion + digest pipeline proceeds naturally. Observationally
  // equivalent when the hex is pre-computed in Rust. Audit flags the
  // single L1448 entry.
  DefMacro!("\\pgfsys@invoke{}", "#1", locked => true);

  def_macro_noop("\\pgfsys@markposition{}")?;

  //===================================================================
  // 13. Invisibility
  //===================================================================

  RawTeX!("\\def\\pgfsys@begininvisible#1\\pgfsys@endinvisible{}");

  //===================================================================
  // 10. Shading
  //===================================================================

  // Perl L632-639: \lxSVG@sh@create — expands shading ranges into interval calls
  DefMacro!("\\lxSVG@sh@create", sub[_args] {
    let mut toks = vec![T_CS!("\\lxSVG@sh@intervals")];
    toks.extend(gullet::do_expand(T_CS!("\\pgf@sys@shading@ranges"))?.unlist());
    toks.push(T_BEGIN!());
    toks.push(T_BEGIN!());
    toks.push(T_CS!("\\pgf@sys@shading@end@pos"));
    toks.push(T_END!());
    toks.push(T_BEGIN!());
    toks.push(T_CS!("\\pgf@sys@shading@end@pos"));
    toks.push(T_END!());
    toks.push(T_BEGIN!());
    toks.push(T_CS!("\\pgf@sys@shading@end@rgb"));
    toks.push(T_END!());
    toks.push(T_BEGIN!());
    toks.push(T_CS!("\\pgf@sys@shading@end@rgb"));
    toks.push(T_END!());
    toks.push(T_END!());
    toks.push(T_BEGIN!());
    toks.push(T_END!());
    toks
  });

  // Perl L641-642: \lxSVG@sh@interval@ — stash a single stop
  DefMacro!("\\lxSVG@sh@interval@{}{}",
    "\\lxSVG@sh@stashstop{\\lxSVG@sh@stop{#1}{\\pgf@sys@shading@end@pos}#2}");

  // Perl L644-646: \lxSVG@sh@stashstop — digest arg and push to pgf_sh_stops list
  DefPrimitive!("\\lxSVG@sh@stashstop{}", sub[args] {
    if let Some(arg) = args.into_iter().next() {
      if arg.is_some() {
        let tokens = arg.revert()?;
        let digested = stomach::digest(tokens)?;
        state::push_value("pgf_sh_stops", digested)?;
      }
    }
  });

  // Perl L648-654: \lxSVG@sh@stop — creates <svg:stop> element
  DefConstructor!("\\lxSVG@sh@stop{Dimension}{Dimension}{Float}{Float}{Float}",
    sub[document, args, _props] {
      let start_dim = args.first().and_then(|a| a.as_ref()).and_then(|a| a.get_dimension());
      let end_dim = args.get(1).and_then(|a| a.as_ref()).and_then(|a| a.get_dimension());
      let r: f64 = args.get(2).and_then(|a| a.as_ref()).map(|a| a.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let g: f64 = args.get(3).and_then(|a| a.as_ref()).map(|a| a.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let b: f64 = args.get(4).and_then(|a| a.as_ref()).map(|a| a.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      if let (Some(s), Some(e)) = (start_dim, end_dim) {
        let s_px = dim_to_px(s);
        let e_px = dim_to_px(e);
        let offset = if e_px != 0.0 {
          floatformat(s_px / e_px)
        } else {
          "0".to_string()
        };
        let stopcolor = format!("#{:02X}{:02X}{:02X}",
          (r * 255.0).round() as u8,
          (g * 255.0).round() as u8,
          (b * 255.0).round() as u8);
        document.insert_element("svg:stop", Vec::new(), Some(string_map!(
          "offset" => offset,
          "stop-color" => stopcolor
        )))?;
      }
    },
    sizer => 0
  );

  // Perl L656-657: \lxSVG@sh@interval — forwards to interval@
  DefMacro!("\\lxSVG@sh@interval{}{}{}{}",
    "\\lxSVG@sh@interval@{#1}{#3}");

  // Perl L659-664: \lxSVG@sh@intervals — recursive interval processing
  DefMacro!("\\lxSVG@sh@intervals{}", sub[args] {
    if let Some(pt) = args.into_iter().next() {
      if pt.is_some() {
        let expanded = gullet::do_expand(pt.revert()?)?;
        let s = expanded.to_string();
        if s.trim().is_empty() {
          return Ok(Tokens::default());
        }
        let mut toks = vec![T_CS!("\\lxSVG@sh@interval")];
        toks.extend(expanded.unlist());
        toks.push(T_CS!("\\lxSVG@sh@intervals"));
        return Ok(Tokens::new(toks));
      }
    }
    Ok(Tokens::default())
  });

  // Perl L667-689: \lxSVG@sh@defstripes — creates linearGradient constructors
  DefPrimitive!("\\lxSVG@sh@defstripes{}{Number}", sub[args] {
    let name = args.first().map(|a| a.to_string()).unwrap_or_default();
    let flag: i64 = args.get(1).map(|a| a.value_of()).unwrap_or(0);
    // Collect digested stops from state
    let stops: Vec<Digested> = match state::lookup_value("pgf_sh_stops") { Some(Stored::VecDequeStored(vd)) => {
      vd.into_iter().filter_map(|s| if let Stored::Digested(d) = s { Some(d) } else { None }).collect()
    } _ => { vec![] }};
    state::assign_value("pgf_sh_stops", Stored::VecDequeStored(VecDeque::new()), Scope::Global);
    let x = dim_to_px(read_dim_register("\\pgf@x"));
    let y = dim_to_px(read_dim_register("\\pgf@y"));
    let is_vertical = flag == 1;

    // Define \@pgfshading<name>! as a primitive
    let shading_cs = T_CS!(format!("\\@pgfshading{}!", name));
    let closure: PrimitiveBody = PrimitiveBody::Closure(Rc::new(move |_args| {
      let objcount = svg_next_object();
      let zero_sizer: Option<latexml_core::definition::SizingClosure> = Some(Rc::new(|_| Ok((Dimension::default(), Dimension::default(), Dimension::default()))));

      // Define \lxSVG@sh@defs constructor — emits linearGradient with stops
      let stops_clone = stops.clone();
      let defs_replacement: ReplacementClosure = Rc::new(move |document, _args, _props| {
        document.open_element("svg:defs", None, None)?;
        let mut grad_attrs = string_map!("id" => format!("pgfsh{}", objcount));
        if is_vertical {
          grad_attrs.insert("gradientTransform".to_string(), "rotate(90)".to_string());
        }
        document.open_element("svg:linearGradient", Some(grad_attrs), None)?;
        for stop in &stops_clone {
          document.absorb(stop, None)?;
        }
        document.close_element("svg:linearGradient")?;
        document.close_element("svg:defs")?;
        Ok(())
      });
      def_constructor(T_CS!("\\lxSVG@sh@defs"), None, Some(defs_replacement),
        ConstructorOptions { sizer: zero_sizer.clone(), ..Default::default() });

      // Define \lxSVG@sh constructor — emits rect with gradient fill
      let sh_replacement: ReplacementClosure = Rc::new(move |document, _args, _props| {
        document.insert_element("svg:rect", Vec::new(), Some(string_map!(
          "width" => format!("{}", x),
          "height" => format!("{}", y),
          "style" => format!("fill:url(#pgfsh{});stroke:none", objcount)
        )))?;
        Ok(())
      });
      def_constructor(T_CS!("\\lxSVG@sh"), None, Some(sh_replacement),
        ConstructorOptions { sizer: zero_sizer, ..Default::default() });

      // Define \lxSVG@pos macro
      let pos_toks = mouth::tokenize_internal(
        &format!("\\pgfpoint{{{x}}}{{{y}}}"));
      let _ = def_macro(T_CS!("\\lxSVG@pos"), None, pos_toks, None);

      Ok(vec![Digested::default()])
    }));
    let lock_key = format!("\\@pgfshading{}!:locked", name);
    state::install_definition(Primitive {
      cs: shading_cs,
      replacement: Some(closure),
      ..Primitive::default()
    }, Some(Scope::Global));
    state::assign_value(&lock_key, true, Scope::Global);
  });

  // Perl L691-714: \lxSVG@sh@defcircles — creates radialGradient constructors
  DefPrimitive!("\\lxSVG@sh@defcircles{}", sub[args] {
    let name = args.first().map(|a| a.to_string().trim().to_string()).unwrap_or_default();
    // Collect digested stops from state
    let stops: Vec<Digested> = match state::lookup_value("pgf_sh_stops") { Some(Stored::VecDequeStored(vd)) => {
      vd.into_iter().filter_map(|s| if let Stored::Digested(d) = s { Some(d) } else { None }).collect()
    } _ => { vec![] }};
    state::assign_value("pgf_sh_stops", Stored::VecDequeStored(VecDeque::new()), Scope::Global);
    // Perl: Dimension(ToString(Expand(T_CS('\pgf@sys@shading@end@pos'))))->pxValue
    let endpos_tokens = gullet::do_expand(T_CS!("\\pgf@sys@shading@end@pos"))?;
    let endpos_str = endpos_tokens.to_string();
    let endpos_dim = endpos_str.trim().parse::<f64>().ok()
      .map(|pts| Dimension::new((pts * 65536.0) as i64))
      .unwrap_or_else(|| {
        // Try parsing as TeX dimension (e.g. "28.45274pt")
        let s = endpos_str.trim().trim_end_matches("pt");
        let pts: f64 = s.parse().unwrap_or(1.0);
        Dimension::new((pts * 65536.0) as i64)
      });
    let endpos = dim_to_px(endpos_dim);
    let pgfx = dim_to_px(read_dim_register("\\pgf@x"));
    let pgfy = dim_to_px(read_dim_register("\\pgf@y"));
    let fx = floatformat(pgfx * 8.0 / (endpos * 16.0) + 0.5);
    let fy = floatformat(pgfy * 8.0 / (endpos * 16.0) + 0.5);

    let shading_cs = T_CS!(format!("\\@pgfshading{}!", name));
    let fx_clone = fx;
    let fy_clone = fy;
    let closure: PrimitiveBody = PrimitiveBody::Closure(Rc::new(move |_args| {
      let objcount = svg_next_object();
      let fx_c = fx_clone.clone();
      let fy_c = fy_clone.clone();
      let zero_sizer: Option<latexml_core::definition::SizingClosure> = Some(Rc::new(|_| Ok((Dimension::default(), Dimension::default(), Dimension::default()))));

      // Define \lxSVG@sh@defs — radialGradient with stops
      let stops_clone = stops.clone();
      let defs_replacement: ReplacementClosure = Rc::new(move |document, _args, _props| {
        document.open_element("svg:defs", None, None)?;
        document.open_element("svg:radialGradient", Some(string_map!(
          "id" => format!("pgfsh{}", objcount),
          "fx" => fx_c.clone(),
          "fy" => fy_c.clone()
        )), None)?;
        for stop in &stops_clone {
          document.absorb(stop, None)?;
        }
        document.close_element("svg:radialGradient")?;
        document.close_element("svg:defs")?;
        Ok(())
      });
      def_constructor(T_CS!("\\lxSVG@sh@defs"), None, Some(defs_replacement),
        ConstructorOptions { sizer: zero_sizer.clone(), ..Default::default() });

      // Define \lxSVG@sh — circle with gradient fill
      let sh_replacement: ReplacementClosure = Rc::new(move |document, _args, _props| {
        document.insert_element("svg:circle", Vec::new(), Some(string_map!(
          "cx" => format!("{}", endpos),
          "cy" => format!("{}", endpos),
          "r" => format!("{}", endpos),
          "style" => format!("fill:url(#pgfsh{});stroke:none", objcount)
        )))?;
        Ok(())
      });
      def_constructor(T_CS!("\\lxSVG@sh"), None, Some(sh_replacement),
        ConstructorOptions { sizer: zero_sizer, ..Default::default() });

      // Define \lxSVG@pos macro
      let pos_toks = mouth::tokenize_internal(
        &format!("\\pgfpoint{{{}}}{{{}}}", 2.0 * endpos, 2.0 * endpos));
      let _ = def_macro(T_CS!("\\lxSVG@pos"), None, pos_toks, None);

      Ok(vec![Digested::default()])
    }));
    let lock_key = format!("\\@pgfshading{}!:locked", name);
    state::install_definition(Primitive {
      cs: shading_cs,
      replacement: Some(closure),
      ..Primitive::default()
    }, Some(Scope::Global));
    state::assign_value(&lock_key, true, Scope::Global);
  });

  // Perl L716-724: \lxSVG@sh@insert — wraps content in translate group
  // NOTE: Perl uses ptValue not pxValue here (comment says "Something odd with scales")
  DefConstructor!("\\lxSVG@sh@insert{Dimension}{Dimension}{}",
    sub[document, args, _props] {
      let x = args.first().and_then(|a| a.as_ref()).and_then(|a| a.get_dimension())
        .map(|d| d.value_of() as f64 / 65536.0).unwrap_or(0.0);
      let y = args.get(1).and_then(|a| a.as_ref()).and_then(|a| a.get_dimension())
        .map(|d| d.value_of() as f64 / 65536.0).unwrap_or(0.0);
      document.open_element("svg:g", Some(string_map!(
        "transform" => format!("translate({} {})", floatformat(x), floatformat(y))
      )), None)?;
      if let Some(Some(content)) = args.get(2) {
        document.absorb(content, None)?;
      }
      document.close_element("svg:g")?;
    },
    sizer => 0
  );

  // Perl L728-732: \lxSVG@process — update bounding box
  DefMacro!("\\lxSVG@process{}{}",
    "\\ifdim\\pgf@picmaxx<#1\\global\\pgf@picmaxx=#1\\fi\\ifdim\\pgf@picmaxy<#2\\global\\pgf@picmaxy=#2\\fi\\ifdim\\pgf@picminx>#1\\global\\pgf@picminx=#1\\fi\\ifdim\\pgf@picminy>#2\\global\\pgf@picminy=#2\\fi");

  // Perl L737-746: \pgfsys@shadinginsidepgfpicture
  RawTeX!("\\def\\pgfsys@shadinginsidepgfpicture#1{#1\\lxSVG@sh@defs\\pgf@process{\\lxSVG@pos}\\pgf@x=-.5\\pgf@x\\relax\\pgf@y=-.5\\pgf@y\\relax\\lxSVG@sh@insert{\\pgf@x}{\\pgf@y}{\\lxSVG@sh}}");

  // Perl L748-762: \pgfsys@shadingoutsidepgfpicture
  RawTeX!("\\def\\pgfsys@shadingoutsidepgfpicture#1{\\begingroup\\lxSVG@installcommands#1\\setbox\\pgfpic=\\hbox to0pt{\\lxSVG@sh@defs\\lxSVG@sh}\\pgf@process{\\lxSVG@pos}\\pgf@picminx=0pt\\pgf@picminy=0pt\\pgf@picmaxx=\\pgf@x\\pgf@picmaxy=\\pgf@y\\def\\pgf@shift@baseline{0pt}\\pgfsys@typesetpicturebox{\\pgfpic}\\endgroup}");

  // Perl L766-773: \pgfsys@horishading
  DefMacro!("\\pgfsys@horishading{}{}{}", sub[args] {
    let name = args.first().map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let height = args.get(1).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let specs = args.get(2).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let mut toks = Vec::new();
    toks.push(T_CS!("\\pgf@parsefunc"));
    toks.push(T_BEGIN!());
    toks.extend(specs.unlist());
    toks.push(T_END!());
    toks.push(T_CS!("\\lxSVG@sh@create"));
    toks.push(T_CS!("\\pgf@process"));
    toks.push(T_BEGIN!());
    toks.push(T_CS!("\\pgfpoint"));
    toks.push(T_BEGIN!());
    toks.push(T_CS!("\\pgf@sys@shading@end@pos"));
    toks.push(T_END!());
    toks.push(T_BEGIN!());
    toks.extend(height.unlist());
    toks.push(T_END!());
    toks.push(T_END!());
    toks.push(T_CS!("\\lxSVG@sh@defstripes"));
    toks.push(T_BEGIN!());
    toks.extend(name.unlist());
    toks.push(T_END!());
    toks.push(T_BEGIN!());
    toks.push(CharToken!('0'));
    toks.push(T_END!());
    toks
  });

  // Perl L776-782: \pgfsys@vertshading
  DefMacro!("\\pgfsys@vertshading{}{}{}", sub[args] {
    let name = args.first().map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let height = args.get(1).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let specs = args.get(2).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let mut toks = Vec::new();
    toks.push(T_CS!("\\pgf@parsefunc"));
    toks.push(T_BEGIN!());
    toks.extend(specs.unlist());
    toks.push(T_END!());
    toks.push(T_CS!("\\lxSVG@sh@create"));
    toks.push(T_CS!("\\pgf@process"));
    toks.push(T_BEGIN!());
    toks.push(T_CS!("\\pgfpoint"));
    toks.push(T_BEGIN!());
    toks.push(T_CS!("\\pgf@sys@shading@end@pos"));
    toks.push(T_END!());
    toks.push(T_BEGIN!());
    toks.extend(height.unlist());
    toks.push(T_END!());
    toks.push(T_END!());
    toks.push(T_CS!("\\lxSVG@sh@defstripes"));
    toks.push(T_BEGIN!());
    toks.extend(name.unlist());
    toks.push(T_END!());
    toks.push(T_BEGIN!());
    toks.push(CharToken!('1'));
    toks.push(T_END!());
    toks
  });

  // Perl L784-789: \pgfsys@radialshading
  DefMacro!("\\pgfsys@radialshading{}{}{}", sub[args] {
    let name = args.first().map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let point = args.get(1).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let specs = args.get(2).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let mut toks = Vec::new();
    toks.push(T_CS!("\\pgf@parsefunc"));
    toks.push(T_BEGIN!());
    toks.extend(specs.unlist());
    toks.push(T_END!());
    toks.push(T_CS!("\\lxSVG@sh@create"));
    toks.push(T_CS!("\\pgf@process"));
    toks.push(T_BEGIN!());
    toks.extend(point.unlist());
    toks.push(T_END!());
    toks.push(T_CS!("\\lxSVG@sh@defcircles"));
    toks.push(T_BEGIN!());
    toks.extend(name.unlist());
    toks.push(T_END!());
    toks
  });

  // Perl L792-796: \pgfsys@functionalshading — not implementable (PostScript functions)
  DefMacro!("\\pgfsys@functionalshading{}{}{}{}", sub[_args] {

    mouth::tokenize_internal(
      "\\let\\lxSVG@sh@defs\\relax\\let\\lxSVG@sh\\relax\\let\\lxSVG@pos\\relax").unlist()
  });

  //===================================================================
  // PGF sentinel tokens — make non-expandable
  //===================================================================
  // PGF defines self-referential sentinel macros like:
  //   \def\pgfkeys@mainstop{\pgfkeys@mainstop}
  //   \def\pgfkeysnovalue{\pgfkeys@novalue}
  //   \def\pgfkeysvaluerequired{\pgfkeysvaluerequired}
  // These are used with \ifx for sentinel comparisons and as delimiters
  // in parameter patterns. They should NEVER be expanded.
  //
  // In our engine, self-referential macros trigger recursion detection
  // which replaces the expansion with empty tokens, breaking the
  // sentinel pattern (the sentinel disappears, causing pgfkeys to read
  // past it). This manifests as an infinite loop when babel + tikz
  // are loaded together (babel's AtBeginDocument hooks trigger pgfkeys
  // processing where the sentinel is expanded).
  //
  // Fix: override as locked non-expandable primitives. This preserves:
  // - \ifx comparisons: \futurelet + \ifx compares meanings, and
  //   \let-copied primitives have equal meaning.
  // - Delimiter matching: TeX matches delimiters by token identity
  //   (CS name), not meaning, so Primitives work as delimiters.
  DefPrimitive!("\\pgfkeys@mainstop", {}, locked => true);
  DefPrimitive!("\\pgfkeysvaluerequired", {}, locked => true);

});
