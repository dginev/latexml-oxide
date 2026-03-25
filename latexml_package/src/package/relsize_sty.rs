use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // \relsize{} and \relscale{} are DefPrimitive with subs in Perl.
  // We approximate with macros since font scaling subs aren't available.
  // The Perl implementation does MergeFont(scale => 1.2**$s).
  DefMacro!("\\relsize{}", None);
  DefMacro!("\\relscale{}", None);

  DefMacro!("\\textscale{}{}", "\\begingroup\\relscale{#1}#2\\endgroup");

  DefMacro!("\\larger Optional:1",         "\\relsize{+#1}");
  DefMacro!("\\smaller Optional:1",        "\\relsize{-#1}");
  DefMacro!("\\textlarger Optional:1 {}",  "{\\relsize{+#1}#2}");
  DefMacro!("\\textsmaller Optional:1 {}", "{\\relsize{-#1}#2}");

  DefMacro!("\\RSpercentTolerance", None);
  DefMacro!("\\RSsmallest",         "999pt");
  DefMacro!("\\RSlargest",          "1pt");

  DefMacro!("\\mathlarger Optional:1",  "\\relsize{+#1}");
  DefMacro!("\\mathsmaller Optional:1", "\\relsize{-#1}");
});
