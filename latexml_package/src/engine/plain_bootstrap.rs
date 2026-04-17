// plain_bootstrap — Bootstrap code for reading plain.tex for LaTeXML.
// Corresponds to Perl Engine/plain_bootstrap.pool.ltxml.
//
// Loaded BEFORE the plain dump. Contains stubs that override plain.tex's
// own allocation mechanisms with LaTeXML's versions.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: plain_bootstrap.pool.ltxml L19-27 — CSS-based TeX logo
  DefConstructor!("\\TeX", "<ltx:text class='ltx_TeX_logo' cssstyle='letter-spacing:-0.2em; margin-right:0.2em'
  >T<ltx:text cssstyle='font-variant:small-caps;font-size:120%;' yoffset='-0.2ex'
  >e</ltx:text>X</ltx:text>",
    locked => true,
    enter_horizontal => true,
    sizer => sub[_whatsit] { Ok((Dimension!("1.9em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });

  //======================================================================
  // Perl: plain_bootstrap.pool.ltxml L32-33
  // Use LaTeXML's register allocation to avoid allocating same slot twice
  DefMacro!("\\alloc@{}{}{}{}{}", r"\lx@alloc@{#2}{\count1#1}{#3}{#5}", locked => true);
  DefMacro!("\\ch@ck{}{}{}", None, locked => true);

  //======================================================================
  // Perl: plain_bootstrap.pool.ltxml L37-40
  // Use LaTeXML's conditional machinery
  DefMacro!("\\newif DefToken", sub[(cs)] {
    def_conditional(cs, None, None, ConditionalOptions::default())
  });

  //======================================================================
  // Perl: plain_bootstrap.pool.ltxml L43
  DefPrimitive!("\\leavevmode", { enter_horizontal(); });
});
