use crate::package::*;

LoadDefinitions!(state, {
  DefMacro!(T_CS!("\\space"), None, T_SPACE!());

  // ... TODO middle content port ... //
  // These should be 0 width, but perhaps also shifted?
  DefMacro!("\\llap{}", "\\hbox to 0pt{#1}");
  DefMacro!("\\rlap{}", "\\hbox to 0pt{#1}");
  DefMacro!("\\m@th",   "\\mathsurround=0pt ");

  // \strutbox
  DefMacro!("\\strut", "");
  RawTeX!("\\newbox\\strutbox");

  //======================================================================
  // TeX Book, Appendix B. p. 354

  // TODO: Not yet done!!
  // tabbing stuff!!!

  DefMacro!("\\settabs", "");

  //======================================================================
  // TeX Book, Appendix B. p. 355

  DefPrimitive!("\\hang", None);

  // TODO: \item, \itemitem not done!
  // This could probably be adopted from LaTeX, if the <itemize> could auto-open
  // and close!
  DefConstructor!("\\item{}",     "#1");
  DefConstructor!("\\itemitem{}", "#1");

  DefMacro!("\\textindent{}", "#1");

  // Conceivably this should enclose the next para in a block?
  // Or add attribute to it? Or...
  DefPrimitive!("\\narrower", None);

});
