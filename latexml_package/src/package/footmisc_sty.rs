use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  for option in &["perpage", "para", "side", "ragged", "bottom",
    "marginal", "flushmargin", "hang", "norule", "splitrule",
    "split", "multiple"] {
    DeclareOption!(option, None);
  }
  // footmisc.sty:65-70: `symbol` marks footnotes with `\fnsymbol{footnote}`, `symbol*` with `\@fnsymbol\c@footnote`
  // in the `lamport*-robust` set, which prints the number past its sixteen symbols (:522-531 `\FN@orange`). The
  // package's own set is `lamport*` (:578-598 `\setfnsymbol{lamport*}`): sixteen symbols, then the counter error. Perl
  // declares both options as no-ops (footmisc.sty.ltxml:19-22), marks 1, 2, 3 where pdflatex prints "∗ † ‡"
  // (KNOWN_PERL_ERRORS #1057). Witness 1907.00006 (`[perpage,symbol*]`). Guard
  // `perfect_kernel_batch78k::footmisc_symbol_marks_footnotes_with_symbols`. Residuals (KPE #1057): `symbol`'s
  // `\fnsymbol` is the kernel's (sect08.rs, the radix form of latex.ltx's nine symbols), not this `\@fnsymbol`, so
  // from mark 10 on it differs from footmisc's set ("∗∗" for "§§") and never raises the counter error; `perpage` is
  // a no-op and the counter is never reset per page; `\DefineFNsymbols`/`\setfnsymbol` below are no-ops. Witness
  // 0807.1434 (its `\setfnsymbol{numbers}` and `perpage`: "∗†", "††" where the PDF numbers each page's notes).
  DeclareOption!("symbol", r"\renewcommand\thefootnote{\fnsymbol{footnote}}");
  DeclareOption!("symbol*", r"\renewcommand\thefootnote{\@fnsymbol\c@footnote}\FN@robusttrue");
  RawTeX!(r"\newif\ifFN@robust \FN@robustfalse");
  ProcessOptions!();
  RawTeX!(r"\def\FN@lamport@star#1{\ifcase#1\or \TextOrMath\textasteriskcentered *\or \TextOrMath\textdagger\dagger
\or \TextOrMath\textdaggerdbl\ddagger\or \TextOrMath\textsection\mathsection\or \TextOrMath\textparagraph\mathparagraph
\or \TextOrMath\textbardbl\|\or \TextOrMath{\textasteriskcentered\textasteriskcentered}{**}%
\or \TextOrMath{\textdagger\textdagger}{\dagger\dagger}\or \TextOrMath{\textdaggerdbl\textdaggerdbl}{\ddagger\ddagger}%
\or \TextOrMath{\textsection\textsection}{\mathsection\mathsection}%
\or \TextOrMath{\textparagraph\textparagraph}{\mathparagraph\mathparagraph}%
\or \TextOrMath{\textasteriskcentered\textasteriskcentered\textasteriskcentered}{***}%
\or \TextOrMath{\textdagger\textdagger\textdagger}{\dagger\dagger\dagger}%
\or \TextOrMath{\textdaggerdbl\textdaggerdbl\textdaggerdbl}{\ddagger\ddagger\ddagger}%
\or \TextOrMath{\textsection\textsection\textsection}{\mathsection\mathsection\mathsection}%
\or \TextOrMath{\textparagraph\textparagraph\textparagraph}{\mathparagraph\mathparagraph\mathparagraph}%
\else\ifFN@robust\@arabic#1\else\@ctrerr\fi\fi}
\let\@fnsymbol\FN@lamport@star");

  // could define & use these, but...
  DefMacro!("\\DefineFNsymbols OptionalMatch:* {}{}", None);
  DefMacro!("\\setfnsymbol{}",                        None);
  // footmisc.sty:496-512 `\DefineFNsymbolsTM*{<name>}{<text/math pairs>}`, the text-or-math form of
  // `\DefineFNsymbols` (a named footnote-symbol set for `\setfnsymbol`), read as that one is
  // (run 340: 127 first errors, witnesses 1904.04281, 1904.11899, 1107.5789).
  DefMacro!("\\DefineFNsymbolsTM OptionalMatch:* {}{}", None);

  DefMacro!("\\mpfootnotemark",    "\\footnotemark");
  DefMacro!("\\mpfootnoterule",    "\\footnoterule");
  DefMacro!("\\pagefootnoterule",  "\\footnoterule");
  DefMacro!("\\splitfootnoterule", "\\footnoterule");
  DefMacro!("\\footnotelayout",    "\\@empty");
  DefMacro!("\\footnotehint",      None);

  DefRegister!("\\footnotemargin"       => Dimension!("1.8em"));
  DefRegister!("\\footnotebaselineskip" => Glue!("12pt"));
});
