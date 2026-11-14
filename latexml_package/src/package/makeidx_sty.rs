use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: makeidx.sty.ltxml — minus its `\makeindex` no-op (makeidx.sty.ltxml:18):
  // real makeidx.sty:44-51 defines NO `\makeindex`; the kernel's stays, and
  // ours (latex_constructs.rs, OXIDIZED_DESIGN #163) allocates `\@indexfile`,
  // which manyind.sty:25-30/189-216 and robustindex.sty:86-118 write to
  // themselves at `\begin{document}` (`undefined:\@indexfile` — mindsample,
  // robustmanual, multisample). Guard:
  // `perfect_kernel_batch54::makeidx_keeps_the_allocating_makeindex`.
  DefMacro!("\\see{}{}", "\\emph{\\seename} #1");
  DefMacro!("\\seealso{}{}", "\\emph{\\alsoname} #1");
  DefMacro!("\\printindex", "\\begin{theindex}\\end{theindex}", locked => true);
  DefMacro!("\\seename", "see");
  DefMacro!("\\alsoname", "see also");
});
