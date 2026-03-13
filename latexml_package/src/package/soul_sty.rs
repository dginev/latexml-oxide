use crate::prelude::*;

LoadDefinitions!({
  // \sodef \cs {font}{letterspacing}{innerspace}{outerspace}
  // Simplified: defines \cs as a macro wrapping \lx@soul@letterspaced
  DefPrimitive!("\\sodef Token {} {Dimension}{Dimension}{Dimension}",
    sub[(cs, _font, _letterspace, _innerspace, _outerspace)] {
      // Define \cs as an alias for the generic letter-spacing constructor
      Let!(cs, T_CS!("\\lx@soul@letterspaced"));
    });

  // Generic letter-spacing constructor used by \sodef-created commands
  DefConstructor!("\\lx@soul@letterspaced {}",
    "<ltx:text _noautoclose='1'>#1</ltx:text>",
    enter_horizontal => true);

  RawTeX!("\\sodef\\textso{}{0.25em}{0.65em}{.55em}");
  RawTeX!("\\sodef\\sloppyword{}{0em}{.33em}{.33em}");

  DefMacro!("\\resetso", "\\sodef\\so{}{0.25em}{0.65em}{.55em}");

  // Small caps
  DefMacro!("\\capsfont", "\\scshape");
  RawTeX!("\\sodef\\textcaps{\\capsfont}{0.28em}{0.37em}{.37em}");

  // Ignorable caps customization
  DefMacro!("\\capsdef {} {Dimension}{Dimension}{Dimension}", None);
  DefMacro!("\\capssave{}", None);
  DefMacro!("\\capsselect{}", None);
  DefMacro!("\\capsreset", None);

  // Underline
  DefConstructor!("\\textul{}",
    "<ltx:text framed='underline' _noautoclose='1'>#1</ltx:text>",
    enter_horizontal => true);

  // Customizing underlines
  DefMacro!("\\setulcolor{}", None);
  DefMacro!("\\setul{Dimension}{Dimension}", None);
  DefMacro!("\\resetul", None);
  DefMacro!("\\setuldepth{}", None);
  DefMacro!("\\setuloverlap{Dimension}", None);

  // Strike-out
  DefConstructor!("\\textst{}",
    "<ltx:text cssstyle='text-decoration:line-through;' _noautoclose='1'>#1</ltx:text>",
    enter_horizontal => true);

  // Customizing strikeout
  DefMacro!("\\setstcolor{}", None);

  // Highlighting — simplified: use background color
  DefConstructor!("\\lx@texthl@color{}",
    "<ltx:text _noautoclose='1'>#1</ltx:text>",
    enter_horizontal => true,
    bounded => true);
  DefMacro!("\\texthl", "\\@ifpackageloaded{color}{\\lx@texthl@color}{\\textul}");

  // Customizing highlight
  DefMacro!("\\sethlcolor{}", None);

  // Aliases
  Let!("\\so", "\\textso");
  Let!("\\caps", "\\textcaps");
  Let!("\\ul", "\\textul");
  Let!("\\st", "\\textst");
  Let!("\\hl", "\\texthl");

  // Ignorable commands
  DefMacro!("\\soulomit{}", "#1");
  DefMacro!("\\soulaccent{}", None);
  DefMacro!("\\soulregister{}{}", None);
  Let!("\\soulfont", "\\soulregister");
});
