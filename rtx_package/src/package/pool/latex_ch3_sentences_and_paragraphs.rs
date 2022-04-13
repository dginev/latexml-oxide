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
  DefConstructor!("\\LaTeX", "LaTeX");
  DefConstructor!("\\LaTeXe", "LaTeX2e");
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
    before_digest   => {
      DefMacro!(T_CS!("\\f@shape"), None, Tokens!(T_LETTER!("i"),T_LETTER!("t")));
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
