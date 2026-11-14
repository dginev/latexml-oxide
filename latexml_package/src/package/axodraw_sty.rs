//! axodraw.sty — J.A.M. Vermaseren's Feynman-diagram package, version 1
//! ("Made by J.A.M. Vermaseren 14-feb-1994").
//!
//! axodraw v1 is neither in TeX Live (which has its successor `axodraw2`) nor,
//! mostly, shipped with the papers that load it: run 336 had 1,507 arXiv
//! papers with `\Text` undefined (149 as their first error; witnesses
//! hep-th0506235, hep-ph0512261, hep-ph0305218, 0710.3255, 0809.0800). Its
//! drawing layer is PostScript: every command is `\put(\axoxoff,\axoyoff)`
//! around a `\special{"…}` that calls a procedure the package installs with
//! `\special{! …}` (dvips header code), so a raw load draws nothing even where
//! the file is present. This binding keeps v1's TeX layer — the same command
//! signatures, `\Set*` state macros and `\put` placement — and replaces each
//! `\special` with `\lx@axo@draw{procedure}{arguments}`, which evaluates the
//! procedure's geometry here, transcribed from the v1 PostScript, into the
//! picture model (`ltx:line`, `ltx:bezier`, `ltx:arc`, `ltx:circle`,
//! `ltx:polygon`, `ltx:rect`, `ltx:ellipse`) that `{picture}` and the
//! pstricks binding emit and the SVG post-processor renders.
//!
//! A paper's own copy of axodraw.sty (about one zip in 830 ships one) is
//! answered by this binding too, as any binding outranks a raw file: raw, its
//! drawing is lost all the same. axodraw2 (TeX Live) and axodraw4j
//! (JaxoDraw's 2008 keyval forerunner of axodraw2, shipped by papers:
//! 1301.5845, 1701.08134) are other packages, read from their files
//! (axodraw2_sty.rs keeps the versioned-name fallback off axodraw2).
//!
//! An argument is what v1 splices into PostScript, so it may be a PostScript
//! expression over a dvips-header constant (`{20 Tdensity mul}`, 0901.1195,
//! 1701.01554): `ps_value` evaluates the arithmetic.
//!
//! The command set is the union of the v1 variants arXiv papers ship (all
//! share one PostScript geometry): the original 1994 file (hep-ph9702393,
//! hep-ph0101185) and the colordvi-aware one adding `\SetColor`, `\CCirc`,
//! `\CBox(c)`, `\COval`, `\CText`, `\C2Text` and the triangles (hep-ph0101235).
//!
//! Units, v1 semantics (both sides deliberate in v1, and relied on by papers:
//! 0809.0800 writes `\unitlength 0.6pt\SetScale{0.6}`): graphics coordinates
//! are PostScript points scaled by `\SetScale` and ignore `\unitlength`;
//! `\Text`/`\rText` place TeX text at `\unitlength` multiples, unscaled. v1
//! draws in big points; this reads them as TeX points, the correction
//! axodraw2 made (axodraw2.sty:3080 `/gs{gsave 1.00375 div dup scale}`), so a
//! label set at the same coordinates as a line meets it.
use std::{collections::VecDeque, f64::consts::PI};

use latexml_core::common::numeric_ops::UNITY_F64;

use crate::{
  package::color_sty::{color_key, lookup_color},
  prelude::*,
};

type Pt = (f64, f64);
type Attrs = HashMap<String, String>;

/// PostScript's current transformation matrix: user `(x,y)` maps to
/// `(a·x + c·y + e, b·x + d·y + f)` in picture points.
#[derive(Clone, Copy, Debug)]
struct Ctm {
  a: f64,
  b: f64,
  c: f64,
  d: f64,
  e: f64,
  f: f64,
}

impl Ctm {
  /// v1's `gs` (`gsave dup scale`): every procedure draws in user space
  /// scaled by `\axoscale`.
  fn scaled(s: f64) -> Self {
    Ctm {
      a: s,
      b: 0.0,
      c: 0.0,
      d: s,
      e: 0.0,
      f: 0.0,
    }
  }

  /// PostScript `tx ty translate`.
  fn translate(self, tx: f64, ty: f64) -> Self {
    let (e, f) = self.apply((tx, ty));
    Ctm { e, f, ..self }
  }

  /// PostScript `angle rotate` (degrees, anticlockwise).
  fn rotate(self, degrees: f64) -> Self {
    let (s, c) = degrees.to_radians().sin_cos();
    Ctm {
      a: self.a * c + self.c * s,
      b: self.b * c + self.d * s,
      c: self.c * c - self.a * s,
      d: self.d * c - self.b * s,
      ..self
    }
  }

  fn apply(&self, (x, y): Pt) -> Pt {
    (
      self.a * x + self.c * y + self.e,
      self.b * x + self.d * y + self.f,
    )
  }
}

/// PostScript `round`: the nearest integer, a half rounding up.
fn ps_round(x: f64) -> f64 { (x + 0.5).floor() }
/// PostScript `dy dx atan`, in degrees.
fn ps_atan(dy: f64, dx: f64) -> f64 { dy.atan2(dx).to_degrees() }
fn cosd(degrees: f64) -> f64 { degrees.to_radians().cos() }
fn sind(degrees: f64) -> f64 { degrees.to_radians().sin() }

const BLACK: &str = "#000000";
const WHITE: &str = "#ffffff";

/// PostScript `setgray`: 0 black, 1 white.
fn gray(level: f64) -> String {
  let v = (level.clamp(0.0, 1.0) * 255.0).round() as u8;
  format!("#{v:02x}{v:02x}{v:02x}")
}

/// A length in points as a px attribute value (the picture model's unit; to
/// the 0.01px, as `{picture}`'s coordinates are).
fn px(pt: f64) -> String {
  let v = Dimension::new((pt * UNITY_F64).round() as i64).px_value(Some(2));
  if v == v.trunc() {
    format!("{}", v as i64)
  } else {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').to_string()
  }
}

fn flatten(points: &[Pt]) -> Vec<f64> { points.iter().flat_map(|(x, y)| [*x, *y]).collect() }

fn px_points(points: &[Pt]) -> String {
  points
    .iter()
    .map(|(x, y)| format!("{},{}", px(*x), px(*y)))
    .collect::<Vec<_>>()
    .join(" ")
}

/// v1's dash fitting (`/dashline`, `/dashcarc`): the `[dsize dsize]` pattern
/// is stretched so a path of length `r` holds an odd number `2n+1` of dashes
/// and gaps, beginning and ending with a dash.
fn fit_dash(r: f64, dsize: f64) -> f64 {
  let n = ps_round(r / (dsize * 2.0) - 0.5).max(0.0);
  r / (n * 2.0 + 1.0)
}

/// A v1 argument as the number the PostScript procedure receives: the
/// arguments are spliced into PostScript, so beside a number one may be a
/// PostScript expression — `{6.283 #3 mul 360 div … Tdensity mul}`
/// (0901.1195, 1701.01554), naming constants a dvips header defined
/// (`\special{! /Tdensity {0.10} def}`, recorded by `\special` as
/// `dvips_header:<name>`). Evaluated on an operand stack with the arithmetic
/// operators; `None` for anything else, a stack not left with one value, or a
/// value that is not finite (`1 0 div`, `-1 sqrt`, or the text `inf`, which
/// PostScript would reject: dvips fails the page).
fn ps_value(text: &str) -> Option<f64> { ps_expression(text).filter(|v| v.is_finite()) }

fn ps_expression(text: &str) -> Option<f64> {
  let text = text.trim();
  if let Ok(v) = text.parse::<f64>() {
    return Some(v);
  }
  let mut stack: Vec<f64> = Vec::new();
  for word in text.split_whitespace() {
    if let Ok(v) = word.parse::<f64>() {
      stack.push(v);
      continue;
    }
    match word {
      "add" | "sub" | "mul" | "div" | "exch" | "atan" => {
        let b = stack.pop()?;
        let a = stack.pop()?;
        match word {
          "add" => stack.push(a + b),
          "sub" => stack.push(a - b),
          "mul" => stack.push(a * b),
          "div" => stack.push(a / b),
          "atan" => stack.push(ps_atan(a, b).rem_euclid(360.0)),
          _ => stack.extend([b, a]),
        }
      },
      "neg" | "abs" | "sqrt" | "sin" | "cos" | "dup" => {
        let a = stack.pop()?;
        match word {
          "neg" => stack.push(-a),
          "abs" => stack.push(a.abs()),
          "sqrt" => stack.push(a.sqrt()),
          "sin" => stack.push(sind(a)),
          "cos" => stack.push(cosd(a)),
          _ => stack.extend([a, a]),
        }
      },
      name => {
        let value = lookup_string(&format!("dvips_header:{name}"));
        stack.push(value.trim().parse::<f64>().ok()?);
      },
    }
  }
  match stack.as_slice() {
    [v] => Some(*v),
    _ => None,
  }
}

/// The most pieces one procedure call draws: winding counts and axis
/// subdivisions beyond it (`{20000000}` windings) are refused with the "Cannot
/// draw" warning, as the PostScript would run for hours, not converted for as
/// long. Real diagrams stay below a few hundred.
const MAX_PIECES: f64 = 10_000.0;

/// A count of pieces (windings, half wiggles, axis marks) a procedure may draw.
fn drawable_count(count: f64) -> bool { count.abs() <= MAX_PIECES }

/// The number of passes of PostScript's `first 1 limit {…} for` — none when
/// `limit < first` (a NaN or `-inf` limit too) — or `None` for an infinite pass
/// count or one beyond [`MAX_PIECES`].
fn ps_for_count(first: f64, limit: f64) -> Option<usize> {
  let passes = if limit >= first {
    (limit - first).floor() + 1.0
  } else {
    0.0
  };
  (passes.is_finite() && drawable_count(passes)).then_some(passes as usize)
}

/// The value of a v1 state macro (`\axowidth` is `0.5 `, …) as a number; one
/// PostScript cannot read (a direct `\def\axoscale{0,5}`; `\SetScale` and
/// `\SetWidth` refuse such values themselves) is reported, once, and read as 0.
fn axo_number(cs: &str) -> Result<f64> {
  let text = do_expand(Tokens::new(vec![T_CS!(cs)]))?.to_string();
  Ok(ps_value(&text).unwrap_or_else(|| {
    if !already_reported(&format!("axodraw_value_{cs}_{}", text.trim())) {
      Warn!(
        "unexpected",
        "axodraw",
        format!("Cannot read {cs} = '{}' as a number; using 0", text.trim())
      );
    }
    0.0
  }))
}

/// A v1 color name (a colordvi/dvipsnam name: `Red`, `Blue`, …) as a color
/// attribute; blank — v1 without colordvi, `\def\axocolor{ }` — is black.
/// The names are dvips's whichever way they are known: plainly (colordvi,
/// `color` with `usenames`, or its on-demand dvipsnam load), or as color's
/// `named` model when `color` loaded dvipsnam.def for its `dvips` option
/// without `usenames` (hep-ph0612301: `\documentclass[dvips]{article}`).
fn axo_color(name: &str) -> String {
  let name = name.trim();
  if name.is_empty() {
    return BLACK.to_string();
  }
  let named = format!("named_{name}");
  if lookup_value(&color_key(name)).is_none() && lookup_value(&color_key(&named)).is_some() {
    lookup_color(&named)
  } else {
    lookup_color(name)
  }
}

/// The drawing state a v1 procedure is called with.
struct Pen {
  /// `\axoscale` (`\SetScale`).
  scale:  f64,
  /// `\axowidth` (`\SetWidth`), in user units.
  width:  f64,
  /// `\axocolor` (`\SetColor`) as a color attribute.
  color:  String,
  /// `\axoxo`/`\axoyo` (`\SetScaledOffset`), added to every point.
  offset: Pt,
  /// `\fsize` (`\SetPFont`), the PostScript text size in user units.
  fsize:  f64,
}

impl Pen {
  fn current() -> Result<Pen> {
    let color = do_expand(Tokens::new(vec![T_CS!("\\axocolor")]))?.to_string();
    let color = color.trim();
    Ok(Pen {
      scale:  axo_number("\\axoscale")?,
      width:  axo_number("\\axowidth")?,
      color:  axo_color(color),
      offset: (axo_number("\\axoxo")?, axo_number("\\axoyo")?),
      fsize:  axo_number("\\fsize")?,
    })
  }
}

/// The elements one procedure call draws, in the coordinates of its `\put`.
struct Drawing {
  pen:        Pen,
  base:       Ctm,
  elements:   Vec<Attrs>,
  /// An element was not drawn: its numbers were not finite.
  degenerate: bool,
}

impl Drawing {
  fn new(pen: Pen) -> Self {
    let base = Ctm::scaled(pen.scale);
    Drawing {
      pen,
      base,
      elements: Vec::new(),
      degenerate: false,
    }
  }

  /// A user point: v1 adds `\axoxo`/`\axoyo` before the procedure runs.
  fn at(&self, x: f64, y: f64) -> Pt { (x + self.pen.offset.0, y + self.pen.offset.1) }

  /// Add an element computed from `numbers`. A degenerate argument (a zero
  /// amplitude, coincident curve points) makes the PostScript divide by
  /// zero, failing the dvips page; only the element it breaks is dropped,
  /// and the drawing reported.
  fn push(&mut self, qname: &str, numbers: &[f64], mut attrs: Attrs) {
    if numbers.iter().any(|v| !v.is_finite()) {
      self.degenerate = true;
      return;
    }
    attrs.insert("_qname".to_string(), qname.to_string());
    self.elements.push(attrs);
  }

  fn stroke(&self, color: &str, width: f64) -> Attrs {
    map!(
      "stroke" => color.to_string(),
      "stroke-width" => px(width * self.pen.scale)
    )
  }

  /// A dash pattern `[d d]`; a dash size of zero (`\DashLine(…){0}`, which
  /// PostScript's `setdash` rejects) leaves the line solid.
  fn dashed(&self, attrs: &mut Attrs, dash: Option<f64>) {
    if let Some(d) = dash.filter(|d| d.is_finite() && *d > 0.0) {
      let d = px(d * self.pen.scale);
      attrs.insert("stroke-dasharray".to_string(), format!("{d} {d}"));
    }
  }

  /// `newpath … lineto … stroke` with the current line width.
  fn line(&mut self, ctm: Ctm, points: &[Pt], dash: Option<f64>) {
    let width = self.pen.width;
    self.line_width(ctm, points, dash, width);
  }

  fn line_width(&mut self, ctm: Ctm, points: &[Pt], dash: Option<f64>, width: f64) {
    let mapped: Vec<Pt> = points.iter().map(|p| ctm.apply(*p)).collect();
    let mut attrs = self.stroke(&self.pen.color.clone(), width);
    attrs.insert("points".to_string(), px_points(&mapped));
    self.dashed(&mut attrs, dash);
    self.push("ltx:line", &flatten(&mapped), attrs);
  }

  /// One stroked `curveto` segment.
  fn curve(&mut self, ctm: Ctm, points: [Pt; 4]) {
    let mapped: Vec<Pt> = points.iter().map(|p| ctm.apply(*p)).collect();
    let mut attrs = self.stroke(&self.pen.color.clone(), self.pen.width);
    attrs.insert("points".to_string(), px_points(&mapped));
    self.push("ltx:bezier", &flatten(&mapped), attrs);
  }

  /// `x y r a1 a2 arc stroke` under a translation: PostScript raises `a2` by
  /// 360 until it is at least `a1`, so the arc runs anticlockwise from `a1`;
  /// a sweep of 360 or more is the full circle.
  fn arc(&mut self, ctm: Ctm, r: f64, a1: f64, a2: f64, dash: Option<f64>) {
    let sweep = if a2 >= a1 {
      a2 - a1
    } else {
      (a2 - a1).rem_euclid(360.0)
    };
    let (cx, cy) = ctm.apply((0.0, 0.0));
    let mut attrs = self.stroke(&self.pen.color.clone(), self.pen.width);
    attrs.insert("x".to_string(), px(cx));
    attrs.insert("y".to_string(), px(cy));
    attrs.insert("r".to_string(), px(r * self.pen.scale));
    self.dashed(&mut attrs, dash);
    if sweep >= 360.0 {
      attrs.insert("fill".to_string(), "none".to_string());
      self.push("ltx:circle", &[cx, cy, r, sweep], attrs);
    } else if sweep > 0.0 {
      attrs.insert("angle1".to_string(), format!("{a1}"));
      attrs.insert("angle2".to_string(), format!("{}", a1 + sweep));
      self.push("ltx:arc", &[cx, cy, r, sweep], attrs);
    }
  }

  /// `arcn`: clockwise from `a1` to `a2`, the anticlockwise arc from `a2`.
  fn arcn(&mut self, ctm: Ctm, r: f64, a1: f64, a2: f64, dash: Option<f64>) {
    self.arc(ctm, r, a2, a1, dash);
  }

  /// A circle of radius `r` around the `ctm` origin, filled and/or stroked.
  fn circle(&mut self, ctm: Ctm, r: f64, fill: Option<&str>, stroke: Option<&str>) {
    let (cx, cy) = ctm.apply((0.0, 0.0));
    let mut attrs = match stroke {
      Some(color) => self.stroke(color, self.pen.width),
      None => map!("stroke" => "none".to_string()),
    };
    attrs.insert("x".to_string(), px(cx));
    attrs.insert("y".to_string(), px(cy));
    attrs.insert("r".to_string(), px(r * self.pen.scale));
    attrs.insert("fill".to_string(), fill.unwrap_or("none").to_string());
    self.push("ltx:circle", &[cx, cy, r], attrs);
  }

  fn polygon(&mut self, ctm: Ctm, points: &[Pt], fill: Option<&str>, stroke: Option<&str>) {
    let mapped: Vec<Pt> = points.iter().map(|p| ctm.apply(*p)).collect();
    let mut attrs = match stroke {
      Some(color) => self.stroke(color, self.pen.width),
      None => map!("stroke" => "none".to_string()),
    };
    attrs.insert("points".to_string(), px_points(&mapped));
    attrs.insert("fill".to_string(), fill.unwrap_or("none").to_string());
    self.push("ltx:polygon", &flatten(&mapped), attrs);
  }

  /// v1's `abox` (the rectangle on corners `p1`, `p2`).
  fn rect(&mut self, p1: Pt, p2: Pt, fill: Option<&str>, stroke: Option<&str>) {
    let (x1, y1) = self.base.apply(p1);
    let (x2, y2) = self.base.apply(p2);
    let mut attrs = match stroke {
      Some(color) => self.stroke(color, self.pen.width),
      None => map!("stroke" => "none".to_string()),
    };
    attrs.insert("x".to_string(), px(x1.min(x2)));
    attrs.insert("y".to_string(), px(y1.min(y2)));
    attrs.insert("width".to_string(), px((x2 - x1).abs()));
    attrs.insert("height".to_string(), px((y2 - y1).abs()));
    attrs.insert("fill".to_string(), fill.unwrap_or("none").to_string());
    self.push("ltx:rect", &[x1, y1, x2, y2], attrs);
  }

  /// v1's `/oval`: `x1 rotate x2 radius div 1.0 scale` on the circle of
  /// radius `height` — an ellipse of semi-axes `width` (along the rotated x)
  /// and `height`.
  fn ellipse(
    &mut self,
    c: Pt,
    height: f64,
    width: f64,
    angle: f64,
    fill: Option<&str>,
    stroke: Option<&str>,
  ) {
    let (cx, cy) = self.base.apply(c);
    let mut attrs = match stroke {
      Some(color) => self.stroke(color, self.pen.width),
      None => map!("stroke" => "none".to_string()),
    };
    attrs.insert("x".to_string(), px(cx));
    attrs.insert("y".to_string(), px(cy));
    attrs.insert("rx".to_string(), px(width.abs() * self.pen.scale));
    attrs.insert("ry".to_string(), px(height.abs() * self.pen.scale));
    attrs.insert("fill".to_string(), fill.unwrap_or("none").to_string());
    if angle != 0.0 {
      attrs.insert(
        "transform".to_string(),
        format!("rotate({angle},{},{})", px(cx), px(cy)),
      );
    }
    self.push("ltx:ellipse", &[cx, cy, width, height, angle], attrs);
  }

  /// `nwidth = width + 1`, the arrowhead scale of `/arrowright` etc.
  fn arrow_size(&self) -> f64 { self.pen.width + 1.0 }

  /// `/arrowright`: the filled triangle pointing along +x.
  fn arrow_right(&mut self, ctm: Ctm) {
    let n = self.arrow_size();
    let color = self.pen.color.clone();
    self.polygon(
      ctm,
      &[(1.5 * n, 0.0), (-1.5 * n, 1.2 * n), (-1.5 * n, -1.2 * n)],
      Some(&color),
      None,
    );
  }

  /// `/arrowup` (`up = true`) or `/arrowdown`: pointing along ±y.
  fn arrow_vertical(&mut self, ctm: Ctm, up: bool) {
    let n = self.arrow_size();
    let s = if up { 1.0 } else { -1.0 };
    let color = self.pen.color.clone();
    self.polygon(
      ctm,
      &[
        (0.0, 1.5 * n * s),
        (1.2 * n, -1.5 * n * s),
        (-1.2 * n, -1.5 * n * s),
      ],
      Some(&color),
      None,
    );
  }

  /// The frame of a straight procedure: origin at `p1`, x along `p1→p2`;
  /// returns it with the length `dr`.
  fn segment_frame(&self, p1: Pt, p2: Pt) -> (Ctm, f64) {
    let (dx, dy) = (p2.0 - p1.0, p2.1 - p1.1);
    let dr = (dx * dx + dy * dy).sqrt();
    (self.base.translate(p1.0, p1.1).rotate(ps_atan(dy, dx)), dr)
  }

  /// `/gluon`: a head and a tail of one winding each and `num-1` windings of
  /// body, as one path of `curveto`s.
  fn gluon(&mut self, p1: Pt, p2: Pt, ampi: f64, num: f64) {
    let (f, dr) = self.segment_frame(p1, p2);
    let num = ps_round(num - 0.5);
    let inc = dr / (num * 2.0 + 2.0);
    let amp8 = (ampi * 0.9).abs();
    let mut x1 = inc * 2.0;
    let mut current = (0.0, 0.0);
    let mut curveto = |d: &mut Drawing, c1: Pt, c2: Pt, end: Pt| {
      d.curve(f, [current, c1, c2, end]);
      current = end;
    };
    curveto(
      self,
      (inc * 0.1, ampi * 0.5),
      (inc * 0.5, ampi),
      (inc * 1.4, ampi),
    );
    curveto(self, (x1 + amp8, ampi), (x1 + amp8, -ampi), (x1, -ampi));
    // `2 1 num {…} for`, on an integer counter
    for _ in 0..ps_for_count(2.0, num).unwrap_or(0) {
      curveto(
        self,
        (x1 - amp8, -ampi),
        (x1 - amp8, ampi),
        (x1 + inc, ampi),
      );
      x1 += inc * 2.0;
      curveto(self, (x1 + amp8, ampi), (x1 + amp8, -ampi), (x1, -ampi));
    }
    curveto(
      self,
      (x1 - amp8, -ampi),
      (x1 - amp8, ampi),
      (x1 + inc * 0.6, ampi),
    );
    curveto(
      self,
      (x1 + inc * 1.5, ampi),
      (dr - inc * 0.1, ampi * 0.5),
      (dr, 0.0),
    );
  }

  /// `/photon` (`zigzag = false`): `2·num - 0.5` (rounded) half wiggles, each
  /// a separately stroked `curveto` of alternating sign; `/zigzag`: the same
  /// half periods as two `lineto`s.
  fn photon(&mut self, p1: Pt, p2: Pt, ampi: f64, num: f64, zigzag: bool) {
    let (mut f, dr) = self.segment_frame(p1, p2);
    let num = ps_round(num * 2.0 - 0.5);
    let x2 = dr / num;
    let mut sign = 1.0;
    // `1 1 num {…} for`, on an integer counter
    for _ in 0..ps_for_count(1.0, num).unwrap_or(0) {
      if zigzag {
        self.line(f, &[(0.0, 0.0), (x2 / 2.0, ampi * sign), (x2, 0.0)], None);
      } else {
        // v1 `/pi 3.141592` (π to 6.5e-7, immaterial at any drawing size)
        let a = 4.0 / 3.0 * x2 / PI;
        let h = 4.0 / 3.0 * ampi * sign;
        self.curve(f, [(0.0, 0.0), (a, h), (x2 - a, h), (x2, 0.0)]);
      }
      sign = -sign;
      f = f.translate(x2, 0.0);
    }
  }

  /// `/photonarc`: `2·num` (rounded) half wiggles on the arc, each one
  /// `curveto` in a frame rotated by `phi` per step.
  fn photon_arc(&mut self, c: Pt, radius: f64, arcstart: f64, arcend: f64, ampli: f64, num: f64) {
    let num = ps_round(num * 2.0);
    let arcend = if arcend < arcstart {
      arcend + 360.0
    } else {
      arcend
    };
    let phi = (arcend - arcstart) / num;
    let mut f = self.base.translate(c.0, c.1).rotate(arcstart);
    let (cp, sp) = (cosd(phi), sind(phi));
    let (cp2, sp2) = (cosd(phi / 2.0), sind(phi / 2.0));
    let mut ampli = ampli;
    // `1 1 num {…} for`, on an integer counter
    for _ in 0..ps_for_count(1.0, num).unwrap_or(0) {
      let beta = radius * phi / (180.0 * ampli);
      let tt = (sp - cp * beta) / (cp + sp * beta);
      let amp1 = ((radius + ampli) * 8.0 * (beta * cp2 - sp2)
        - (beta * (4.0 + cp) + (tt * cp * 3.0 - sp * 4.0)) * radius)
        / ((beta - tt) * 3.0);
      let x1 = ((radius + ampli) * 8.0 * cp2 - (1.0 + cp) * radius) / 3.0 - amp1;
      self.curve(f, [
        (radius, 0.0),
        (x1, (x1 - radius) * beta),
        (amp1, (amp1 - radius * cp) * tt + radius * sp),
        (radius * cp, radius * sp),
      ]);
      ampli = -ampli;
      f = f.rotate(phi);
    }
  }

  /// `/gluearc`: the straight gluon laid along the arc — a point `x` along
  /// the arc and `a` from the centre maps to `(a·cos(x·const), a·sin(x·const))`,
  /// with the control radii corrected (`amp4`…`amp7`) for the bending.
  fn gluon_arc(&mut self, c: Pt, radius: f64, arcstart: f64, arcend: f64, ampi: f64, num: f64) {
    let f = self.base.translate(c.0, c.1).rotate(arcstart);
    let darc = arcend - arcstart;
    // v1 `3.141592` (π)
    let dr = darc / 180.0 * PI * radius;
    let konst = darc / dr;
    let num = ps_round(num - 0.5);
    let inc = dr / (num * 2.0 + 2.0);
    let amp8 = ampi * 0.9;
    let amp1 = radius + ampi;
    let amp2 = radius - ampi;
    let amp3 = radius + ampi / 2.0;
    let amp4 = amp1 / cosd((inc + amp8) * konst);
    let amp5 = amp2 / cosd(amp8 * konst);
    let amp6 = amp1 / cosd((inc * 0.6 + amp8) * konst);
    let amp7 = amp1 / cosd(inc * 0.9 * konst);
    let amp8 = amp8.abs();
    let p = |x: f64, a: f64| -> Pt { (cosd(x * konst) * a, sind(x * konst) * a) };
    let mut x1 = inc * 2.0;
    let mut current = (radius, 0.0);
    let mut curveto = |d: &mut Drawing, c1: Pt, c2: Pt, end: Pt| {
      d.curve(f, [current, c1, c2, end]);
      current = end;
    };
    curveto(
      self,
      p(inc * 0.1, amp3),
      p(inc * 0.5, amp7),
      p(inc * 1.4, amp1),
    );
    curveto(self, p(x1 + amp8, amp6), p(x1 + amp8, amp5), p(x1, amp2));
    // `2 1 num {…} for`, on an integer counter
    for _ in 0..ps_for_count(2.0, num).unwrap_or(0) {
      curveto(
        self,
        p(x1 - amp8, amp5),
        p(x1 - amp8, amp4),
        p(x1 + inc, amp1),
      );
      x1 += inc * 2.0;
      curveto(self, p(x1 + amp8, amp4), p(x1 + amp8, amp5), p(x1, amp2));
    }
    curveto(
      self,
      p(x1 - amp8, amp5),
      p(x1 - amp8, amp6),
      p(x1 + inc * 0.6, amp1),
    );
    curveto(
      self,
      p(x1 + inc * 1.5, amp7),
      p(dr - inc * 0.1, amp3),
      p(dr, radius),
    );
  }

  /// `/makecurve` (`/docurve`): the cubic through consecutive points whose
  /// tangents are fitted from the neighbours, the end tangents extrapolated.
  fn smooth_curve(points: &[Pt]) -> Vec<[Pt; 4]> {
    let docurve = |p0: Pt, p1: Pt, p2: Pt, p3: Pt| -> [Pt; 4] {
      let ((x0, y0), (x1, y1), (x2, y2), (x3, y3)) = (p0, p1, p2, p3);
      let c1 = (
        (x1 * 2.0 + x2) / 3.0,
        ((y1 - y0) / (x1 - x0) * (x2 - x0) + (y2 - y0) / (x2 - x0) * (x1 - x0) + y1 + y0 * 2.0)
          / 3.0,
      );
      let c2 = (
        (x1 + x2 * 2.0) / 3.0,
        ((y2 - y3) / (x2 - x3) * (x1 - x3) + (y1 - y3) / (x1 - x3) * (x2 - x3) + y2 + y3 * 2.0)
          / 3.0,
      );
      [p1, c1, c2, p2]
    };
    let n = points.len();
    let mut segments = Vec::new();
    if n < 3 {
      return segments;
    }
    let ((x1, y1), (x2, y2), (x3, y3)) = (points[0], points[1], points[2]);
    let x0 = x1 * 2.0 - x2;
    let y0 =
      ((y3 - y2) / (x3 - x2) - (y2 - y1) / (x2 - x1)) * 2.0 * ((x2 - x1) * (x2 - x1) / (x3 - x1))
        + y1 * 2.0
        - y2;
    segments.push(docurve((x0, y0), points[0], points[1], points[2]));
    for w in points.windows(4) {
      segments.push(docurve(w[0], w[1], w[2], w[3]));
    }
    let ((x0, y0), (x1, y1), (x2, y2)) = (points[n - 3], points[n - 2], points[n - 1]);
    let x3 = x2 * 2.0 - x1;
    let y3 =
      ((y2 - y1) / (x2 - x1) - (y1 - y0) / (x1 - x0)) * 2.0 * ((x2 - x1) * (x2 - x1) / (x2 - x0))
        + y2 * 2.0
        - y1;
    segments.push(docurve(
      points[n - 3],
      points[n - 2],
      points[n - 1],
      (x3, y3),
    ));
    segments
  }

  /// `/logaxis` (`linear = false`) and `/linaxis`: the axis line and its
  /// hash marks, the minor ones at 0.6 of the line width. `false` (nothing
  /// drawn) when a step is not finite (`perdec` 0: `x1 = rr/0`) or a mark
  /// count is beyond [`MAX_PIECES`]; the PostScript `for` loops are counted
  /// up front and run on an integer counter, so no float step can stall.
  /// Guard: `perfect_kernel_batch64w::axodraw_v1_unbounded_arguments`.
  #[allow(clippy::too_many_arguments)]
  fn axis(
    &mut self,
    p1: Pt,
    p2: Pt,
    count: f64,
    per: f64,
    hashsize: f64,
    offset: f64,
    width: f64,
    linear: bool,
  ) -> bool {
    let (f, rr) = self.segment_frame(p1, p2);
    let mut major = Vec::new();
    let mut minor = Vec::new();
    if linear {
      let (numdec, perdec) = (count, per);
      // `/x1 rr numdec perdec mul div def /y1 rr numdec div def
      // /offset offset x1 mul def`
      let x1 = rr / (numdec * perdec);
      let y1 = rr / numdec;
      let offset = offset * x1;
      if !(x1.is_finite() && y1.is_finite() && offset.is_finite()) {
        return false;
      }
      // `0 1 numdec {…} for`, `offset cvi 1 numdec perdec mul offset add {…} for`
      let start = offset.trunc();
      let (Some(n_major), Some(n_minor)) = (
        ps_for_count(0.0, numdec),
        ps_for_count(start, numdec * perdec + offset),
      ) else {
        return false;
      };
      major.extend((0..n_major).map(|i| i as f64 * y1 - offset));
      minor.extend((0..n_minor).map(|i| (start + i as f64) * x1 - offset));
    } else {
      let nlogs = count;
      let mut offset = if offset != 0.0 {
        offset.log10()
      } else {
        offset
      };
      offset -= offset.trunc();
      let lsize = rr / nlogs;
      if !(lsize.is_finite() && offset.is_finite()) {
        return false;
      }
      // `0 1 nlogs {…} for`, eight minor marks a decade
      let Some(n) = ps_for_count(0.0, nlogs).filter(|n| drawable_count(*n as f64 * 9.0)) else {
        return false;
      };
      for i in 0..n {
        let k = i as f64;
        if k >= offset {
          major.push((k - offset) * lsize);
        }
        for j in 2..=9 {
          let xx = f64::from(j).log10() + k;
          if xx >= offset {
            minor.push((xx - offset) * lsize);
          }
        }
      }
    }
    self.line_width(f, &[(0.0, 0.0), (rr, 0.0)], None, width);
    for (marks, length, w) in [(major, 1.2, width), (minor, 0.8, width * 0.6)] {
      for v in marks.into_iter().filter(|v| *v >= 0.0 && *v <= rr) {
        self.line_width(f, &[(v, 0.0), (v, hashsize * length)], None, w);
      }
    }
    true
  }

  /// Evaluate one v1 procedure on its `\special` arguments; `false` for an
  /// unknown procedure or a malformed argument list.
  fn draw(&mut self, procedure: &str, args: &[String]) -> bool {
    let n = |i: usize| -> Option<f64> { args.get(i).and_then(|a| ps_value(a)) };
    let nums = |count: usize| -> Option<Vec<f64>> {
      if args.len() < count {
        return None;
      }
      (0..count).map(n).collect()
    };
    let color = |i: usize| -> String { axo_color(args.get(i).map(String::as_str).unwrap_or("")) };
    let ink = self.pen.color.clone();
    match procedure {
      "line" | "dashline" | "arrowline" | "longarrow" | "dasharrowline" => {
        let dashed = procedure.starts_with("dash");
        let Some(v) = nums(if dashed { 5 } else { 4 }) else {
          return false;
        };
        let (p1, p2) = (self.at(v[0], v[1]), self.at(v[2], v[3]));
        let (f, dr) = self.segment_frame(p1, p2);
        match procedure {
          "line" => self.line(self.base, &[p1, p2], None),
          "dashline" => self.line(self.base, &[p1, p2], Some(fit_dash(dr, v[4]))),
          "arrowline" => {
            self.line(self.base, &[p1, p2], None);
            self.arrow_right(f.translate(dr / 2.0, 0.0));
          },
          "longarrow" => {
            let w = self.pen.width;
            self.line(f, &[(0.0, 0.0), (dr - w * 3.0, 0.0)], None);
            self.arrow_right(f.translate(dr - w * 1.5, 0.0));
          },
          _ => {
            let half = dr / 2.0;
            let dash = Some(fit_dash(half, v[4]));
            self.line(f, &[(0.0, 0.0), (half, 0.0)], dash);
            let g = f.translate(half, 0.0);
            self.line(g, &[(0.0, 0.0), (half, 0.0)], dash);
            self.arrow_right(g);
          },
        }
      },
      "gluon" | "photon" | "zigzag" => {
        let Some(v) = nums(6) else { return false };
        if !drawable_count(v[5] * 2.0) {
          return false;
        }
        let (p1, p2) = (self.at(v[0], v[1]), self.at(v[2], v[3]));
        if procedure == "gluon" {
          self.gluon(p1, p2, v[4], v[5]);
        } else {
          self.photon(p1, p2, v[4], v[5], procedure == "zigzag");
        }
      },
      "photonarc" | "gluearc" => {
        let Some(v) = nums(7) else { return false };
        if !drawable_count(v[6] * 2.0) {
          return false;
        }
        let c = self.at(v[0], v[1]);
        if procedure == "photonarc" {
          self.photon_arc(c, v[2], v[3], v[4], v[5], v[6]);
        } else {
          self.gluon_arc(c, v[2], v[3], v[4], v[5], v[6]);
        }
      },
      "carc" | "dashcarc" | "arrowarc" | "longarrowarc" | "dasharrowarc" | "arrowarcn"
      | "longarrowarcn" | "dasharrowarcn" => {
        let dashed = procedure.starts_with("dash");
        let Some(v) = nums(if dashed { 6 } else { 5 }) else {
          return false;
        };
        let c = self.at(v[0], v[1]);
        let f = self.base.translate(c.0, c.1);
        let (radius, a1, a2) = (v[2], v[3], v[4]);
        // `/dashcarc`: the arc length, its sweep raised by 360 once.
        let dash_for = |from: f64, to: f64| -> Option<f64> {
          let sweep = if to - from < 0.0 {
            to - from + 360.0
          } else {
            to - from
          };
          Some(fit_dash(sweep * PI / 180.0 * radius, v[5]))
        };
        // `540 width 1 add mul 3.14159 div radius div`: the angle the
        // arrowhead of a `\LongArrowArc` takes off the arc.
        let arcmid = 540.0 * (self.pen.width + 1.0) / PI / radius;
        match procedure {
          "carc" => self.arc(f, radius, a1, a2, None),
          "dashcarc" => self.arc(f, radius, a1, a2, dash_for(a1, a2)),
          "arrowarc" => {
            self.arc(f, radius, a1, a2, None);
            let a2 = if a1 > a2 { a2 + 360.0 } else { a2 };
            self.arrow_vertical(f.rotate((a1 + a2) / 2.0).translate(radius, 0.0), true);
          },
          "longarrowarc" => {
            let a2 = if a1 > a2 { a2 + 360.0 } else { a2 };
            self.arc(f, radius, a1, a2 - arcmid, None);
            self.arrow_vertical(f.rotate(a2 - arcmid / 2.0).translate(radius, 0.0), true);
          },
          "dasharrowarc" => {
            let a2 = if a2 < a1 { a2 + 360.0 } else { a2 };
            let mid = (a1 + a2) / 2.0;
            self.arc(f, radius, a1, mid, dash_for(a1, mid));
            self.arc(f, radius, mid, a2, dash_for(mid, a2));
            self.arrow_vertical(f.rotate(mid).translate(radius, 0.0), true);
          },
          "arrowarcn" => {
            self.arcn(f, radius, a1, a2, None);
            let a1 = if a1 < a2 { a1 + 360.0 } else { a1 };
            self.arrow_vertical(f.rotate((a1 + a2) / 2.0).translate(radius, 0.0), false);
          },
          "longarrowarcn" => {
            let a1 = if a1 < a2 { a1 + 360.0 } else { a1 };
            self.arcn(f, radius, a1, a2 + arcmid, None);
            self.arrow_vertical(f.rotate(a2 + arcmid / 2.0).translate(radius, 0.0), false);
          },
          _ => {
            let a1 = if a1 < a2 { a1 + 360.0 } else { a1 };
            let mid = (a1 + a2) / 2.0;
            self.arc(f, radius, mid, a1, dash_for(mid, a1));
            self.arc(f, radius, a2, mid, dash_for(a2, mid));
            self.arrow_vertical(f.rotate(mid).translate(radius, 0.0), false);
          },
        }
      },
      "vertex" | "bcirc" | "gcirc" | "ccirc" => {
        let Some(v) = nums(3) else { return false };
        let c = self.at(v[0], v[1]);
        let f = self.base.translate(c.0, c.1);
        match procedure {
          "vertex" => self.circle(f, v[2], Some(&ink), None),
          "bcirc" => self.circle(f, v[2], Some(WHITE), Some(&ink)),
          "gcirc" => {
            let Some(g) = n(3) else { return false };
            self.circle(f, v[2], Some(&gray(g)), Some(BLACK));
          },
          _ => self.circle(f, v[2], Some(&color(4)), Some(&color(3))),
        }
      },
      "box" | "bbox" | "gbox" | "cbox" | "boxc" | "bboxc" | "gboxc" | "cboxc" => {
        let Some(v) = nums(4) else { return false };
        let (p1, p2) = if procedure.ends_with("boxc") {
          // `\Boxc(x,y)(w,h)`: centred on (x,y).
          let (hw, hh) = (v[2] / 2.0, v[3] / 2.0);
          (self.at(v[0] - hw, v[1] - hh), self.at(v[0] + hw, v[1] + hh))
        } else {
          (self.at(v[0], v[1]), self.at(v[2], v[3]))
        };
        match procedure.trim_end_matches('c') {
          "box" => self.rect(p1, p2, None, Some(&ink)),
          "bbox" => self.rect(p1, p2, Some(WHITE), Some(&ink)),
          "gbox" => {
            let Some(g) = n(4) else { return false };
            self.rect(p1, p2, Some(&gray(g)), Some(BLACK));
          },
          _ => self.rect(p1, p2, Some(&color(5)), Some(&color(4))),
        }
      },
      "triangle" | "btriangle" | "gtriangle" | "ctriangle" => {
        let Some(v) = nums(6) else { return false };
        let points = [
          self.at(v[0], v[1]),
          self.at(v[2], v[3]),
          self.at(v[4], v[5]),
        ];
        let base = self.base;
        match procedure {
          "triangle" => self.polygon(base, &points, None, Some(&ink)),
          "btriangle" => self.polygon(base, &points, Some(WHITE), Some(&ink)),
          "gtriangle" => {
            let Some(g) = n(6) else { return false };
            self.polygon(base, &points, Some(&gray(g)), Some(BLACK));
          },
          _ => self.polygon(base, &points, Some(&color(7)), Some(&color(6))),
        }
      },
      "oval" | "goval" | "coval" => {
        let Some(v) = nums(5) else { return false };
        let c = self.at(v[0], v[1]);
        match procedure {
          "oval" => self.ellipse(c, v[2], v[3], v[4], None, Some(&ink)),
          "goval" => {
            let Some(g) = n(5) else { return false };
            self.ellipse(c, v[2], v[3], v[4], Some(&gray(g)), Some(BLACK));
          },
          _ => self.ellipse(c, v[2], v[3], v[4], Some(&color(6)), Some(&color(5))),
        }
      },
      "curve" | "dashcurve" => {
        // `\Curve{(x1,y1)(x2,y2)…}` (v1 `\kromme` reads the pairs): the
        // numbers in order, the dash size first for `\DashCurve`.
        let joined = args.join(",");
        let Some(v): Option<Vec<f64>> = joined
          .split([',', '(', ')'])
          .map(str::trim)
          .filter(|s| !s.is_empty())
          .map(ps_value)
          .collect()
        else {
          return false;
        };
        let (dash, coords) = if procedure == "dashcurve" {
          match v.split_first() {
            Some((d, rest)) => (Some(*d), rest),
            None => return false,
          }
        } else {
          (None, &v[..])
        };
        let points: Vec<Pt> = coords
          .as_chunks::<2>()
          .0
          .iter()
          .map(|[x, y]| self.at(*x, *y))
          .collect();
        if points.len() == 2 {
          self.line(self.base, &points, None);
        }
        let segments = Drawing::smooth_curve(&points);
        match dash {
          // `/centerdash`: `[ddsize]` with `ddsize` fitting the path length
          // in whole periods. The curve is flattened into one polyline so
          // the pattern runs continuously along it.
          Some(dsize) => {
            let mut flat = Vec::new();
            for seg in &segments {
              for k in 0..=16 {
                let t = f64::from(k) / 16.0;
                let u = 1.0 - t;
                let b = |i: usize| -> f64 {
                  let (p0, p1, p2, p3) = (seg[0], seg[1], seg[2], seg[3]);
                  let pick = |p: Pt| if i == 0 { p.0 } else { p.1 };
                  u * u * u * pick(p0)
                    + 3.0 * u * u * t * pick(p1)
                    + 3.0 * u * t * t * pick(p2)
                    + t * t * t * pick(p3)
                };
                if k > 0 || flat.is_empty() {
                  flat.push((b(0), b(1)));
                }
              }
            }
            let length: f64 = flat
              .windows(2)
              .map(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt())
              .sum();
            // v1 `cvi`s the period count; one shorter than a period would
            // divide by zero (a dvips error): one period.
            let periods = (length / dsize / 2.0).trunc().max(1.0);
            let ddsize = length / (periods * 2.0);
            if flat.len() > 1 {
              self.line(self.base, &flat, Some(ddsize));
            }
          },
          None => {
            for seg in segments {
              self.curve(self.base, seg);
            }
          },
        }
      },
      "logaxis" | "linaxis" => {
        let linear = procedure == "linaxis";
        let Some(v) = nums(if linear { 9 } else { 8 }) else {
          return false;
        };
        let (p1, p2) = (self.at(v[0], v[1]), self.at(v[2], v[3]));
        if linear {
          return self.axis(p1, p2, v[4], v[5], v[6], v[7], v[8], true);
        }
        return self.axis(p1, p2, v[4], 0.0, v[5], v[6], v[7], false);
      },
      "btext" | "gtext" | "ctext" | "b2text" | "g2text" | "c2text" => {
        // `x,y[,gray | ,color,background],textwidth`: the box `/btext` and
        // `/b2text` fit around the text — `fsize` wider than it, `4/3·fsize`
        // (two lines: `7/3·fsize`) high; the text is set by `\lx@axo@labels`.
        let Some(hsize) = args.last().and_then(|a| ps_value(a)) else {
          return false;
        };
        let Some(v) = nums(2) else { return false };
        let fsize = self.pen.fsize;
        let half_w = (hsize + fsize) / 2.0;
        let half_h = if procedure.contains('2') {
          fsize * 7.0 / 6.0
        } else {
          fsize * 2.0 / 3.0
        };
        let (p1, p2) = (
          self.at(v[0] - half_w, v[1] - half_h),
          self.at(v[0] + half_w, v[1] + half_h),
        );
        match procedure.chars().next() {
          Some('b') => self.rect(p1, p2, Some(WHITE), Some(&ink)),
          Some('g') => {
            let Some(g) = n(2) else { return false };
            self.rect(p1, p2, Some(&gray(g)), Some(BLACK));
          },
          _ => self.rect(p1, p2, Some(&color(3)), Some(&color(2))),
        }
      },
      _ => return false,
    }
    true
  }

  fn into_stored(self) -> Stored {
    Stored::VecDequeStored(
      self
        .elements
        .into_iter()
        .map(Stored::HashString)
        .collect::<VecDeque<_>>(),
    )
  }
}

/// A point of the graphics (user, scaled) space as `\put` coordinates: v1's
/// `\put(\axoxoff,\axoyoff)` origin, in `\unitlength`s, plus the point in
/// points.
fn graphics_put(x: f64, y: f64) -> Result<(f64, f64)> {
  let pen = Pen::current()?;
  let unit = unitlength_sp()? as f64 / UNITY_F64;
  let unit = if unit == 0.0 { 1.0 } else { unit };
  let to_units = |v: f64, offset: f64| (v + offset) * pen.scale / unit;
  Ok((
    axo_number("\\axoxoff")? + to_units(x, pen.offset.0),
    axo_number("\\axoyoff")? + to_units(y, pen.offset.1),
  ))
}

/// `\put(X,Y){\makebox(0,0)[pos]{<text at size>}}`, rotated by `angle`: a
/// PostScript-font text of v1 (`/ptext`, `/btext`) set in TeX at its scaled
/// size, as axodraw2 does (axodraw2.sty:2885-2900 `\PText`).
fn ps_label(x: f64, y: f64, angle: &str, pos: &str, text: Tokens) -> Result<Tokens> {
  let (px_, py_) = graphics_put(x, y)?;
  let pen = Pen::current()?;
  let size = pen.fsize * pen.scale;
  let mut body = mouth::tokenize_internal(TeXString::assembled(format!(
    "\\makebox(0,0)[{pos}]{{\\fontsize{{{size}}}{{{size}}}\\selectfont "
  )))
  .unlist();
  body.extend(text.unlist());
  body.push(T_END!());
  let angle = angle.trim();
  let mut out =
    mouth::tokenize_internal(TeXString::assembled(format!("\\put({px_},{py_}){{"))).unlist();
  if angle.is_empty() || angle.parse::<f64>().ok() == Some(0.0) {
    out.extend(body);
  } else {
    out.extend(
      mouth::tokenize_internal(TeXString::assembled(format!("\\rotatebox{{{angle}}}{{"))).unlist(),
    );
    out.extend(body);
    out.push(T_END!());
  }
  out.push(T_END!());
  Ok(Tokens::new(out))
}

fn arg_tokens(args: &[ArgWrap], i: usize) -> Tokens {
  args
    .get(i)
    .and_then(|a| a.revert().ok())
    .unwrap_or_default()
}

fn arg_number(args: &[ArgWrap], i: usize) -> Result<f64> {
  Ok(
    do_expand(arg_tokens(args, i))?
      .to_string()
      .trim()
      .parse()
      .unwrap_or(0.0),
  )
}

LoadDefinitions!({
  // axodraw.sty v1 `\input{epsf.sty}`: papers rely on its `\epsfbox`.
  RequirePackage!("epsf");
  // The colordvi-aware v1 (51 of the 63 copies papers ship in a 64k-zip
  // sample, hep-ph0101235) `\openin`s colordvi.sty and, as TeX Live has it,
  // `\input`s it: its color names for `\SetColor`/`\CCirc`/… and its
  // `\Red{…}`, `\textBlack` text commands (1301.5498 `\Black{$Z$}`;
  // 0709.2086, 1712.05312 `\SetColor{Purple}` with no color package). The
  // union of the variants loads it too. Its `\background{White}\textBlack`
  // that follow are the page defaults, not repeated (a raw copy's
  // `\background{White}` marked every element `backgroundcolor="#FFFFFF"`).
  // colordvi defines its own `\DefineNamedColor`, which in TeX is a
  // `\special`-level name only; a color/xcolor already loaded keeps its own
  // (`\DefineNamedColor{named}{MyBlue}…` then `\color[named]{MyBlue}`).
  // Guard: `perfect_kernel_batch64w::axodraw_v1_keeps_color_definenamedcolor`.
  let color_package = lookup_definition(&T_CS!("\\DefineNamedColor"))?.is_some();
  if color_package {
    Let!("\\lx@axo@DefineNamedColor", "\\DefineNamedColor");
  }
  RequirePackage!("colordvi");
  if color_package {
    Let!("\\DefineNamedColor", "\\lx@axo@DefineNamedColor");
  }

  // `\lx@axo@draw{procedure}{arguments}` stands for v1's
  // `\special{"arguments … procedure showpage}`: the arguments are the
  // comma-separated values the `\special` passes (its numbers already
  // expanded, as `\special` expands them), the state macros are read when
  // it is digested. Its elements are box-free: it measures nothing, like
  // the `{picture}` drawing objects.
  DefConstructor!("\\lx@axo@draw{}{}",
  sub[document, _args, props] {
    if let Some(Stored::VecDequeStored(elements)) = props.get("elements") {
      for element in elements {
        if let Stored::HashString(attrs) = element {
          let mut attrs = attrs.clone();
          let qname = attrs.remove("_qname").unwrap_or_default();
          document.insert_element(&qname, Vec::new(), Some(attrs))?;
        }
      }
    }
  },
  sizer => 0,
  // No reversion: like the `\special` it stands for, it has no TeX source to
  // show, so a math `tex` attribute around an inline diagram keeps the
  // `\put` alone, as without the binding (not the PostScript arguments).
  // Guard: `perfect_kernel_batch64w::axodraw_v1_diagram_in_math_tex_attribute`.
  reversion => "",
  properties => sub[args] {
    let procedure = args[0].as_ref().map(|d| d.to_string()).unwrap_or_default();
    let procedure = procedure.trim();
    let spec = args[1].as_ref().map(|d| d.to_string()).unwrap_or_default();
    let values: Vec<String> = spec.split(',').map(|s| s.trim().to_string()).collect();
    let mut drawing = Drawing::new(Pen::current()?);
    if !drawing.draw(procedure, &values) {
      Warn!("unexpected", "axodraw",
        format!("Cannot draw axodraw '{procedure}' with arguments '{}'", spec.trim()));
    } else if drawing.degenerate {
      Warn!("unexpected", "axodraw",
        format!("Degenerate axodraw '{procedure}' with arguments '{}': part of it is not drawn", spec.trim()));
    }
    Ok(stored_map!("elements" => drawing.into_stored()))
  });

  // `\lx@axo@pstext{x}{y}{angle}{pos}{text}`: `\PText`'s PostScript-font
  // text at a graphics point.
  DefMacro!("\\lx@axo@pstext{}{}{}{}{}", sub[args] {
    let (x, y) = (arg_number(&args, 0)?, arg_number(&args, 1)?);
    let angle = do_expand(arg_tokens(&args, 2))?.to_string();
    let pos = arg_tokens(&args, 3).to_string();
    ps_label(x, y, &angle, &pos, arg_tokens(&args, 4))
  });
  // `\lx@axo@ifvalue{\Set…}{value}{assignment}`: the assignment when the
  // value is a number PostScript can use; else a warning, and the pen keeps
  // its value (dvips would fail the page on it: `\SetWidth{1 0 div}`).
  // Guard: `perfect_kernel_batch64w::axodraw_v1_unbounded_arguments`.
  DefMacro!("\\lx@axo@ifvalue{}{}{}", sub[args] {
    let value = do_expand(arg_tokens(&args, 1))?.to_string();
    if ps_value(&value).is_some() {
      Ok(arg_tokens(&args, 2))
    } else {
      Warn!("unexpected", "axodraw", format!("Cannot read {} value '{}' as a number; the setting is unchanged",
        arg_tokens(&args, 0), value.trim()));
      Ok(Tokens!())
    }
  });

  // `\lx@axo@labels{x}{y}{line1}{line2}`: the text of `\BText`, `\GText`,
  // `\CText` (one line, centred on (x,y)) and of `\B2Text`, … (two lines,
  // centred half a text size above and below it; v1 `/b2text` sets the
  // baselines at `fsize/6` and `-5fsize/6`).
  DefMacro!("\\lx@axo@labels{}{}{}{}", sub[args] {
    let (x, y) = (arg_number(&args, 0)?, arg_number(&args, 1)?);
    let (first, second) = (arg_tokens(&args, 2), arg_tokens(&args, 3));
    if second.is_empty() {
      ps_label(x, y, "", "", first)
    } else {
      let fsize = Pen::current()?.fsize;
      let mut out = ps_label(x, y + fsize / 2.0, "", "", first)?.unlist();
      out.extend(ps_label(x, y - fsize / 2.0, "", "", second)?.unlist());
      Ok(Tokens::new(out))
    }
  });

  // The TeX layer of axodraw.sty v1, its `\special`s replaced by
  // `\lx@axo@draw`. The state macros (with v1's trailing spaces, PostScript
  // separators) and `\Set*` are verbatim; `\SetColor`/`\IfColor` are the
  // colordvi-aware variant's (graphics color only — text keeps its own).
  // `\GluonArc` is axodraw2's name for `\GlueArc` (axodraw2.sty:980).
  RawTeX!(
    r"\def\axowidth{0.5 }\def\axoscale{1.0 }\def\axoxoff{0 }\def\axoyoff{0 }
\def\axoxo{0 }\def\axoyo{0 }\def\firstcall{1}\def\axocolor{ }
\def\SetWidth#1{\lx@axo@ifvalue{\SetWidth}{#1}{\def\axowidth{#1 }}}
\def\SetScale#1{\lx@axo@ifvalue{\SetScale}{#1}{\def\axoscale{#1 }}}
\def\SetOffset(#1,#2){\def\axoxoff{#1 }\def\axoyoff{#2 }}
\def\SetScaledOffset(#1,#2){\def\axoxo{#1 }\def\axoyo{#2 }}
\def\SetColor#1{\def\axocolor{#1 }}
\def\IfColor#1#2{#1}
\def\pfont{Times-Roman }\def\fsize{10 }
\def\SetPFont#1#2{\def\pfont{#1 }\def\fsize{#2 }}
\def\lx@axo@put{\put(\axoxoff,\axoyoff)}
\def\Gluon(#1,#2)(#3,#4)#5#6{\lx@axo@put{\lx@axo@draw{gluon}{#1,#2,#3,#4,#5,#6}}}
\def\Photon(#1,#2)(#3,#4)#5#6{\lx@axo@put{\lx@axo@draw{photon}{#1,#2,#3,#4,#5,#6}}}
\def\ZigZag(#1,#2)(#3,#4)#5#6{\lx@axo@put{\lx@axo@draw{zigzag}{#1,#2,#3,#4,#5,#6}}}
\def\PhotonArc(#1,#2)(#3,#4,#5)#6#7{\lx@axo@put{\lx@axo@draw{photonarc}{#1,#2,#3,#4,#5,#6,#7}}}
\def\GlueArc(#1,#2)(#3,#4,#5)#6#7{\lx@axo@put{\lx@axo@draw{gluearc}{#1,#2,#3,#4,#5,#6,#7}}}
\let\GluonArc\GlueArc
\def\ArrowArc(#1,#2)(#3,#4,#5){\lx@axo@put{\lx@axo@draw{arrowarc}{#1,#2,#3,#4,#5}}}
\def\LongArrowArc(#1,#2)(#3,#4,#5){\lx@axo@put{\lx@axo@draw{longarrowarc}{#1,#2,#3,#4,#5}}}
\def\DashArrowArc(#1,#2)(#3,#4,#5)#6{\lx@axo@put{\lx@axo@draw{dasharrowarc}{#1,#2,#3,#4,#5,#6}}}
\def\ArrowArcn(#1,#2)(#3,#4,#5){\lx@axo@put{\lx@axo@draw{arrowarcn}{#1,#2,#3,#4,#5}}}
\def\LongArrowArcn(#1,#2)(#3,#4,#5){\lx@axo@put{\lx@axo@draw{longarrowarcn}{#1,#2,#3,#4,#5}}}
\def\DashArrowArcn(#1,#2)(#3,#4,#5)#6{\lx@axo@put{\lx@axo@draw{dasharrowarcn}{#1,#2,#3,#4,#5,#6}}}
\def\ArrowLine(#1,#2)(#3,#4){\lx@axo@put{\lx@axo@draw{arrowline}{#1,#2,#3,#4}}}
\def\LongArrow(#1,#2)(#3,#4){\lx@axo@put{\lx@axo@draw{longarrow}{#1,#2,#3,#4}}}
\def\DashArrowLine(#1,#2)(#3,#4)#5{\lx@axo@put{\lx@axo@draw{dasharrowline}{#1,#2,#3,#4,#5}}}
\def\Line(#1,#2)(#3,#4){\lx@axo@put{\lx@axo@draw{line}{#1,#2,#3,#4}}}
\def\DashLine(#1,#2)(#3,#4)#5{\lx@axo@put{\lx@axo@draw{dashline}{#1,#2,#3,#4,#5}}}
\def\CArc(#1,#2)(#3,#4,#5){\lx@axo@put{\lx@axo@draw{carc}{#1,#2,#3,#4,#5}}}
\def\DashCArc(#1,#2)(#3,#4,#5)#6{\lx@axo@put{\lx@axo@draw{dashcarc}{#1,#2,#3,#4,#5,#6}}}
\def\Vertex(#1,#2)#3{\lx@axo@put{\lx@axo@draw{vertex}{#1,#2,#3}}}
\def\BCirc(#1,#2)#3{\lx@axo@put{\lx@axo@draw{bcirc}{#1,#2,#3}}}
\def\GCirc(#1,#2)#3#4{\lx@axo@put{\lx@axo@draw{gcirc}{#1,#2,#3,#4}}}
\def\CCirc(#1,#2)#3#4#5{\lx@axo@put{\lx@axo@draw{ccirc}{#1,#2,#3,#4,#5}}}
\def\EBox(#1,#2)(#3,#4){\lx@axo@put{\lx@axo@draw{box}{#1,#2,#3,#4}}}
\def\BBox(#1,#2)(#3,#4){\lx@axo@put{\lx@axo@draw{bbox}{#1,#2,#3,#4}}}
\def\GBox(#1,#2)(#3,#4)#5{\lx@axo@put{\lx@axo@draw{gbox}{#1,#2,#3,#4,#5}}}
\def\CBox(#1,#2)(#3,#4)#5#6{\lx@axo@put{\lx@axo@draw{cbox}{#1,#2,#3,#4,#5,#6}}}
\def\Boxc(#1,#2)(#3,#4){\lx@axo@put{\lx@axo@draw{boxc}{#1,#2,#3,#4}}}
\def\BBoxc(#1,#2)(#3,#4){\lx@axo@put{\lx@axo@draw{bboxc}{#1,#2,#3,#4}}}
\def\GBoxc(#1,#2)(#3,#4)#5{\lx@axo@put{\lx@axo@draw{gboxc}{#1,#2,#3,#4,#5}}}
\def\CBoxc(#1,#2)(#3,#4)#5#6{\lx@axo@put{\lx@axo@draw{cboxc}{#1,#2,#3,#4,#5,#6}}}
\def\ETri(#1,#2)(#3,#4)(#5,#6){\lx@axo@put{\lx@axo@draw{triangle}{#1,#2,#3,#4,#5,#6}}}
\def\BTri(#1,#2)(#3,#4)(#5,#6){\lx@axo@put{\lx@axo@draw{btriangle}{#1,#2,#3,#4,#5,#6}}}
\def\GTri(#1,#2)(#3,#4)(#5,#6)#7{\lx@axo@put{\lx@axo@draw{gtriangle}{#1,#2,#3,#4,#5,#6,#7}}}
\def\CTri(#1,#2)(#3,#4)(#5,#6)#7#8{\lx@axo@put{\lx@axo@draw{ctriangle}{#1,#2,#3,#4,#5,#6,#7,#8}}}
\def\Oval(#1,#2)(#3,#4)(#5){\lx@axo@put{\lx@axo@draw{oval}{#1,#2,#3,#4,#5}}}
\def\GOval(#1,#2)(#3,#4)(#5)#6{\lx@axo@put{\lx@axo@draw{goval}{#1,#2,#3,#4,#5,#6}}}
\def\COval(#1,#2)(#3,#4)(#5)#6#7{\lx@axo@put{\lx@axo@draw{coval}{#1,#2,#3,#4,#5,#6,#7}}}
\def\Curve#1{\lx@axo@put{\lx@axo@draw{curve}{#1}}}
\def\DashCurve#1#2{\lx@axo@put{\lx@axo@draw{dashcurve}{#2,#1}}}
\def\LogAxis(#1,#2)(#3,#4)(#5,#6,#7,#8){\lx@axo@put{\lx@axo@draw{logaxis}{#1,#2,#3,#4,#5,#6,#7,#8}}}
\def\LinAxis(#1,#2)(#3,#4)(#5,#6,#7,#8,#9){\lx@axo@put{\lx@axo@draw{linaxis}{#1,#2,#3,#4,#5,#6,#7,#8,#9}}}"
  );
  // v1 `\Text`/`\rText`: `\raise\dimen1\hbox to\z@{\kern\dimen0
  // \makebox(0,0)[#3]{#4}\hss}` with `\dimen0 = \axoxoff\unitlength + #1
  // \unitlength` — exactly a `\put` from the picture origin, in
  // `\unitlength`s and not scaled by `\SetScale`. `\rText`'s angle letter is
  // v1's `\rotl`/`\rotr`/`\rotu` (rotate.tex) of the text inside the box.
  // The PostScript texts (`\PText`, `\BText`, …) are set in TeX at their
  // graphics point and scaled size.
  RawTeX!(
    r"\def\lx@axo@at(#1,#2){\@tempdima\axoxoff\p@\advance\@tempdima#1\p@
\@tempdimb\axoyoff\p@\advance\@tempdimb#2\p@
\edef\lx@axo@xy{(\strip@pt\@tempdima,\strip@pt\@tempdimb)}\expandafter\put\lx@axo@xy}
\def\Text(#1,#2)[#3]#4{\lx@axo@at(#1,#2){\makebox(0,0)[#3]{#4}}\ignorespaces}
\def\rText(#1,#2)[#3][#4]#5{\lx@axo@angleof{#4}\lx@axo@at(#1,#2){\makebox(0,0)[#3]{%
\ifnum\lx@axo@angle=\z@#5\else\rotatebox{\lx@axo@angle}{#5}\fi}}\ignorespaces}
\def\lx@axo@angleof#1{\def\lx@axo@tmp{#1}\def\lx@axo@angle{0}%
\def\lx@axo@tmpb{l}\ifx\lx@axo@tmp\lx@axo@tmpb\def\lx@axo@angle{90}\fi
\def\lx@axo@tmpb{r}\ifx\lx@axo@tmp\lx@axo@tmpb\def\lx@axo@angle{-90}\fi
\def\lx@axo@tmpb{u}\ifx\lx@axo@tmp\lx@axo@tmpb\def\lx@axo@angle{180}\fi}
\def\PText(#1,#2)(#3)[#4]#5{\lx@axo@pstext{#1}{#2}{#3}{#4}{#5}\ignorespaces}
\newdimen\lx@axo@wd
\def\lx@axo@measure#1#2{\setbox\@tempboxa\hbox{\fontsize{\fsize}{\fsize}\selectfont#1}\lx@axo@wd\wd\@tempboxa
\setbox\@tempboxa\hbox{\fontsize{\fsize}{\fsize}\selectfont#2}%
\ifdim\wd\@tempboxa>\lx@axo@wd\lx@axo@wd\wd\@tempboxa\fi}
\def\lx@axo@textbox#1#2#3#4#5#6{\lx@axo@measure{#5}{#6}%
\lx@axo@put{\lx@axo@draw{#1}{#2,#3#4,\strip@pt\lx@axo@wd}}\lx@axo@labels{#2}{#3}{#5}{#6}}
\def\BText(#1,#2)#3{\lx@axo@textbox{btext}{#1}{#2}{}{#3}{}}
\def\GText(#1,#2)#3#4{\lx@axo@textbox{gtext}{#1}{#2}{,#3}{#4}{}}
\def\CText(#1,#2)#3#4#5{\lx@axo@textbox{ctext}{#1}{#2}{,#3,#4}{#5}{}}
\def\B2Text(#1,#2)#3#4{\lx@axo@textbox{b2text}{#1}{#2}{}{#3}{#4}}
\def\G2Text(#1,#2)#3#4#5{\lx@axo@textbox{g2text}{#1}{#2}{,#3}{#4}{#5}}
\def\C2Text(#1,#2)#3#4#5#6{\lx@axo@textbox{c2text}{#1}{#2}{,#3,#4}{#5}{#6}}"
  );
});
