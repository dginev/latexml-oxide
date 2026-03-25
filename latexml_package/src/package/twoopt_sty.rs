use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: twoopt.sty.ltxml
  // \newcommandtwoopt, \renewcommandtwoopt, \providecommandtwoopt:
  // These use complex sub{} bodies with convert2optArgs helper to build
  // parameters with two optional arguments and then call DefMacroI.
  // Stub: define as no-ops that absorb their arguments.
  // The real implementation would need runtime DefMacroI with dynamic parameter construction.

  // \newcommandtwoopt * \cs [Number] [] [] {}
  DefMacro!("\\newcommandtwoopt OptionalMatch:* DefToken [Number][][]{}", None);

  // \renewcommandtwoopt * \cs [Number] [] [] {}
  DefMacro!("\\renewcommandtwoopt OptionalMatch:* DefToken [Number][][]{}", None);

  // \providecommandtwoopt * \cs [Number] [] [] {}
  DefMacro!("\\providecommandtwoopt OptionalMatch:* DefToken [Number][][]{}", None);
});
