//! pict2e.sty — extended LaTeX picture environment.
//!
//! pict2e enhances the standard LaTeX picture environment with arbitrary-
//! slope `\line` / `\vector`, smooth curves, quadratic/cubic Bezier
//! handling and a PostScript-like path interface. LaTeX picture is rendered
//! as XML/SVG in our pipeline regardless of driver, so the driver-dispatch
//! chain (p2e-pdftex.def, p2e-dvips.def, …) has no XML meaning and is not
//! raw-loaded; the user-facing commands are bound here.
//!
//! Under `\pdfoutput=0` (pdftex.rs:11, and Perl) pict2e.cfg would pick the
//! dvips driver and pict2e.sty:160 `\ifnum\pIIe@mode>\z@` would define the
//! path interface `\moveto`/`\lineto`/`\curveto`/`\circlearc`/`\closepath`/
//! `\strokepath`/`\fillpath` (pict2e.sty:742-774). An earlier stub skipped
//! them ("No suitable driver" was a stale reading), so fancyqr-doc and
//! curve2e-manual (curve2e.sty:19 requires pict2e, :92 `\let\originalmoveto
//! \moveto`) failed on `undefined:\moveto` while same-host Perl (raw load)
//! converted with an empty picture. Here the path is accumulated per subpath
//! and stroked/filled as `ltx:line` polylines (curves and arcs sampled), which
//! renders what Perl drops. Witness 2503.14673 (pict2e error blocking 1 paper).
use std::cell::RefCell;

use latexml_core::state::lookup_register;

use crate::prelude::*;

thread_local! {
  /// The current pict2e path: one point list per subpath (`\moveto` opens one),
  /// in `\unitlength` multiples. `\strokepath`/`\fillpath` consume it.
  static PICT2E_PATH: RefCell<Vec<Vec<(f64, f64)>>> = const { RefCell::new(Vec::new()) };
}

/// A picture coordinate: pict2e's `\@defaultunitsset` takes a bare number
/// (× `\unitlength`) or a dimension with a unit; the latter is converted to
/// `\unitlength` multiples so the polyline scaling stays uniform.
fn pic_coord(tokens: Tokens) -> Result<f64> {
  let text = Expand!(tokens).to_string();
  let text = text.trim();
  if let Ok(v) = text.parse::<f64>() {
    return Ok(v);
  }
  let split = text
    .find(|c: char| c.is_ascii_alphabetic())
    .unwrap_or(text.len());
  let (num, unit) = text.split_at(split);
  let num: f64 = num.trim().parse().unwrap_or(0.0);
  if unit.is_empty() {
    return Ok(num);
  }
  let unit_pt = convert_unit(unit.trim()) / 65536.0;
  let unitlength = match lookup_register("\\unitlength", Vec::new())? {
    Some(RegisterValue::Dimension(d)) => d.pt_value(None),
    _ => 1.0,
  };
  Ok(if unitlength == 0.0 {
    0.0
  } else {
    num * unit_pt / unitlength
  })
}

fn path_push(point: (f64, f64), new_subpath: bool) {
  PICT2E_PATH.with(|p| {
    let mut p = p.borrow_mut();
    if new_subpath || p.is_empty() {
      p.push(Vec::new());
    }
    p.last_mut().expect("subpath").push(point);
  });
}

fn path_current() -> Option<(f64, f64)> {
  PICT2E_PATH.with(|p| p.borrow().last().and_then(|s| s.last().copied()))
}

/// Emit one `\lx@pic@polyline{}{closed}(x,y)…` per subpath and clear the path.
fn path_flush(closed: &str) -> Result<Tokens> {
  let subpaths = PICT2E_PATH.with(|p| std::mem::take(&mut *p.borrow_mut()));
  let mut out = Vec::new();
  for sub in subpaths.iter().filter(|s| s.len() > 1) {
    let mut pairs = String::new();
    for (x, y) in sub {
      pairs.push_str(&format!("({x},{y})"));
    }
    out.extend(
      Invocation!(T_CS!("\\lx@pic@polyline"), vec![
        Tokens!(),
        Tokens::new(ExplodeText!(closed))
      ])
      .unlist(),
    );
    out.extend(ExplodeText!(&pairs));
  }
  Ok(Tokens::new(out))
}

LoadDefinitions!({
  def_macro_noop("\\OriginalPictureCmds")?;
  // pict2e.sty:686-740 — the polygonal-line family that curve2e.sty:240
  // `\renewcommand*`s (sapthesis-doc, unifith-doc: `\polyline` undefined).
  // `\lx@pic@polyline{terminators}{closed}` (latex_constructs.rs) reads the
  // `(x,y)…` pairs and emits one `ltx:line`.
  DefMacro!("\\polyline []", "\\lx@pic@polyline{}{0}");
  DefMacro!("\\Line", "\\lx@pic@polyline{}{0}");
  DefMacro!("\\polyvector", "\\lx@pic@polyline{->}{0}");
  DefMacro!("\\Vector", "\\lx@pic@polyline{->}{0}");
  DefMacro!("\\polygon OptionalMatch:*", "\\lx@pic@polyline{}{1}");
  def_macro_noop("\\pIIe@vector@ltx")?;
  def_macro_noop("\\pIIe@vector@pst")?;

  // The path interface, pict2e.sty:742-774 (mode > 0 branch). Guard:
  // `perfect_kernel_batch56::pict2e_path_interface_strokes_a_polyline`.
  DefPrimitive!("\\moveto Match:( Until:, Until:)", sub[(_open, x, y)] {
    path_push((pic_coord(x)?, pic_coord(y)?), true);
    Ok(Vec::new())
  });
  DefPrimitive!("\\lineto Match:( Until:, Until:)", sub[(_open, x, y)] {
    path_push((pic_coord(x)?, pic_coord(y)?), false);
    Ok(Vec::new())
  });
  // Cubic Bezier from the current point, sampled (pict2e.sty:754 `\curveto`).
  DefPrimitive!(
    "\\curveto Match:( Until:, Until:) Match:( Until:, Until:) Match:( Until:, Until:)",
    sub[(_o1, x1, y1, _o2, x2, y2, _o3, x3, y3)] {
      let (x0, y0) = path_current().unwrap_or((0.0, 0.0));
      let (x1, y1) = (pic_coord(x1)?, pic_coord(y1)?);
      let (x2, y2) = (pic_coord(x2)?, pic_coord(y2)?);
      let (x3, y3) = (pic_coord(x3)?, pic_coord(y3)?);
      for i in 1..=8 {
        let t = f64::from(i) / 8.0;
        let u = 1.0 - t;
        let x = u * u * u * x0 + 3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t * x3;
        let y = u * u * u * y0 + 3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t * y3;
        path_push((x, y), false);
      }
      Ok(Vec::new())
    }
  );
  // `\circlearc[mode]{cx}{cy}{r}{start}{end}` (pict2e.sty:765, `\pIIe@arc`):
  // mode 0 = a new subpath at the arc's start, 1 = `\lineto` the start,
  // 2 = the arc continues the current subpath. Angles in degrees.
  DefPrimitive!("\\circlearc [Number]{}{}{}{}{}", sub[(mode, cx, cy, r, start, end)] {
    let mode = mode.value_of();
    let (cx, cy, r) = (pic_coord(cx)?, pic_coord(cy)?, pic_coord(r)?);
    let start = pic_coord(start)?;
    let end = pic_coord(end)?;
    let sweep = end - start;
    let steps = ((sweep.abs() / 10.0).ceil() as usize).max(1);
    for i in 0..=steps {
      let a = (start + sweep * (i as f64) / (steps as f64)).to_radians();
      let point = (cx + r * a.cos(), cy + r * a.sin());
      if i == 0 && mode == 2 && path_current().is_some() {
        continue;
      }
      path_push(point, i == 0 && mode == 0);
    }
    Ok(Vec::new())
  });
  DefPrimitive!("\\closepath", sub[()] {
    PICT2E_PATH.with(|p| {
      let mut p = p.borrow_mut();
      if let Some(sub) = p.last_mut()
        && let Some(first) = sub.first().copied()
      {
        sub.push(first);
      }
    });
    Ok(Vec::new())
  });
  DefMacro!("\\strokepath", sub[()] { path_flush("0") });
  DefMacro!("\\fillpath", sub[()] { path_flush("1") });
});
