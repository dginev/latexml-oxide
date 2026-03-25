use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: titlesec.sty.ltxml — stub for section title formatting
  // No styling implemented; just stubs to avoid errors

  DefMacro!("\\titlelabel{}", None);
  // \titleformat: star and normal forms
  // Simplified: just ignore the arguments
  DefMacro!("\\titleformat", "\\@ifstar{\\lx@titleformat@star}{\\lx@titleformat}");
  // \titleformat*{cmd}{format}
  DefMacro!("\\lx@titleformat@star {}{}", None);
  // \titleformat{cmd}[shape]{format}{label}{sep}{before}[after]
  DefMacro!("\\lx@titleformat {} [] {}{}{}{}[]", None);

  DefMacro!("\\chaptertitlename",                        "\\chaptername");
  DefMacro!("\\titlespacing OptionalMatch:* {}{}{}{}[]", None);

  DefMacro!("\\filright",  "\\raggedright");
  DefMacro!("\\filcenter", "\\centering");
  DefMacro!("\\filleft",   "\\raggedleft");
  DefMacro!("\\fillast",   None);
  DefMacro!("\\filinner",  "\\filleft");
  DefMacro!("\\filouter",  "\\filright");
  DefRegister!("\\wordsep", Dimension(0));

  DefMacro!("\\titleline[]{}", None);
  DefMacro!("\\titlerule", "\\@ifstar{\\lx@titlerule@star}{\\lx@titlerule}");
  DefMacro!("\\lx@titlerule@star []{}", None);
  DefMacro!("\\lx@titlerule []", None);

  DefConditional!("\\iftitlemeasuring");
  DefMacro!("\\assignpagestyle{}{}", None);
  DefMacro!("\\sectionbreak",       None);
  DefMacro!("\\subsectionbreak",    None);
  DefMacro!("\\subsubsectionbreak", None);
  DefMacro!("\\paragraphbreak",     None);
  DefMacro!("\\subparagraphbreak",  None);

  DefMacro!("\\titleclass{}[]{} []", None);
});
