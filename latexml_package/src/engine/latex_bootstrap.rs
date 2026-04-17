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
  DefMacro!("\\LaTeX", "LaTeX");
  DefMacro!("\\LaTeXe", "LaTeX2e");
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
  DefMacro!("\\@definecounter", "\\newcounter", locked => true);
  DefMacro!("\\try@load@fontshape", "", locked => true);
  DefMacro!("\\define@newfont", "", locked => true);

  //======================================================================
  // Perl: latex_bootstrap.pool.ltxml L58
  Let!("\\@@input", "\\input"); // Save TeX's version.
});
