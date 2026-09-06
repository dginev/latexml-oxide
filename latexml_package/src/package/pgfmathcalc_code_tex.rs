//! pgfmathcalc.code.tex — PGF math calculation macros
//! Perl: pgfmathcalc.code.tex.ltxml (34 lines)
//!
//! Loads the raw TeX code and provides \pgfmathsetmacro in Rust.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L20: Load pgf's TeX code for math calc first
  InputDefinitions!("pgfmathcalc.code", extension => Some(Cow::Borrowed("tex")), noltxml => true);

  // Perl L24-32: \pgfmathsetmacro — evaluates expression and defines macro.
  // Perl's DefMacroI call explicitly passes `scope => 'local'`, so the
  // new CS vanishes when the enclosing group closes — essential for
  // tikz/pgf loops that reuse the same CS (e.g. \foreach \n …). The
  // Rust port had dropped the scope, leaking redefinitions upward past
  // the begingroup/endgroup wrapper. Restore Scope::Local to match Perl.
  DefPrimitive!("\\pgfmathsetmacro{}{}", sub[(cs, expression)] {
    let expr_str = do_expand(expression)?.to_string();
    bgroup();
    let result = pgfmath_code_tex::pgfmathparse_eval(&expr_str);
    egroup()?;
    let result_tokens = mouth::tokenize_internal(TeXString::assembled(result));
    def_macro(
      cs.unlist().into_iter().next().unwrap_or_else(|| T_CS!("\\pgfmathresult")),
      None,
      result_tokens,
      Some(ExpandableOptions { scope: Some(Scope::Local), ..Default::default() }))?;
    Ok(Vec::new())
  });

  // pgfmathcalc.code.tex:366-468 `\pgfmathpointintersectionoflineandarc`:
  // pgf bisects the arc's parametric angle until the angle from #2 to the
  // arc point EQUALS (`\ifdim\x pt=\q pt`, :447) the angle from #2 to #1 in
  // its own fixed-point trig — the loop's only exit. Our trig is float
  // (pgfmath_code_tex.rs `\pgfmathcos@`/`sin@`/`atantwo@`), so the two
  // never meet exactly and a rounded-rectangle corner border query (a
  // self-loop wire — tikz-cd `\ar[loop]`, zx-calculus `\zxLoopAboveDots`;
  // callout nodes, arXiv 2201.09268) spun 2 boxes per iteration into the
  // cycle fatal. The intersection is closed form: the ray from #2 toward #1
  // meets the ellipse (center #3, radii #6) at a quadratic's roots; the root
  // on the arc (nearest to it otherwise — pgf's "best estimate" `\n`) is the
  // parametric angle. Same point set as the bisection's limit, no loop.
  // Guard: `perfect_kernel_batch56::line_and_arc_intersection_is_closed_form`.
  RawTeX!(r"\def\pgfmathpointintersectionoflineandarc#1#2#3#4#5#6{%
  \pgf@process{%
    \pgfmathanglebetweenpoints{#2}{#1}%
    \let\x\pgfmathresult
    \pgfmath@in@{and }{#6}%
    \ifpgfmath@in@\pgf@polar@#6\@@\else\pgf@polar@#6 and #6\@@\fi
    \edef\xarcradius{\the\pgf@x}%
    \edef\yarcradius{\the\pgf@y}%
    \pgfmathsetmacro\s{#4}%
    \pgfmathsetmacro\e{#5}%
    \pgf@process{#2}\edef\lx@lineandarc@q{\the\pgf@x,\the\pgf@y}%
    \pgf@process{#3}\edef\lx@lineandarc@c{\the\pgf@x,\the\pgf@y}%
    \lx@pgf@lineandarc
    \pgfpointadd{#3}{\pgfpointpolar{\n}{\xarcradius and \yarcradius}}%
  }%
}");
  DefPrimitive!("\\lx@pgf@lineandarc", sub[_args] {
    fn num(s: &str) -> f64 { s.trim().trim_end_matches("pt").trim().parse::<f64>().unwrap_or(0.0) }
    fn pair(s: &str) -> (f64, f64) {
      let mut it = s.split(',');
      (num(it.next().unwrap_or("0")), num(it.next().unwrap_or("0")))
    }
    let expand = |cs: &str| -> Result<String> { Ok(do_expand(Tokens!(T_CS!(cs)))?.to_string()) };
    let x = num(&expand("\\x")?);
    let s = num(&expand("\\s")?);
    let e = num(&expand("\\e")?);
    let rx = num(&expand("\\xarcradius")?).abs().max(1e-6);
    let ry = num(&expand("\\yarcradius")?).abs().max(1e-6);
    let (qx, qy) = pair(&expand("\\lx@lineandarc@q")?);
    let (cx, cy) = pair(&expand("\\lx@lineandarc@c")?);
    // pgfmathcalc.code.tex:395-406: the arc's angles mod 360, straddling zero
    // when the end comes before the start.
    let norm = |t: f64| t.rem_euclid(360.0);
    let (ss, ee) = (norm(s), norm(e));
    let on_arc = |t: f64| if ee < ss { t >= ss || t <= ee } else { ss <= t && t <= ee };
    let arc_distance = |t: f64| {
      if on_arc(t) { 0.0 } else {
        let d = |a: f64| { let m = (t - a).rem_euclid(360.0); m.min(360.0 - m) };
        d(ss).min(d(ee))
      }
    };
    let (dx, dy) = (x.to_radians().cos(), x.to_radians().sin());
    let (u, v, a, b) = ((qx - cx) / rx, (qy - cy) / ry, dx / rx, dy / ry);
    let (qa, qb, qc) = (a * a + b * b, 2.0 * (u * a + v * b), u * u + v * v - 1.0);
    let disc = qb * qb - 4.0 * qa * qc;
    // The default `\n`: the middle of the arc (:412-413). A ray that MISSES
    // the arc (the shapes' `\anchorborder` never sends one — it takes the
    // straight edges through `\pgfpointintersectionoflines`) gets the true
    // off-arc root nearest the arc where pgf's bisection would clamp inside
    // [#4,#5]; both are "meaningless points" per pgf's own comment (:356).
    let mut n = (s + e) / 2.0;
    if disc >= 0.0 && qa > 0.0 {
      let sq = disc.sqrt();
      let mut best: Option<(bool, f64, f64)> = None; // (forward ray, distance to the arc, angle)
      for lam in [(-qb + sq) / (2.0 * qa), (-qb - sq) / (2.0 * qa)] {
        let (px, py) = (qx + lam * dx, qy + lam * dy);
        let t = norm(((py - cy) / ry).atan2((px - cx) / rx).to_degrees());
        let cand = (lam >= 0.0, arc_distance(t), t);
        let better = match best {
          None => true,
          Some((fwd, dist, _)) => (cand.0 && !fwd) || (cand.0 == fwd && cand.1 < dist),
        };
        if better { best = Some(cand); }
      }
      if let Some((_, _, t)) = best { n = t; }
    }
    let text = format!("{n:.5}");
    let text = text.trim_end_matches('0').trim_end_matches('.').to_string();
    def_macro(
      T_CS!("\\n"),
      None,
      mouth::tokenize_internal(TeXString::assembled(text)),
      Some(ExpandableOptions { scope: Some(Scope::Local), ..Default::default() }))?;
    Ok(Vec::new())
  });
});
