//! xylatexml.tex — LaTeXML SVG driver for xy-pic
//! Perl: xylatexml.tex.ltxml (1093 lines)
//!
//! Full port of the SVG drawing primitives for xy-pic diagrams.
use crate::prelude::*;

/// Helper: convert dimension to px value
/// Perl: $dim->pxValue  =>  (sp / 65536) * (DPI / 72.27)
fn dim_to_px(d: Dimension) -> f64 {
  let dpi = state::lookup_int("DPI");
  let dpi = if dpi > 0 { dpi as f64 } else { 100.0 };
  (d.value_of() as f64 / 65536.0) * (dpi / 72.27)
}

/// Helper: read an xy register as dimension
fn xy_reg_dim(name: &str) -> Dimension {
  match state::lookup_register(name, Vec::new()) {
    Ok(Some(RegisterValue::Dimension(d))) => d,
    Ok(Some(RegisterValue::Number(n))) => Dimension::new(n.value_of()),
    _ => Dimension::new(0),
  }
}

/// Helper: read an xy register as number
fn xy_reg_num(name: &str) -> i64 {
  match state::lookup_register(name, Vec::new()) {
    Ok(Some(RegisterValue::Number(n))) => n.value_of(),
    _ => 0,
  }
}

/// Helper: read a macro's expansion as a string
fn macro_string(cs: &str) -> String {
  gullet::do_expand(T_CS!(cs))
    .map(|t| t.to_string())
    .unwrap_or_default()
}

/// Helper: read a macro's raw definition body (no expansion).
/// Mirrors Perl's `ToString(LookupDefinition(T_CS('\\cs'))->getExpansion)` — it
/// returns the *right-hand side* of the `\def`, not the result of evaluating it.
/// Required when the body itself contains drawing directives (e.g. `\dir{-->}`)
/// that must not be re-invoked: expanding them while deciphering the curve style
/// re-enters the curve pipeline and loops without bound.
fn macro_body(cs: &str) -> String {
  let Ok(Some(defn)) = state::lookup_definition(&T_CS!(cs)) else {
    return String::new();
  };
  match defn.get_expansion() {
    Some(ExpansionBody::Tokens(tks)) => tks.to_string(),
    _ => String::new(),
  }
}

/// Helper: get cos/sin direction (Perl: xy_getOrientation)
fn xy_get_orientation() -> (f64, f64) {
  let c: f64 = macro_string("\\cosDirection").parse().unwrap_or(1.0);
  let s: f64 = macro_string("\\sinDirection").parse().unwrap_or(0.0);
  (c, s)
}

/// Helper: get stroke color from font
fn xy_stroke_color() -> String {
  // Read font color from state (Perl: LookupValue('font')->getColor)
  if let Some(font) = state::lookup_font() {
    if let Some(color) = font.get_color() {
      return color.to_attribute();
    }
  }
  String::from("#000000")
}

/// Helper: get stroke/fill attributes from state
fn xy_fill_stroke() -> (String, String) {
  let color = xy_stroke_color();
  let stroke = if state::lookup_bool("xy_stroke") {
    color.clone()
  } else {
    String::from("none")
  };
  let fill = if state::lookup_bool("xy_fill") {
    color
  } else {
    String::from("none")
  };
  (stroke, fill)
}

/// Helper: build an SVG path string from mixed items
/// Perl: xy_packpath — joins strings and Dimension->pxValue
fn xy_packpath(parts: &[XyPathPart]) -> String {
  parts
    .iter()
    .map(|p| match p {
      XyPathPart::Cmd(s) => s.to_string(),
      XyPathPart::Dim(d) => format!("{}", dim_to_px(*d)),
      XyPathPart::Px(v) => format!("{}", v),
    })
    .collect::<Vec<_>>()
    .join(" ")
}

enum XyPathPart {
  Cmd(&'static str),
  Dim(Dimension),
  // Px is reserved for callers that already have pixel-unit values; currently
  // no call site constructs this variant, but xy_packpath handles it.
  #[allow(dead_code)]
  Px(f64),
}

/// Helper: insert an empty SVG element with attributes
/// Uses floatToElement to find the right insertion point (matching Perl's "^" prefix).
/// This ensures SVG elements like svg:path go into svg:g, not ltx:text.
fn svg_empty_element(
  document: &mut Document,
  tag: &str,
  attrs: HashMap<String, String>,
) -> Result<()> {
  let savenode = document.float_to_element(tag, false)?;
  document.open_element(tag, Some(attrs), None)?;
  document.close_element(tag)?;
  if let Some(saved) = savenode {
    document.set_node(&saved);
  }
  Ok(())
}

/// Helper: capture common xy SVG element properties at digest time.
/// Returns (stroke, fill, dashes) for use in after_digest handlers.
fn xy_capture_stroke_fill() -> (String, String, String) {
  let (stroke, fill) = xy_fill_stroke();
  let dashes = state::lookup_string("xy_linepattern");
  (stroke, fill, dashes)
}

/// Helper: read SVG path attributes from props at construction time and emit element.
fn xy_emit_path(
  document: &mut Document,
  props: &latexml_core::common::arena::SymHashMap<Stored>,
) -> Result<()> {
  let path = match props.get("xy_path") {
    Some(Stored::String(s)) => arena::to_string(*s),
    _ => return Ok(()), // no path → skip
  };
  let stroke = match props.get("xy_stroke") {
    Some(Stored::String(s)) => arena::to_string(*s),
    _ => String::from("#000000"),
  };
  let fill = match props.get("xy_fill") {
    Some(Stored::String(s)) => arena::to_string(*s),
    _ => String::from("none"),
  };
  let mut attrs = string_map!("d" => path, "stroke" => stroke, "fill" => fill);
  if let Some(Stored::String(d)) = props.get("xy_dashes") {
    // Avoid allocating the owned dashes string when it's empty (the
    // common "no dash pattern" case, fires on every solid xy line).
    if !arena::with(*d, |s| s.is_empty()) {
      attrs.insert(String::from("stroke-dasharray"), arena::to_string(*d));
    }
  }
  svg_empty_element(document, "svg:path", attrs)
}

/// Helper: emit SVG circle from props.
fn xy_emit_circle(
  document: &mut Document,
  props: &latexml_core::common::arena::SymHashMap<Stored>,
) -> Result<()> {
  let cx = match props.get("xy_cx") {
    Some(Stored::String(s)) => arena::to_string(*s),
    _ => String::from("0"),
  };
  let cy = match props.get("xy_cy") {
    Some(Stored::String(s)) => arena::to_string(*s),
    _ => String::from("0"),
  };
  let r = match props.get("xy_r") {
    Some(Stored::String(s)) => arena::to_string(*s),
    _ => String::from("0"),
  };
  let stroke = match props.get("xy_stroke") {
    Some(Stored::String(s)) => arena::to_string(*s),
    _ => String::from("#000000"),
  };
  let fill = match props.get("xy_fill") {
    Some(Stored::String(s)) => arena::to_string(*s),
    _ => String::from("none"),
  };
  let mut attrs = string_map!("cx" => cx, "cy" => cy, "r" => r, "stroke" => stroke, "fill" => fill);
  if let Some(Stored::String(d)) = props.get("xy_dashes") {
    if !arena::with(*d, |s| s.is_empty()) {
      attrs.insert(String::from("stroke-dasharray"), arena::to_string(*d));
    }
  }
  svg_empty_element(document, "svg:circle", attrs)
}

/// Helper: format a float for SVG output, rounding to 2 decimal places
fn fmt2(v: f64) -> String {
  let r = (v * 100.0).round() / 100.0;
  if r == 0.0 {
    String::from("0")
  } else {
    format!("{}", r)
  }
}

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("color");

  // RawTeX blocks from Perl (L25-51, L157-173, L187-230, L277-278, L328-331,
  //   L335, L348, L387-416, L401-416, L579-600, L750-756)
  // These redefine xy internals to use our LaTeXML SVG constructors.
  ::latexml_core::stomach::raw_tex(concat!(
    // Perl L25-51: Register latexml driver, set up dimension registers
    "\\xyprovide{latexml}{LaTeXML Driver}{0.8.6}{Bruce Miller}{\\url{https://dlmf.nist.gov/LaTeXML/}}{}\n",
    "\\newdriver{\n",
    "\\xyaddsupport{latexml}\\lx@xy@latexmlon\n",
    "\\xyaddsupport{curve}\\lx@xy@curveon\n",
    "\\xyaddsupport{frame}\\lx@xy@frameon\n",
    "\\xyaddsupport{tips}\\lx@xy@tipson\n",
    "\\xyaddsupport{line}\\lx@xy@lineon\n",
    "\\xyaddsupport{rotate}\\lx@xy@rotateon\n",
    "\\xyaddsupport{color}\\lx@xy@coloron\n",
    "\\xyaddsupport{crayon}\\lx@xy@crayonon\n",
    "\\xyaddsupport{matrix}\\lx@xy@matrixon\n",
    "\\xyaddsupport{arrow}\\lx@xy@arrowon\n",
    "\\xyaddsupport{graph}\\lx@xy@graphon\n",
    "\\xyaddsupport{arc}\\lx@xy@arcon\n",
    "\\xyaddsupport{knot}\\lx@xy@polyon\n",
    "\\xyaddsupport{poly}\\lx@xy@knoton\n",
    "\\xyaddsupport{tile}\\lx@xy@tileon\n",
    "\\xyaddsupport{web}\\lx@xy@webon\n",
    "}\n",
    "\\newdimen\\xydashl@\\xydashl@=5pt\\relax\n",
    "\\newdimen\\xydashh@\\xydashh@=2.0pt\\relax\n",
    "\\newdimen\\xydashw@\\xydashw@=0.4pt\\relax\n",
    "\\newdimen\\xybsqll@\\xybsqll@=3.53554pt\\relax\n",
    "\\newdimen\\xybsqlh@\\xybsqlh@=1.46448pt\\relax\n",
    "\\newdimen\\xybsqlw@\\xybsqlw@=0.4pt\\relax\n",
  ))?;

  // Perl L157-172: \imposeDirection@i — calculate direction from K-angle
  // Redefines to use \lx@xy@calculate@direction instead of font metrics
  ::latexml_core::stomach::raw_tex(concat!(
    r"\def\imposeDirection@i{%", "
",
    r" \count@@=\K@ \multiply\count@@ by8 \advance\count@@\Direction",
    r" \count@=\count@@ \advance\count@\KK@ \divide\count@64 \advance\count@\m@ne",
    r" \loop@\ifnum127<\count@ \advance\count@-128 \repeat@",
    r" \chardef\DirectionChar\count@",
    r" \advance\count@@16 \divide\count@@\KK@ \advance\count@@\m@ne",
    r" \loop@\ifnum127<\count@@ \advance\count@@-128 \repeat@",
    r" \chardef\SemiDirectionChar\count@@",
    r" \lx@xy@calculate@direction",
    r"}",
  ))?;

  // Perl L187-230: \straight@typeset — route through \lx@xy@straight@typeset
  // instead of original \straighth@/\straightv@ which use font metrics
  ::latexml_core::stomach::raw_tex(concat!(
    r"\def\straight@typeset{%", "
",
    r" \ifInvisible@ \let\next@=\relax",
    r" \else \DN@{\lx@xy@straight@typeset}%", "
",
    r" \fi \checkoverlap@@ \next@}",
    r"\def\lx@xy@straight@typeset{\setbox\z@=\hbox{%", "
",
    r" \setbox8=\copy\lastobjectbox@",
    r" \A@=\wd8\relax \B@=\dp8\relax \advance\B@\ht8\relax",
    r" \ifdim \A@=\z@ \count@@=\m@ne",
    r" \else \dimen@=\sd@X\d@X \divide\dimen@\A@ \count@@=\dimen@ \fi",
    r" \Spread@@",
    r" \ifdim\d@X>\z@ \advance\X@c-\wd8\relax\fi",
    r" \dimen@=-\sd@X\wd8\relax",
    r" \multiply\dimen@\K@dYdX \divide\dimen@\K@",
    r" \ifdim\d@X>\z@ \advance\Y@c\dimen@ \advance\Y@c-\Leftness@\dimen@",
    r" \else \advance\Y@c\Leftness@\dimen@ \fi",
    r" \dimen@=\wd8\relax \A@=\sd@X\d@X \advance\A@-\dimen@",
    r" \B@=\sd@X\dimen@ \multiply\B@\K@dYdX \divide\B@\K@",
    r" \advance\B@-\d@Y \B@=\sd@Y\B@",
    r" \count@=\count@@ \advance\count@\m@ne",
    r" \ifnum\z@<\count@ \divide\A@\count@ \divide\B@\count@ \fi",
    r" \A@=-\sd@X\A@ \B@=\sd@Y\B@ \wd8=\A@",
    r"  \count@=\z@",
    r" \loop@\ifnum\count@<\count@@ \advance\count@\@ne",
    r"   \lx@xy@move@to{\X@c}{\Y@c}{\copy8}\advance\X@c\A@\relax",
    r"   \advance\Y@c\B@ \repeat@}%", "
",
    r" \ht\z@=\z@ \wd\z@=\z@ \dp\z@=\z@ {\Drop@@}}",
  ))?;

  // Perl L328-335: Spread macros — route line drawing through \lx@xy@drawline
  ::latexml_core::stomach::raw_tex(concat!(
    r"\def\solidSpread@{\lx@xy@solidpat\lx@xy@drawline}",
    r"\def\dottedSpread@#1{\lx@xy@dotpat\lx@xy@drawline}",
    r"\def\dashedSpread@{\lx@xy@dashpat\lx@xy@drawline}",
    r"\def\squiggledSpread@{\lx@xy@solidpat\lx@xy@drawsquiggles}",
    r"\def\dashsquiggledSpread@{\lx@xy@dashpat\lx@xy@drawsquiggles}",
  ))?;

  // Perl L387-416: Tip macros + Tip@ redefinitions
  ::latexml_core::stomach::raw_tex(concat!(
    r"\def\atip@@{\lx@xy@tip{1}}",
    r"\def\btip@@{\lx@xy@tip{-1}}",
    r"\def\Tip@@{\lx@xy@tip{1.5}\lx@xy@tip{-1.5}}",
    r"\def\Ttip@@{\lx@xy@tip{2}\lx@xy@tip{-2}}",
    r"\def\stopper@@{\lx@xy@stopper}",
    r"\def\hook@@{\lx@xy@hook{0}}",
    r"\def\ahook@@{\lx@xy@hook{1}}",
    r"\def\bhook@@{\lx@xy@hook{-1}}",
    r"\def\aturn@@{\lx@xy@turn{1}}",
    r"\def\bturn@@{\lx@xy@turn{-1}}",
    r"\def\solidpoint@{\pointlike@{\lx@xy@fill@on\lx@xy@point}\jot}",
    r"\def\hollowpoint@{\pointlike@{\lx@xy@fill@off\lx@xy@point}\jot}",
    // \Tip@, \Ttip@ redefined without kerns (Perl L401-416)
    r"\def\Tip@{%", "
",
    r" \Tip@@ \egroup",
    r" \U@c=2.5pt \D@c=2.5pt \L@c=2.5pt \R@c=2.5pt \Edge@c={\circleEdge}%", "
",
    r" \def\Leftness@{.5}\def\Upness@{.5}%", "
",
    r" \def\Drop@@{\styledboxz@}\def\Connect@@{\straight@{\dottedSpread@\jot}}}",
    r"\def\Ttip@{%", "
",
    r" \Ttip@@ \egroup",
    r" \U@c=3.2pt \D@c=3.2pt \L@c=3.2pt \R@c=3.2pt \Edge@c={\circleEdge}%", "
",
    r" \def\Leftness@{.5}\def\Upness@{.5}%", "
",
    r" \def\Drop@@{\styledboxz@}\def\Connect@@{\straight@{\dottedSpread@\jot}}}",
  ))?;

  // Perl L579-600: Spline macros
  ::latexml_core::stomach::raw_tex(concat!(
    r"\def\xysplinespecialcases@{\splineset@@}",
    r"\def\splineset@@{",
    r"\readsplineparams@",
    r"\ifdim\dimen5<\dimen7",
    r"\ifx\splineinfo@\squineinfo@",
    r"\L@c\dimexpr(\X@p+2\A@)/3\relax",
    r"\U@c\dimexpr(\Y@p+2\B@)/3\relax",
    r"\R@c\dimexpr(\X@c+2\A@)/3\relax",
    r"\D@c\dimexpr(\Y@c+2\B@)/3\relax",
    r"\fi",
    r"\lx@xy@shavespline",
    r"\lx@xy@crv@decipher",
    r"\lx@xy@spline@",
    r"\fi",
    r"}",
  ))?;

  // Perl L750-756: Frame drop redef
  ::latexml_core::stomach::raw_tex(concat!(
    r"\def\frmDrop@#1{%", "
",
    r" \ifx\frmradius@@\z@ \addtoDrop@@{\let\frmradius@@=\z@}%", "
",
    r" \else \expandafter\addtoDrop@@\expandafter{%", "
",
    r" \expandafter\def\expandafter\frmradius@@\expandafter{\frmradius@@}}\fi",
    r" \addtoDrop@@{\setboxz@h{#1}\styledboxz@}}",
  ))?;

  // Perl L953-957: Polyline macros
  ::latexml_core::stomach::raw_tex(concat!(
    r"\def\xypolyline@Special{\lx@xy@stroke@on\lx@xy@fill@off\lx@xy@poly}",
    r"\def\xypolyfill@Special{\lx@xy@stroke@off\lx@xy@fill@on\lx@xy@poly}",
    r"\def\xypolyeofill@Special{\lx@xy@stroke@off\lx@xy@fill@on\lx@xy@poly}",
    r"\def\xypolydot@Special{\lx@xy@stroke@on\lx@xy@dotpat\lx@xy@fill@off\lx@xy@poly}",
    r"\def\xypolydash@Special{\lx@xy@stroke@on\lx@xy@dashpat\lx@xy@fill@off\lx@xy@poly}",
  ))?;

  // Perl L721: buildcircle
  ::latexml_core::stomach::raw_tex(r"\def\buildcircle@{\lx@xy@crv@decipher\lx@xy@buildcircle@}")?;

  // Perl L1057-1058: Matrix extension
  ::latexml_core::stomach::raw_tex(concat!(
    r"\let\lx@xy@prentry@@norm@save\prentry@@norm",
    r"\def\prentry@@norm{\lx@xy@prentry@@norm@save\lx@xy@notealignment}",
  ))?;


  // Line pattern management (Perl L82-90)
  DefPrimitive!("\\lx@xy@solidpat", { state::assign_value("xy_linepattern", Stored::None, None); });
  DefPrimitive!("\\lx@xy@dashpat", { state::assign_value("xy_linepattern", Stored::String(arena::pin("5")), None); });
  DefPrimitive!("\\lx@xy@dotpat", { state::assign_value("xy_linepattern", Stored::String(arena::pin("1 2")), None); });
  DefPrimitive!("\\lx@xy@cldashpat", { state::assign_value("xy_linepattern", Stored::String(arena::pin("5")), None); });
  DefPrimitive!("\\lx@xy@cldotpat", { state::assign_value("xy_linepattern", Stored::String(arena::pin("1 2")), None); });

  // Stroke/fill state (Perl L92-97)
  state::assign_value("xy_fill", false, None);
  state::assign_value("xy_stroke", true, None);
  DefPrimitive!("\\lx@xy@stroke@on", { state::assign_value("xy_stroke", true, None); });
  DefPrimitive!("\\lx@xy@stroke@off", { state::assign_value("xy_stroke", false, None); });
  DefPrimitive!("\\lx@xy@fill@on", { state::assign_value("xy_fill", true, None); });
  DefPrimitive!("\\lx@xy@fill@off", { state::assign_value("xy_fill", false, None); });

  // Color support (Perl L101-109)
  DefMacro!("\\xycolor@ {}", "");
  DefMacro!("\\xylocalColor@ {}{}", "\\def\\preStyle@@{\\addtostyletoks@{\\bgroup\\lx@xy@usecolor{#1}{#2}}}\\def\\postStyle@@{\\addtostyletoks@{\\egroup}}\\modXYstyle@");
  DefPrimitive!("\\lx@xy@usecolor {}{}", sub[(spec, model)] {
    // Perl L101-109: MergeFont(color => ParseColor($model, $spec))
    let model_str = model.to_string();
    let spec_str = spec.to_string();
    let model_opt = if model_str.trim().is_empty() { None } else { Some(model_str.trim()) };
    let color = crate::package::color_sty::parse_color(model_opt, spec_str.trim());
    MergeFont!(color => color);
  });

  // Direction calculation (Perl L175-179)
  DefPrimitive!("\\lx@xy@calculate@direction", {
    let direction = match state::lookup_register("\\Direction", Vec::new()) {
      Ok(Some(RegisterValue::Number(n))) => n.value_of(),
      _ => 0,
    };
    let k = 1024i64;
    let kangle = ((direction + 8 * k) % (8 * k)) as f64;
    let q = kangle / 2.0 / k as f64;
    let quad = q as i32;
    let delta = 2.0 * (q - quad as f64) - 1.0;
    let norm = 1.0 / (1.0 + delta * delta).sqrt();
    let (dx, dy) = match quad {
      0 => (delta, -1.0),
      1 => (1.0, delta),
      2 => (-delta, 1.0),
      _ => (-1.0, -delta),
    };
    def_macro(T_CS!("\\cosDirection"), None, Tokenize!(&s!("{:.6}", dx * norm)), None)?;
    def_macro(T_CS!("\\sinDirection"), None, Tokenize!(&s!("{:.6}", dy * norm)), None)?;
  });

  // \lx@xy@move@to — position content at (x,y) in SVG (Perl L236-242)
  // CRITICAL: reads {Dimension}{Dimension}{} and applies translate(x,y)
  DefConstructor!("\\lx@xy@move@to {Dimension}{Dimension}{}",
    sub[document, args, _props] {
      let x = args.first().and_then(|a| a.as_ref())
        .and_then(|t| t.get_dimension()).unwrap_or(Dimension::new(0));
      let y = args.get(1).and_then(|a| a.as_ref())
        .and_then(|t| t.get_dimension()).unwrap_or(Dimension::new(0));
      let xpx = dim_to_px(x);
      let ypx = dim_to_px(y);
      let transform = s!("translate({},{})", fmt2(xpx), fmt2(ypx));
      let g_attrs = string_map!("transform" => transform);
      document.open_element("svg:g", Some(g_attrs), None)?;
      if let Some(Some(content)) = args.get(2) {
        document.absorb(content, None)?;
      }
      document.close_element("svg:g")?;
    }
  );

  // \zerodot — dot for dotted lines (Perl L262-269)
  DefConstructor!("\\zerodot",
    sub[document, _args, props] { xy_emit_path(document, props)?; },
    properties => {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      stored_map!(
        "xy_path" => "M -2 -1 l 1 1",
        "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
        "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0)
      )
    }
  );

  // \Droprule@ macro (Perl L277-278)
  DefMacro!("\\Droprule@", "\\setboxz@h{\\lx@xy@droprule}\\advance\\X@p-\\X@c\\Drop@@");

  // \lx@xy@droprule — horizontal/vertical rule (Perl L280-293)
  DefConstructor!("\\lx@xy@droprule",
    sub[document, _args, props] { xy_emit_path(document, props)?; },
    properties => {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let path = xy_packpath(&[
        XyPathPart::Cmd("M"), XyPathPart::Dim(xy_reg_dim("\\X@c")), XyPathPart::Dim(xy_reg_dim("\\Y@c")),
        XyPathPart::Cmd("L"), XyPathPart::Dim(xy_reg_dim("\\X@p")), XyPathPart::Dim(xy_reg_dim("\\Y@p")),
      ]);
      stored_map!("xy_path" => path, "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
        "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0))
    }
  );

  // \squiggle@@ — squiggle line fragment (Perl L309-323)
  DefConstructor!("\\squiggle@@",
    sub[document, _args, props] { xy_emit_path(document, props)?; },
    properties => {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let (c, s) = xy_get_orientation();
      let l = xy_reg_dim("\\xybsqll@");
      let r_px = dim_to_px(l) * 0.66;
      let w_px = dim_to_px(l) * c;
      let h_px = dim_to_px(l) * s;
      let path = s!("M {} {} a {} {} 60 0 0 {} {} a {} {} 60 0 1 {} {}",
        fmt2(-w_px), fmt2(-h_px),
        fmt2(r_px), fmt2(r_px), fmt2(w_px), fmt2(h_px),
        fmt2(r_px), fmt2(r_px), fmt2(w_px), fmt2(h_px));
      stored_map!("xy_path" => path, "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
        "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0))
    }
  );

  // \lx@xy@drawline — discards the \repeat@ iteration, emits a single line (Perl L335)
  DefMacro!("\\lx@xy@drawline Until:\\repeat@", "\\lx@xy@drawline@");

  // \line@@ — line segment (Perl L296-307)
  DefConstructor!("\\line@@",
    sub[document, _args, props] { xy_emit_path(document, props)?; },
    properties => {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let (c, s) = xy_get_orientation();
      let l = xy_reg_dim("\\xydashl@");
      let x = dim_to_px(l) * c;
      let y = dim_to_px(l) * s;
      let path = s!("M 0 0 L {} {}", fmt2(x), fmt2(y));
      stored_map!("xy_path" => path, "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
        "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0))
    }
  );

  // DEBUG: trace drawline coordinates
  // \lx@xy@drawline@ — connecting line from X@p,Y@p to X@c,Y@c (Perl L336-344)
  DefConstructor!("\\lx@xy@drawline@",
    sub[document, _args, props] { xy_emit_path(document, props)?; },
    properties => {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let path = xy_packpath(&[
        XyPathPart::Cmd("M"), XyPathPart::Dim(xy_reg_dim("\\X@p")), XyPathPart::Dim(xy_reg_dim("\\Y@p")),
        XyPathPart::Cmd("L"), XyPathPart::Dim(xy_reg_dim("\\X@c")), XyPathPart::Dim(xy_reg_dim("\\Y@c")),
      ]);
      stored_map!("xy_path" => path, "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
        "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0))
    }
  );

  // \lx@xy@drawsquiggles — discard iteration, emit squiggles (Perl L348)
  DefMacro!("\\lx@xy@drawsquiggles Until:\\repeat@", "\\lx@xy@drawsquiggles@");

  // \lx@xy@drawsquiggles@ — squiggle path (Perl L349-379)
  DefConstructor!("\\lx@xy@drawsquiggles@",
    sub[document, _args, props] { xy_emit_path(document, props)?; },
    properties => {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let x0 = xy_reg_dim("\\X@p"); let y0 = xy_reg_dim("\\Y@p");
      let x1 = xy_reg_dim("\\X@c"); let y1 = xy_reg_dim("\\Y@c");
      let l = xy_reg_dim("\\xybsqll@");
      let r_px = dim_to_px(l) * 0.66;
      let is_dashed = !dashes.is_empty();
      let w = x1.value_of() - x0.value_of();
      let h = y1.value_of() - y0.value_of();
      let dist = ((w as f64 * w as f64 + h as f64 * h as f64).sqrt()) as i64;
      let l_val = l.value_of();
      let mut n = if l_val != 0 {
        ((dist as f64 * 0.5 + dist as f64).sqrt() / l_val as f64) as i32
      } else { 0 };
      if n % 2 == 1 { n += 1; }
      if is_dashed && n % 4 == 0 { n += 2; }
      let dx_px = dim_to_px(x1) - dim_to_px(x0);
      let dy_px = dim_to_px(y1) - dim_to_px(y0);
      let ddx = dx_px / n as f64;
      let ddy = dy_px / n as f64;
      let step = if is_dashed { 4 } else { 2 };
      let mut path_str = String::new();
      let mut cx = dim_to_px(x0);
      let mut cy = dim_to_px(y0);
      let mut i = 0;
      while i < n {
        if is_dashed || i == 0 {
          path_str.push_str(&s!("M {} {} ", fmt2(cx), fmt2(cy)));
        }
        path_str.push_str(&s!("a {} {} 60 0 0 {} {} ", fmt2(r_px), fmt2(r_px), fmt2(ddx), fmt2(ddy)));
        path_str.push_str(&s!("a {} {} 60 0 1 {} {} ", fmt2(r_px), fmt2(r_px), fmt2(ddx), fmt2(ddy)));
        cx += ddx * step as f64;
        cy += ddy * step as f64;
        i += step;
      }
      let trimmed_path = path_str.trim().to_string();
      stored_map!(
        "xy_path" => trimmed_path,
        "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
        "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0)
      )
    }
  );

  // Arrow tips (Perl L428-446)
  // \lx@xy@tip — half an arrow tip using arc path
  // Applies style factors from %xy_tips_factors (Perl L424-426)
  DefConstructor!("\\lx@xy@tip {}",
    sub[document, _args, props] { xy_emit_path(document, props)?; },
    properties => sub[args] {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let stretch_str = args.first().and_then(|a| a.as_ref())
        .map(|t| t.to_string()).unwrap_or_else(|| String::from("1"));
      let stretch: f64 = stretch_str.parse().unwrap_or(1.0);
      // Apply tip style factors (Perl %xy_tips_factors). Probe the state
      // value in place — no need to allocate an owned String for a
      // 2-3 char comparison against a small literal set.
      let (lf, wf): (f64, f64) = state::with_value("xy_tips_style", |v| {
        match v {
          Some(Stored::String(s)) => arena::with(*s, |style| match style {
            "cm" => (0.5, 1.7),
            "eu" => (0.5, 1.5),
            "lu" => (0.5, 0.5),
            _ => (1.0, 1.0),
          }),
          _ => (1.0, 1.0),
        }
      });
      let (c, s) = xy_get_orientation();
      let l_px = dim_to_px(xy_reg_dim("\\xydashl@")) * lf;
      let w_px = dim_to_px(xy_reg_dim("\\xydashh@")) * wf * stretch;
      let r_px = l_px * 2.0;
      let dx = -l_px * c - w_px * s;
      let dy = -l_px * s + w_px * c;
      let sweep = if stretch < 0.0 { 1 } else { 0 };
      let path = s!("M 0 0 A {} {} 45 0 {} {} {}",
        fmt2(r_px), fmt2(r_px), sweep, fmt2(dx), fmt2(dy));
      Ok(stored_map!("xy_path" => path, "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
        "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0)))
    }
  );

  // \lx@xy@stopper — "|" tip (Perl L448-460)
  DefConstructor!("\\lx@xy@stopper",
    sub[document, _args, props] { xy_emit_path(document, props)?; },
    properties => {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let (c, s) = xy_get_orientation();
      let l_px = dim_to_px(xy_reg_dim("\\xydashl@"));
      let dx = -l_px * s;
      let dy = l_px * c;
      let path = s!("M {} {} L {} {}", fmt2(dx), fmt2(dy), fmt2(-dx), fmt2(-dy));
      stored_map!("xy_path" => path, "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
        "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0))
    }
  );

  // \lx@xy@hook — "(" tip (Perl L462-480)
  DefConstructor!("\\lx@xy@hook {Number}",
    sub[document, _args, props] { xy_emit_path(document, props)?; },
    properties => sub[args] {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let offset: f64 = args.first().and_then(|a| a.as_ref())
        .map(|t| t.value_of() as f64).unwrap_or(0.0);
      let (c, s) = xy_get_orientation();
      let l_px = dim_to_px(xy_reg_dim("\\xybsqll@")) * std::f64::consts::FRAC_1_SQRT_2;
      let x0 = if offset != 0.0 { 0.0 } else { l_px };
      let y0_val = l_px * (offset + 1.0);
      let y1_val = y0_val - l_px * 2.0;
      let mx = x0 * c - y0_val * s;
      let my_val = x0 * s + y0_val * c;
      let ex = x0 * c - y1_val * s;
      let ey = x0 * s + y1_val * c;
      let path = s!("M {} {} A {} {} 180 0 1 {} {}",
        fmt2(mx), fmt2(my_val), fmt2(l_px), fmt2(l_px), fmt2(ex), fmt2(ey));
      Ok(stored_map!("xy_path" => path, "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
        "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0)))
    }
  );

  // \lx@xy@turn — quarter circle tip (Perl L481-495)
  DefConstructor!("\\lx@xy@turn {Number}",
    sub[document, _args, props] { xy_emit_path(document, props)?; },
    properties => sub[args] {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let offset: f64 = args.first().and_then(|a| a.as_ref())
        .map(|t| t.value_of() as f64).unwrap_or(1.0);
      let (c, s) = xy_get_orientation();
      let l_px = dim_to_px(xy_reg_dim("\\xybsqll@")) * std::f64::consts::FRAC_1_SQRT_2;
      let sweep = if offset > 0.0 { 0 } else { 1 };
      let ex = l_px * -(c + offset * s);
      let ey = l_px * (offset * c - s);
      let path = s!("M 0 0 A {} {} 90 0 {} {} {}",
        fmt2(l_px), fmt2(l_px), sweep, fmt2(ex), fmt2(ey));
      Ok(stored_map!("xy_path" => path, "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
        "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0)))
    }
  );

  // \lx@xy@point — filled/empty circle (Perl L497-505)
  DefConstructor!("\\lx@xy@point",
    sub[document, _args, props] { xy_emit_circle(document, props)?; },
    properties => {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let r_px = dim_to_px(xy_reg_dim("\\xybsqll@")) * 0.5;
      stored_map!(
        "xy_cx" => "0", "xy_cy" => "0", "xy_r" => fmt2(r_px),
        "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
        "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0)
      )
    }
  );

  // \cirbuild@ — circle (Perl L522-557)
  DefConstructor!("\\cirbuild@",
    sub[document, _args, props] {
      let is_full = match props.get("xy_full") {
        Some(Stored::Bool(b)) => *b,
        _ => true,
      };
      if is_full {
        xy_emit_circle(document, props)?;
      } else {
        xy_emit_path(document, props)?;
      }
    },
    properties => {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let r = xy_reg_dim("\\R@");
      let r_px = dim_to_px(r);
      let xc_px = r_px;
      // Probe xy_circle_dir in place: we need both an "is empty or 0"
      // boolean (the full-circle path) and a parsed i64 (the arc path).
      // Compute both from the interned SymStr without allocating an
      // owned String.
      let (cd_empty_or_zero, cd_val): (bool, i64) = state::with_value("xy_circle_dir", |v| {
        match v {
          Some(Stored::String(s)) => arena::with(*s, |cd| {
            let empty_or_zero = cd.is_empty() || cd == "0";
            let val: i64 = cd.parse().unwrap_or(0);
            (empty_or_zero, val)
          }),
          _ => (true, 0),
        }
      });
      if cd_empty_or_zero {
        // Full circle — Perl: width => 2*R@, height => R@, depth => R@
        let d = Dimension::new(r.value_of() * 2);
        stored_map!(
          "xy_full" => true,
          "xy_cx" => fmt2(xc_px), "xy_cy" => "0", "xy_r" => fmt2(r_px),
          "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
          "width" => d, "height" => r, "depth" => r
        )
      } else {
        // Partial arc
        let d1 = xy_reg_num("\\count@@");
        let d2 = xy_reg_num("\\count@");
        let (a1, a2) = if cd_val > 0 {
          if d1 < d2 { ((d1 - 4) * 45, (d2 - 4) * 45) }
          else { ((d1 - 4) * 45, (d2 - 4 + 8) * 45) }
        } else {
          if d1 < d2 { ((d2 - 4) * 45, (d1 - 4 + 8) * 45) }
          else { ((d2 - 4) * 45, (d1 - 4) * 45) }
        };
        let theta1 = (a1 as f64) * std::f64::consts::PI / 180.0;
        let theta2 = (a2 as f64) * std::f64::consts::PI / 180.0;
        let x0 = xc_px + r_px * theta1.cos();
        let y0 = r_px * theta1.sin();
        let x1 = xc_px + r_px * theta2.cos();
        let y1 = r_px * theta2.sin();
        let a = a2 - a1;
        let large = if a > 180 { 1 } else { 0 };
        let path = s!("M {} {} A {} {} {} {} 0 {} {}",
          fmt2(x1), fmt2(y1), fmt2(r_px), fmt2(r_px), a, large, fmt2(x0), fmt2(y0));
        stored_map!(
          "xy_full" => false,
          "xy_path" => path, "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
          "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0)
        )
      }
    }
  );
  DefPrimitive!("\\lx@xy@circdir {}", sub[(dir)] {
    state::assign_value("xy_circle_dir", Stored::String(arena::pin(dir.to_string())), None);
  });
  // Perl L510-515: CIR macros
  ::latexml_core::stomach::raw_tex(concat!(
    r"\let\lx@xy@CIRfull@orig\CIRfull@",
    r"\let\lx@xy@CIRcw@orig\CIRcw@",
    r"\let\lx@xy@CIRacw@orig\CIRacw@",
    r"\def\CIRfull@{\lx@xy@circdir{}\lx@xy@CIRfull@orig}",
    r"\def\CIRcw@{\lx@xy@circdir{-1}\lx@xy@CIRcw@orig}",
    r"\def\CIRacw@{\lx@xy@circdir{+1}\lx@xy@CIRacw@orig}",
  ))?;

  // \lx@xy@spline@ — cubic Bezier spline with multiplicity (Perl L656-692)
  DefConstructor!("\\lx@xy@spline@",
    sub[document, _args, props] { xy_emit_path(document, props)?; },
    properties => {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let x0 = xy_reg_dim("\\X@p"); let y0 = xy_reg_dim("\\Y@p");
      let x1 = xy_reg_dim("\\X@c"); let y1 = xy_reg_dim("\\Y@c");
      let lc = xy_reg_dim("\\L@c"); let uc = xy_reg_dim("\\U@c");
      let rc = xy_reg_dim("\\R@c"); let dc = xy_reg_dim("\\D@c");
      let mult: i32 = state::with_value("xy_multiplicity", |v| match v {
        Some(Stored::String(s)) => arena::with(*s, |m| m.parse().unwrap_or(1)),
        _ => 1,
      });
      let mut path = String::new();
      // Main curve (for odd multiplicity)
      if mult % 2 == 1 {
        path.push_str(&xy_packpath(&[
          XyPathPart::Cmd("M"), XyPathPart::Dim(x0), XyPathPart::Dim(y0),
          XyPathPart::Cmd("C"), XyPathPart::Dim(lc), XyPathPart::Dim(uc),
          XyPathPart::Dim(rc), XyPathPart::Dim(dc),
          XyPathPart::Dim(x1), XyPathPart::Dim(y1),
        ]));
      }
      // Double/triple offset curves (Perl L667-687)
      if mult > 1 {
        let sep = xy_reg_dim("\\xydashh@");
        let sep = if mult == 2 { Dimension::new(sep.value_of() / 2) } else { sep };
        // xy_linediff: perpendicular offset for line segments (Perl L642-649)
        let linediff = |sep: Dimension, ax: Dimension, ay: Dimension, bx: Dimension, by: Dimension| -> (f64, f64) {
          let diffx = (bx.value_of() - ax.value_of()) as f64;
          let diffy = (by.value_of() - ay.value_of()) as f64;
          let length = (diffx * diffx + diffy * diffy).sqrt();
          if length == 0.0 { return (0.0, 0.0); }
          let s = sep.value_of() as f64;
          (s * diffy / length, s * diffx / length)
        };
        let (dx0, dy0) = linediff(sep, x0, y0, lc, uc);
        let (dx1, dy1) = linediff(sep, lc, uc, rc, dc);
        let (dx2, dy2) = linediff(sep, rc, dc, x1, y1);
        let dx1a = (dx0 + dx1) * 0.5; let dy1a = (dy0 + dy1) * 0.5;
        let dx1b = (dx1 + dx2) * 0.5; let dy1b = (dy1 + dy2) * 0.5;
        // Perl L677-687: offset dimension by sp-valued offset, then convert to px
        let dp = |d: Dimension, off: f64| fmt2(dim_to_px(Dimension::new(d.value_of() - off as i64)));
        // Upper offset curve
        if !path.is_empty() { path.push(' '); }
        path.push_str(&s!("M {} {} C {} {} {} {} {} {}",
          dp(x0, -dy0), dp(y0, dx0),
          dp(lc, -dx1a), dp(uc, -dy1a),
          dp(rc, -dx1b), dp(dc, -dy1b),
          dp(x1, dy2), dp(y1, -dx2)));
        // Lower offset curve
        path.push_str(&s!(" M {} {} C {} {} {} {} {} {}",
          dp(x0, dy0), dp(y0, -dx0),
          dp(lc, dx1a), dp(uc, dy1a),
          dp(rc, dx1b), dp(dc, dy1b),
          dp(x1, -dy2), dp(y1, dx2)));
      }
      stored_map!("xy_path" => path, "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
        "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0))
    }
  );

  // \lx@xy@shavespline — trim spline endpoints (Perl L694-719)
  DefPrimitive!("\\lx@xy@shavespline", {
    let pt = 65536i64; // 1pt in sp
    let a_sp = xy_reg_dim("\\dimen5").value_of();
    let b_sp = xy_reg_dim("\\dimen7").value_of();
    let a = a_sp as f64 / pt as f64;
    let b = b_sp as f64 / pt as f64;
    if a != 0.0 || b != 0.0 {
      let xp = xy_reg_dim("\\X@p").value_of() as f64;
      let yp = xy_reg_dim("\\Y@p").value_of() as f64;
      let lc = xy_reg_dim("\\L@c").value_of() as f64;
      let uc = xy_reg_dim("\\U@c").value_of() as f64;
      let rc = xy_reg_dim("\\R@c").value_of() as f64;
      let dc = xy_reg_dim("\\D@c").value_of() as f64;
      let xc = xy_reg_dim("\\X@c").value_of() as f64;
      let yc = xy_reg_dim("\\Y@c").value_of() as f64;
      let xu = lc - xp; let xv = rc - xu - lc; let xw = xc - 3.0 * rc + 3.0 * lc - xp;
      let yu = uc - yp; let yv = dc - yu - uc; let yw = yc - 3.0 * dc + 3.0 * uc - yp;
      let aab = 2.0 * a + b;
      let abb = a + 2.0 * b;
      let new_xp = xp + a * (3.0 * xu + (3.0 * xv + xw * a) * a);
      let new_yp = yp + a * (3.0 * yu + (3.0 * yv + yw * a) * a);
      let new_lc = xp + aab * xu + a * (abb * xv + xw * a * b);
      let new_uc = yp + aab * yu + a * (abb * yv + yw * a * b);
      let new_rc = xp + abb * xu + b * (aab * xv + xw * b * a);
      let new_dc = yp + abb * yu + b * (aab * yv + yw * b * a);
      let new_xc = xp + b * (3.0 * xu + (3.0 * xv + xw * b) * b);
      let new_yc = yp + b * (3.0 * yu + (3.0 * yv + yw * b) * b);
      state::assign_register("\\X@p", RegisterValue::Dimension(Dimension::new(new_xp as i64)), None, Vec::new())?;
      state::assign_register("\\Y@p", RegisterValue::Dimension(Dimension::new(new_yp as i64)), None, Vec::new())?;
      state::assign_register("\\L@c", RegisterValue::Dimension(Dimension::new(new_lc as i64)), None, Vec::new())?;
      state::assign_register("\\U@c", RegisterValue::Dimension(Dimension::new(new_uc as i64)), None, Vec::new())?;
      state::assign_register("\\R@c", RegisterValue::Dimension(Dimension::new(new_rc as i64)), None, Vec::new())?;
      state::assign_register("\\D@c", RegisterValue::Dimension(Dimension::new(new_dc as i64)), None, Vec::new())?;
      state::assign_register("\\X@c", RegisterValue::Dimension(Dimension::new(new_xc as i64)), None, Vec::new())?;
      state::assign_register("\\Y@c", RegisterValue::Dimension(Dimension::new(new_yc as i64)), None, Vec::new())?;
    }
  });

  // \lx@xy@crv@decipher — parse curve drop/connection styles (Perl L609-640)
  DefPrimitive!("\\lx@xy@crv@decipher", {
    // Parse \xycrvdrop@ and \xycrvconn@ to determine line pattern and multiplicity.
    // Perl uses LookupDefinition(...)->getExpansion which returns the raw macro body;
    // expanding via do_expand re-invokes drawing macros such as \dir{-->} and causes
    // the curve pipeline to recurse into itself (see SYNC_STATUS xy-pic OOM repro).
    let drop = macro_body("\\xycrvdrop@").trim().to_string();
    let conn = macro_body("\\xycrvconn@").trim().to_string();
    if !drop.is_empty() {
      // Check for "=<spacing>{char}" pattern (dotted with given spacing)
      if let Some(rest) = drop.strip_prefix("=<") {
        if let Some(end_angle) = rest.find('>') {
          let spacing_str = rest[..end_angle].trim();
          // Parse spacing as dimension string like "5pt", "3.5pt"
          // Perl: Dimension($s)->pxValue
          if let Some(num_part) = spacing_str.strip_suffix("pt") {
            if let Ok(val) = num_part.parse::<f64>() {
              let sp_dim = Dimension::new((val * 65536.0) as i64);
              let sp_px = dim_to_px(sp_dim) as i32;
              let pattern = s!("1 {sp_px}");
              state::assign_value("xy_linepattern", Stored::String(arena::pin(&pattern)), None);
            }
          }
        }
      } else if drop.contains("\\zerodot") {
        state::assign_value("xy_linepattern", Stored::String(arena::pin("1 2")), None);
      }
    }
    if !conn.is_empty() {
      // Strip leading "!<letter>" prefix
      let conn_stripped = if conn.len() > 1 && conn.starts_with('!') {
        let skip = if conn.as_bytes().get(1).is_some_and(|b| b.is_ascii_alphabetic()) { 2 } else { 1 };
        &conn[skip..]
      } else { &conn };
      // Match \dir{<type>} or \dir<n>{<type>}
      if let Some(dir_start) = conn_stripped.find("\\dir") {
        let after_dir = &conn_stripped[dir_start + 4..];
        // Check for optional digit
        let (n_opt, rest) = if after_dir.starts_with(|c: char| c.is_ascii_digit()) {
          let n: i32 = after_dir[..1].parse().unwrap_or(1);
          (Some(n), &after_dir[1..])
        } else {
          (None, after_dir)
        };
        // Extract {type}
        if let Some(brace_start) = rest.find('{') {
          if let Some(brace_end) = rest.find('}') {
            let mut t = rest[brace_start + 1..brace_end].to_string();
            let mut n = n_opt.unwrap_or(1);
            if t == ":" { n = 2; t = String::from("."); }
            if t == "=" { n = 2; t = String::from("-"); }
            if n > 1 {
              state::assign_value("xy_multiplicity", Stored::String(arena::pin(n.to_string())), None);
            }
            match t.as_str() {
              "--" => { state::assign_value("xy_linepattern", Stored::String(arena::pin("5")), None); },
              "." => { state::assign_value("xy_linepattern", Stored::String(arena::pin("1 2")), None); },
              _ => {}, // "-" or empty = solid
            }
          }
        }
      }
    }
  });

  // \lx@xy@buildcircle@ — ellipse from \R@ and \L@ (Perl L722-741)
  DefConstructor!("\\lx@xy@buildcircle@",
    sub[document, _args, props] {
      let cx = match props.get("xy_cx") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let cy = match props.get("xy_cy") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let rx = match props.get("xy_rx") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let ry = match props.get("xy_ry") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let stroke = match props.get("xy_stroke") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("#000000") };
      let fill = match props.get("xy_fill") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("none") };
      let mut attrs = string_map!("cx" => cx, "cy" => cy, "rx" => rx, "ry" => ry, "stroke" => stroke, "fill" => fill);
      if let Some(Stored::String(d)) = props.get("xy_dashes") {
        if !arena::with(*d, |s| s.is_empty()) {
          attrs.insert(String::from("stroke-dasharray"), arena::to_string(*d));
        }
      }
      svg_empty_element(document, "svg:ellipse", attrs)?;
    },
    properties => {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let rx_px = dim_to_px(xy_reg_dim("\\R@"));
      let ry_px = dim_to_px(xy_reg_dim("\\L@"));
      let xc_px = rx_px;
      let yc_px = ry_px + dim_to_px(xy_reg_dim("\\xydashl@"));
      stored_map!(
        "xy_cx" => fmt2(xc_px), "xy_cy" => fmt2(yc_px),
        "xy_rx" => fmt2(rx_px), "xy_ry" => fmt2(ry_px),
        "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes
      )
    }
  );

  // \framed@@ — rectangular frame with optional rounded corners (Perl L773-809)
  DefConstructor!("\\framed@@ {Dimension}",
    sub[document, _args, props] { xy_emit_path(document, props)?; },
    properties => sub[args] {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let r = args.first().and_then(|a| a.as_ref())
        .and_then(|t| t.get_dimension()).unwrap_or(Dimension::new(0));
      let x = xy_reg_dim("\\X@c"); let y = xy_reg_dim("\\Y@c");
      let l = xy_reg_dim("\\L@c"); let u = xy_reg_dim("\\U@c");
      let rc = xy_reg_dim("\\R@c"); let d = xy_reg_dim("\\D@c");
      let w_sp = l.value_of() + rc.value_of();
      let h_sp = u.value_of() + d.value_of();
      let x0 = dim_to_px(x) - dim_to_px(l);
      let y0 = dim_to_px(y) - dim_to_px(d);
      let x1 = x0 + dim_to_px(Dimension::new(w_sp));
      let y1 = y0 + dim_to_px(Dimension::new(h_sp));
      let r_sp = r.value_of().min(w_sp / 2).min(h_sp / 2);
      let path = if r_sp <= 0 {
        s!("M {} {} L {} {} L {} {} L {} {} Z",
          fmt2(x0), fmt2(y0), fmt2(x1), fmt2(y0), fmt2(x1), fmt2(y1), fmt2(x0), fmt2(y1))
      } else {
        let r_px = dim_to_px(Dimension::new(r_sp));
        s!("M {} {} A {} {} 90 0 0 {} {} L {} {} A {} {} 90 0 0 {} {} L {} {} A {} {} 90 0 0 {} {} L {} {} A {} {} 90 0 0 {} {} Z",
          fmt2(x0 + r_px), fmt2(y0), fmt2(r_px), fmt2(r_px), fmt2(x0), fmt2(y0 + r_px),
          fmt2(x0), fmt2(y1 - r_px), fmt2(r_px), fmt2(r_px), fmt2(x0 + r_px), fmt2(y1),
          fmt2(x1 - r_px), fmt2(y1), fmt2(r_px), fmt2(r_px), fmt2(x1), fmt2(y1 - r_px),
          fmt2(x1), fmt2(y0 + r_px), fmt2(r_px), fmt2(r_px), fmt2(x1 - r_px), fmt2(y0))
      };
      Ok(stored_map!("xy_path" => path, "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
        "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0)))
    }
  );

  // \circled@ — circle frame (Perl L811-829)
  DefConstructor!("\\circled@ {Dimension}",
    sub[document, _args, props] { xy_emit_circle(document, props)?; },
    properties => sub[args] {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let r_arg = args.first().and_then(|a| a.as_ref())
        .and_then(|t| t.get_dimension()).unwrap_or(Dimension::new(0));
      let x = xy_reg_dim("\\X@c"); let y = xy_reg_dim("\\Y@c");
      let l = xy_reg_dim("\\L@c"); let u = xy_reg_dim("\\U@c");
      let rc = xy_reg_dim("\\R@c"); let d = xy_reg_dim("\\D@c");
      let w_v = l.value_of() + rc.value_of();
      let h_v = u.value_of() + d.value_of();
      let r = if r_arg.value_of() != 0 { r_arg }
        else { Dimension::new(w_v.max(h_v) / 2) };
      Ok(stored_map!(
        "xy_cx" => fmt2(dim_to_px(x)), "xy_cy" => fmt2(dim_to_px(y)),
        "xy_r" => fmt2(dim_to_px(r)),
        "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes
      ))
    }
  );

  // \ellipsed@ — ellipse frame (Perl L831-848)
  DefConstructor!("\\ellipsed@ {Dimension}{Dimension}",
    sub[document, _args, props] {
      let cx = match props.get("xy_cx") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let cy = match props.get("xy_cy") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let rx = match props.get("xy_rx") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let ry = match props.get("xy_ry") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let stroke = match props.get("xy_stroke") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("#000000") };
      let fill = match props.get("xy_fill") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("none") };
      let mut attrs = string_map!("cx" => cx, "cy" => cy, "rx" => rx, "ry" => ry, "stroke" => stroke, "fill" => fill);
      if let Some(Stored::String(d)) = props.get("xy_dashes") {
        if !arena::with(*d, |s| s.is_empty()) {
          attrs.insert(String::from("stroke-dasharray"), arena::to_string(*d));
        }
      }
      svg_empty_element(document, "svg:ellipse", attrs)?;
    },
    properties => sub[args] {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let rx = args.first().and_then(|a| a.as_ref())
        .and_then(|t| t.get_dimension()).unwrap_or(Dimension::new(0));
      let ry = args.get(1).and_then(|a| a.as_ref())
        .and_then(|t| t.get_dimension()).unwrap_or(Dimension::new(0));
      let x = xy_reg_dim("\\X@c"); let y = xy_reg_dim("\\Y@c");
      Ok(stored_map!(
        "xy_cx" => fmt2(dim_to_px(x)), "xy_cy" => fmt2(dim_to_px(y)),
        "xy_rx" => fmt2(dim_to_px(rx)), "xy_ry" => fmt2(dim_to_px(ry)),
        "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes
      ))
    }
  );

  // \shaded@@ — shaded path (Perl L850-869)
  DefConstructor!("\\shaded@@ {Dimension}",
    sub[document, _args, props] { xy_emit_path(document, props)?; },
    properties => sub[args] {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let r = args.first().and_then(|a| a.as_ref())
        .and_then(|t| t.get_dimension()).unwrap_or(Dimension::new(0));
      let x = xy_reg_dim("\\X@c"); let y = xy_reg_dim("\\Y@c");
      let l = xy_reg_dim("\\L@c"); let u = xy_reg_dim("\\U@c");
      let rc = xy_reg_dim("\\R@c"); let d = xy_reg_dim("\\D@c");
      let w_sp = l.value_of() + rc.value_of();
      let h_sp = u.value_of() + d.value_of();
      let x0 = dim_to_px(x) - dim_to_px(l) + dim_to_px(r);
      let y0 = dim_to_px(y) - dim_to_px(d);
      let x1 = x0 + dim_to_px(Dimension::new(w_sp));
      let y1 = y0 + dim_to_px(Dimension::new(h_sp));
      let path = s!("M {} {} L {} {} L {} {}", fmt2(x0), fmt2(y0), fmt2(x1), fmt2(y0), fmt2(x1), fmt2(y1));
      Ok(stored_map!("xy_path" => path, "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
        "width" => Dimension::new(0), "height" => Dimension::new(0), "depth" => Dimension::new(0)))
    }
  );

  // Bracket macros (Perl L871-878)
  DefMacro!("\\lbraced",        "\\lx@xy@bracketed{\\{}{L}");
  DefMacro!("\\rbraced",        "\\lx@xy@bracketed{\\}}{R}");
  DefMacro!("\\ubraced",        "\\lx@xy@bracketed{\\{}{U}");
  DefMacro!("\\dbraced",        "\\lx@xy@bracketed{\\{}{D}");
  DefMacro!("\\lparenthesized", "\\lx@xy@bracketed{(}{L}");
  DefMacro!("\\rparenthesized", "\\lx@xy@bracketed{)}{R}");
  DefMacro!("\\uparenthesized", "\\lx@xy@bracketed{(}{U}");
  DefMacro!("\\dparenthesized", "\\lx@xy@bracketed{(}{D}");

  // \lx@xy@bracketed — positioned bracket/brace in SVG (Perl L880-916)
  DefConstructor!("\\lx@xy@bracketed {}{}",
    sub[document, args, props] {
      let stroke = match props.get("xy_stroke") {
        Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("#000000")
      };
      let x = match props.get("xy_x") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let y = match props.get("xy_y") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let angle = match props.get("xy_angle") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let xscale = match props.get("xy_xscale") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("1") };
      let yscale = match props.get("xy_yscale") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("1") };
      let transform = s!("translate({},{}) rotate({}) scale({},{})", x, y, angle, xscale, yscale);
      let attrs = string_map!("transform" => transform, "stroke" => stroke);
      let savenode = document.float_to_element("svg:text", false)?;
      document.open_element("svg:text", Some(attrs), None)?;
      if let Some(Some(content)) = args.first() {
        document.absorb(content, None)?;
      }
      document.close_element("svg:text")?;
      if let Some(saved) = savenode { document.set_node(&saved); }
    },
    properties => sub[args] {
      let (stroke, _fill) = xy_fill_stroke();
      let orientation_d = args.get(1).and_then(|a| a.as_ref())
        .map(|t| t.to_string().to_uppercase()).unwrap_or_default();
      let _x = xy_reg_dim("\\X@c"); let _y = xy_reg_dim("\\Y@c");
      let l = xy_reg_dim("\\L@c"); let u = xy_reg_dim("\\U@c");
      let rc = xy_reg_dim("\\R@c"); let d = xy_reg_dim("\\D@c");
      let w_px = dim_to_px(Dimension::new(l.value_of() + rc.value_of()));
      let h_px = dim_to_px(Dimension::new(u.value_of() + d.value_of()));
      // Approximate char size (w0=6pt, ht0=10pt) since we can't easily query
      let w0_px = dim_to_px(Dimension::new(6 * 65536));
      let ht0_px = dim_to_px(Dimension::new(10 * 65536));
      let (px, py, angle, xscale, yscale) = match orientation_d.as_str() {
        "L" => (-w0_px, 0.0, 0.0, 1.0, if ht0_px != 0.0 { h_px / ht0_px } else { 1.0 }),
        "R" => (w_px, 0.0, 0.0, 1.0, if ht0_px != 0.0 { h_px / ht0_px } else { 1.0 }),
        "U" => ((w_px + ht0_px) * 0.5, h_px + w0_px, -90.0, 1.0,
          if ht0_px != 0.0 { w_px / ht0_px } else { 1.0 }),
        "D" => ((w_px - ht0_px) * 0.5, -(h_px + w0_px), 90.0, 1.0,
          if ht0_px != 0.0 { w_px / ht0_px } else { 1.0 }),
        _ => (0.0, 0.0, 0.0, 1.0, 1.0),
      };
      Ok(stored_map!(
        "xy_x" => fmt2(px), "xy_y" => fmt2(py), "xy_angle" => fmt2(angle),
        "xy_xscale" => fmt2(xscale), "xy_yscale" => fmt2(yscale),
        "xy_stroke" => stroke
      ))
    }
  );

  // \blacked@@ — filled black rect (Perl L918-929)
  DefConstructor!("\\blacked@@",
    sub[document, _args, props] {
      let x = match props.get("xy_x") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let y = match props.get("xy_y") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let w = match props.get("xy_w") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let h = match props.get("xy_h") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("0") };
      let stroke = match props.get("xy_stroke") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("#000000") };
      let fill = match props.get("xy_fill") { Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("#000000") };
      let attrs = string_map!("x" => x, "y" => y, "width" => w, "height" => h,
        "stroke" => stroke, "fill" => fill);
      svg_empty_element(document, "svg:rect", attrs)?;
    },
    properties => {
      let (stroke, _fill) = xy_fill_stroke();
      let w = xy_reg_dim("\\dimen@");
      let h = xy_reg_dim("\\dimen@ii");
      let b = xy_reg_dim("\\B@");
      let d_px = dim_to_px(Dimension::new(-b.value_of()));
      let y_px = dim_to_px(b);
      let ht_px = dim_to_px(h) + d_px;
      stored_map!(
        "xy_x" => "0", "xy_y" => fmt2(y_px), "xy_w" => fmt2(dim_to_px(w)), "xy_h" => fmt2(ht_px),
        "xy_stroke" => stroke, "xy_fill" => "#000000"
      )
    }
  );

  // \xyscale@@ — scale transform (Perl L988-1001)
  DefConstructor!("\\xyscale@@ {}{}",
    sub[document, _args, props] {
      let transform = match props.get("xy_transform") {
        Some(Stored::String(s)) => arena::to_string(*s), _ => String::new()
      };
      let attrs = string_map!("transform" => transform);
      let savenode = document.float_to_element("svg:g", false)?;
      document.open_element("svg:g", Some(attrs), None)?;
      if let Some(Stored::Digested(box_d)) = props.get("xy_box") {
        document.absorb(box_d, None)?;
      }
      document.close_element("svg:g")?;
      if let Some(saved) = savenode { document.set_node(&saved); }
    },
    properties => sub[args] {
      let xscale = args.first().and_then(|a| a.as_ref())
        .map(|t| t.to_string()).unwrap_or_else(|| String::from("1"));
      let yscale = args.get(1).and_then(|a| a.as_ref())
        .map(|t| t.to_string()).unwrap_or_else(|| String::from("1"));
      let lc = xy_reg_dim("\\L@c"); let up = xy_reg_dim("\\U@p"); let rp = xy_reg_dim("\\R@p");
      let t1x = dim_to_px(Dimension::new(lc.value_of() - rp.value_of()));
      let t1y = -dim_to_px(up);
      let t2x = -t1x; let t2y = -t1y;
      let transform = s!("translate({},{}) scale({},{}) translate({},{})",
        fmt2(t1x), fmt2(t1y), xscale, yscale, fmt2(t2x), fmt2(t2y));
      let box_val = state::lookup_value("box0").unwrap_or(Stored::None);
      state::assign_value("box0", Stored::None, None);
      Ok(stored_map!("xy_transform" => transform, "xy_box" => box_val))
    }
  );

  // \xyRotate@@ — rotation transform (Perl L1014-1029, second definition overrides)
  DefConstructor!("\\xyRotate@@ {}",
    sub[document, _args, props] {
      let transform = match props.get("xy_transform") {
        Some(Stored::String(s)) => arena::to_string(*s), _ => String::new()
      };
      let attrs = string_map!("transform" => transform);
      let savenode = document.float_to_element("svg:g", false)?;
      document.open_element("svg:g", Some(attrs), None)?;
      if let Some(Stored::Digested(box_d)) = props.get("xy_box") {
        document.absorb(box_d, None)?;
      }
      document.close_element("svg:g")?;
      if let Some(saved) = savenode { document.set_node(&saved); }
    },
    properties => sub[args] {
      let kangle: i64 = args.first().and_then(|a| a.as_ref())
        .map(|t| t.value_of()).unwrap_or(0);
      let lc = xy_reg_dim("\\L@c"); let up = xy_reg_dim("\\U@p"); let rp = xy_reg_dim("\\R@p");
      let t1x = dim_to_px(Dimension::new(lc.value_of() - rp.value_of()));
      let t1y = -dim_to_px(up);
      let t2x = -t1x; let t2y = -t1y;
      // Direction from K-angle (same as xy_direction)
      let k = 1024i64;
      let kf = ((kangle + 8 * k) % (8 * k)) as f64;
      let q = kf / 2.0 / k as f64;
      let quad = q as i32;
      let delta = 2.0 * (q - quad as f64) - 1.0;
      let norm = 1.0 / (1.0 + delta * delta).sqrt();
      let (dx, dy) = match quad {
        0 => (delta, -1.0), 1 => (1.0, delta), 2 => (-delta, 1.0), _ => (-1.0, -delta),
      };
      let angle = (dy * norm).atan2(dx * norm) * 180.0 / std::f64::consts::PI;
      let transform = s!("translate({},{}) rotate({}) translate({},{})",
        fmt2(t1x), fmt2(t1y), angle as i32, fmt2(t2x), fmt2(t2y));
      let box_val = state::lookup_value("box0").unwrap_or(Stored::None);
      state::assign_value("box0", Stored::None, None);
      Ok(stored_map!("xy_transform" => transform, "xy_box" => box_val))
    }
  );

  // \doSpecialRotate@@ — align with current direction (Perl L1032-1047)
  DefConstructor!("\\doSpecialRotate@@ Until:@@",
    sub[document, _args, props] {
      let transform = match props.get("xy_transform") {
        Some(Stored::String(s)) => arena::to_string(*s), _ => String::new()
      };
      let attrs = string_map!("transform" => transform);
      let savenode = document.float_to_element("svg:g", false)?;
      document.open_element("svg:g", Some(attrs), None)?;
      if let Some(Stored::Digested(box_d)) = props.get("xy_box") {
        document.absorb(box_d, None)?;
      }
      document.close_element("svg:g")?;
      if let Some(saved) = savenode { document.set_node(&saved); }
    },
    properties => sub[_args] {
      let lc = xy_reg_dim("\\L@c"); let up = xy_reg_dim("\\U@p"); let rp = xy_reg_dim("\\R@p");
      let t1x = dim_to_px(Dimension::new(lc.value_of() - rp.value_of()));
      let t1y = -dim_to_px(up);
      let t2x = -t1x; let t2y = -t1y;
      let (c, s) = xy_get_orientation();
      let angle = s.atan2(c) * 180.0 / std::f64::consts::PI;
      let transform = s!("translate({},{}) rotate({}) translate({},{})",
        fmt2(t1x), fmt2(t1y), fmt2(angle), fmt2(t2x), fmt2(t2y));
      let box_val = state::lookup_value("box0").unwrap_or(Stored::None);
      state::assign_value("box0", Stored::None, None);
      Ok(stored_map!("xy_transform" => transform, "xy_box" => box_val))
    }
  );

  // Frame variant macros (Perl L764-770)
  ::latexml_core::stomach::raw_tex(concat!(
    r"\expandafter\def\csname frm{.}\endcsname{\addtoDrop@@{\lx@xy@dotpat}\csname frm{-}\endcsname}",
    r"\expandafter\def\csname frm{o-}\endcsname{\addtoDrop@@{\lx@xy@dashpat}\csname frm{-}\endcsname}",
    r"\expandafter\def\csname frm{--}\endcsname{\addtoDrop@@{\lx@xy@dashpat}\csname frm{-}\endcsname}",
    r"\expandafter\def\csname frm{.o}\endcsname{\addtoDrop@@{\lx@xy@dotpat}\csname frm{o}\endcsname}",
    r"\expandafter\def\csname frm{-o}\endcsname{\addtoDrop@@{\lx@xy@dashpat}\csname frm{o}\endcsname}",
    r"\expandafter\def\csname frm{.e}\endcsname{\addtoDrop@@{\lx@xy@dotpat}\csname frm{e}\endcsname}",
    r"\expandafter\def\csname frm{-e}\endcsname{\addtoDrop@@{\lx@xy@dashpat}\csname frm{e}\endcsname}",
  ))?;

  // Frame fill/emph macros (Perl L759-760)
  DefMacro!("\\frame@fill@@ {}", "\\lx@xy@fill@on\\lx@xy@stroke@off\\framed@@{#1}");
  DefMacro!("\\frame@emph@@ {}", "\\lx@xy@fill@on\\lx@xy@stroke@on\\lx@xy@solidpat\\framed@@{#1}");

  // Line extension Let bindings (Perl L951)
  ::latexml_core::stomach::raw_tex(concat!(
    r"\let\xy@polystyle@@\xy@polystyle@",
    r"\let\xylinewidth@@\xylinewidth@",
  ))?;

  // Infrastructure: globalize edge values from \OBJECT@x so they survive
  // \halign cell groups. Currently unused but retained for potential future hookup.
  DefPrimitive!("\\lx@xy@globalize@edges", {
    let dc = xy_reg_dim("\\D@c");
    let uc = xy_reg_dim("\\U@c");
    let lc = xy_reg_dim("\\L@c");
    let rc = xy_reg_dim("\\R@c");
    assign_register("\\D@c", RegisterValue::Dimension(dc), Some(Scope::Global), Vec::new())?;
    assign_register("\\U@c", RegisterValue::Dimension(uc), Some(Scope::Global), Vec::new())?;
    assign_register("\\L@c", RegisterValue::Dimension(lc), Some(Scope::Global), Vec::new())?;
    assign_register("\\R@c", RegisterValue::Dimension(rc), Some(Scope::Global), Vec::new())?;
  });
  // Hook into \object to globalize edge values right after OBJECT@x completes.
  // \object (xy.tex L942) = \hbox\bgroup\resetStyle@\object@
  // After OBJECT@x's \egroup closes the \bgroup, the remaining tokens from
  // OBJECT@x's \toks@ execute. We can't easily intercept between OBJECT@x and
  // the caller. Instead, override \idfromc@ which is called right after \drop@'s
  // \object call sets up the entry position.
  // Actually: override \drop@ itself, reading the centered values AFTER \object:



  // \lx@xy@notealignment — record alignment for matrix measurement (Perl L1062-1066)

  // Records the current Alignment object globally so \xymatrix@measureit can access
  // it after the \halign is finished. Sets preserve_structure to prevent pruning.
  DefPrimitive!("\\lx@xy@notealignment", {
    if let Some(alignment_d) = state::lookup_alignment() {
      if let Some(alignment) = alignment_d.alignment_cell() {
        alignment.borrow_mut().set_property("preserve_structure", Stored::Bool(true));
      }
      // Save globally so we can access it in \xymatrix@measureit
      state::assign_value("xymatrix_alignment", Stored::Digested(alignment_d.clone()), Some(Scope::Global));
    }
  });

  // \xymatrix@measureit — matrix dimension measurement (Perl L1068-1090)
  // xy-pic's own measurement uses \lastbox which doesn't work in LaTeXML.
  // Instead, read the saved alignment's computed row heights and column widths,
  // and define \Hrow@N, \Wcol@N, \H@max, \W@max macros for xy-pic.
  DefPrimitive!("\\xymatrix@measureit", {
    let alignment_d = state::lookup_value("xymatrix_alignment")
      .and_then(|v| if let Stored::Digested(d) = v { Some(d.clone()) } else { None });
    if let Some(ref alignment_d) = alignment_d {
      if let Some(alignment) = alignment_d.alignment_cell() {
        // Normalize to compute row/column dimensions
        alignment.borrow_mut().normalize()?;
        let row_heights = alignment.borrow().get_row_heights().to_vec();
        let col_widths = alignment.borrow().get_column_widths().to_vec();
        // Define \Hrow@1, \Hrow@2, ... and find \H@max
        // Must be global scope — Perl DefMacroI defaults to global
        let global_opts = |_: &str| -> Option<ExpandableOptions> {
          Some(ExpandableOptions { scope: Some(Scope::Global), ..Default::default() })
        };
        let mut h_max = Dimension::default();
        for (i, h) in row_heights.iter().enumerate() {
          let name = s!("\\Hrow@{}", i + 1);
          def_macro(T_CS!(&name), None, Tokenize!(&h.to_string()), global_opts(&name))?;
          h_max = h_max.larger(*h);
        }
        // Add fake last row (Perl L1077-1078)
        let last_idx = row_heights.len() + 1;
        let name = s!("\\Hrow@{}", last_idx);
        def_macro(T_CS!(&name), None, Tokenize!("0pt"), global_opts(&name))?;
        def_macro(T_CS!("\\H@max"), None, Tokenize!(&h_max.to_string()), global_opts(""))?;
        // Define \Wcol@1, \Wcol@2, ... and find \W@max
        let mut w_max = Dimension::default();
        for (j, w) in col_widths.iter().enumerate() {
          let name = s!("\\Wcol@{}", j + 1);
          def_macro(T_CS!(&name), None, Tokenize!(&w.to_string()), global_opts(&name))?;
          w_max = w_max.larger(*w);
        }
        def_macro(T_CS!("\\W@max"), None, Tokenize!(&w_max.to_string()), global_opts(""))?;
        def_macro(T_CS!("\\HW@max"), None, Tokenize!(&h_max.larger(w_max).to_string()), global_opts(""))?;
        // Reset counters (Perl L1086-1088)
        assign_register("\\Col", RegisterValue::Number(Number::new(0)), Some(Scope::Global), Vec::new())?;
        assign_register("\\Row", RegisterValue::Number(Number::new(0)), Some(Scope::Global), Vec::new())?;
        assign_register("\\count@@", RegisterValue::Number(Number::new(0)), Some(Scope::Global), Vec::new())?;
      }
    }
  });

  // Tips style management (Perl L934-943)
  state::assign_value("xy_tips_style", Stored::String(arena::pin("xy")), None);
  state::assign_value("xy_tips_pending_style", Stored::String(arena::pin("xy")), None);
  DefPrimitive!("\\SelectTips {}{}", sub[(style, _size)] {
    let style_s = style.to_string();
    if matches!(style_s.as_str(), "xy" | "cm" | "eu" | "lu") {
      state::assign_value("xy_tips_pending_style", Stored::String(arena::pin(&style_s)), None);
      state::assign_value("xy_tips_style", Stored::String(arena::pin(&style_s)), None);
    }
  });
  DefPrimitive!("\\UseTips", {
    // Probe xy_tips_pending_style in place and just re-pin the SymStr
    // on xy_tips_style when present — no owned String needed.
    let sym = state::with_value("xy_tips_pending_style", |v| match v {
      Some(Stored::String(s)) if !arena::with(*s, |p| p.is_empty()) => *s,
      _ => arena::pin("xy"),
    });
    state::assign_value("xy_tips_style", Stored::String(sym), None);
  });
  DefPrimitive!("\\NoTips", {
    state::assign_value("xy_tips_style", Stored::String(arena::pin("xy")), None);
  });

  // \lx@xy@poly — polyline with stroke styling (Perl L962-983)
  // Perl L959-960: cap/join code arrays
  // our @xy_cap_codes  = (qw(butt round square));
  // our @xy_join_codes = (qw(miter round bevel));
  DefConstructor!("\\lx@xy@poly {}",
    sub[document, _args, props] {
      let path = match props.get("xy_path") {
        Some(Stored::String(s)) => arena::to_string(*s),
        _ => return Ok(()),
      };
      let stroke = match props.get("xy_stroke") {
        Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("#000000"),
      };
      let fill = match props.get("xy_fill") {
        Some(Stored::String(s)) => arena::to_string(*s), _ => String::from("none"),
      };
      let mut attrs = string_map!("d" => path, "stroke" => stroke, "fill" => fill);
      if let Some(Stored::String(d)) = props.get("xy_dashes") {
        if !arena::with(*d, |s| s.is_empty()) {
          attrs.insert(String::from("stroke-dasharray"), arena::to_string(*d));
        }
      }
      // Stroke styling attributes (Perl L963-964)
      if let Some(Stored::String(s)) = props.get("xy_thickness") {
        let th = arena::to_string(*s);
        if !th.is_empty() && th != "0" { attrs.insert(String::from("stroke-width"), th); }
      }
      if let Some(Stored::String(s)) = props.get("xy_cap") {
        let cap = arena::to_string(*s);
        if !cap.is_empty() { attrs.insert(String::from("stroke-linecap"), cap); }
      }
      if let Some(Stored::String(s)) = props.get("xy_join") {
        let join = arena::to_string(*s);
        if !join.is_empty() { attrs.insert(String::from("stroke-linejoin"), join); }
      }
      if let Some(Stored::String(s)) = props.get("xy_miter") {
        let miter = arena::to_string(*s);
        if !miter.is_empty() { attrs.insert(String::from("stroke-miterlimit"), miter); }
      }
      svg_empty_element(document, "svg:path", attrs)?;
    },
    properties => sub[args] {
      let (stroke, fill, dashes) = xy_capture_stroke_fill();
      let points_str = args.first().and_then(|a| a.as_ref())
        .map(|t| t.to_string()).unwrap_or_default();
      let dpi = state::lookup_int("DPI");
      let dpi = if dpi > 0 { dpi as f64 } else { 100.0 };
      let pt_px = dpi / 72.27;
      let points: Vec<f64> = points_str.split_whitespace()
        .filter_map(|s| s.parse::<f64>().ok())
        .map(|v| (v * pt_px * 100.0).round() / 100.0)
        .collect();
      // Stroke styling (Perl L975-982)
      let th = dim_to_px(xy_reg_dim("\\xylinethick@"));
      // Cap/join/miter — these are digested from \xylinecap@, \xylinejoin@, \xylinemiter@
      let cap_idx: usize = macro_string("\\xylinecap@").trim().parse().unwrap_or(0);
      let join_idx: usize = macro_string("\\xylinejoin@").trim().parse().unwrap_or(0);
      let miter_str = macro_string("\\xylinemiter@").trim().to_string();
      let cap_codes = ["butt", "round", "square"];
      let join_codes = ["miter", "round", "bevel"];
      let cap = cap_codes.get(cap_idx).unwrap_or(&"butt");
      let join = join_codes.get(join_idx).unwrap_or(&"miter");
      if points.len() >= 2 {
        let mut path = s!("M {} {}", points[0], points[1]);
        let mut i = 2;
        while i + 1 < points.len() {
          path.push_str(&s!(" L {} {}", points[i], points[i + 1]));
          i += 2;
        }
        Ok(stored_map!(
          "xy_path" => path, "xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes,
          "xy_thickness" => fmt2(th), "xy_cap" => *cap, "xy_join" => *join, "xy_miter" => miter_str
        ))
      } else {
        Ok(stored_map!("xy_stroke" => stroke, "xy_fill" => fill, "xy_dashes" => dashes))
      }
    }
  );

  // Enable features — messages (Perl L55-70)
  // Most are no-ops; \coloron and \crayonon trigger their setup macros (Perl L61-62)
  DefMacro!("\\lx@xy@latexmlon", "");
  DefMacro!("\\lx@xy@curveon", "");
  DefMacro!("\\lx@xy@frameon", "");
  DefMacro!("\\lx@xy@tipson", "");
  DefMacro!("\\lx@xy@lineon", "");

  // Perl L950-957: line styles extension stubs
  // Use our definitions, NOT the raw TeX stubs
  Let!("\\xy@polystyle@@", "\\xy@polystyle@");
  // Perl L952: Use our definitions, NOT the raw TeX stubs
  // These contain @ in CS names — use DefMacro!/Let! which bypass catcode issues
  DefMacro!("\\xylinewidth@{}", "");
  DefMacro!("\\xylinewidth@i{}", "");
  DefMacro!("\\xyshape@thicker@", "");
  DefMacro!("\\xyshape@thinner@", "");
  Let!("\\xylinewidth@@", "\\xylinewidth@");
  DefMacro!("\\xypolyline@Special", "\\lx@xy@stroke@on\\lx@xy@fill@off\\lx@xy@poly");
  DefMacro!("\\xypolyfill@Special", "\\lx@xy@stroke@off\\lx@xy@fill@on\\lx@xy@poly");
  DefMacro!("\\xypolyeofill@Special", "\\lx@xy@stroke@off\\lx@xy@fill@on\\lx@xy@poly");
  DefMacro!("\\xypolydot@Special", "\\lx@xy@stroke@on\\lx@xy@dotpat\\lx@xy@fill@off\\lx@xy@poly");
  DefMacro!("\\xypolydash@Special", "\\lx@xy@stroke@on\\lx@xy@dashpat\\lx@xy@fill@off\\lx@xy@poly");
  DefMacro!("\\lx@xy@rotateon", "");
  DefMacro!("\\lx@xy@coloron", "\\xystandardcolors@");
  DefMacro!("\\lx@xy@crayonon", "\\installCrayolaColors@");
  DefMacro!("\\lx@xy@matrixon", "");
  DefMacro!("\\lx@xy@arrowon", "");
  DefMacro!("\\lx@xy@graphon", "");
  DefMacro!("\\lx@xy@arcon", "");
  DefMacro!("\\lx@xy@polyon", "");
  DefMacro!("\\lx@xy@knoton", "");
  DefMacro!("\\lx@xy@tileon", "");
  DefMacro!("\\lx@xy@webon", "");
});
