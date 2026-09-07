//! pstricks.sty — PSTricks graphics package
//! PSTricks requires DVI backend; we raw-load real pstricks.sty to set
//! up internal state (\ifpst@useCalc, \ifpst@psfonts, …) then override
//! user-facing drawing commands as HTML-friendly no-ops.
//! Perl: pstricks.sty.ltxml (44L) + pstricks_support.sty.ltxml (1057L)
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

fn ps_fmt_px(v: f64) -> String {
  if v == v.round() && v.abs() < 1e10 {
    format!("{}", v as i64)
  } else {
    format!("{v}")
  }
}

fn ps_fmt_pt(v: f64) -> String {
  if v == v.round() {
    format!("{v:.1}pt")
  } else {
    format!("{v}pt")
  }
}

fn ps_units() -> Result<(f64, f64)> {
  let unit = |name: &str| -> Result<f64> {
    Ok(match lookup_register(name, Vec::new())? {
      Some(RegisterValue::Dimension(d)) => d.pt_value(None),
      _ => 28.45274, // 1cm, pstricks_support:417
    })
  };
  Ok((unit("\\psxunit")?, unit("\\psyunit")?))
}

fn ps_pair(arg: &Option<Digested>) -> Option<(f64, f64)> {
  match arg.as_ref()?.data() {
    DigestedData::RegisterValue(RegisterValue::Pair(p)) => Some((p.x.0, p.y.0)),
    _ => None,
  }
}

/// `{pspicture}(c0)(c1)` — Perl pstricks_support.sty.ltxml:527-536: with one
/// pair, it is the far corner and the origin is (0,0); width/height are the
/// corner differences in `\psxunit`/`\psyunit`; the body is translated by the
/// negated origin (the same attributes the LaTeX `{picture}` binding emits).
fn pspicture_properties(
  first: &Option<Digested>,
  second: &Option<Digested>,
) -> Result<SymHashMap<Stored>> {
  let (ux, uy) = ps_units()?;
  let a = ps_pair(first).unwrap_or((0.0, 0.0));
  let (c0, c1) = match ps_pair(second) {
    Some(b) => (a, b),
    None => ((0.0, 0.0), a),
  };
  let (w, h) = ((c1.0 - c0.0) * ux, (c1.1 - c0.1) * uy);
  let (ox, oy) = (c0.0 * ux, c0.1 * uy);
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

/// `\lx@ps@put(x,y){body}` — the LaTeX `\put` transform in pstricks units.
fn ps_put_properties(coords: &Option<Digested>) -> Result<SymHashMap<Stored>> {
  let (ux, uy) = ps_units()?;
  let (x, y) = ps_pair(coords).unwrap_or((0.0, 0.0));
  Ok(stored_map!(
    "transform" => Stored::String(pin(format!("translate({},{})", ps_fmt_px(ps_px(x * ux)), ps_fmt_px(ps_px(y * uy)))))
  ))
}

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("xcolor");
  // Perl pstricks.sty.ltxml L40-42:
  //   InputDefinitions('pstricks', type => 'sty', noltxml => 1);
  // The raw-load executes the 7 \newifs (\ifpst@useCalc,
  // \ifpst@psfonts, \ifpstGSfonts, \if@check@engine, \ifpst@xetex,
  // \ifpst@autopdf, \ifpst@distiller) plus the option processing
  // machinery. Downstream pstricks.tex (line 1228 \ifpst@useCalc)
  // depends on these being defined. Without the raw-load the hand-
  // stub left \ifpst@useCalc/\ifpst@psfonts undefined; witness:
  // 1907.03162 (`\usepackage{pstricks, pst-plot, pst-eps, pst-grad}`
  // → 2 Error:undefined diagnostics, vs Perl 0 errors).
  InputDefinitions!("pstricks", extension => Some(Cow::Borrowed("sty")), noltxml => true);
  // Perl pstricks.sty.ltxml L44: `RequirePackage('pstricks_support')`.
  // pstricks_support defines color-CS shorthands (`\blue`, `\red`, …)
  // that PSTricks-using papers (e.g. arxiv 1107.3732) reference inside
  // `\tikzpicture{\node{\blue{…}}}`. Without it those CSes are undefined.
  RequirePackage!("pstricks_support");

  // `\psset` is the real family-aware xkeyval one from the raw load above
  // (pst-xkey.tex:60-63 `\def\psset{\@testopt\pss@t{\pst@famlist}}`,
  // `\setkeys+[psset]{#1}{#2}`). A no-op here lost every key BODY: raw
  // pst-node.tex:1248-1257 defines its `\psk@mnodesize`/`\psk@mnode`/
  // `\psk@mcol` internals only as the side effect of `\psset[pst-node]
  // {mnodesize=-1pt,…}`, and `\psm@endnode` (:1224) then read them —
  // `{psmatrix}` under pstricks-add (dsptricks 101 errors, pst-eucl,
  // egpeirce; Perl fails the same way with its constructor `\psset`).
  // Guard: `perfect_kernel_batch56::psset_dispatches_family_key_bodies`.

  // Perl pstricks_support.sty.ltxml L849-861: `\newpsobject{name}{oldname}{keyval}`
  // dynamically defines `\<name>` to forward to `\<oldname>` with the saved
  // `<keyval>` baked into the optional argument. The paper's drawing object is
  // then drawn by the resolved `\<oldname>` (typically `\psline`, `\psdots`).
  // Perl stores `oldname` and `keyval` in two LookupValue keys so the
  // generated forwarder can read them at call time, and additionally merges
  // a user-supplied `[opt]` into the saved `keyval`.
  //
  // Witness: physics/9710028 uses
  //   \newpsobject{PST@Border}{psline}{linewidth=.0015,linestyle=solid}
  // then later calls `\PST@Border(...)`. With the prior no-op stub
  // `\PST@Border` stayed undefined and Rust errored; Perl recovered.
  DefPrimitive!("\\newpsobject{}{}{}", sub[(newname, oldname, keyval)] {
    let newcs    = s!("\\{}", newname.to_string());
    let oldcs    = s!("\\{}", oldname.to_string());
    let keystr   = keyval.to_string();
    let new_tok  = T_CS!(newcs);
    let params   = parse_parameters("OptionalMatch:* []", &new_tok, true)?;
    // Generated forwarder closure: read OptionalMatch:* and []; emit
    //   \<old>(*)([combined-key])
    // combined-key = saved-key + ',' + user-key (Perl L855).
    let oldcs_owned = oldcs;
    let key_owned   = keystr;
    let body_closure: ExpansionBody = ExpansionBody::Closure(Rc::new(move |args| {
      let star = args.first().map(|a| !a.is_none()).unwrap_or(false);
      let usr  = args.get(1)
        .and_then(|a| match a.as_tokens() { Ok(Some(t)) => Some(t.to_string()), _ => None })
        .unwrap_or_default();
      let combined = match (key_owned.is_empty(), usr.is_empty()) {
        (false, false) => s!("{},{}", key_owned, usr),
        (false, true)  => key_owned.clone(),
        (true,  false) => usr,
        (true,  true)  => String::new(),
      };
      let mut out = vec![T_CS!(oldcs_owned.clone())];
      if star { out.push(T_OTHER!("*")); }
      if !combined.is_empty() {
        out.push(T_OTHER!("["));
        out.extend(Explode!(combined));
        out.push(T_OTHER!("]"));
      }
      // Perl L856 — emit the suffix only; the paren-coords tuple is
      // consumed by the resolved \psline (or sibling) macro's own
      // `\lx@psgobble@parens` chain.
      Ok(Tokens::new(out))
    }));
    def_macro(new_tok, params, Some(body_closure), None)?;
  });

  def_macro_noop("\\newpsstyle{}{}")?;

  // PSCoordList-emulator. Perl's pstricks_support.sty.ltxml uses parameter
  // type `PSCoordList` (variable-arity `(x,y)(x,y)...`) to absorb the paren
  // tuples that follow most pstricks drawing commands. Without it those
  // tuples leak as raw text into the document — opening an `<ltx:p>` that
  // doesn't auto-close before subsequent block content (witness:
  // hep-ph0102192 minipage-in-figure failure). Recursive `\@ifnextchar`
  // idiom: peek for `(`; consume one tuple; recurse.
  RawTeX!("\\def\\lx@psgobble@parens{\\@ifnextchar({\\lx@psgobble@one}{}}");
  RawTeX!("\\def\\lx@psgobble@one(#1){\\lx@psgobble@parens}");
  // `\lx@psgobble@shape`: consume an OPTIONAL leading `{<arrows>}` brace group
  // (e.g. `\psline{->}…`), THEN the trailing `(x,y)(x,y)…` coordinate tuples.
  // The pstricks open-curve commands take an OPTIONAL arrow spec before the
  // coordinates; the previous signatures declared it as a MANDATORY `{}` arg,
  // so when arrows were absent (`\pscurve[opts](x,y)…`) the `{}` swallowed the
  // first `(` and `\lx@psgobble@parens` then saw a digit, stopped, and left the
  // remaining coordinates as stray picture text. Following an open `\put{…}`
  // `<ltx:text>`, that stray text trapped every later block in an un-closeable
  // `<ltx:text>` (witness 1112.2096). Peeking for the leading brace makes the
  // arrow spec optional without over-gobbling any trailing document braces.
  RawTeX!("\\def\\lx@psgobble@shape{\\@ifnextchar\\bgroup{\\lx@psgobble@arrows}{\\lx@psgobble@parens}}");
  RawTeX!("\\def\\lx@psgobble@arrows#1{\\lx@psgobble@parens}");

  // Drawing commands — all no-ops for HTML, but MUST consume their
  // optional `{arrows}` + trailing `(x,y)…` coordinate tuples so nothing
  // leaks as text. Open curves use `\lx@psgobble@shape` (optional arrows);
  // closed shapes have no arrows so `\lx@psgobble@parens` (coords only)
  // suffices and is equally arrow-tolerant via the peek.
  DefMacro!("\\psline OptionalMatch:* []", "\\lx@psgobble@shape");
  DefMacro!("\\psframe OptionalMatch:* []", "\\lx@psgobble@shape");
  // \pscircle[par](x,y){radius}: one coord pair + braced radius (real pstricks
  // `\def\pscircle(#1){#2}` style). The coordinate pair is OPTIONAL, default
  // (0,0) — Perl pstricks_support.sty.ltxml:700 `ZeroPSCoord` = ReadPSCoord
  // || ZeroPair; dsptricks.sty L535 calls `\pscircle[…]{\PZCROC\dspUnitX}`
  // with no pair (witness dspTricksManual expected:Pair).
  DefMacro!("\\pscircle OptionalMatch:* [] OptionalPair {}", "\\lx@psgobble@parens");
  DefMacro!("\\psarc OptionalMatch:* []{}{}{}{}", "\\lx@psgobble@parens");
  DefMacro!("\\psbezier OptionalMatch:* []", "\\lx@psgobble@shape");
  DefMacro!("\\pscurve OptionalMatch:* []", "\\lx@psgobble@shape");
  DefMacro!("\\psecurve OptionalMatch:* []", "\\lx@psgobble@shape");
  DefMacro!("\\psccurve OptionalMatch:* []", "\\lx@psgobble@shape");
  DefMacro!("\\parabola OptionalMatch:* []", "\\lx@psgobble@shape");
  DefMacro!("\\pspolygon OptionalMatch:* []", "\\lx@psgobble@shape");
  DefMacro!("\\psdots OptionalMatch:* []", "\\lx@psgobble@shape");
  DefMacro!("\\psdot OptionalMatch:* []", "\\lx@psgobble@shape");
  // \qline(x1,y1)(x2,y2) — two coordinate pairs, real pstricks
  // `\def\qline(#1)(#2){…}` (pstricks.tex L2150). The `{}{}` signature read
  // two brace/single-token args instead of the parenthesised coordinate
  // pairs, so `\qline(0,0)(1,1)` mis-parsed (consumed `\qline` + `(` + `0`)
  // and dumped the remainder `,0)(1,1)` as stray picture text.
  def_macro_noop("\\qline Pair Pair")?;

  // \Rput[refpoint](x,y){body} — placement at coords (real pstricks
  // defines this in pstricks.tex / pst-code-put.tex, raw-loaded by
  // Perl's `InputDefinitions('pstricks', noltxml=>1)`. Our Rust binding
  // doesn't raw-load pstricks.sty, so define a HTML-shrug stub here:
  // emit just the body, dropping placement. Witness: 0905.1885
  // (`\Rput[t](0,0){$H_1$}`).
  DefMacro!("\\Rput OptionalMatch:* [] Pair {}", "#4");
  DefMacro!("\\rput OptionalMatch:* [] Pair {}", "#4");
  DefMacro!("\\uput OptionalMatch:* {} [] Pair {}", "#5");
  // \qdisk(x0,y0){radius} — coordinate pair + braced radius, real pstricks
  // `\def\qdisk(#1)#2{…}` (pstricks.tex L3411). The old `{}{}` signature read
  // two brace/single-token args instead of `(coord){radius}`, so
  // `\qdisk(3,2.5){2.5pt}` consumed only `\qdisk` + `(` + `3` and left the
  // remainder `,2.5){2.5pt}` as stray picture text. Following a `\put{…}`
  // (whose `<ltx:text>` was still open), that stray text trapped every
  // subsequent block (proof/theorem/section/bibliography) inside an
  // un-closeable `<ltx:text>` → "ltx:* isn't allowed in <ltx:text>" cascade.
  // Witness: arXiv:1112.2096 (`\put(2.5,1.4){$a$}` then `\qdisk(3,2.5){2.5pt}`
  // inside `\mbox{\scalebox{\begin{pspicture}…}}`).
  def_macro_noop("\\qdisk Pair {}")?;

  // Text placement — since batch 56ao the body SURVIVES inside an
  // `<ltx:g transform>` (`\lx@ps@put`, Perl :879-888), and `{pspicture}` is
  // a real `<ltx:picture>`, so a placed label no longer lands in the
  // surrounding paragraph (the hep-ph/0102192 minipage-in-figure cascade
  // that once forced the body to be dropped; re-verified 0 errors).
  // Runaway-safe placement gobbler shared by \rput/\cput. Consumes (and
  // drops) optional [refpoint], optional {angle}, optional (coords), and the
  // mandatory {body}. The PREVIOUS def used a *delimited* `(#1)` parameter
  // (`\def\lx@rput@parens(#1)#2{}`): for the braced-angle / no-coords form
  // `\rput{angle}{body}` there is no `(`, so TeX scanned FORWARD eating
  // tokens — including `\end{pspicture}` — until the next `(` anywhere
  // later. That swallowed the env end, so pspicture's `end_mode` never fired
  // and its mode-switch frame leaked, tripping `\endgroup Attempt to close a
  // group that switched to mode restricted_horizontal` (witness 1505.07999 +
  // the ~17-paper `\endgroup` mode-leak cluster). Perl avoids this with
  // `OptionalBracketed`+`ZeroPSCoord` (coords optional); we PEEK for `(`
  // instead of requiring it. (Body still dropped — see the <ltx:p>-cascade
  // note above; faithful `<ltx:g>`-with-body is the separate TODO.)
  RawTeX!("\\def\\lx@put@cb(#1)#2{\\lx@ps@put(#1){#2}}");      // (coords){body} -> placed body
  // `{group}` with no `(` after it: the group WAS the body (Perl's ZeroPSCoord
  // leniency, origin (0,0)); with a `(` after it, it was the rotation angle.
  RawTeX!("\\def\\lx@put@b#1{\\@ifnextchar(\\lx@put@cb{\\lx@ps@put(0,0){#1}}}");
  RawTeX!("\\def\\lx@put@s{\\@ifnextchar(\\lx@put@cb\\lx@put@b}");    // ( -> coords; else {angle}|{body}
  RawTeX!("\\def\\lx@put@opt[#1]{\\lx@put@s}");                       // [refpoint] -> continue
  RawTeX!("\\def\\lx@put@start{\\@ifnextchar[\\lx@put@opt\\lx@put@s}");
  RawTeX!("\\def\\rput{\\@ifstar\\lx@put@start\\lx@put@start}");
  RawTeX!("\\def\\lx@uput@parens#1(#2)#3{\\lx@ps@put(#2){#3}}"); // {dist}(coord){text} → placed text
  RawTeX!("\\def\\lx@uput@bracket[#1]{\\lx@uput@parens}");
  RawTeX!("\\def\\uput{\\@ifstar\\lx@uput@i\\lx@uput@i}");
  RawTeX!("\\def\\lx@uput@i{\\@ifnextchar[\\lx@uput@bracket{\\lx@uput@parens}}");
  // \cput shares the runaway-safe gobbler (same delimited-`(` hazard).
  RawTeX!("\\def\\cput{\\@ifstar\\lx@put@start\\lx@put@start}");

  // Box commands
  DefMacro!("\\psframebox OptionalMatch:* []{}", "#2");
  DefMacro!("\\psshadowbox OptionalMatch:* []{}", "#2");
  DefMacro!("\\pscirclebox OptionalMatch:* []{}", "#2");
  DefMacro!("\\psovalbox OptionalMatch:* []{}", "#2");
  DefMacro!("\\psdblframebox OptionalMatch:* []{}", "#2");

  // Environment — Perl pstricks_support.sty.ltxml:520-560: `\begin{pspicture}
  // *[baseline](x0,y0)(x1,y1)` is an `<ltx:picture>` sized by the two corners
  // in `\psxunit`/`\psyunit` (the FIRST corner is optional: a lone pair is
  // the far corner, origin (0,0)), the body inside a `<ltx:g>` translated by
  // the negated origin, `\par` let to `\relax`. The former `[]{}` signature
  // swallowed the `(` of the first pair and leaked `x0,y0)(x1,y1)` as text in
  // EVERY pstricks picture (batch 56ao; the LaTeX `{picture}` binding in
  // latex_constructs/sect13.rs is the model).
  DefEnvironment!("{pspicture} OptionalMatch:* [] Pair OptionalPair",
    "<ltx:picture width='#width' height='#height' origin-x='#origin-x' origin-y='#origin-y'\
      fill='none' stroke='none' unitlength='#unitlength'>\
      ?#transform(<ltx:g transform='#transform'>#body</ltx:g>)(#body)\
    </ltx:picture>",
    mode => "inline_internal_vertical",
    before_digest => { Let!("\\par", "\\relax"); },
    properties => sub[args] { pspicture_properties(&args[2], &args[3]) }
  );
  DefEnvironment!("{pspicture*} OptionalMatch:* [] Pair OptionalPair",
    "<ltx:picture width='#width' height='#height' origin-x='#origin-x' origin-y='#origin-y'\
      clip='true' fill='none' stroke='none' unitlength='#unitlength'>\
      ?#transform(<ltx:g transform='#transform'>#body</ltx:g>)(#body)\
    </ltx:picture>",
    mode => "inline_internal_vertical",
    before_digest => { Let!("\\par", "\\relax"); },
    properties => sub[args] { pspicture_properties(&args[2], &args[3]) }
  );
  // `\rput`-family bodies (Perl :879-888 `\rput@start` → `<ltx:g transform>`
  // … `\put@end`): the same shape as the LaTeX `\put` constructor, in
  // pstricks units. Rotation/refpoint/labelsep are dropped (presentation).
  DefConstructor!("\\lx@ps@put Pair {}",
    "<ltx:g transform='#transform'>#2</ltx:g>",
    alias => "\\rput",
    mode  => "restricted_horizontal",
    properties => sub[args] { ps_put_properties(&args[0]) }
  );

  // Grid
  def_macro_noop("\\psgrid OptionalMatch:* []{}")?;

  // Misc
  def_macro_noop("\\pscustom OptionalMatch:* []{}")?;
  def_macro_noop("\\psclip{}")?;
  def_macro_noop("\\endpsclip")?;
  def_macro_noop("\\SpecialCoor")?;
  def_macro_noop("\\NormalCoor")?;
  def_macro_noop("\\degrees[]")?;
  def_macro_noop("\\radians")?;

  // \multips(rotation)(translation){n}{stuff} — pstricks "multiple put"
  // for drawing N copies of an object along a translated step. Rust port
  // doesn't raw-load pstricks.tex so this CS would otherwise be undefined.
  // Use RawTeX with a `\def` that consumes the paren-delimited args plus
  // the two brace args; the body is a no-op since pstricks output is
  // already suppressed in pspicture stubs. Same pattern as
  // `iopart_support_sty.rs:185`'s `\def\pt(#1){...}`.
  // Witness: math0104011 (was 17 errors → 0 with this stub).
  RawTeX!("\\def\\multips(#1)(#2)#3#4{}");
});
