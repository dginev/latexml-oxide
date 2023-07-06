use crate::package::*;
LoadDefinitions!({

  DefMacro!("\\@saveprimitive{}{}", "\\let#2#1");

  Let!("\\@xp", "\\expandafter");
  Let!("\\@nx", "\\noexpand");
  DefRegister!("\\@emptytoks" => Tokens!());
  DefMacro!("\\@ifempty {}", r"\@xifempty#1@@..\@nil");
  RawTeX!(r"
  \def\@oparg#1[#2]{\@ifnextchar[{#1}{#1[#2]}}
  \long\def\@ifempty#1{\@xifempty#1@@..\@nil}
  \long\def\@xifempty#1#2@#3#4#5\@nil{%
    \ifx#3#4\@xp\@firstoftwo\else\@xp\@secondoftwo\fi}
  \long\def\@ifnotempty#1{\@ifempty{#1}{}}");

  DefMacro!("\\FN@", "\\futurelet\\@let@token");
  DefMacro!("\\DN@", "\\def\\next@");
  DefMacro!("\\RifM@", "\\relax\\ifmmode");
  DefMacro!("\\setboxz@h", "\\setbox\\z@\\hbox");
  DefMacro!("\\wdz@", "\\wd\\z@");
  DefMacro!("\\boxz@", "\\box\\z@");
  DefMacro!("\\relaxnext@", "\\let\\@let@token\\relax");

  // Do we need to worry about the skip space issues...?
  Let!("\\new@ifnextchar", "\\@ifnextchar");
  // \@ifstar already in LaTeX.pool
  DefRegister!("\\ex@" => Dimension::from_str("1pt")?);    // Just fake it...
  // Hmm.... how should we detect whether there"\s already punctuation?
  DefMacro!("\\@addpunct{}", "#1");

  DefMacro!("\\mathhexbox{}{}{}", r###"\text{$\m@th\mathchar"#1#2#3$}"###);

});