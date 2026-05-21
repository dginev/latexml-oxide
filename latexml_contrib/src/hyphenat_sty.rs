use latexml_package::prelude::*;


LoadDefinitions!({
  // These don't really apply in latexml, as our linebreak considerations are much softer than
  // PDF's.
  DefMacro!("\\BreakableBackslash", "\\textbackslash");
  DefMacro!("\\BreakableColon", ":");
  DefMacro!("\\BreakableHyphen", "-");
  DefMacro!("\\BreakablePeriod", ".");
  DefMacro!("\\BreakableSlash", "/");
  DefMacro!("\\BreakableUnderscore", "\\textunderscore");
  DefMacro!(
    "\\bshyp",
    "\\ifmmode\\backslash\\else\\BreakableBackslash\\fi"
  );
  DefMacro!("\\colonhyp", ":");
  DefMacro!("\\dothyp", ".");
  DefMacro!("\\fshyp", "/");
  DefMacro!("\\hyp", "-");
  def_macro_noop("\\langwohyphens")?;
  def_macro_noop("\\nhttfamily")?;
  DefMacro!("\\nohyphens{}", "#1");
  def_macro_noop("\\textnhtt")?;
  def_macro_noop("\\touchextrattfonts")?;
  def_macro_noop("\\touchttfonts")?;
  DefMacro!("\\prw@zbreak", "\\nobreak\\hskip\\z@skip");
});
