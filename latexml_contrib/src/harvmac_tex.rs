use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("harvmac", noltxml => true, extension => Some(Cow::Borrowed("tex")));
  // ar5iv-bindings/bindings/harvmac.tex.ltxml L16-22: override \eqn{\label}{body}
  // to (a) capture `(\secsym\the\meqno)` into the given label macro at invocation
  // time (mirrors the `\xdef #1{(\secsym\the\meqno)}` in raw harvmac.tex L130)
  // and (b) emit the body either directly (when IN_MATH) or wrapped in
  // \lx@begin@display@math ... \lx@end@display@math — the Perl port's XML-mode
  // substitute for the raw `$$#2\eqno#1\eqlabeL#1$$` plain-TeX wrapping.
  DefMacro!("\\eqn{}{}", sub[args] {
    let mut it = args.into_iter();
    let label_toks: Tokens = it.next().unwrap().into();
    let content: Tokens = it.next().unwrap().into();
    let label_cs = label_toks.unlist().into_iter().next().unwrap_or_else(|| T_CS!("\\relax"));
    let expanded = gullet::do_expand(Tokenize!(r"(\secsym\the\meqno)"))?;
    def_macro(label_cs, None, expanded, None)?;
    if state::lookup_bool_sym(pin!("IN_MATH")) {
      Ok(content)
    } else {
      let mut out = vec![T_CS!("\\lx@begin@display@math")];
      out.extend(content.unlist());
      out.push(T_CS!("\\lx@end@display@math"));
      Ok(Tokens::new(out))
    }
  });
  // ar5iv-bindings/bindings/harvmac.tex.ltxml L30-34: override \listtoc and
  // \writetoc to no-ops. The raw harvmac.tex version of \listtoc runs
  // `\openin\ch@ckfile=\jobname.toc` + `\input\jobname.toc`, producing
  // `Can't find TeX file <jobname>.toc` when the .toc hasn't been written
  // (single-pass processing). arxiv 0711.4787 used to fail here.
  // `locked=>1` matches Perl — prevents the raw harvmac.tex \def\listtoc /
  // \def\writetoc definitions from overriding these.
  DefMacro!("\\listtoc", "", locked => true);
  DefMacro!("\\writetoc", sub[_args] {
    Warn!(
      "expected",
      "TOC",
      "harvmac.tex.ltxml has not yet implemented Table-of-contents"
    );
    Ok(Tokens!())
  }, locked => true);
});
