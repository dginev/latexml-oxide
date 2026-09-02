use crate::prelude::*;
LoadDefinitions!({
  DefMacro!("\\@saveprimitive{}{}", "\\let#2#1");

  Let!("\\@xp", "\\expandafter");
  Let!("\\@nx", "\\noexpand");
  DefRegister!("\\@emptytoks" => Tokens!());
  DefMacro!("\\@ifempty {}", r"\@xifempty#1@@..\@nil");
  TeX!(
    r"
  \def\@oparg#1[#2]{\@ifnextchar[{#1}{#1[#2]}}
  \long\def\@ifempty#1{\@xifempty#1@@..\@nil}
  \long\def\@xifempty#1#2@#3#4#5\@nil{%
    \ifx#3#4\@xp\@firstoftwo\else\@xp\@secondoftwo\fi}
  \long\def\@ifnotempty#1{\@ifempty{#1}{}}"
  );

  DefMacro!("\\FN@", "\\futurelet\\@let@token");
  DefMacro!("\\DN@", "\\def\\next@");
  DefMacro!("\\RifM@", "\\relax\\ifmmode");
  DefMacro!("\\setboxz@h", "\\setbox\\z@\\hbox");
  DefMacro!("\\wdz@", "\\wd\\z@");
  DefMacro!("\\boxz@", "\\box\\z@");
  DefMacro!("\\relaxnext@", "\\let\\@let@token\\relax");

  // Perl (amsgen.sty.ltxml:42 "Do we need to worry about the skip space
  // issues...?") Lets this to `\@ifnextchar`, which SKIPS spaces; the real
  // `\new@ifnextchar` (amsgen.sty:54-62) does not — that is its whole point.
  // bibleref.sty:969 `\bibleverse{Psalm} (Einzahl)` then took the `(` after
  // the space as its `(chapter:verse)` opener and `\@bibleverse(#1:` scanned
  // to the end of the document (en/de-bibleref-german, 12 `Until::` misses
  // each, sweep 28). KNOWN_PERL_ERRORS #113. Guard:
  // `perfect_kernel_batch51::new_ifnextchar_keeps_space`.
  TeX!(
    r"
  \long\def\new@ifnextchar#1#2#3{%
    \let\reserved@d= #1%
    \def\reserved@a{#2}\def\reserved@b{#3}%
    \futurelet\@let@token\new@ifnch
  }
  \def\new@ifnch{%
    \ifx\@let@token\reserved@d \let\reserved@b\reserved@a \fi
    \reserved@b
  }"
  );
  // \@ifstar already in LaTeX.pool
  DefRegister!("\\ex@" => Dimension::from_str("1pt")?);
  // Just fake it...
  // Hmm.... how should we detect whether there"\s already punctuation?
  DefMacro!("\\@addpunct{}", "#1");

  DefMacro!(
    "\\mathhexbox{}{}{}",
    r###"\text{$\m@th\mathchar"#1#2#3$}"###
  );
});
