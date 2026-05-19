// latex_bootstrap — Bootstrap code for reading latex.ltx for LaTeXML.
// Corresponds to Perl Engine/latex_bootstrap.pool.ltxml.
//
// Loaded BEFORE the LaTeX dump. Contains stubs that override latex.ltx's
// own mechanisms with LaTeXML's versions, plus CSS-based logos.
use crate::prelude::*;

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


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
  // Perl uses `DefMacro` here, but the dump overwrites `\newcounter` with
  // the raw latex.ltx Expandable body that expands to `\@definecounter`.
  // If `\@definecounter` is a macro that re-expands to `\newcounter`, we
  // get an infinite loop after dump load. Perl's `DefMacro` is fine
  // because Perl's Token-list expansion of `\newcounter` is a Token, and
  // Perl's `installDefinition` would skip the dump-overwrite if our
  // `\newcounter` were locked (it's not — Perl bypasses lock too). The
  // working semantics in Perl actually rely on `\@definecounter`
  // resolving to the bootstrap Primitive at substitution time: by the
  // time `\@definecounter` is invoked (inside the dump-loaded
  // `\newcounter` body), the active `\newcounter` is the dump
  // Expandable — same loop. The Perl loop test shows it doesn't
  // actually loop in user code because... TODO investigate. For now,
  // use `Let!` to snapshot the Primitive at bootstrap time, breaking
  // the cycle. Token-stream-equivalent for downstream callers that
  // simply invoke `\@definecounter{...}`.
  Let!("\\@definecounter", "\\newcounter");
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
  // \@nil: kernel pattern-boundary marker some shims dereference.
  // \@expl@str@if@eq@@nnTF: expl3 internal predicate used in compat
  //   rollback (4-arg gobble matches \str_if_eq:nnTF semantics).
  // Witness 2408.00879, 2408.02823, 2406.00475.
  DefRegister!("\\tracingstacklevels" => Number::new(0));
  def_macro_noop("\\@nil")?;
  def_macro_noop("\\@expl@str@if@eq@@nnTF{}{}{}{}")?;
});
