//! pstricks_support.sty — PSTricks drawing support
//! Perl: pstricks_support.sty.ltxml — 1057 lines
//! The parameter types (PSCoord, PSDimension, PSAngle, Arrows), the
//! `{pspicture}` environment, the `DefPSConstructor` objects that draw into
//! an `ltx:picture` (`\psline` → `ltx:line`, `\psframe` → `ltx:rect`, …),
//! the `\rput` family and the helpers around them. Loaded by both
//! pstricks.sty (pstricks_sty.rs) and plain-TeX pstricks.tex
//! (pstricks_tex.rs), after the raw pstricks load.
//!
//! The graphics parameters are read from where the RAW `\psset` keeps them
//! (pst-xkey.tex:60-63; the key bodies in pstricks.tex and pstricks-*.tex),
//! not from Perl's own `setGraphParams` store (:318-342): this port keeps the
//! real `\psset`, so `\pslinewidth`, `\pslinecolor`, `\pslinestyle`,
//! `\psk@fillstylename`, `\psfillcolor`, `\psk@dash`, … are the state.
use super::{
  color_sty::{color_key, lookup_color_obj},
  xcolor_sty::parse_xcolor,
};
use crate::prelude::*;

/// Perl `Dimension::pxValue`: TeX points → CSS px at the `DPI` value (100 by
/// default), rounded to 2 decimals (mirrors `latex_constructs::px_value`).
fn ps_px(pt: f64) -> f64 {
  let dpi = lookup_value("DPI")
    .and_then(|v| {
      if let Stored::Number(n) = v {
        Some(n.0 as f64)
      } else {
        None
      }
    })
    .unwrap_or(100.0);
  (pt * dpi / 72.27 * 100.0).round() / 100.0
}

/// A rounded number as Perl stringifies it (`39.37`, `0`, `1`).
fn ps_fmt_px(v: f64) -> String {
  if v == v.round() && v.abs() < 1e10 {
    format!("{}", v as i64)
  } else {
    format!("{v}")
  }
}

/// Perl `Dimension::ptValue` of a value in points (2 decimals), as Perl
/// stringifies it: `0.8`, `1`.
fn ps_pt_num(pt: f64) -> String { ps_fmt_px((pt * 100.0).round() / 100.0) }

/// Perl `Dimension::ptValue`: two decimals, integers with one.
fn ps_fmt_pt(v: f64) -> String {
  let v = (v * 100.0).round() / 100.0;
  if v == v.round() {
    format!("{v:.1}pt")
  } else {
    format!("{v}pt")
  }
}

/// 1cm in points, the default `\psunit`/`\psxunit`/`\psyunit`
/// (pstricks_support.sty.ltxml:416-418).
const PS_CM: f64 = 28.452_755_905_511_81;

/// A dimension in points at full scaled-point precision (Perl keeps the sp
/// integer until the final `ptValue`/`pxValue`).
fn dim_pt(d: Dimension) -> f64 { d.value_of() as f64 / UNITY_F64 }

/// A dimension register's value in points; `default` when it is not one.
fn ps_register_pt(name: &str, default: f64) -> Result<f64> {
  Ok(match lookup_register(name, Vec::new())? {
    Some(RegisterValue::Dimension(d)) => dim_pt(d),
    _ => default,
  })
}

/// `(\psxunit, \psyunit)` in points (1cm each by default, :416-418).
fn ps_units() -> Result<(f64, f64)> {
  Ok((
    ps_register_pt("\\psxunit", PS_CM)?,
    ps_register_pt("\\psyunit", PS_CM)?,
  ))
}

/// Perl `Gullet::readFloat`, which is `undef` when neither digits nor an
/// integer follow (the Rust `read_float` answers 0 there).
fn read_ps_float() -> Result<Option<f64>> {
  let negative = read_optional_signs()?;
  let Some(token) = read_x_token(None, false, None)? else {
    return Ok(None);
  };
  let numeric = token.with_str(|s| {
    matches!(
      s,
      "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "." | "`" | "'" | "\""
    )
  }) || lookup_definition(&token)?.is_some_and(|defn| defn.register_type().is_some());
  unread_one(token);
  if !numeric {
    return Ok(None);
  }
  let value = read_float()?.0;
  Ok(Some(if negative { -value } else { value }))
}

/// pstricks `\xdef`s an angle or length argument before it looks at it
/// (pstricks.tex:790-802, SpecialCoor's `\pssetlength`/`\pst@@getangle`):
/// inside a braced argument, its rest is fully expanded here, so a macro that
/// expands to `(1,1)` or `! code` is seen as one.
fn ps_expand_braced_rest() -> Result<()> {
  let rest = take_rest_of_mouth();
  if !rest.is_empty() {
    unread(do_expand(Tokens::new(rest))?);
  }
  Ok(())
}

/// Perl `ReadPSDimension` (pstricks_support.sty.ltxml:25-38): a dimension
/// register, a number with a unit, or a bare number scaled by `scale`
/// (`\psunit` by default). In points.
///
/// In a braced argument, `! <code>` is a PostScript length (pstricks.tex:
/// 1001-1004 `\special@length`), which takes the whole argument: this port
/// cannot run it, so it is zero, and none of it comes back as text (the tail
/// of a plain length does, as from `\special@length`'s assignment `#3
/// #1#2\@psunit`; OXIDIZED_DESIGN #317).
fn read_ps_dimension(scale: Option<f64>) -> Result<f64> {
  let scale = match scale {
    Some(s) => s,
    None => ps_register_pt("\\psunit", PS_CM)?,
  };
  if in_braced_read() {
    ps_expand_braced_rest()?;
    skip_spaces()?;
  }
  if in_braced_read() && if_next(T_OTHER!("!"))? {
    let code = Tokens::new(take_rest_of_mouth());
    Warn!(
      "unexpected",
      "!",
      s!("The PostScript length '{code}' is not evaluated; treated as zero.")
    );
    return Ok(0.0);
  }
  let sign = if read_optional_signs()? { -1.0 } else { 1.0 };
  if let Some(value) = read_register_value_coerce(RegisterType::Dimension, true)? {
    let d: Dimension = value.into();
    return Ok(sign * dim_pt(d));
  }
  if let Some(value) = read_ps_float()? {
    return Ok(match read_unit()? {
      Some((num, den)) => sign * value * num as f64 / den as f64,
      None => sign * value * scale,
    });
  }
  Warn!("expected", "<number>", "Missing number, treated as zero.");
  Ok(0.0)
}

/// One coordinate component, read by `read_ps_dimension` from its own
/// tokens. A component wrapped in braces is read inside them (`({36.5},1)`).
fn ps_component_pt(tokens: &[Token], unit: f64) -> Result<f64> {
  let mut tokens: Vec<Token> = tokens
    .iter()
    .filter(|t| t.get_catcode() != Catcode::SPACE)
    .copied()
    .collect();
  if tokens.len() >= 2
    && tokens
      .first()
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
    && tokens
      .last()
      .is_some_and(|t| t.get_catcode() == Catcode::END)
  {
    tokens.remove(0);
    tokens.pop();
  }
  if tokens.is_empty() {
    return Ok(0.0);
  }
  reading_from_mouth(Mouth::default(), move || {
    unread_vec(tokens);
    read_ps_dimension(Some(unit))
  })
}

/// A pstricks coordinate: a point in points, or one this port cannot place.
#[derive(Clone, Copy, Debug)]
enum PsCoord {
  Point(f64, f64),
  /// A node reference `(A)`, `([angle=45]A)`, `(>A)`; PostScript code `(!…)`;
  /// an algebraic expression `(*…)`, `(+…)`: pstricks resolves them in the
  /// PostScript interpreter or from the node positions, neither of which is
  /// modelled here (pst-flags' `\pscircle(!…)`, retopstricks' node polygons).
  Unresolved,
}

/// The position of the first top-level `text` token: outside braces and
/// outside `[…]` (a node reference's options, `([angle=45,nodesep=1]A)`).
fn ps_top_level_position(tokens: &[Token], text: &str) -> Option<usize> {
  let (mut braces, mut brackets) = (0i32, 0i32);
  tokens.iter().position(|t| match t.get_catcode() {
    Catcode::BEGIN => {
      braces += 1;
      false
    },
    Catcode::END => {
      braces -= 1;
      false
    },
    Catcode::SPACE => false,
    _ => t.with_str(|s| {
      match s {
        "[" => brackets += 1,
        "]" => brackets -= 1,
        _ => {},
      }
      braces == 0 && brackets == 0 && s == text
    }),
  })
}

/// One coordinate, as pstricks' `\SpecialCoor` classifies it
/// (pstricks.tex:830-890 `\pst@@CheckCoorType` / `\special@coor`, in force
/// after :806): `(A|B)` takes x from A and y from B (`\mixed@coor`, :930-941);
/// a first token that is a letter, `[`, `>`, `!`, `*` or `+` is a node
/// reference, PostScript or an algebraic expression (unresolved here);
/// `(r;a)` is polar (`\polar@coor`, :943-949: the radius read by
/// `\pssetlength`, a bare number in `\psunit`, the angle by `\pst@@getangle`
/// in the current `\degrees` unit); otherwise `(x,y)` is cartesian, each
/// component by Perl's `ReadPSDimension` in `\psxunit`/`\psyunit`. Perl's
/// `ReadPair` (latex_constructs.pool.ltxml:4839-4863) knows only the
/// cartesian form: on a node reference it warns "Missing number" and reads on
/// past the `)` for a comma.
fn ps_coord_of(tokens: &[Token]) -> Result<PsCoord> {
  if let Some(bar) = ps_top_level_position(tokens, "|") {
    return Ok(
      match (
        ps_coord_of(&tokens[..bar])?,
        ps_coord_of(&tokens[bar + 1..])?,
      ) {
        (PsCoord::Point(x, _), PsCoord::Point(_, y)) => PsCoord::Point(x, y),
        _ => PsCoord::Unresolved,
      },
    );
  }
  let Some(first) = tokens.iter().find(|t| t.get_catcode() != Catcode::SPACE) else {
    return Ok(PsCoord::Unresolved);
  };
  if first.get_catcode() == Catcode::LETTER
    || first.with_str(|s| matches!(s, "[" | ">" | "!" | "*" | "+"))
  {
    return Ok(PsCoord::Unresolved);
  }
  if let Some(semicolon) = ps_top_level_position(tokens, ";") {
    let radius = ps_component_pt(&tokens[..semicolon], ps_register_pt("\\psunit", PS_CM)?)?;
    let angle = ps_tokens_angle(&tokens[semicolon + 1..])?;
    return Ok(match angle {
      Some(angle) => {
        let (sin, cos) = angle.to_radians().sin_cos();
        PsCoord::Point(radius * cos, radius * sin)
      },
      None => PsCoord::Unresolved,
    });
  }
  let Some(comma) = ps_top_level_position(tokens, ",") else {
    return Ok(PsCoord::Unresolved);
  };
  let (ux, uy) = ps_units()?;
  let x = ps_component_pt(&tokens[..comma], ux)?;
  let y = ps_component_pt(&tokens[comma + 1..], uy)?;
  Ok(PsCoord::Point(x, y))
}

/// A polar coordinate's angle, in degrees (`read_ps_angle` over its tokens).
fn ps_tokens_angle(tokens: &[Token]) -> Result<Option<f64>> {
  let tokens = tokens.to_vec();
  let angle = reading_from_mouth(Mouth::default(), move || {
    unread_vec(tokens);
    read_ps_angle()
  })?;
  Ok(match angle {
    ArgWrap::Float(angle) => Some(angle.0),
    _ => None,
  })
}

/// Perl `ReadPSCoord` (:103-105): `(…)` → a `PsCoord`, `None` when no `(`
/// follows. The coordinate is read to its `)` and fully expanded first, as
/// pstricks' `\xdef\pst@tempg{#1}` does (batch 56aq), so a coordinate this
/// port cannot place is consumed instead of derailing the picture.
fn read_ps_coord() -> Result<Option<PsCoord>> {
  skip_spaces()?;
  if !if_next(T_OTHER!("("))? {
    return Ok(None);
  }
  read_token()?; // (
  let Some(inner) = read_until(&Tokens!(T_OTHER!(")")))? else {
    return Ok(None);
  };
  let inner = do_expand(inner)?.unlist();
  Ok(Some(ps_coord_of(&inner)?))
}

/// A `PsCoord` as an argument: a point is a `Pair` in points; an unresolved
/// coordinate is an empty argument, not a pair. `{pspicture}` and `\rput`
/// place that at the origin (`ps_coord_or_origin`, Perl's `ZeroPSCoord`
/// leniency), while a drawing object draws nothing (`ps_arg_coord`): an
/// object at a position it does not have is geometry pdflatex never draws.
fn ps_coord_arg(coord: PsCoord) -> ArgWrap {
  use latexml_core::common::pair::Pair;
  match coord {
    PsCoord::Point(x, y) => ArgWrap::Pair(Pair::new(Float(x), Float(y))),
    PsCoord::Unresolved => ArgWrap::Tokens(Tokens::new(Vec::new())),
  }
}

/// A `PSCoord` argument: `None` when it was not given.
fn ps_arg_coord(arg: Option<&Digested>) -> Option<PsCoord> {
  Some(match arg?.data() {
    DigestedData::RegisterValue(RegisterValue::Pair(p)) => PsCoord::Point(p.x.0, p.y.0),
    _ => PsCoord::Unresolved,
  })
}

/// A placement `PSCoord` argument (`{pspicture}`, `\rput`): `None` when it
/// was not given; an unresolved coordinate stands for the origin.
fn ps_coord_or_origin(arg: Option<&Digested>) -> Option<(f64, f64)> {
  Some(match ps_arg_coord(arg)? {
    PsCoord::Point(x, y) => (x, y),
    PsCoord::Unresolved => (0.0, 0.0),
  })
}

/// `{pspicture}(c0)(c1)` — Perl pstricks_support.sty.ltxml:527-536: with one
/// pair, it is the far corner and the origin is (0,0); width/height are the
/// corner differences (the pairs are already in points, see `read_ps_coord`);
/// the body is translated by the negated origin (the same attributes the
/// LaTeX `{picture}` binding emits).
fn pspicture_properties(
  first: Option<&Digested>,
  second: Option<&Digested>,
) -> Result<SymHashMap<Stored>> {
  let (ux, _) = ps_units()?;
  let a = ps_coord_or_origin(first).unwrap_or((0.0, 0.0));
  let (c0, c1) = match ps_coord_or_origin(second) {
    Some(b) => (a, b),
    None => ((0.0, 0.0), a),
  };
  let (w, h) = (c1.0 - c0.0, c1.1 - c0.1);
  let (ox, oy) = c0;
  let mut map = stored_map!(
    "width"      => Stored::String(pin(ps_fmt_pt(w))),
    "height"     => Stored::String(pin(ps_fmt_pt(h))),
    "unitlength" => Stored::String(pin(ps_fmt_pt(ux)))
  );
  if ox != 0.0 || oy != 0.0 {
    map.insert("origin-x", Stored::String(pin(ps_fmt_pt(ox))));
    map.insert("origin-y", Stored::String(pin(ps_fmt_pt(oy))));
    map.insert(
      "transform",
      Stored::String(pin(format!(
        "translate({},{})",
        ps_fmt_px(ps_px(-ox)),
        ps_fmt_px(ps_px(-oy))
      ))),
    );
  }
  Ok(map)
}

/// `\lx@ps@put(x,y)` — the LaTeX `\put` transform of the `<ltx:g>` it opens.
fn ps_put_properties(coords: Option<&Digested>) -> Result<SymHashMap<Stored>> {
  let (x, y) = ps_coord_or_origin(coords).unwrap_or((0.0, 0.0));
  Ok(stored_map!(
    "transform" => Stored::String(pin(format!("translate({},{})", ps_fmt_px(ps_px(x)), ps_fmt_px(ps_px(y)))))
  ))
}

/// A `PSCoordList` argument: the pairs as its reader wrote them,
/// `(<x>pt,<y>pt)…` (Perl's reversion of a `PairList`), an unresolved one
/// as `(?)`. `None` when any of them is unresolved.
fn ps_coord_list(arg: Option<&Digested>) -> Option<Vec<(f64, f64)>> {
  let Some(arg) = arg else {
    return Some(Vec::new());
  };
  let text = arg.to_string();
  text
    .split(')')
    .filter(|pair| !pair.trim().is_empty())
    .map(|pair| {
      let (x, y) = pair.trim().strip_prefix('(')?.split_once(',')?;
      Some((
        x.trim().trim_end_matches("pt").parse().ok()?,
        y.trim().trim_end_matches("pt").parse().ok()?,
      ))
    })
    .collect()
}

/// A `PSDimension` argument, in points.
fn ps_dimension_arg(arg: Option<&Digested>) -> Option<f64> {
  match arg?.data() {
    DigestedData::RegisterValue(RegisterValue::Dimension(d)) => Some(dim_pt(*d)),
    _ => None,
  }
}

/// A `PSAngle` argument, in degrees.
fn ps_angle_arg(arg: Option<&Digested>) -> Option<f64> { arg?.to_string().trim().parse().ok() }

/// Perl `trunc2` (:425-427) of a `PSAngle`: 2 decimals (a zero angle is
/// still an object, so `0`); nothing when the angle is missing.
fn ps_trunc2(v: Option<f64>) -> Option<String> { v.map(|v| ps_fmt_px((v * 100.0).round() / 100.0)) }

/// `x`/`y` of an object's centre (Perl `properties`, `pxValue`); `false`
/// when the centre is unresolved.
fn ps_set_center(whatsit: &mut Whatsit, n: usize) -> bool {
  let Some(PsCoord::Point(x, y)) = ps_arg_coord(whatsit.get_arg(n)) else {
    return false;
  };
  whatsit.set_property("x", Stored::from(ps_fmt_px(ps_px(x))));
  whatsit.set_property("y", Stored::from(ps_fmt_px(ps_px(y))));
  true
}

/// `r` of a circle, wedge or arc, in px (see the objects' comment); SVG
/// renders no negative radius.
fn ps_set_radius(whatsit: &mut Whatsit, n: usize) {
  if let Some(r) = ps_dimension_arg(whatsit.get_arg(n)) {
    whatsit.set_property("r", Stored::from(ps_fmt_px(ps_px(r.abs()))));
  }
}

/// `angle1`/`angle2` of a wedge or arc (`&trunc2(#n)`); whether both are
/// resolved. An arc at angles it does not have (a node's, PostScript's) is
/// geometry pdflatex never draws, as for an unresolved coordinate
/// (`ps_arg_coord`); Perl draws it from 0°.
fn ps_set_angles(whatsit: &mut Whatsit, n1: usize, n2: usize) -> bool {
  let mut resolved = true;
  for (key, n) in [("angle1", n1), ("angle2", n2)] {
    match ps_trunc2(ps_angle_arg(whatsit.get_arg(n))) {
      Some(angle) => whatsit.set_property(key, Stored::from(angle)),
      None => resolved = false,
    }
  }
  resolved
}

/// `\pscurve`, `\psecurve`, `\psccurve` (Perl :783-796): the points of the
/// `PSCoordList` (argument 3, after the arrows).
fn ps_curve_geometry(whatsit: &mut Whatsit) -> Result<bool> {
  let Some(points) = ps_coord_list(whatsit.get_arg(3)) else {
    return Ok(false);
  };
  whatsit.set_property("points", Stored::from(ps_points_px(&points)));
  let arrows = ps_arrows_arg(whatsit, 2);
  ps_terminators(whatsit, arrows)?;
  Ok(true)
}

/// The `attributes` of the three curves (Perl :786, :791, :796).
const PS_CURVE_ATTRIBUTES: &[&str] = &["linecolor", "linewidth", "dash", "showpoints", "curvature"];

/// `\psdot`, `\psdots` (Perl :807-810, :817-820): the line colour fills
/// the dots; the size is the `tbarsize` for the `|` style, else `dotsize`.
fn ps_dots_style(whatsit: &mut Whatsit) -> Result<()> {
  if let Some(color) = ps_color("\\pslinecolor")? {
    whatsit.set_property("myfill", Stored::from(color));
  }
  let tbar = ps_macro_text("\\psk@dotstyle")? == "|";
  let size = if tbar {
    ps_macro_numbers("\\psk@tbarsize")?.first().copied()
  } else {
    ps_macro_numbers("\\psk@@dotsize")?.first().copied()
  };
  if let Some(size) = size {
    whatsit.set_property("sz", Stored::from(ps_pt_num(size)));
  }
  Ok(())
}

/// The `attributes` of the dots (Perl :806, :816).
const PS_DOTS_ATTRIBUTES: &[&str] = &["dotstyle", "dotscale", "dotangle", "showpoints"];

/// Pairs in points as an SVG `points` list in px (`x,y x,y`).
fn ps_points_px(points: &[(f64, f64)]) -> String {
  points
    .iter()
    .map(|(x, y)| format!("{},{}", ps_fmt_px(ps_px(*x)), ps_fmt_px(ps_px(*y))))
    .collect::<Vec<_>>()
    .join(" ")
}

/// Perl `PairList(ZeroPair, @pairs)` when the list is shorter than `min`
/// (the first point defaults to the origin, :667, :679, :760).
fn ps_with_origin(mut points: Vec<(f64, f64)>, min: usize) -> Vec<(f64, f64)> {
  if points.len() < min {
    points.insert(0, (0.0, 0.0));
  }
  points
}

/// Perl `%angleVals` (:152-154).
fn ps_named_angle(name: &str) -> f64 {
  match name {
    "N" | "U" | "r" => 0.0,
    "W" | "L" | "u" => 90.0,
    "S" | "D" | "l" => 180.0,
    "E" | "R" | "d" => 270.0,
    "ur" | "ru" => 45.0,
    "ul" | "lu" => 135.0,
    "dl" | "ld" => 225.0,
    "dr" | "rd" => 315.0,
    _ => 0.0,
  }
}

/// Perl `ReadPSAngle` + `LaTeXML::PSAngle->new` (:147-215): optional `:` and
/// `*` prefixes, then a direction name or a number in `\degrees` units,
/// scaled to 360. A `*` angle undoes the accumulated `\rput` rotation
/// (`_psActiveSRotation`), which this port does not track: `\rput`'s
/// rotation is dropped with the rest of its placement transform.
///
/// In its braced argument (`{PSAngle}`), the angle is read as pstricks reads
/// it, whole (pstricks.tex:990-999 `\special@angle#1#2)#3\@nil`, SpecialCoor):
/// a coordinate `(x,y)` is the angle of that vector (`\pst@coor exch
/// \tx@Atan`), a node's `(A)` one this port cannot place, as `! <code>` is
/// PostScript it cannot run — both an unresolved angle, whose arc or wedge is
/// not drawn (`ps_set_angles`; pst-eucl.tex:528 `\psarc(0,0){…}{(#2)}{(#4)}`:
/// `\pstMarkAngle`) — and text after a number is `\pst@checknum`'s "Bad
/// number", 0 substituted (:534-563). The argument is expanded first, as
/// pstricks `\xdef`s it (`ps_expand_braced_rest`). Perl's reader takes neither
/// form (angle 0, the rest dropped); none of the argument may come back as
/// text (OXIDIZED_DESIGN #317).
fn read_ps_angle() -> Result<ArgWrap> {
  if in_braced_read() {
    ps_expand_braced_rest()?;
  }
  skip_spaces()?;
  for prefix in [":", "*"] {
    if if_next(T_OTHER!(prefix))? {
      read_token()?;
      skip_spaces()?;
    }
  }
  if in_braced_read() {
    if if_next(T_OTHER!("!"))? {
      take_rest_of_mouth();
      return Ok(ArgWrap::Tokens(Tokens::new(Vec::new())));
    }
    if if_next(T_OTHER!("("))? {
      let coord = read_ps_coord()?;
      take_rest_of_mouth();
      return Ok(match coord {
        Some(PsCoord::Point(x, y)) if x != 0.0 || y != 0.0 => {
          ArgWrap::Float(Float(y.atan2(x).to_degrees().rem_euclid(360.0)))
        },
        // PostScript's `atan` of (0,0) fails; `\tx@Atan` stops it with 0.
        Some(PsCoord::Point(..)) => ArgWrap::Float(Float(0.0)),
        _ => ArgWrap::Tokens(Tokens::new(Vec::new())),
      });
    }
    // No angle at all is the caller's "Missing argument"; its text goes too.
    let angle = read_ps_angle_value()?;
    let rest = take_rest_of_mouth();
    // An undefined control sequence the scan met was reported; TeX discards
    // it (as `read_braced` does).
    if !matches!(angle, ArgWrap::None)
      && let Some(first) = rest
        .iter()
        .find(|t| t.get_catcode() != Catcode::SPACE && !is_error_stub(t))
    {
      Error!(
        "unexpected",
        first.stringify(),
        s!(
          "Bad number: text '{}' after the angle. 0 substituted.",
          Tokens::new(rest.clone())
        )
      );
      return Ok(ArgWrap::Float(Float(0.0)));
    }
    return Ok(angle);
  }
  read_ps_angle_value()
}

/// The direction name or number of a `PSAngle`, after its prefixes.
fn read_ps_angle_value() -> Result<ArgWrap> {
  let mut name = String::new();
  for letter in [
    "N", "W", "S", "E", "U", "L", "D", "R", "u", "d", "l", "r", "u", "d",
  ] {
    if if_next(T_LETTER!(letter))? {
      read_token()?;
      skip_spaces()?;
      name.push_str(letter);
    }
  }
  if !name.is_empty() {
    return Ok(ArgWrap::Float(Float(ps_named_angle(&name))));
  }
  let Some(value) = read_ps_float()? else {
    return Ok(ArgWrap::None);
  };
  let degrees = match lookup_value("\\degrees") {
    Some(Stored::Float(f)) if f.0 != 0.0 => f.0,
    Some(Stored::Number(n)) if n.0 != 0 => n.0 as f64,
    _ => 360.0,
  };
  Ok(ArgWrap::Float(Float(value * 360.0 / degrees)))
}

/// The replacement text of a raw pstricks value macro, trimmed; `None` when
/// it is not a macro.
fn ps_macro_body(cs: &str) -> Result<Option<Tokens>> {
  Ok(match lookup_definition(&T_CS!(cs))? {
    Some(defn) => match defn.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => Some(body.clone()),
      _ => None,
    },
    None => None,
  })
}

/// The value of a raw pstricks value macro (`\pslinestyle`, `\psk@dash`, …)
/// as pstricks reads it, fully expanded (`\csname psls@\pslinestyle
/// \endcsname`), trimmed; empty when it is not a macro. Some keys keep their
/// value unexpanded (pstricks.tex:1162 `\def\pslinestyle{#1}`, :1334
/// `\def\psk@fillstylename{#1}`): egameps.sty:62-63 sets
/// `linestyle=\@branchstyle`.
fn ps_macro_text(cs: &str) -> Result<String> {
  Ok(match ps_macro_body(cs)? {
    Some(body) => do_expand(body)?.to_string().trim().to_string(),
    None => String::new(),
  })
}

/// The numbers of a raw pstricks value macro (`\psk@dash` = ` 5.0  3.0 `).
fn ps_macro_numbers(cs: &str) -> Result<Vec<f64>> {
  Ok(
    ps_macro_text(cs)?
      .split_whitespace()
      .filter_map(|n| n.parse().ok())
      .collect(),
  )
}

/// The top-level `{…}` groups of a macro's replacement text, unbraced.
fn ps_brace_groups(text: &str) -> Vec<&str> {
  let mut groups = Vec::new();
  let (mut depth, mut start) = (0usize, 0usize);
  for (k, c) in text.char_indices() {
    match c {
      '{' => {
        if depth == 0 {
          start = k + 1;
        }
        depth += 1;
      },
      '}' if depth > 0 => {
        depth -= 1;
        if depth == 0 {
          groups.push(text[start..k].trim());
        }
      },
      _ => {},
    }
  }
  groups
}

/// A pstricks colour as the raw `\psset` stored it (pstricks.tex:1080
/// `\pst@getcolor{#1}\pslinecolor`, :1203 `\psfillcolor`), as an SVG colour.
/// With xcolor (pstricks.sty requires it) `\pst@getcolor` is `\XC@getcolor`,
/// whose value is `\xcolor@{}{<spec>}{<model>}{<spec>}` in any xcolor model
/// (`[HTML]{FF8000}` keeps `HTML`, xcolor_sty.rs `\XC@getcolor@lxmodel`);
/// Perl's `setGraphParams` (:336-338) stores `LookupColor(...)->toHex`. An
/// empty one names no colour. A bare name is a default pstricks.tex set
/// while it loaded, before pstricks.sty let `\pst@getcolor` to xcolor's, or
/// any colour under plain-TeX `\input pstricks`, whose own `\pst@getcolor`
/// stores the name: it is resolved to hex through the colour table (a user
/// colour is no SVG colour, and LaTeX's `green` is not SVG's), except Perl's
/// own defaults `black` and `white` (:400-402), which SVG reads the same.
fn ps_color(cs: &str) -> Result<Option<String>> {
  // Unexpanded: `\xcolor@` expands to its `<spec>` alone (xcolor.sty:603).
  let value =
    ps_macro_body(cs)?.map_or_else(String::new, |body| body.to_string().trim().to_string());
  if let Some(payload) = value.strip_prefix("\\xcolor@") {
    return Ok(xcolor_payload_color(payload));
  }
  if value.is_empty() || value == "black" || value == "white" {
    return Ok((!value.is_empty()).then_some(value));
  }
  Ok(Some(if lookup_value(&color_key(&value)).is_some() {
    lookup_color_obj(&value).to_attribute()
  } else if let Some(color) = ps_native_color(&value)? {
    color
  } else {
    value
  }))
}

/// The colour of an `\xcolor@{…}{…}{<model>}{<spec>}` body (xcolor.sty:603);
/// none when the spec is empty.
fn xcolor_payload_color(payload: &str) -> Option<String> {
  match ps_brace_groups(payload).as_slice() {
    [.., model, spec] if !spec.is_empty() => {
      Some(parse_xcolor(Some(model), spec, None).to_attribute())
    },
    _ => None,
  }
}

/// A colour pstricks defined itself, outside the colour table: under plain
/// `\input pstricks`, pstricks-color.tex:23-27 `\@newcolor` keeps it as the
/// PostScript operator its `\newgray`/`\newrgbcolor`/`\newhsbcolor`/
/// `\newcmykcolor` (:93-104) build, in `\csname\string\color@<name>\endcsname`:
/// the defaults (:105-115) and the user's own. `green` is pstricks' `0 1 0
/// setrgbcolor`, #00FF00, not SVG's #008000.
fn ps_native_color(name: &str) -> Result<Option<String>> {
  let Some(body) = ps_macro_body(&s!("\\\\color@{name}"))? else {
    return Ok(None);
  };
  let body = body.to_string();
  // A colour `\definecolor` made is stored in the xcolor form.
  if let Some(payload) = body.trim().strip_prefix("\\xcolor@") {
    return Ok(xcolor_payload_color(payload));
  }
  let words: Vec<&str> = body.split_whitespace().collect();
  let Some((operator, values)) = words.split_last() else {
    return Ok(None);
  };
  let (model, arity) = match *operator {
    "setgray" => ("gray", 1),
    "setrgbcolor" => ("rgb", 3),
    "sethsbcolor" => ("hsb", 3),
    "setcmykcolor" => ("cmyk", 4),
    _ => return Ok(None),
  };
  if values.len() != arity || values.iter().any(|v| v.parse::<f64>().is_err()) {
    return Ok(None);
  }
  Ok(Some(
    parse_xcolor(Some(model), &values.join(","), None).to_attribute(),
  ))
}

/// Whether a raw pstricks boolean key is on (`\define@boolkey[psset]
/// {pstricks}[]{showpoints}` → `\ifshowpoints`, pstricks.tex:1046).
fn ps_if(cs: &str) -> Result<bool> {
  let token = T_CS!(cs);
  if lookup_definition(&token)?.is_none() {
    return Ok(false);
  }
  Ok(if_condition(&token)?.unwrap_or(false))
}

/// The `arrows` default a `\psset{arrows=A-B}` left (Perl `LookupValue
/// ('\psarrows')`, :133). pstricks-arrows.tex:239-251 keeps `\psk@arrowB`
/// = B but `\psk@arrowA` = the `\pst@arrowtable` image of A (`<` → `>`,
/// :87-88), so A is read back through the table.
fn ps_default_arrows() -> Result<Option<String>> {
  let (a, b) = (
    ps_macro_text("\\psk@arrowA")?,
    ps_macro_text("\\psk@arrowB")?,
  );
  if a.is_empty() && b.is_empty() {
    return Ok(None);
  }
  let table = ps_macro_text("\\pst@arrowtable")?;
  let user_a = table
    .split(',')
    .filter_map(|entry| entry.split_once('-'))
    .find(|(_, image)| *image == a)
    .map_or(a.as_str(), |(spec, _)| spec);
  Ok(Some(format!("{user_a}-{b}")))
}

/// Perl `ReadArrows` (:126-133): the next `{…}` only when it holds arrow
/// characters and a `-`; otherwise it is put back and the `arrows` default
/// applies.
fn read_ps_arrows() -> Result<ArgWrap> {
  skip_spaces()?;
  if if_next(T_BEGIN!())? {
    let arrows = read_arg(ExpansionLevel::Off)?;
    let text = arrows.to_string();
    let is_arrows = text.contains('-')
      && text
        .chars()
        .all(|c| c.is_whitespace() || "()-><|cCo*[]".contains(c));
    if is_arrows {
      return Ok(ArgWrap::Tokens(arrows));
    }
    let mut back = vec![T_BEGIN!()];
    back.extend(arrows.unlist());
    back.push(T_END!());
    unread_vec(back);
  }
  Ok(match ps_default_arrows()? {
    Some(arrows) => ArgWrap::Tokens(Tokens::new(Explode!(arrows))),
    None => ArgWrap::None,
  })
}

/// Perl `arrowLength` (:445-449): `(linewidth × arrowsize-num +
/// arrowsize-dim) × arrowlength`, from the raw `\psk@arrowsize` (`1.5 2.`)
/// and `\psk@arrowlength` (`1.4`).
fn ps_arrow_length() -> Result<Option<String>> {
  let size = ps_macro_numbers("\\psk@arrowsize")?;
  let length = ps_macro_numbers("\\psk@arrowlength")?;
  let linewidth = ps_register_pt("\\pslinewidth", 0.8)?;
  Ok(match (size.as_slice(), length.first()) {
    ([dim, num, ..], Some(al)) => Some(ps_pt_num((linewidth * num + dim) * al)),
    _ => None,
  })
}

/// Perl `reverseArrow` (:452-459): `{A-B}` → `{B'-A'}` with every bracket
/// and arrow head mirrored, its own example being `{->}` → `{<-}`. Perl's
/// `/([^\-]+)-([^\-]+)/` needs a head on both sides, so a one-sided
/// `{->}` only mirrored to `{-<}`: a head at the arc's far end pointing back,
/// where pdflatex's `\psarcn{->}` ends at its first angle, the arc's start
/// here (its angles are swapped).
fn ps_reverse_arrow(arrows: &str) -> Option<String> {
  if arrows.is_empty() {
    return None;
  }
  let swapped = match arrows.split_once('-') {
    Some((a, b)) if !b.contains('-') => format!("{b}-{a}"),
    _ => arrows.to_string(),
  };
  Some(
    swapped
      .chars()
      .map(|c| match c {
        '>' => '<',
        '<' => '>',
        '[' => ']',
        ']' => '[',
        '(' => ')',
        ')' => '(',
        c => c,
      })
      .collect(),
  )
}

/// Perl `psTerminators` (:461-467).
fn ps_terminators(whatsit: &mut Whatsit, arrows: Option<String>) -> Result<()> {
  if let Some(arrows) = arrows {
    let arrows: String = arrows.chars().filter(|c| !c.is_whitespace()).collect();
    if !arrows.is_empty() && arrows != "-" {
      whatsit.set_property("terminators", Stored::from(arrows));
      if let Some(length) = ps_arrow_length()? {
        whatsit.set_property("arrowlength", Stored::from(length));
      }
    }
  }
  Ok(())
}

/// The explicit-or-default `Arrows` argument of an object.
fn ps_arrows_arg(whatsit: &Whatsit, n: usize) -> Option<String> {
  whatsit
    .get_arg(n)
    .map(|a| a.to_string())
    .filter(|s| !s.is_empty())
}

/// Perl `psGetLinecolor` (:357-361).
fn ps_get_linecolor(whatsit: &mut Whatsit, used: &mut Vec<&'static str>) -> Result<()> {
  let color = if ps_macro_text("\\pslinestyle")? == "none" {
    Some("none".to_string())
  } else {
    ps_color("\\pslinecolor")?
  };
  if let Some(color) = color {
    whatsit.set_property("linecolor", Stored::from(color));
  }
  used.push("linecolor");
  Ok(())
}

/// Perl `psGetFill` (:363-373). A starred object is filled with the line
/// colour and not stroked. Otherwise the fill colour applies when the
/// `fillstyle` fills: Perl's rule is "anything but `none`" ("fillstyle is
/// assumed solid or none", pstricks.sty.ltxml:27), which matched pdflatex
/// only while Perl's `\psfillcolor` defaulted to `none`. The raw state
/// defaults it to `white` (pstricks.tex:1212), so the hatch styles
/// (`vlines`, `hlines`, `crosshatch`, `dots`, …: pstricks.tex:1236-1330,
/// lines only) would paint a white background pdflatex never draws: only
/// `solid`, `eofill` and the starred `<style>*` variants fill.
fn ps_get_fill(whatsit: &mut Whatsit, used: &mut Vec<&'static str>) -> Result<()> {
  let starred = whatsit.get_arg(1).is_some_and(|a| a.to_string() == "*");
  if starred {
    if let Some(color) = ps_color("\\pslinecolor")? {
      whatsit.set_property("fill", Stored::from(color));
    }
    whatsit.set_property("linecolor", Stored::from("none"));
  } else {
    let fillstyle = ps_macro_text("\\psk@fillstylename")?;
    let fills = fillstyle == "solid" || fillstyle == "eofill" || fillstyle.ends_with('*');
    let fill = if fills {
      ps_color("\\psfillcolor")?
    } else {
      Some("none".to_string())
    };
    if let Some(fill) = fill {
      whatsit.set_property("fill", Stored::from(fill));
    }
    whatsit.set_property(
      "linewidth",
      Stored::from(ps_pt_num(ps_register_pt("\\pslinewidth", 0.8)?)),
    );
    ps_get_linecolor(whatsit, used)?;
  }
  used.extend(["fill", "linecolor", "linewidth", "fillstyle", "fillcolor"]);
  Ok(())
}

/// Perl `psGetDash` (:344-355): `dashed` uses the `dash` pattern
/// (`\psk@dash`, pstricks.tex:1105-1134, trailing zero pairs dropped:
/// the default `5pt 3pt 0pt 0pt` is Perl's `5,3`), `dotted` is `1pt`
/// on, `dotsep` off. Perl also applies an explicitly set `dash` to a
/// `solid` line; pdflatex dashes only a `dashed` one.
fn ps_get_dash(whatsit: &mut Whatsit, used: &mut Vec<&'static str>) -> Result<()> {
  let dash = match ps_macro_text("\\pslinestyle")?.as_str() {
    "dashed" => {
      let mut pattern = ps_macro_numbers("\\psk@dash")?;
      while pattern.len() >= 2
        && pattern[pattern.len() - 1] == 0.0
        && pattern[pattern.len() - 2] == 0.0
      {
        pattern.truncate(pattern.len() - 2);
      }
      (!pattern.is_empty()).then(|| {
        pattern
          .iter()
          .map(|v| ps_pt_num(*v))
          .collect::<Vec<_>>()
          .join(",")
      })
    },
    "dotted" => {
      let dotsep = ps_macro_numbers("\\psk@dotsep")?
        .first()
        .copied()
        .unwrap_or(3.0);
      Some(format!("1,{}", ps_pt_num(dotsep)))
    },
    _ => None,
  };
  if let Some(dash) = dash {
    whatsit.set_property("dash", Stored::from(dash));
  }
  used.push("dash");
  Ok(())
}

/// Perl `LookupValue('\ps' . $param)` (:387-388) over the raw storage.
/// Parameters Perl only knows once `\psset` assigned them are omitted at
/// their raw default (`linearc` 0, `arcsep` 0, `dotscale` 1, `showpoints`
/// false), as Perl's are unset then.
fn ps_parameter(param: &str) -> Result<Option<String>> {
  let nonzero_pt = |pt: f64| (pt != 0.0).then(|| ps_pt_num(pt));
  Ok(match param {
    "linewidth" => Some(ps_pt_num(ps_register_pt("\\pslinewidth", 0.8)?)),
    "linearc" => nonzero_pt(ps_register_pt("\\pslinearc", 0.0)?),
    "arcsepA" => ps_macro_numbers("\\psk@arcsepA")?
      .first()
      .copied()
      .and_then(nonzero_pt),
    "arcsepB" => ps_macro_numbers("\\psk@arcsepB")?
      .first()
      .copied()
      .and_then(nonzero_pt),
    "showpoints" => ps_if("\\ifshowpoints")?.then(|| "true".to_string()),
    "dotstyle" => Some(ps_macro_text("\\psk@dotstyle")?).filter(|s| !s.is_empty()),
    "dotscale" => {
      let scale = ps_macro_numbers("\\psk@dotscale")?;
      match scale.as_slice() {
        [x, y, ..] if (*x, *y) != (1.0, 1.0) => Some(if x == y {
          ps_fmt_px(*x)
        } else {
          format!("{} {}", ps_fmt_px(*x), ps_fmt_px(*y))
        }),
        _ => None,
      }
    },
    "curvature" => {
      let curvature = ps_macro_numbers("\\psk@curvature")?;
      (!curvature.is_empty()).then(|| {
        curvature
          .iter()
          .map(|v| ps_fmt_px(*v))
          .collect::<Vec<_>>()
          .join(" ")
      })
    },
    "fillcolor" => ps_color("\\psfillcolor")?,
    "xunit" => Some(ps_pt_num(ps_register_pt("\\psxunit", PS_CM)?)),
    "yunit" => Some(ps_pt_num(ps_register_pt("\\psyunit", PS_CM)?)),
    _ => None,
  })
}

/// Perl `psDefaultParameters` (:375-389): the graphics parameters an object
/// lists as its `attributes`, each through its getter or as stored.
fn ps_default_parameters(
  whatsit: &mut Whatsit,
  cmd: &str,
  attributes: &[&'static str],
) -> Result<()> {
  assign_value(
    "_ps@LastPSCmd",
    Stored::String(pin(cmd)),
    Some(Scope::Global),
  );
  let mut used: Vec<&'static str> = Vec::new();
  for param in attributes {
    if used.contains(param) {
      continue;
    }
    match *param {
      "dash" => ps_get_dash(whatsit, &mut used)?,
      "linecolor" => ps_get_linecolor(whatsit, &mut used)?,
      "fill" => ps_get_fill(whatsit, &mut used)?,
      _ => {
        if let Some(value) = ps_parameter(param)? {
          whatsit.set_property(param, Stored::from(value));
        }
      },
    }
  }
  Ok(())
}

/// Whether an object is drawn into a picture: inside a `{pspicture}` (the
/// flag its `before_digest` sets) or a LaTeX `{picture}`. Elsewhere (a
/// `\psline` in running text or a caption, an object in a `{psmatrix}` cell)
/// pdflatex draws it over the text from the current point, in no box of its
/// own. Perl's objects auto-open an `ltx:picture` there, which its
/// `afterClose` (latex_constructs.pool.ltxml:4943-4950) sizes from the
/// object's whatsit, of no size. The Rust tag (latex_constructs/sect13.rs)
/// has no such sizing: the picture was unsized (an SVG a browser draws at
/// 300×150, flipped) and swallowed the rest of the paragraph. Such an object
/// draws nothing, as before this port drew any (ffslides.cls:138-152, the
/// header and footer rules; DIVERGENCES #301 row 14).
fn ps_in_picture() -> bool {
  lookup_bool("lx_in_pspicture")
    || with_stacked_values("current_environment", |envs| {
      envs
        .iter()
        .any(|v| matches!(&**v, Stored::String(s) if with(*s, |e| e == "picture")))
    })
}

/// Run an object's `geometry`, which sets its coordinate properties and
/// answers whether every coordinate is resolved; the object draws (`draw`)
/// only then and only in a picture.
fn ps_draw(
  whatsit: &mut Whatsit,
  geometry: impl FnOnce(&mut Whatsit) -> Result<bool>,
) -> Result<bool> {
  let in_picture = ps_in_picture();
  let draw = in_picture && geometry(whatsit)?;
  if draw {
    whatsit.set_property("draw", Stored::Bool(true));
  } else {
    // Counted, not a warning: the corpus can see what was left undrawn.
    let why = if in_picture {
      "a coordinate this port cannot place"
    } else {
      "outside a picture"
    };
    Info!("pstricks", "undrawn", &s!("object not drawn: {why}"));
  }
  Ok(draw)
}

/// Perl `afterPSObject` (:476-479) after the object's own `afterDigest`
/// (`DefPSConstructor`, :491-507): the geometry, then the parameters of a
/// drawn object, then close the group its macro opened around the local
/// `\psset` — on every path, an error included, or the group leaks.
fn after_ps_object(
  whatsit: &mut Whatsit,
  cmd: &str,
  attributes: &[&'static str],
  geometry: impl FnOnce(&mut Whatsit) -> Result<bool>,
) -> Result<()> {
  let drawn = ps_draw(whatsit, geometry).and_then(|draw| {
    if draw {
      ps_default_parameters(whatsit, cmd, attributes)
    } else {
      Ok(())
    }
  });
  let closed = endgroup();
  drawn.and(closed)
}

/// Perl `DefPSConstructor` (:491-507), its macro half: `\cmd*[<params>]`
/// → `\begingroup\psset{<params>}\cmd@*`, the parameters set locally inside
/// a group that the constructor `\cmd@` closes (`after_ps_object`). Perl
/// opens the group with `{` and closes it with `Digest(T_END)` (:479, :500),
/// but that `}` is unread into its own mouth first, which retracts the brace
/// (Gullet.pm:343-358), so it never leaves the alignment brace ledger where
/// the scanned `{` put it: inside an `\halign` cell every later `&`/`\cr`
/// went unrecognised (pst-node's `{psmatrix}` under pstricks-add;
/// dspTricksManual's `{dspBlocks}`: 0 → 17 errors and a Fatal). A semi-simple
/// group scopes the `\psset` the same way and has no brace to count.
fn def_ps_object_macro(cmd: &str) -> Result<()> {
  let token = T_CS!(cmd);
  let params = parse_parameters("OptionalMatch:* []", &token, true)?;
  let constructor = s!("{cmd}@");
  let body = ExpansionBody::Closure(Rc::new(move |args: Vec<ArgWrap>| {
    let mut expansion = vec![T_CS!("\\begingroup")];
    if let Some(keys) = args.get(1).filter(|a| !a.is_none()) {
      expansion.push(T_CS!("\\psset"));
      expansion.push(T_BEGIN!());
      if let Ok(Some(keys)) = keys.as_tokens() {
        expansion.extend(keys.unlist_ref().iter().copied());
      }
      expansion.push(T_END!());
    }
    expansion.push(T_CS!(constructor.clone()));
    if args.first().is_some_and(|a| !a.is_none()) {
      expansion.push(T_OTHER!("*"));
    }
    Ok(Tokens::new(expansion))
  }));
  def_macro(token, params, Some(body), None)
}

#[rustfmt::skip]
LoadDefinitions!({
  // Parameter types — Perl pstricks_support.sty.ltxml:25-250. `PSCoord`
  // is a `Pair` in points; `OptionalPSCoord` needs no definition (the spec
  // parser derives `Optional<Type>` from the prefix).
  DefParameterType!(PSCoord, sub[_inner, _extra] {
    read_ps_coord()?.map_or(ArgWrap::None, ps_coord_arg)
  });
  // `ReadZeroPSCoord` (:111-113): a missing pair is the origin.
  DefParameterType!(ZeroPSCoord, sub[_inner, _extra] {
    ps_coord_arg(read_ps_coord()?.unwrap_or(PsCoord::Point(0.0, 0.0)))
  });
  // `ReadPSCoordList` (:115-119). The pairs travel as their reversion
  // `(<x>pt,<y>pt)…`, an unresolved one as `(?)`, undigested
  // (`ps_coord_list` reads them back).
  DefParameterType!(PSCoordList, sub[_inner, _extra] {
    let mut list = String::new();
    while let Some(coord) = read_ps_coord()? {
      match coord {
        PsCoord::Point(x, y) => list.push_str(&s!("({}pt,{}pt)", x, y)),
        PsCoord::Unresolved => list.push_str("(?)"),
      }
    }
    ArgWrap::Tokens(Tokens::new(Explode!(list)))
  },
  predigest => sub[arg] { Ok(arg.undigested()) });
  DefParameterType!(PSDimension, sub[_inner, _extra] {
    ArgWrap::Dimension(Dimension::new_f64(read_ps_dimension(None)? * UNITY_F64))
  });
  DefParameterType!(Arrows, sub[_inner, _extra] { read_ps_arrows()? },
    predigest => sub[arg] { Ok(arg.undigested()) }
    optional => true,
    reversion => sub[arg, _inner, _extra] {
      let mut tokens = vec![T_BEGIN!()];
      tokens.extend(arg);
      tokens.push(T_END!());
      Ok(Tokens::new(tokens))
    });
  DefParameterType!(PSAngle, sub[_inner, _extra] { read_ps_angle()? });

  // Transform management — Perl L130-200. `\psset` / `\@@@ackscale` are
  // Perl DefConstructor / DefPrimitive — the Rust stubs are DefMacro
  // drops because the full body needs PSDim*/PSAngle/PSOrigin parameter
  // types that aren't ported (see WISDOM #41 — TeXDelimiter / Pair /
  // PSDim are all structural parameter-type gaps). DP-audit flags both
  // entries; safe until the parameter types land (no \edef site
  // observes these CSes; pstricks use is stomach-time invocation).
  //
  // Intentional divergence (WISDOM #44 class): the transform tracker
  // (_psActiveTransform / ackTransform, Perl :256-298) is not ported to
  // Rust. The \@@@ackscale DefPrimitive → DefMacro flip (L181)
  // is a no-op arg-consumer whose observable behavior under an
  // HTML/MathML backend is identical to a constructor body of "".
  // `\pst@object`, `\use@par`, `\addto@par` are the RAW pstricks.tex
  // definitions (pstricks.tex:1453-1461: `\pst@object{name}` sets `\pst@par`,
  // reads `*`/`[…]` and dispatches to `\<name>@i`; `\use@par` :1441 applies the
  // stored key list; `\addto@par` :1425 extends it). The former stub
  // `\pst@object{}` → `#1` typeset the
  // object's NAME as text and never reached `\<name>@i`: pst-node's
  // `\psm@beginnode` (pst-node.tex:1206-1209, `\pst@object{psm@beginnode}`)
  // opened no node box, so psmatrix's v-part `\psm@endnode@i`
  // (`\unskip\endgroup\psm@endmath\egroup`, :1215-1221) closed the
  // alignment's own cell frame and the template's trailing `\endgroup` met
  // the row frame ("`\endgroup` Attempt to close non-boxing group";
  // dsptricks 98, every psmatrix). Perl has no such stub. The drawing
  // objects are this file's own constructors (II Basic graphics objects).
  // `\psset` itself stays the raw pst-xkey.tex definition (pstricks_sty.rs),
  // so the family key BODIES run: `linecolor=` & co. call `\pst@getcolor`
  // (pstricks.tex `\pst@getcolor{name}\psk@linecolor`). pstricks.sty:150-176,
  // verbatim in shape: with xcolor loaded (its 2004+ contract — our binding
  // supplies `\XC@getcolor`/`\XC@usecolor`, xcolor.sty:1373-1396) the helpers
  // are xcolor's; otherwise pstricks' own over `\color@<name>` storage, which
  // is exactly how color_sty.rs stores a `\definecolor`, plus the three grays.
  // `\pst@usecolor` writes the PostScript colour (`\c@lor@to@ps`): no
  // PostScript backend, so the fallback emits nothing. Guards:
  // `perfect_kernel_batch56::psset_dispatches_family_key_bodies` (control:
  // `\psset{linewidth=2pt,linecolor=red}` converts clean),
  // `perfect_kernel_gemini::xcolor_pst_getcolor_and_usecolor` (xcolor first).
  RawTeX!(r"\@ifpackageloaded{xcolor}{\let\pst@getcolor\XC@getcolor\let\pst@usecolor\XC@usecolor}{%
\def\pst@getcolor#1#2{\@ifundefined{\string\color@#1}{\@pstrickserr{Color `#1' not defined}\@eha}{\edef#2{#1}}}%
\def\pst@usecolor#1{}%
\definecolor{darkgray}{gray}{.25}\definecolor{gray}{gray}{.5}\definecolor{lightgray}{gray}{.75}}");
  def_macro_noop("\\psset@special{}")?;

  // Perl pstricks_support.sty.ltxml L580-606: register 22 pstricks keyvals
  // covering dot/arrow sizes, line styling, frame/arc/label spacing, and
  // coordinate units. Perl types PSDimFloat / PSAngle / PSDimension /
  // PSDimDim / PSOrigin / PSRegisterDimension / Float aren't registered
  // Rust types; register with the untyped placeholder ("") since no
  // consumer coerces these values (the raw `\psset` stores them, and the
  // drawing objects read that storage). Author code
  // that tests `\@ifundefined{KV@pstricks@dotsize@default}` now sees the
  // Perl-equivalent answer.
  for key in ["dotsize", "tbarsize", "dotangle",
              "arrowsize", "arrowlength", "arrowinset",
              "dotsep", "dash", "linewidth", "linearc", "framearc",
              "origin", "framesep", "labelsep", "doublesep",
              "arcsep", "arcsepA", "arcsepB",
              "unit", "xunit", "yunit", "runit"] {
    DefKeyVal!("pstricks", key, "");
  }

  // Graphics parameters — Perl L200-350
  DefRegister!("\\pslinewidth" => Dimension!("0.8pt"));
  DefRegister!("\\psunit" => Dimension!("1cm"));
  DefRegister!("\\psxunit" => Dimension!("1cm"));
  DefRegister!("\\psyunit" => Dimension!("1cm"));
  DefRegister!("\\pst@dima" => Dimension::new(0));
  DefRegister!("\\pst@dimb" => Dimension::new(0));

  // 3 The pspicture environment — Perl pstricks_support.sty.ltxml:520-563:
  // `\begin{pspicture}*[baseline](x0,y0)(x1,y1)` is an `<ltx:picture>` sized
  // by the two corners in `\psxunit`/`\psyunit` (the FIRST corner is
  // optional: a lone pair is the far corner, origin (0,0)), the body inside a
  // `<ltx:g>` translated by the negated origin, `\par` let to `\relax`. The
  // former `[]{}` signature swallowed the `(` of the first pair and leaked
  // `x0,y0)(x1,y1)` as text in EVERY pstricks picture (batch 56ao; the LaTeX
  // `{picture}` binding in latex_constructs/sect13.rs is the model). Defined
  // here, as in Perl, so that plain-TeX `\input pstricks` (pstricks_tex.rs,
  // which loads this file and not pstricks.sty) has it too: there the raw
  // `\pspicture` ran, and every object in it auto-opened an unsized picture.
  DefEnvironment!("{pspicture} OptionalMatch:* [] PSCoord OptionalPSCoord",
    "<ltx:picture width='#width' height='#height' origin-x='#origin-x' origin-y='#origin-y'\
      fill='none' stroke='none' unitlength='#unitlength'>\
      ?#transform(<ltx:g transform='#transform'>#body</ltx:g>)(#body)\
    </ltx:picture>",
    mode => "inline_internal_vertical",
    before_digest => { Let!("\\par", "\\relax"); assign_value("lx_in_pspicture", Stored::Bool(true), None); },
    properties => sub[args] { pspicture_properties(args[2].as_ref(), args[3].as_ref()) }
  );
  DefEnvironment!("{pspicture*} OptionalMatch:* [] PSCoord OptionalPSCoord",
    "<ltx:picture width='#width' height='#height' origin-x='#origin-x' origin-y='#origin-y'\
      clip='true' fill='none' stroke='none' unitlength='#unitlength'>\
      ?#transform(<ltx:g transform='#transform'>#body</ltx:g>)(#body)\
    </ltx:picture>",
    mode => "inline_internal_vertical",
    before_digest => { Let!("\\par", "\\relax"); assign_value("lx_in_pspicture", Stored::Bool(true), None); },
    properties => sub[args] { pspicture_properties(args[2].as_ref(), args[3].as_ref()) }
  );

  // II Basic graphics objects — Perl pstricks_support.sty.ltxml:661-843,
  // in Perl's order. Each `DefPSConstructor` is the `\cmd[*][<params>]`
  // macro (`def_ps_object_macro`) plus the constructor `\cmd@` whose
  // arguments follow `OptionalMatch:*`; its `after_digest` runs Perl's
  // `afterDigest` geometry, then `psTerminators` for an `Arrows` object,
  // then `afterPSObject` (`after_ps_object`). An object draws only when all
  // its coordinates are resolved (a node reference, PostScript or algebraic
  // coordinate is not, `PsCoord`) and it is inside a picture
  // (`ps_in_picture`); otherwise it is consumed and draws nothing.
  //
  // Geometry is in px, the unit of the enclosing `{pspicture}` frame and of
  // Perl's own `\psframe`/`\pscircle` centres/`\psellipse` (`pxValue`). Perl
  // writes the `\psline`/`\pspolygon`/`\psbezier`/`\pscurve`/`\psdots`
  // points and the `\pscircle`/`\pswedge`/`\psarc` radii with `ptValue`
  // (:663, :675, :701, :717, :733, :756, :784, :805) into that same px
  // space, so every line stopped at 72.27% of its length and every circle
  // was drawn at 72.27% of its radius around a px centre, off the frame and
  // the other objects pdflatex joins them to. Stroke widths, dash patterns
  // and dot sizes keep Perl's `ptValue`, as the LaTeX `{picture}` does.
  // Witnesses: egpeirce-doc, srdp-mathematik, egameps, lsc, dspTricksManual.
  //
  // 6 Lines and polygons.
  def_ps_object_macro("\\psline")?;
  DefConstructor!("\\psline@ OptionalMatch:* Arrows PSCoordList",
    "?#draw(<ltx:line stroke='#linecolor' stroke-width='#linewidth' stroke-dasharray='#dash'\
      terminators='#terminators' arrowlength='#arrowlength' arc='#linearc'\
      points='#points' fill='#fill'/>)",
    alias => "\\psline",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\psline", &["fill", "linecolor", "linewidth", "linearc", "dash", "showpoints"], |whatsit| {
        let Some(points) = ps_coord_list(whatsit.get_arg(3)) else { return Ok(false) };
        whatsit.set_property("points", Stored::from(ps_points_px(&ps_with_origin(points, 2))));
        let arrows = ps_arrows_arg(whatsit, 2);
        ps_terminators(whatsit, arrows)?;
        Ok(true)
      })
    });
  // `DefSimplePSConstructor` (:509-514) builds the `psDefaultParameters`
  // hook and never attaches it, so Perl's `\qline` has no stroke and is
  // invisible under the picture's `stroke='none'`; pdflatex draws it
  // (pstricks.tex `\qline` strokes with the current line parameters).
  DefConstructor!("\\qline PSCoordList",
    "?#draw(<ltx:line points='#points' stroke='#linecolor' stroke-width='#linewidth'\
      stroke-dasharray='#dash'/>)",
    after_digest => sub[whatsit] {
      let draw = ps_draw(whatsit, |whatsit| {
        let Some(points) = ps_coord_list(whatsit.get_arg(1)) else { return Ok(false) };
        whatsit.set_property("points", Stored::from(ps_points_px(&points)));
        Ok(true)
      })?;
      if draw {
        ps_default_parameters(whatsit, "\\qline", &["linecolor", "linewidth", "dash"])?;
      }
    });
  def_ps_object_macro("\\pspolygon")?;
  DefConstructor!("\\pspolygon@ OptionalMatch:* PSCoordList",
    "?#draw(<ltx:polygon stroke='#linecolor' stroke-width='#linewidth' stroke-dasharray='#dash'\
      arc='#linearc' points='#points' fill='#fill' showpoints='#showpoints'/>)",
    alias => "\\pspolygon",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\pspolygon", &["fill", "linecolor", "linewidth", "linearc", "dash", "showpoints"], |whatsit| {
        let Some(points) = ps_coord_list(whatsit.get_arg(2)) else { return Ok(false) };
        whatsit.set_property("points", Stored::from(ps_points_px(&ps_with_origin(points, 3))));
        Ok(true)
      })
    });
  // `rx` is Perl's `arcValue` (:430-443), whose own comment says its logic
  // "isn't right" (it takes `ptValue` of the `framearc` Float, i.e. 0) and
  // keeps the intended formula commented out: pstricks' `framearc` radius,
  // half the smaller side times `framearc` (`cornersize=relative`, the
  // default), or `linearc` (`cornersize=absolute`), pstricks.tex:2175-2184.
  // The corners may come in any order (ffslides.cls:102-103 sets a negative
  // `yunit`): the rect is their bounding box, as pdflatex draws it; Perl's
  // `c1 - c0` (:693-694) gave a negative `height`, which SVG does not render.
  def_ps_object_macro("\\psframe")?;
  DefConstructor!("\\psframe@ OptionalMatch:* PSCoordList",
    "?#draw(<ltx:rect stroke='#linecolor' stroke-width='#linewidth' stroke-dasharray='#dash'\
      x='#x' y='#y' width='#pxwidth' height='#pxheight' rx='#arcval' fill='#fill'/>)",
    alias => "\\psframe",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\psframe", &["fill", "linecolor", "linewidth", "dash"], |whatsit| {
        let Some(corners) = ps_coord_list(whatsit.get_arg(2)) else { return Ok(false) };
        let (c0, c1) = match corners.as_slice() {
          [c0, c1, ..] => (*c0, *c1),
          [c1] => ((0.0, 0.0), *c1),
          [] => ((0.0, 0.0), (0.0, 0.0)),
        };
        let (w, h) = ((c1.0 - c0.0).abs(), (c1.1 - c0.1).abs());
        let relative = ps_macro_text("\\psk@cornersize")?.contains("true");
        let arc = if relative {
          let framearc = ps_macro_numbers("\\psk@framearc")?.first().copied().unwrap_or(0.0);
          framearc * w.min(h) / 2.0
        } else {
          ps_register_pt("\\pslinearc", 0.0)?
        };
        if arc != 0.0 {
          whatsit.set_property("arcval", Stored::from(ps_fmt_px(ps_px(arc))));
        }
        whatsit.set_property("x", Stored::from(ps_fmt_px(ps_px(c0.0.min(c1.0)))));
        whatsit.set_property("y", Stored::from(ps_fmt_px(ps_px(c0.1.min(c1.1)))));
        whatsit.set_property("pxwidth", Stored::from(ps_fmt_px(ps_px(w))));
        whatsit.set_property("pxheight", Stored::from(ps_fmt_px(ps_px(h))));
        Ok(true)
      })
    });

  // 7 Arcs, circles and ellipses.
  def_ps_object_macro("\\pscircle")?;
  DefConstructor!("\\pscircle@ OptionalMatch:* ZeroPSCoord {PSDimension}",
    "?#draw(<ltx:circle stroke='#linecolor' stroke-width='#linewidth' stroke-dasharray='#dash'\
      x='#x' y='#y' r='#r' fill='#fill'/>)",
    alias => "\\pscircle",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\pscircle", &["fill", "linecolor", "linewidth", "dash"], |whatsit| {
        ps_set_radius(whatsit, 3);
        Ok(ps_set_center(whatsit, 2))
      })
    });
  // Perl :707-713: filled with the line colour, never stroked (so that
  // `linestyle` cannot touch it). Witness arXiv:1112.2096: `\qdisk(3,2.5)
  // {2.5pt}` inside `\mbox{\scalebox{…}{\begin{pspicture}…}}`, after a
  // `\put(2.5,1.4){$a$}`; the former `{}{}` signature read `(` and `3` and
  // left `,2.5){2.5pt}` as stray picture text that trapped every later block
  // (proof, theorem, section, bibliography) in an un-closeable `<ltx:text>`.
  DefConstructor!("\\qdisk PSCoord {PSDimension}",
    "?#draw(<ltx:circle x='#x' y='#y' r='#r' fill='#myfill' stroke='none'/>)",
    after_digest => sub[whatsit] {
      ps_draw(whatsit, |whatsit| {
        if let Some(color) = ps_color("\\pslinecolor")? {
          whatsit.set_property("myfill", Stored::from(color));
        }
        ps_set_radius(whatsit, 2);
        Ok(ps_set_center(whatsit, 1))
      })?;
    });
  def_ps_object_macro("\\pswedge")?;
  DefConstructor!("\\pswedge@ OptionalMatch:* ZeroPSCoord {PSDimension} {PSAngle} {PSAngle}",
    "?#draw(<ltx:wedge stroke='#linecolor' stroke-width='#linewidth' stroke-dasharray='#dash'\
      x='#x' y='#y' r='#r' angle1='#angle1' angle2='#angle2' fill='#fill'/>)",
    alias => "\\pswedge",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\pswedge", &["fill", "linecolor", "linewidth", "dash"], |whatsit| {
        ps_set_radius(whatsit, 3);
        let angles = ps_set_angles(whatsit, 4, 5);
        Ok(ps_set_center(whatsit, 2) && angles)
      })
    });
  // One pair is the radii of an ellipse centred at the origin (:728); a
  // negative unit makes a radius negative, which SVG does not render.
  def_ps_object_macro("\\psellipse")?;
  DefConstructor!("\\psellipse@ OptionalMatch:* PSCoord OptionalPSCoord",
    "?#draw(<ltx:ellipse stroke='#linecolor' stroke-width='#linewidth' stroke-dasharray='#dash'\
      x='#x' y='#y' rx='#rx' ry='#ry' fill='#fill'/>)",
    alias => "\\psellipse",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\psellipse", &["fill", "linecolor", "linewidth", "dash"], |whatsit| {
        let first = ps_arg_coord(whatsit.get_arg(2)).unwrap_or(PsCoord::Point(0.0, 0.0));
        let (c, r) = match (first, ps_arg_coord(whatsit.get_arg(3))) {
          (PsCoord::Point(x, y), Some(PsCoord::Point(rx, ry))) => ((x, y), (rx, ry)),
          (PsCoord::Point(rx, ry), None) => ((0.0, 0.0), (rx, ry)),
          _ => return Ok(false),
        };
        whatsit.set_property("x", Stored::from(ps_fmt_px(ps_px(c.0))));
        whatsit.set_property("y", Stored::from(ps_fmt_px(ps_px(c.1))));
        whatsit.set_property("rx", Stored::from(ps_fmt_px(ps_px(r.0.abs()))));
        whatsit.set_property("ry", Stored::from(ps_fmt_px(ps_px(r.1.abs()))));
        Ok(true)
      })
    });
  def_ps_object_macro("\\psarc")?;
  DefConstructor!("\\psarc@ OptionalMatch:* Arrows ZeroPSCoord {PSDimension} {PSAngle} {PSAngle}",
    "?#draw(<ltx:arc stroke='#linecolor' stroke-width='#linewidth' stroke-dasharray='#dash'\
      terminators='#terminators' arrowlength='#arrowlength' x='#x' y='#y' r='#r'\
      angle1='#angle1' angle2='#angle2' arcsepA='#arcsepA' arcsepB='#arcsepB'\
      fill='#fill' showpoints='#showpoints'/>)",
    alias => "\\psarc",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\psarc",
        &["fill", "linecolor", "linewidth", "dash", "showpoints", "arcsepA", "arcsepB"], |whatsit| {
        ps_set_radius(whatsit, 4);
        let angles = ps_set_angles(whatsit, 5, 6);
        let arrows = ps_arrows_arg(whatsit, 2);
        ps_terminators(whatsit, arrows)?;
        Ok(ps_set_center(whatsit, 3) && angles)
      })
    });
  // The clockwise arc: angles and arc separations swapped, arrows reversed
  // (Perl :741-749; its `Arrows` is explicit, not `#!ARROWS`).
  def_ps_object_macro("\\psarcn")?;
  DefConstructor!("\\psarcn@ OptionalMatch:* Arrows ZeroPSCoord {PSDimension} {PSAngle} {PSAngle}",
    "?#draw(<ltx:arc stroke='#linecolor' stroke-width='#linewidth' stroke-dasharray='#dash'\
      terminators='#terminators' arrowlength='#arrowlength' x='#x' y='#y' r='#r'\
      angle1='#angle2' angle2='#angle1' arcsepA='#arcsepB' arcsepB='#arcsepA'\
      fill='#fill' showpoints='#showpoints'/>)",
    alias => "\\psarcn",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\psarcn",
        &["fill", "linecolor", "linewidth", "dash", "showpoints", "arcsepA", "arcsepB"], |whatsit| {
        ps_set_radius(whatsit, 4);
        let angles = ps_set_angles(whatsit, 5, 6);
        let arrows = ps_arrows_arg(whatsit, 2).and_then(|a| ps_reverse_arrow(&a));
        ps_terminators(whatsit, arrows)?;
        Ok(ps_set_center(whatsit, 3) && angles)
      })
    });

  // 8 Curves.
  def_ps_object_macro("\\psbezier")?;
  DefConstructor!("\\psbezier@ OptionalMatch:* Arrows PSCoordList",
    "?#draw(<ltx:bezier stroke='#linecolor' stroke-width='#linewidth' stroke-dasharray='#dash'\
      terminators='#terminators' arrowlength='#arrowlength' showpoints='#showpoints'\
      points='#points'/>)",
    alias => "\\psbezier",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\psbezier", &["linecolor", "linewidth", "dash", "showpoints"], |whatsit| {
        let Some(points) = ps_coord_list(whatsit.get_arg(3)) else { return Ok(false) };
        whatsit.set_property("points", Stored::from(ps_points_px(&ps_with_origin(points, 4))));
        let arrows = ps_arrows_arg(whatsit, 2);
        ps_terminators(whatsit, arrows)?;
        Ok(true)
      })
    });
  // Perl defines `\parabola` twice (:763 as `ltx:parabola`, :770 as
  // `ltx:bezier`); the second wins: the quadratic Bézier from (x0,y0) over
  // the vertex (x1,y1) to (2x1-x0, y0), in px (the reflected coordinates
  // rounded as the others are).
  def_ps_object_macro("\\parabola")?;
  DefConstructor!("\\parabola@ OptionalMatch:* Arrows PSCoord PSCoord",
    "?#draw(<ltx:bezier stroke='#linecolor' stroke-width='#linewidth' stroke-dasharray='#dash'\
      terminators='#terminators' arrowlength='#arrowlength' showpoints='#showpoints'\
      points='#path' fill='#fill'/>)",
    alias => "\\parabola",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\parabola", &["linecolor", "linewidth", "dash", "showpoints"], |whatsit| {
        let point = |arg: Option<&Digested>| ps_arg_coord(arg).unwrap_or(PsCoord::Point(0.0, 0.0));
        let (PsCoord::Point(x0, y0), PsCoord::Point(x1, y1)) =
          (point(whatsit.get_arg(3)), point(whatsit.get_arg(4))) else { return Ok(false) };
        let (x0, y0, x1, y1) = (ps_px(x0), ps_px(y0), ps_px(x1), ps_px(y1));
        let round2 = |v: f64| (v * 100.0).round() / 100.0;
        let (xc, yc) = (round2(2.0 * x1 - x0), round2(2.0 * y1 - y0));
        whatsit.set_property("path", Stored::from(s!("{},{} {},{} {},{}",
          ps_fmt_px(x0), ps_fmt_px(y0), ps_fmt_px(x1), ps_fmt_px(yc), ps_fmt_px(xc), ps_fmt_px(y0))));
        let arrows = ps_arrows_arg(whatsit, 2);
        ps_terminators(whatsit, arrows)?;
        Ok(true)
      })
    });
  // `noendpoints` / `closed` are not `ltx:curve` attributes in the schema
  // (LaTeXML-picture.rnc Picture.attributes), so the model drops them, as
  // Perl's does.
  def_ps_object_macro("\\pscurve")?;
  DefConstructor!("\\pscurve@ OptionalMatch:* Arrows PSCoordList",
    "?#draw(<ltx:curve stroke='#linecolor' stroke-width='#linewidth' stroke-dasharray='#dash'\
      terminators='#terminators' arrowlength='#arrowlength' points='#points'\
      showpoints='#showpoints' curvature='#curvature'/>)",
    alias => "\\pscurve",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\pscurve", PS_CURVE_ATTRIBUTES, ps_curve_geometry)
    });
  def_ps_object_macro("\\psecurve")?;
  DefConstructor!("\\psecurve@ OptionalMatch:* Arrows PSCoordList",
    "?#draw(<ltx:curve stroke='#linecolor' stroke-width='#linewidth' stroke-dasharray='#dash'\
      terminators='#terminators' arrowlength='#arrowlength' points='#points'\
      showpoints='#showpoints' curvature='#curvature' noendpoints='yes'/>)",
    alias => "\\psecurve",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\psecurve", PS_CURVE_ATTRIBUTES, ps_curve_geometry)
    });
  def_ps_object_macro("\\psccurve")?;
  DefConstructor!("\\psccurve@ OptionalMatch:* Arrows PSCoordList",
    "?#draw(<ltx:curve stroke='#linecolor' stroke-width='#linewidth' stroke-dasharray='#dash'\
      terminators='#terminators' arrowlength='#arrowlength' points='#points'\
      showpoints='#showpoints' curvature='#curvature' closed='yes'/>)",
    alias => "\\psccurve",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\psccurve", PS_CURVE_ATTRIBUTES, ps_curve_geometry)
    });

  // 9 Dots: filled with the line colour; `dotsize` is the dimension part of
  // the `dotsize` (`\psk@@dotsize`, pstricks-dots.tex:18-23) or, for the
  // `|` style, of the `tbarsize` (`\psk@tbarsize`, pstricks-arrows.tex:183),
  // as Perl's `\psdotsize`/`\pstbarsize` `ptValue` (:809-810). `dotangle`
  // is not a `dots` attribute in the schema (dropped, as in Perl).
  def_ps_object_macro("\\psdot")?;
  DefConstructor!("\\psdot@ OptionalMatch:* PSCoord",
    "?#draw(<ltx:dots dotstyle='#dotstyle' dotsize='#sz' dotscale='#dotscale'\
      showpoints='#showpoints' points='#points' fill='#myfill'/>)",
    alias => "\\psdot",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\psdot", PS_DOTS_ATTRIBUTES, |whatsit| {
        let PsCoord::Point(x, y) =
          ps_arg_coord(whatsit.get_arg(2)).unwrap_or(PsCoord::Point(0.0, 0.0)) else { return Ok(false) };
        whatsit.set_property("points", Stored::from(ps_points_px(&[(x, y)])));
        ps_dots_style(whatsit)?;
        Ok(true)
      })
    });
  def_ps_object_macro("\\psdots")?;
  DefConstructor!("\\psdots@ OptionalMatch:* PSCoordList",
    "?#draw(<ltx:dots dotstyle='#dotstyle' dotsize='#sz' dotscale='#dotscale'\
      showpoints='#showpoints' points='#points' fill='#myfill'/>)",
    alias => "\\psdots",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\psdots", PS_DOTS_ATTRIBUTES, |whatsit| {
        let Some(points) = ps_coord_list(whatsit.get_arg(2)) else { return Ok(false) };
        whatsit.set_property("points", Stored::from(ps_points_px(&points)));
        ps_dots_style(whatsit)?;
        Ok(true)
      })
    });

  // 24 Placing and rotating whatever — Perl :866-936. The placed body
  // SURVIVES inside an `<ltx:g transform>` (batch 56ao; Perl's `\rput@start`
  // … `\put@end`, :879-888), in pstricks units; rotation, refpoint and
  // labelsep are dropped (presentation). A coordinate this port cannot place
  // (a node reference `\rput(N){…}`, `\uput[ur](N){$A_1$}`) places the body at
  // the origin, Perl's `ZeroPSCoord` leniency, rather than dropping the label.
  // The `[refpoint]` reaches the `<ltx:g>` as Perl's `pos` (a digested `[]`,
  // measured like Perl's: `x \rput[bl](1,1){Hello} y` is 23.45×12.3 px).
  // `{pspicture}` is a real `<ltx:picture>`, so a placed label no longer
  // lands in the surrounding paragraph (the hep-ph/0102192 minipage-in-figure
  // cascade that once forced the body to be dropped; re-verified 0 errors).
  // Defined here, as in Perl, for plain-TeX `\input pstricks` too. As in
  // Perl, the placement opens the `<ltx:g>` and a separate `\put@end{body}`
  // writes the body and closes it: the placement's whatsit has no box
  // argument, so an `\rput` outside a pspicture auto-opens a picture sized
  // without its body (`x \rput(1,1){Hello} y`: 11.92×8.65 px, as in Perl),
  // while an enclosing box still measures the body through `\put@end`.
  DefConstructor!("\\lx@ps@put [] OptionalPSCoord",
    "<ltx:g transform='#transform' pos='#1'>",
    alias => "\\rput",
    properties => sub[args] { ps_put_properties(args[1].as_ref()) }
  );
  DefConstructor!("\\put@end {}", "#1</ltx:g>",
    alias => "",
    mode  => "restricted_horizontal"
  );
  // \Rput[refpoint](x,y){body} — placement at coords (real pstricks
  // defines this in pstricks.tex / pst-code-put.tex, raw-loaded by Perl's
  // `InputDefinitions('pstricks', noltxml=>1)`): the body, placement dropped.
  // Witness: 0905.1885 (`\Rput[t](0,0){$H_1$}`).
  DefMacro!("\\Rput OptionalMatch:* [] Pair {}", "#4");
  // Runaway-safe placement reader shared by \rput/\cput. Reads the optional
  // [refpoint], optional {angle}, optional (coords), and the mandatory
  // {body}. A former def used a *delimited* `(#1)` parameter
  // (`\def\lx@rput@parens(#1)#2{}`): for the braced-angle / no-coords form
  // `\rput{angle}{body}` there is no `(`, so TeX scanned FORWARD eating
  // tokens — including `\end{pspicture}` — until the next `(` anywhere
  // later. That swallowed the env end, so pspicture's `end_mode` never fired
  // and its mode-switch frame leaked, tripping `\endgroup Attempt to close a
  // group that switched to mode restricted_horizontal` (witness 1505.07999 +
  // the ~17-paper `\endgroup` mode-leak cluster). Perl avoids this with
  // `OptionalBracketed`+`ZeroPSCoord` (coords optional); we PEEK for `(`
  // instead of requiring it.
  RawTeX!("\\def\\lx@put@cb(#1)#2{\\expandafter\\lx@ps@put\\lx@ps@refpoint(#1)\\put@end{#2}}"); // (coords){body} -> placed body
  // `{group}` with no `(` after it: the group WAS the body (Perl's ZeroPSCoord
  // leniency, origin (0,0)); with a `(` after it, it was the rotation angle.
  RawTeX!("\\def\\lx@put@b#1{\\@ifnextchar(\\lx@put@cb{\\expandafter\\lx@ps@put\\lx@ps@refpoint(0,0)\\put@end{#1}}}");
  RawTeX!("\\def\\lx@put@s{\\@ifnextchar(\\lx@put@cb\\lx@put@b}");    // ( -> coords; else {angle}|{body}
  RawTeX!("\\def\\lx@put@opt[#1]{\\def\\lx@ps@refpoint{[#1]}\\lx@put@s}"); // [refpoint] -> continue
  RawTeX!("\\def\\lx@put@start{\\def\\lx@ps@refpoint{}\\@ifnextchar[\\lx@put@opt\\lx@put@s}");
  RawTeX!("\\def\\rput{\\@ifstar\\lx@put@start\\lx@put@start}");
  RawTeX!("\\def\\lx@uput@parens#1(#2)#3{\\lx@ps@put(#2)\\put@end{#3}}"); // {dist}(coord){text} → placed text
  RawTeX!("\\def\\lx@uput@bracket[#1]{\\lx@uput@parens}");
  RawTeX!("\\def\\uput{\\@ifstar\\lx@uput@i\\lx@uput@i}");
  RawTeX!("\\def\\lx@uput@i{\\@ifnextchar[\\lx@uput@bracket{\\lx@uput@parens}}");
  // \cput shares the runaway-safe reader (same delimited-`(` hazard).
  RawTeX!("\\def\\cput{\\@ifstar\\lx@put@start\\lx@put@start}");
  def_macro_noop("\\multirput[]{}{}{}{}")?;

  // 10 Grids — Perl :826-843 `DefSimplePSConstructor('\psgrid', 'OptionalPSCoord
  // OptionalPSCoord OptionalPSCoord', …)`. Its attributes are not `ltx:grid`
  // attributes in the schema (LaTeXML-picture.rnc), so the model drops them
  // and the element stays bare, as in Perl. Perl reads no `*`/`[<params>]`,
  // so `\psgrid[subgriddiv=1](0,0)(2,2)` printed its options and corners as
  // text, and that text swallowed the rest of the picture into an
  // `ltx:text`; the `DefPSConstructor` macro here consumes them.
  def_ps_object_macro("\\psgrid")?;
  DefConstructor!("\\psgrid@ OptionalMatch:* OptionalPSCoord OptionalPSCoord OptionalPSCoord",
    "?#draw(<ltx:grid x0='#x0' y0='#y0' x1='#x1' y1='#y1' x2='#x2' y2='#y2'\
      xunit='#xunit' yunit='#yunit'/>)",
    alias => "\\psgrid",
    after_digest => sub[whatsit] {
      after_ps_object(whatsit, "\\psgrid", &["xunit", "yunit", "gridwidth", "gridcolor", "griddots",
        "gridlabels", "gridlabelcolor", "subgriddiv", "subgridwidth", "subgridcolor", "subgriddots"], |whatsit| {
        let mut corners = Vec::new();
        for n in 2..=4 {
          match ps_arg_coord(whatsit.get_arg(n)) {
            Some(PsCoord::Point(x, y)) => corners.push((x, y)),
            Some(PsCoord::Unresolved) => return Ok(false),
            None => break,
          }
        }
        let zero = (0.0, 0.0);
        let (p0, p1, p2) = match corners.as_slice() {
          [] => {
            let (ux, uy) = ps_units()?;
            (zero, zero, (ux * 10.0, uy * 10.0))
          },
          [p0] => (zero, zero, *p0),
          [p0, p1] => (*p0, *p0, *p1),
          [p0, p1, p2, ..] => (*p0, *p1, *p2),
        };
        for (key, value) in [("x0", p0.0), ("y0", p0.1), ("x1", p1.0), ("y1", p1.1), ("x2", p2.0), ("y2", p2.1)] {
          whatsit.set_property(key, Stored::from(ps_fmt_px(ps_px(value))));
        }
        Ok(true)
      })
    });
  def_macro_noop("\\psaxes[]")?;

  // Custom object and clip — Perl L1000-1057
  def_macro_noop("\\pscustom[]")?;
  // Perl pstricks_support.sty.ltxml L996:
  //   DefEnvironment('{psclip} {}',
  //     '<ltx:clip> <ltx:clippath> #1 </ltx:clippath> #body </ltx:clip>');
  // Prior Rust stubbed both \psclip and \endpsclip to empty DefMacro,
  // losing the <ltx:clip>/<ltx:clippath> wrapping structure entirely.
  // Port as DefEnvironment matching Perl's signature and template.
  // (ltx:clip + ltx:clippath are declared in picture.rnc schema.)
  DefEnvironment!("{psclip}{}",
    "<ltx:clip><ltx:clippath>#1</ltx:clippath>#body</ltx:clip>");

  // Perl pstricks_support.sty.ltxml L995:
  //   DefConstructor('\clipbox [PSDimension] {}',
  //     "<ltx:g bclip='&ptValue(#1)'>#2</ltx:g>");
  // `bclip` is not an `ltx:g` attribute in the schema (LaTeXML-picture.rnc
  // PictureGroup.attributes), so the model drops it, in Perl too: the
  // wrapper keeps the <ltx:g> nesting and passes the raw optional argument.
  DefConstructor!("\\clipbox[]{}", "<ltx:g bclip='#1'>#2</ltx:g>");

  // Arrow tips
  def_macro_noop("\\psoverlay{}")?;
  // `\pst@getangle`, `\pst@number`, `\pst@coor` are pstricks.tex's own VALUE
  // helpers (raw-loaded by pstricks_sty.rs: `\pst@getangle#1#2{\pst@@getangle
  // {#1}\let#2\pst@angle}` :738, `\pst@number` :527 → `\pst@@dimtonum` :265,
  // `\pst@coor` :723 — `\edef`/`\let` into `\psk@…`, no driver output). Perl
  // never defines them: its `\psset` constructor never runs a key body. Rule
  // for this binding: a stub may shadow a raw pstricks helper only when that
  // helper emits driver output (`\pst@object`/`\pstVerb`/`\special`/
  // `\addto@pscode`) with no rendering fallback; a stub over a pure value
  // computer is a bug. With the
  // real `\psset` every angle/length key runs `\pst@getangle{#1}\psk@…`
  // (pst-coil.tex:49-50, pst-node's angleA/arcangleA, pst-3dplot's
  // viewangle): a one-argument stub ate the value and left the target macro
  // undefined (sweep #48 flips: hexgame, xcolor2, srdp-mathematik, ffslides,
  // vocaltract, seminar ×2). Nothing to define here. Guard:
  // `perfect_kernel_batch56::psset_angle_keys_run_the_raw_helpers`.

  // Perl pstricks_support.sty.ltxml L1042-1055: color shorthands. pstricks
  // re-binds these CSes (usually provided by color.sty / xcolor.sty as the
  // named colors) so that `\blue`, `\red`, etc. in figure/node text resolve
  // to a `\color{…}` call. Arxiv 1107.3732 uses `\node[…]{\blue{\small …}}`
  // inside `\tikzpicture`; without these, `\blue` is undefined and errors.
  // Extra length registers — Perl L411-419.
  DefRegister!("\\psframesep" => Dimension!("3pt"));
  DefRegister!("\\pslabelsep" => Dimension!("5pt"));
  DefRegister!("\\psdotsize"  => Dimension!("2pt"));
  DefRegister!("\\psrunit"    => Dimension!("1cm"));

  // Color definition shorthands. pstricks.sty:187-207 defines BOTH the color
  // and a switch macro: `\newrgbcolor{n}{r g b}` = `\gdef\n{\color{n}}` +
  // registration, and documents use `{\n text}` (ffslides-doc.tex:58-59,
  // header-example.txt). Perl's pstricks_support.sty.ltxml:570-573 has only the
  // `\definecolor` half — dead code there, since Perl raw-loads pstricks.sty
  // and the raw `\def` wins; here bindings outrank raw, so the port must be
  // the winning definition. Batch 56bh.
  DefMacro!("\\newgray{}{}",      "\\expandafter\\gdef\\csname #1\\endcsname{\\color{#1}}\\definecolor{#1}{gray}{#2}");
  DefMacro!("\\newrgbcolor{}{}",  "\\expandafter\\gdef\\csname #1\\endcsname{\\color{#1}}\\definecolor{#1}{rgb}{#2}");
  DefMacro!("\\newhsbcolor{}{}",  "\\expandafter\\gdef\\csname #1\\endcsname{\\color{#1}}\\definecolor{#1}{hsb}{#2}");
  DefMacro!("\\newcmykcolor{}{}", "\\expandafter\\gdef\\csname #1\\endcsname{\\color{#1}}\\definecolor{#1}{cmyk}{#2}");

  // Length helpers — Perl L641-651. Perl's closing `Let('\pssetlength',
  // '\setlength')` / `Let('\psaddtolength', '\addtolength')` (:650-651) are
  // NOT ported, nor its `PSDimension` primitives (:641-648): Perl's own
  // `\psset` reads its lengths through `PSDimension` keys (:590 `linewidth`,
  // :591 `linearc`, …) and never through `\pssetlength`, but here the raw
  // `\psset` runs the key bodies, `\pssetlength\pslinewidth{#1}`
  // (pstricks.tex:1078), and the `\setlength` alias made `linewidth=0.05`
  // 0.05pt with two "Illegal unit" warnings instead of pdflatex's (and
  // Perl's) 0.05cm. `\pssetlength` and `\psaddtolength` stay the raw
  // pstricks.tex definitions (`\pssetlength` :668-672, redefined by
  // `\SpecialCoor` :790-796, in force after :806; `\psaddtolength` :673),
  // which read a bare number in `\psunit`;
  // the raw `\psaddtolength` ends its `\advance` with `\afterassignment
  // \pstunit@off`, which `\advance` fires since batch 56is (lsc.sty adds
  // twice in a row). Guard:
  // `pstricks_drawing::psset_lengths_read_bare_numbers_in_psunit`.

  // Angle units — Perl L653-654: `\degrees[<full circle>]` (360 without
  // the optional argument, pstricks.tex:763; Perl stores the missing Float,
  // which its `PSAngle` then reads as 1, scaling every angle by 360) and
  // `\radians` (2π; Perl's `Float(6.28319)`), consulted by `read_ps_angle`.
  // Floats: `Stored::from(f64)` is `Stored::Number` of the floor, which made
  // `\radians` 6 (3.14159 radians came out as 188.5 degrees).
  DefPrimitive!("\\degrees []", sub[(angle)] {
    let v = angle
      .as_ref()
      .and_then(|t| t.to_string().trim().parse::<f64>().ok())
      .unwrap_or(360.0);
    AssignValue!("\\degrees" => Stored::Float(Float(v)), None);
  });
  DefPrimitive!("\\radians", {
    AssignValue!("\\degrees" => Stored::Float(Float(std::f64::consts::TAU)), None);
  });

  // Coordinate modes — Perl L1037-1038 makes `\SpecialCoor`/`\NormalCoor`
  // no-ops, which is sound only beside its own `\pssetlength` primitive.
  // Here `\pssetlength` is the raw one (above), and the two modes are the raw
  // pstricks.tex definitions (:746-806) that switch it: pst-poly.tex:68-71
  // sets `\NormalCoor` around `\pssetlength{\PstPoly@IntermediatePointDim}
  // {\PstPoly@IntermediatePoint}`, because the `\SpecialCoor` form
  // (`\special@length#1#2\@nil#3`, :795, :1001) cannot take the empty
  // default: under a no-op `\NormalCoor` it read past its `\@nil` (hexgame
  // 0 → 1 error, "\@nil undefined"). They are pure value state (they
  // redefine `\pssetlength`, `\pst@@getcoor`, `\pst@@getangle`, `\psput@`),
  // so a stub over them is a bug (the rule at `\pst@getangle` above). The
  // drawing objects read every coordinate form `\SpecialCoor` accepts
  // (`ps_coord_of`), its default since pstricks.tex:806. Guard:
  // `pstricks_drawing::pstricks_coordinate_modes_are_the_raw_ones`.
  // Perl L1039.
  def_macro_noop("\\PSTricksOff")?;

  // Rotation constructors — Perl pstricks_support.sty.ltxml L1002-1006.
  // Produce <ltx:g> SVG-rotate wrappers. `bounded => 1` matches Perl —
  // restricts the body's group scope so subsequent text outside the
  // rotate doesn't inherit the wrapper's font/color side effects.
  // (Perl additionally calls `ackTransform('rotate(...)')` in
  // beforeDigest to compose the rotation onto the
  // `_psActiveTransform` state used downstream by ps-coordinate math
  // — that piece needs the Transform/_psActiveTransform infrastructure
  // which doesn't yet exist on the Rust side; deferred. See
  // pstricks_support.sty.ltxml `sub ackTransform`.)
  DefConstructor!(
    "\\rotateleft{}",
    "<ltx:g transform='rotate(90)'>#1</ltx:g>",
    bounded => true
  );
  DefConstructor!(
    "\\rotateright{}",
    "<ltx:g transform='rotate(-90)'>#1</ltx:g>",
    bounded => true
  );
  DefConstructor!(
    "\\rotatedown{}",
    "<ltx:g transform='rotate(180)'>#1</ltx:g>",
    bounded => true
  );

  // Perl pstricks_support.sty.ltxml L1009-1012: \@@@ackscale and the
  // \scalebox / \@@scalebox / \@@@scalebox trio that threads a scale
  // transform through the PSTricks _psActiveTransform tracker. The
  // tracker (ackTransform) isn't ported to Rust — it lives only in
  // PSTricks post-processing which Rust doesn't implement. Ship
  // \@@@ackscale as a no-op consume-the-arg stub so documents invoking
  // `\scalebox{0.5}{body}` via PSTricks don't hit undefined-CS.
  // `\scalebox` itself is provided by graphics_sty (standard LaTeX form);
  // not overriding it here keeps the tested scalebox golden intact.
  // Intentional DefPrimitive → DefMacro (WISDOM #44, same DVI-only
  // blocker as L18-31 umbrella).
  def_macro_noop("\\@@@ackscale{}")?;

  DefMacro!("\\black", "\\color{black}");
  DefMacro!("\\darkgray", "\\color{darkgray}");
  DefMacro!("\\gray", "\\color{gray}");
  DefMacro!("\\lightgray", "\\color{lightgray}");
  DefMacro!("\\white", "\\color{white}");
  DefMacro!("\\blue", "\\color{blue}");
  DefMacro!("\\red", "\\color{red}");
  DefMacro!("\\green", "\\color{green}");
  DefMacro!("\\yellow", "\\color{yellow}");
  DefMacro!("\\magenta", "\\color{magenta}");
  DefMacro!("\\cyan", "\\color{cyan}");

  // Rotation wrappers — Perl pstricks_support.sty.ltxml L1002-1008.
  // Perl runs `ackTransform(...)` in before_digest to track the
  // transform (the tracker is not ported, see "Transform management"
  // above). Drop the before_digest and emit the <ltx:g> element directly.
  DefConstructor!("\\rotateleft{}",  "<ltx:g transform='rotate(90)'>#1</ltx:g>",
    bounded => true);
  DefConstructor!("\\rotateright{}", "<ltx:g transform='rotate(-90)'>#1</ltx:g>",
    bounded => true);
  DefConstructor!("\\rotatedown{}",  "<ltx:g transform='rotate(180)'>#1</ltx:g>",
    bounded => true);
  // Perl pstricks_support.sty.ltxml L1010-1012: unconditional redef of
  // `\scalebox{}{}`. Body expands to `\@@@ackscale{factor}` (already a
  // no-op consume above) + `\@@scalebox{factor}{body}`, where
  // `\@@scalebox` is `\let` to `\@secondoftwo` (returns the body). Net
  // effect: `\scalebox{0.6}{T}` renders as `T` (scaling dropped — this
  // matches Perl's effective behavior since the DVI-only
  // `\@@@scalebox` constructor never fires in non-PSTricks contexts).
  //
  // Earlier comment said this would shadow `graphics.sty`'s fully-
  // featured `\scalebox{} []{}` and break tested goldens. Empirically
  // safe: the goldens that exercise `\scalebox[ratio]{...}` syntax
  // (graphrot, vmode) load graphicx WITHOUT pstricks, so pstricks_support
  // never runs for them. Papers that load pstricks WITHOUT graphics
  // (1208.6481 stage 14 RUST-REGRESSION witness — `\def\tra{\scalebox{.6}{T}}`)
  // need the fallback or hit `Error:undefined:\scalebox`.
  //
  // Perl L1010 is unconditional; we mirror that. If graphicx is loaded
  // AFTER pstricks, graphicx's fancier form wins (later assignment).
  // If loaded BEFORE, this pstricks form overwrites (matching Perl).
  DefMacro!("\\scalebox{}{}", "#2");
});
