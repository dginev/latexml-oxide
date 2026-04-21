use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("harvmac", noltxml => true, extension => Some(Cow::Borrowed("tex")));
  // ar5iv-bindings/bindings/harvmac.tex.ltxml L30-34: override \listtoc and
  // \writetoc to no-ops. The raw harvmac.tex version of \listtoc runs
  // `\openin\ch@ckfile=\jobname.toc` + `\input\jobname.toc`, producing
  // `Can't find TeX file <jobname>.toc` when the .toc hasn't been written
  // (single-pass processing). arxiv 0711.4787 used to fail here.
  DefMacro!("\\listtoc", "");
  DefMacro!("\\writetoc", {
    Warn!(
      "expected",
      "TOC",
      "harvmac.tex.ltxml has not yet implemented Table-of-contents"
    );
    Vec::<Token>::new()
  });
});
