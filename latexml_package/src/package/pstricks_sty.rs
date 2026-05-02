//! pstricks.sty — PSTricks graphics package (stubs)
//! PSTricks requires DVI backend; we just stub commands to prevent errors.
//! Perl: pstricks.sty.ltxml (44L) + pstricks_support.sty.ltxml (1057L)
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("xcolor");
  // Perl pstricks.sty.ltxml L44: `RequirePackage('pstricks_support')`.
  // pstricks_support defines color-CS shorthands (`\blue`, `\red`, …)
  // that PSTricks-using papers (e.g. arxiv 1107.3732) reference inside
  // `\tikzpicture{\node{\blue{…}}}`. Without it those CSes are undefined.
  RequirePackage!("pstricks_support");

  // Core PSTricks parameter setting
  DefMacro!("\\psset{}", "");
  DefMacro!("\\newpsobject{}{}{}", "");
  DefMacro!("\\newpsstyle{}{}", "");

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
  DefMacro!("\\qline{}{}", "");
  DefMacro!("\\qdisk{}{}", "");

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
  RawTeX!("\\def\\lx@rput@parens(#1)#2{}");
  RawTeX!("\\def\\lx@rput@bracket[#1]{\\lx@rput@parens}");
  RawTeX!("\\def\\rput{\\@ifstar\\lx@rput@i\\lx@rput@i}");
  RawTeX!("\\def\\lx@rput@i{\\@ifnextchar[\\lx@rput@bracket{\\lx@rput@parens}}");
  RawTeX!("\\def\\lx@uput@parens#1(#2)#3{}"); // {dist}(coord){text} → drop
  RawTeX!("\\def\\lx@uput@bracket[#1]{\\lx@uput@parens}");
  RawTeX!("\\def\\uput{\\@ifstar\\lx@uput@i\\lx@uput@i}");
  RawTeX!("\\def\\lx@uput@i{\\@ifnextchar[\\lx@uput@bracket{\\lx@uput@parens}}");
  RawTeX!("\\def\\lx@cput@parens(#1)#2{}");
  RawTeX!("\\def\\lx@cput@bracket[#1]{\\lx@cput@parens}");
  RawTeX!("\\def\\cput{\\@ifstar\\lx@cput@i\\lx@cput@i}");
  RawTeX!("\\def\\lx@cput@i{\\@ifnextchar[\\lx@cput@bracket{\\lx@cput@parens}}");

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
  DefMacro!("\\psgrid OptionalMatch:* []{}", "");

  // Misc
  DefMacro!("\\pscustom OptionalMatch:* []{}", "");
  DefMacro!("\\psclip{}", "");
  DefMacro!("\\endpsclip", "");
  DefMacro!("\\SpecialCoor", "");
  DefMacro!("\\NormalCoor", "");
  DefMacro!("\\degrees[]", "");
  DefMacro!("\\radians", "");

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
