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
  DefMacro!("\\fmtversion", "XXXX/XX/XX");

  DefMacro!("\\today", sub[gullet,args,state] { Ok(Tokens::new(ExplodeText!(today(state)))) });

  // Previously, we used ltx:emph, to preserve the semantic intent,
  // but some folks wrap it around arbitrary blocks of material,
  // more like a font switch.
  DefConstructor!("\\emph{}", "#1", mode => "text".into_option(),
    bounded        => true, font=>Some(fontmap!(emph => true)), alias => "\\emph".into_option(),
    before_digest   => beforeproc!(stomach, inner_state, { DefMacroI!(T_CS!("\\f@shape"), None, T_LETTER!("i")); }),
    after_construct => construct!(doc,args,inner_state, { doc.add_class(&mut doc.get_element().unwrap(), "ltx_emph")?; })
  );

  //======================================================================
  // C.3.2 Making Paragraphs
  //======================================================================
  // \noindent, \indent, \par in TeX.pool.ltxml

  Let!("\\@@par", "\\par");
  // Style parameters
  // \parindent, \baselineskip, \parskip alreadin in TeX.pool.ltxml

  DefPrimitiveI!("\\linespread{}", noprimitive!());

  // ?
  DefMacro!("\\@noligs", "");
  DefConditional!("\\if@endpe");
  DefMacro!("\\@doendpe", "");
  DefMacro!("\\@bsphack", "\\relax"); // what else?
  DefMacro!("\\@esphack", "\\relax");
  DefMacro!("\\@Esphack", "\\relax");
});
