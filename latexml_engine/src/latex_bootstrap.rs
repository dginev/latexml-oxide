// latex_bootstrap — Bootstrap code for reading latex.ltx for LaTeXML.
// Corresponds to Perl Engine/latex_bootstrap.pool.ltxml.
//
// Loaded BEFORE the LaTeX dump. Contains stubs that override latex.ltx's
// own mechanisms with LaTeXML's versions, plus CSS-based logos.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: latex_bootstrap.pool.ltxml L18
  InnerPool!(plain_bootstrap);

  //======================================================================
  // Perl: latex_bootstrap.pool.ltxml L22-44 — CSS-based LaTeX/LaTeXe logos
  DefConstructor!("\\LaTeX","<ltx:text class='ltx_LaTeX_logo' cssstyle='letter-spacing:-0.2em; margin-right:0.1em'
  >L<ltx:text cssstyle='font-variant:small-caps;' yoffset='0.4ex'
  >a</ltx:text
  >T<ltx:text cssstyle='font-variant:small-caps;font-size:120%' yoffset='-0.2ex'
  >e</ltx:text
  >X</ltx:text>",
  enter_horizontal => true, locked => true,
  sizer => { Ok((Dimension!("2.6em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });

  DefConstructor!("\\LaTeXe","<ltx:text class='ltx_LaTeX_logo' cssstyle='letter-spacing:-0.2em; margin-right:0.1em'
  >L<ltx:text cssstyle='font-variant:small-caps;' yoffset='0.4ex'
  >a</ltx:text
  >T<ltx:text cssstyle='font-variant:small-caps;font-size:120%' yoffset='-0.2ex'
  >e</ltx:text
  >X\u{2002}2<ltx:text cssstyle='font-style:italic' yoffset='-0.3ex'
  >\u{03B5}</ltx:text></ltx:text>",
  enter_horizontal => true, locked => true,
  sizer => { Ok((Dimension!("3.7em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });

  //======================================================================
  // Perl: latex_bootstrap.pool.ltxml L49 — register allocation override
  DefMacro!("\\e@alloc{}{}{}{}{}{}", r"\lx@alloc@{#1}{#3}{#2}{#6}", locked => true);
  DefMacro!("\\e@ch@ck{}{}{}{}", "", locked => true);

  // Perl: latex_bootstrap.pool.ltxml L51-54 — counter/font stubs
  DefPrimitive!("\\newcounter{}[]", sub[(cs, default_opt)] {
    let default = if let Some(tks) = default_opt {
      if !tks.is_empty() { Expand!(tks) } else { Tokens!() }
    } else {
      Tokens!()
    };
    let cs_expanded = &Expand!(cs).to_string();
    NewCounter!(cs_expanded, &default.to_string());
  }, locked => true);
  // Perl latex_bootstrap.pool.ltxml:54: `\@definecounter` is the LOCKED
  // macro `\newcounter`, so latex.ltx:15736's raw `\def\@definecounter` is
  // refused during the format build and every kernel counter it creates
  // (`equation`, `footnote`, `enumi`…, and any package's `\@definecounter`
  // later) goes through `NewCounter`, which always defines the counter's
  // `\the<ctr>@ID` formatter (Package.pm:695-703, prefix = the counter
  // name). An unlocked `Let!` snapshot here was overwritten by the raw
  // definition: the dump then carried latex.ltx's `\@definecounter` and
  // no `@ID` formatter at all, so under a class with no binding loaded raw
  // (ptptex's manptp) `\theequation@ID` was undefined and `subequations`
  // ids degenerated to `.1` (`X.1` after `clean_id`), colliding across
  // groups; Perl gives `equation1`/`equation1.1`. No loop is possible:
  // `\newcounter` is the locked primitive, never latex.ltx's macro.
  DefMacro!("\\@definecounter", "\\newcounter", locked => true);
  DefMacro!("\\try@load@fontshape", "", locked => true);
  DefMacro!("\\define@newfont", "", locked => true);

  //======================================================================
  // Perl: latex_bootstrap.pool.ltxml L58
  Let!("\\@@input", "\\input"); // Save TeX's version.

  //======================================================================
  // Dump-replay rollback shims — must be defined BEFORE the dump loads.
  //
  // The kernel dump replays latexrelease/IncludeInRelease blocks that
  // reference TeX primitives the dump-time TeXLive didn't actually
  // include. Predefine them so the dump replay's "Applying:" arm
  // doesn't hit `Error:undefined:`. These need to live in
  // latex_bootstrap (not latex_constructs_rust_only) because the dump
  // load — which is what probes them — runs BEFORE constructs.
  //
  // \tracingstacklevels: TeX primitive added in TL 2021/06/01.
  // \@expl@str@if@eq@@nnTF: expl3 internal predicate used in compat
  //   rollback (4-arg gobble matches \str_if_eq:nnTF semantics).
  // Witness 2408.00879, 2408.02823, 2406.00475.
  // `\@nil` is NOT predefined: latex.ltx never defines it (a delimiter /
  // `\ifx\@nil#1\@nil` sentinel only, e.g. latexrelease.sty:144), and an
  // empty-macro stand-in makes `\ifx\@nil\<any empty macro>` true —
  // polynom.sty:1695 `\pld@MeasureCells@` then stops at an empty
  // `\pld@resultstyle` and leaks its `&\@nil&` (polynom/polydemo "Stray
  // alignment"). The dump-replay `undefined:\@nil` the shim once silenced
  // must be fixed where the `\@parse@version …\@nil` scan runs, never here.
  DefRegister!("\\tracingstacklevels" => Number::new(0));
  def_macro_noop("\\@expl@str@if@eq@@nnTF{}{}{}{}")?;
});
