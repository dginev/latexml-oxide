use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: thmtools.sty.ltxml — theorem tools package (126 lines)
  //
  // TODO: Full port requires:
  // 1. set_savable_theorem_parameters() — tracks which parameters
  //    \declaretheoremstyle and \declaretheorem can set (bodyfont, headfont, etc.)
  //    Perl stores these in LookupValue('THEOREM_PARAMETERS').
  // 2. \declaretheoremstyle — parses OptionalKeyVals, extracts heading/body/note fonts
  //    via get_keyval_value(), then calls amsthm's define_theorem_style().
  //    Need to port the keyval extraction logic.
  // 3. \declaretheorem — parses keyvals (style, parent, numberwithin, sibling, name,
  //    heading, title), resolves style → parameters, calls define_new_theorem().
  //    Need to expose define_new_theorem from latex_ch8_theoremlike_environments.
  // 4. \listoftheorems — generates a list similar to \listoffigures.
  //
  // Perl source: LaTeXML/lib/LaTeXML/Package/thmtools.sty.ltxml
  RequirePackage!("amsthm");
  InputDefinitions!("thmtools", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // TODO: \declaretheoremstyle — Perl extracts keyvals and calls define_theorem_style()
  // Stubbed: absorbs arguments without effect
  DefMacro!("\\declaretheoremstyle OptionalKeyVals:thm {}", "");

  // TODO: \declaretheorem — Perl extracts style/name/numberwithin keyvals,
  // resolves style to parameters, calls define_new_theorem(). Complex logic.
  // Stubbed: absorbs arguments without effect
  DefMacro!("\\declaretheorem OptionalKeyVals:thm {}", "");

  // TODO: \listoftheorems — generates theorem list
  DefMacro!("\\listoftheorems OptionalKeyVals:thm", "");

  DefMacro!("\\listtheoremname", "List of Theorems");
});
