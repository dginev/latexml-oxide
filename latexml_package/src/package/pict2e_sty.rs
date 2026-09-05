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

/// `\unitlength` in pt (1.0 when unset), for dimension → picture-unit conversion.
fn unitlength_pt() -> Result<f64> {
  Ok(match lookup_register("\\unitlength", Vec::new())? {
    Some(RegisterValue::Dimension(d)) => d.pt_value(None),
    _ => 1.0,
  })
}

/// An angle argument (`\pIIe@arc … {start}{end}`): a plain or macro-held number.
fn pic_number(tokens: Tokens) -> Result<f64> {
  let text = Expand!(tokens).to_string();
  Ok(text.trim().parse().unwrap_or(0.0))
}

/// A driver-level `\pIIe@*` argument is a real dimension (pict2e.sty:267-308
/// `\pIIe@add@CP{#1}{#2}`): convert to `\unitlength` multiples.
fn pic_dim(d: Dimension) -> Result<f64> {
  let unit = unitlength_pt()?;
  Ok(if unit == 0.0 {
    0.0
  } else {
    d.pt_value(None) / unit
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

/// Cubic Bezier from the current point, sampled into the current subpath.
fn path_curveto(c1: (f64, f64), c2: (f64, f64), end: (f64, f64)) {
  let (x0, y0) = path_current().unwrap_or((0.0, 0.0));
  for i in 1..=8 {
    let t = f64::from(i) / 8.0;
    let u = 1.0 - t;
    let x = u * u * u * x0 + 3.0 * u * u * t * c1.0 + 3.0 * u * t * t * c2.0 + t * t * t * end.0;
    let y = u * u * u * y0 + 3.0 * u * u * t * c1.1 + 3.0 * u * t * t * c2.1 + t * t * t * end.1;
    path_push((x, y), false);
  }
}

/// `\pIIe@arc[mode]{cx}{cy}{r}{start}{end}` (pict2e.sty:765): mode 0 = a new
/// subpath at the arc's start, 1 = `\lineto` the start, 2 = the arc continues
/// the current subpath. Angles in degrees, sampled every ≤10°.
fn path_arc(mode: i64, (cx, cy, r): (f64, f64, f64), start: f64, end: f64) {
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
}

fn path_close() {
  PICT2E_PATH.with(|p| {
    let mut p = p.borrow_mut();
    if let Some(sub) = p.last_mut()
      && let Some(first) = sub.first().copied()
    {
      sub.push(first);
    }
  });
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

  // The driver level `\pIIe@moveto{dim}{dim}` … (pict2e.sty:267-308) feeds a
  // subpath accumulator; every argument is a real dimension read by the
  // engine (`{Dimension}`), so register and `\dimexpr` coordinates
  // (FramedSyntax.sty:189 `\moveto(\SIXR,\SIYD+#4)`) scale correctly — an
  // earlier string parse of the expansion read a register name as 0 (141
  // zero-point frames in curve2e-manual). Guards:
  // `perfect_kernel_batch56::{pict2e_path_interface_strokes_a_polyline,
  // curve2e_raw_load_renders_arcs_and_vectors}`.
  DefPrimitive!("\\pIIe@moveto {Dimension}{Dimension}", sub[(x, y)] {
    path_push((pic_dim(x)?, pic_dim(y)?), true);
    Ok(Vec::new())
  });
  DefPrimitive!("\\pIIe@lineto {Dimension}{Dimension}", sub[(x, y)] {
    path_push((pic_dim(x)?, pic_dim(y)?), false);
    Ok(Vec::new())
  });
  DefPrimitive!(
    "\\pIIe@curveto {Dimension}{Dimension}{Dimension}{Dimension}{Dimension}{Dimension}",
    sub[(x1, y1, x2, y2, x3, y3)] {
      let c1 = (pic_dim(x1)?, pic_dim(y1)?);
      let c2 = (pic_dim(x2)?, pic_dim(y2)?);
      let end = (pic_dim(x3)?, pic_dim(y3)?);
      path_curveto(c1, c2, end);
      Ok(Vec::new())
    }
  );
  DefPrimitive!("\\pIIe@arc [Number]{Dimension}{Dimension}{Dimension}{}{}", sub[(mode, cx, cy, r, start, end)] {
    let centre = (pic_dim(cx)?, pic_dim(cy)?, pic_dim(r)?);
    path_arc(mode.value_of(), centre, pic_number(start)?, pic_number(end)?);
    Ok(Vec::new())
  });
  DefPrimitive!("\\pIIe@closepath", sub[()] {
    path_close();
    Ok(Vec::new())
  });
  DefMacro!("\\pIIe@strokeGraph", sub[()] { path_flush("0") });
  DefMacro!("\\pIIe@fillGraph", sub[()] { path_flush("1") });
  // The user level, verbatim pict2e.sty:742-774 (the mode > 0 branch a
  // driver enables): `\@defaultunitsset` scales a bare number by
  // `\unitlength` and passes a dimension through. Plain macros, so
  // curve2e.sty:92-96 can `\let\originalmoveto\moveto` and wrap them for its
  // macro-pair `(\P)` form; `\lx@pictii@*` are stable aliases of the same.
  RawTeX!(
    r"\ifx\undefined\pIIe@tempdima \newdimen\pIIe@tempdima \fi
\ifx\undefined\pIIe@tempdimb \newdimen\pIIe@tempdimb \fi
\ifx\undefined\pIIe@tempdimc \newdimen\pIIe@tempdimc \fi
\ifx\undefined\pIIe@tempdimd \newdimen\pIIe@tempdimd \fi
\ifx\undefined\pIIe@tempdime \newdimen\pIIe@tempdime \fi
\ifx\undefined\pIIe@tempdimf \newdimen\pIIe@tempdimf \fi
\def\moveto(#1,#2){\@killglue
  \@defaultunitsset\pIIe@tempdima{#1}\unitlength
  \@defaultunitsset\pIIe@tempdimb{#2}\unitlength
  \pIIe@moveto{\pIIe@tempdima}{\pIIe@tempdimb}\ignorespaces}
\def\lineto(#1,#2){\@killglue
  \@defaultunitsset\pIIe@tempdima{#1}\unitlength
  \@defaultunitsset\pIIe@tempdimb{#2}\unitlength
  \pIIe@lineto{\pIIe@tempdima}{\pIIe@tempdimb}\ignorespaces}
\def\curveto(#1,#2)(#3,#4)(#5,#6){\@killglue
  \@defaultunitsset\pIIe@tempdima{#1}\unitlength
  \@defaultunitsset\pIIe@tempdimb{#2}\unitlength
  \@defaultunitsset\pIIe@tempdimc{#3}\unitlength
  \@defaultunitsset\pIIe@tempdimd{#4}\unitlength
  \@defaultunitsset\pIIe@tempdime{#5}\unitlength
  \@defaultunitsset\pIIe@tempdimf{#6}\unitlength
  \pIIe@curveto{\pIIe@tempdima}{\pIIe@tempdimb}{\pIIe@tempdimc}{\pIIe@tempdimd}{\pIIe@tempdime}{\pIIe@tempdimf}\ignorespaces}
\newcommand*\circlearc[6][0]{\@killglue
  \@defaultunitsset\pIIe@tempdima{#2}\unitlength
  \@defaultunitsset\pIIe@tempdimb{#3}\unitlength
  \@defaultunitsset\pIIe@tempdimc{#4}\unitlength
  \pIIe@arc[#1]{\pIIe@tempdima}{\pIIe@tempdimb}{\pIIe@tempdimc}{#5}{#6}\ignorespaces}
\def\closepath{\pIIe@closepath}
\def\strokepath{\pIIe@strokeGraph}
\def\fillpath{\pIIe@fillGraph}
\let\lx@pictii@moveto\moveto \let\lx@pictii@lineto\lineto \let\lx@pictii@curveto\curveto"
  );
  // pict2e.sty:78-80 arrow-head parameters, :603 `\pIIe@bezier@QtoC`,
  // :779-791 the line cap/join declarations (pdfTeX driver operators).
  RawTeX!(
    r"\newcommand*\pIIe@FAL{1.52}\newcommand*\pIIe@FAW{3.2}\newcommand*\pIIe@CAW{1.5pt}
\newcommand*\pIIe@bezier@QtoC[3]{\@tempdimc#1\relax \advance\@tempdimc-#2\relax
  \divide\@tempdimc\thr@@ \advance\@tempdimc #2\relax #3\@tempdimc}
\ifx\undefined\@arclen \newdimen\@arclen \fi
\ifx\undefined\@arcrad \newdimen\@arcrad \fi
\newcommand*\pIIe@linecap@op{J}\newcommand*\pIIe@linejoin@op{j}
\def\pIIe@linecap{}\def\pIIe@linejoin{}
\def\buttcap{\edef\pIIe@linecap{ 0 \pIIe@linecap@op}}
\def\roundcap{\edef\pIIe@linecap{ 1 \pIIe@linecap@op}}
\def\squarecap{\edef\pIIe@linecap{ 2 \pIIe@linecap@op}}
\def\miterjoin{\edef\pIIe@linejoin{ 0 \pIIe@linejoin@op}}
\def\roundjoin{\edef\pIIe@linejoin{ 1 \pIIe@linejoin@op}}
\def\beveljoin{\edef\pIIe@linejoin{ 2 \pIIe@linejoin@op}}
\def\pIIe@mode{2}"
  );
  // curve2e.sty:273-280 `\@picture` sets `\pict@dimen`/`\pict@offset` for its
  // grid defaults; the picture environment constructor (sect13.rs) records the
  // same pairs, so expose them here.
  DefMacro!(T_CS!("\\pict@dimen"), None, {
    ExplodeText!(
      &lookup_value("PICTURE_DIMEN")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "0,0".into())
    )
  });
  DefMacro!(T_CS!("\\pict@offset"), None, {
    ExplodeText!(
      &lookup_value("PICTURE_OFFSET")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "0,0".into())
    )
  });
});
