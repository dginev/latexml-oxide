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
  DefMacro!("\\langwohyphens", "");
  DefMacro!("\\nhttfamily", "");
  DefMacro!("\\nohyphens{}", "#1");
  DefMacro!("\\textnhtt", "");
  DefMacro!("\\touchextrattfonts", "");
  DefMacro!("\\touchttfonts", "");
  DefMacro!("\\prw@zbreak", "\\nobreak\\hskip\\z@skip");
});
