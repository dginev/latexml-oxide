//**********************************************************************
// C.3. Sentences and Paragraphs
//**********************************************************************
use crate::package::*;

LoadDefinitions!(state, {
  //======================================================================
  // C.3.1 Making Sentences
  //======================================================================
  // quotes;  should these be handled in DOM/construction?
  // dashes:  We'll need some sort of Ligature analog, or something like
  // Omega's OTP, to combine sequences of "-" into endash, emdash,
  // Perhaps it also applies more semantically?
  // Such as interpreting certain sequences as section headings,
  // or math constructs.

  // Spacing; in TeX.pool.ltxml

  // Special Characters; in TeX.pool.ltxml

  // Logos
  // \TeX is in TeX.pool.ltxml
  DefMacro!("\\LaTeX", "LaTeX");
  DefMacro!("\\LaTeXe", "LaTeX2e");
  DefConstructor!("\\LaTeX",
  r###"<ltx:text class='ltx_LaTeX_logo' cssstyle='letter-spacing:-0.2em; margin-right:0.2em'>L<ltx:text
  fontsize='80%' yoffset='0.4ex'>A</ltx:text>T<ltx:text
  yoffset='-0.4ex'>E</ltx:text>X</ltx:text>"###,
  sizer => sub[_w,_s] { Ok((Dimension!("2.6em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });

  DefConstructor!("\\LaTeXe",
  "<ltx:text class='ltx_LaTeX_logo' cssstyle='letter-spacing:-0.2em; margin-right:0.2em'>L<ltx:text
  fontsize='80%' yoffset='0.4ex'>A</ltx:text>T<ltx:text
  yoffset='-0.4ex'>E</ltx:text>X\u{2002}2<ltx:text yoffset='-0.4em'>\u{03B5}</ltx:text></ltx:text>",
  sizer => sub[_w,_s] { Ok((Dimension!("3.7em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });

  DefMacro!("\\fmtname", "LaTeX2e");
  DefMacro!("\\fmtversion", "2018/12/01");

  DefMacro!("\\today", { ExplodeText!(Today!()) });

  // Use fonts (w/ special flag) to propogate emphasis as a font change,
  // but preserve it's "emph"-ness.
  DefConstructor!("\\emph{}", "<ltx:emph _force_font='1'>#1",
    mode => "text",
    bounded        => true,
    font=> { emph => true },
    alias => "\\emph",
    before_digest => sub[stomach,state] {
      let gullet = stomach.get_gullet_mut();
      if Expand!(T_CS!("\\f@shape"),gullet,state).to_string() == "it" {
        DefMacro!(T_CS!("\\f@shape"), None, Tokens!(T_LETTER!("n")));
      } else {
        DefMacro!(T_CS!("\\f@shape"), None, Tokens!(T_LETTER!("i"),T_LETTER!("t")));
      }
    },
    after_construct => sub[doc,args,inner_state] {
      doc.maybe_close_element("ltx:emph", inner_state)?; }
  );

  //======================================================================
  // C.3.2 Making Paragraphs
  //======================================================================
  // \noindent, \indent, \par in TeX.pool.ltxml

  Let!("\\@@par", "\\par");
  // Style parameters
  // \parindent, \baselineskip, \parskip alreadin in TeX.pool.ltxml

  DefPrimitive!("\\linespread{}", None);

  // ?
  DefMacro!("\\@noligs", "");
  DefConditional!("\\if@endpe");
  DefMacro!("\\@doendpe", "");
  DefMacro!("\\@bsphack", "\\relax"); // what else?
  DefMacro!("\\@esphack", "\\relax");
  DefMacro!("\\@Esphack", "\\relax");
});
