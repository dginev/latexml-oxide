//! pstricks.sty — PSTricks graphics package
//! PSTricks requires DVI backend; we raw-load real pstricks.sty to set
//! up internal state (\ifpst@useCalc, \ifpst@psfonts, …) then override
//! user-facing drawing commands as HTML-friendly no-ops.
//! Perl: pstricks.sty.ltxml (44L) + pstricks_support.sty.ltxml (1057L)
use crate::prelude::*;

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

  // Core PSTricks parameter setting
  def_macro_noop("\\psset{}")?;

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

  // Drawing commands — all no-ops for HTML, but MUST consume trailing
  // PSCoordList via `\lx@psgobble@parens` so coords don't leak as text.
  DefMacro!("\\psline OptionalMatch:* []{}", "\\lx@psgobble@parens");
  DefMacro!("\\psframe OptionalMatch:* []{}", "\\lx@psgobble@parens");
  DefMacro!("\\pscircle OptionalMatch:* []{}{}", "\\lx@psgobble@parens");
  DefMacro!("\\psarc OptionalMatch:* []{}{}{}{}", "\\lx@psgobble@parens");
  DefMacro!("\\psbezier OptionalMatch:* []{}", "\\lx@psgobble@parens");
  DefMacro!("\\pscurve OptionalMatch:* []{}", "\\lx@psgobble@parens");
  DefMacro!("\\psecurve OptionalMatch:* []{}", "\\lx@psgobble@parens");
  DefMacro!("\\psccurve OptionalMatch:* []{}", "\\lx@psgobble@parens");
  DefMacro!("\\parabola OptionalMatch:* []{}{}", "\\lx@psgobble@parens");
  DefMacro!("\\pspolygon OptionalMatch:* []{}", "\\lx@psgobble@parens");
  DefMacro!("\\psdots OptionalMatch:* []{}", "\\lx@psgobble@parens");
  DefMacro!("\\psdot OptionalMatch:* []{}", "\\lx@psgobble@parens");
  def_macro_noop("\\qline{}{}")?;

  // \Rput[refpoint](x,y){body} — placement at coords (real pstricks
  // defines this in pstricks.tex / pst-code-put.tex, raw-loaded by
  // Perl's `InputDefinitions('pstricks', noltxml=>1)`. Our Rust binding
  // doesn't raw-load pstricks.sty, so define a HTML-shrug stub here:
  // emit just the body, dropping placement. Witness: 0905.1885
  // (`\Rput[t](0,0){$H_1$}`).
  DefMacro!("\\Rput OptionalMatch:* [] Pair {}", "#4");
  DefMacro!("\\rput OptionalMatch:* [] Pair {}", "#4");
  DefMacro!("\\uput OptionalMatch:* {} [] Pair {}", "#5");
  def_macro_noop("\\qdisk{}{}")?;

  // Text placement — drop both coords AND the text body. Perl's
  // `DefPSConstructor` would wrap the labelled text inside a
  // `<ltx:picture>`, so the picture auto-closes cleanly when block
  // content (e.g. `\begin{minipage}` inside a figure) follows. Rust's
  // pstricks port doesn't yet generate `<ltx:picture>`; emitting the
  // text into the surrounding paragraph traps later block content
  // inside an `<ltx:p>` (witness: hep-ph0102192 minipage-in-figure
  // schema errors). Dropping the text body is a fidelity regression —
  // visible labels like "cocktail"/"thermal" placed via `\rput` are
  // lost — but it eliminates the cascading schema errors. TODO: port
  // `DefPSConstructor` framework so pstricks output lives in
  // `<ltx:picture>` and labels survive.
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
  RawTeX!("\\def\\lx@put@cb(#1)#2{}");      // (coords){body} -> drop
  RawTeX!("\\def\\lx@put@bb#1{}");          // {body} -> drop (no coords)
  RawTeX!("\\def\\lx@put@b#1{\\@ifnextchar(\\lx@put@cb\\lx@put@bb}"); // {angle}; then (coords)? body
  RawTeX!("\\def\\lx@put@s{\\@ifnextchar(\\lx@put@cb\\lx@put@b}");    // ( -> coords; else {angle}|{body}
  RawTeX!("\\def\\lx@put@opt[#1]{\\lx@put@s}");                       // [refpoint] -> continue
  RawTeX!("\\def\\lx@put@start{\\@ifnextchar[\\lx@put@opt\\lx@put@s}");
  RawTeX!("\\def\\rput{\\@ifstar\\lx@put@start\\lx@put@start}");
  RawTeX!("\\def\\lx@uput@parens#1(#2)#3{}"); // {dist}(coord){text} → drop
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

  // Environment
  DefEnvironment!("{pspicture} OptionalMatch:* []{}", "#body");
  DefEnvironment!("{pspicture*} OptionalMatch:* []{}", "#body");

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
