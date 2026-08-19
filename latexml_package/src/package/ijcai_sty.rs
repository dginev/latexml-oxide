use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ijcai.sty.ltxml (PR #2767)
  // borrow a few of the cite-related definitions from natbib
  RequirePackage!("natbib");

  // \author{} : used once, separates multiple authors by \and (\And for last), followed by \\ !!!
  // Bizarrely, \affiliations and \emails should be WITHIN \author!
  // \affiliations{} gives \\ separated set of affiliations, corresponding to author in order
  // \emails{} gives a ", " separated set of emails, corresponding to author in order
  // If the association isn't "obvious" (one-to-one, or only a single affil?),
  // the author is expected to use $^{1,2}$ superscript markers!!
  Let!("\\AND",      "\\and");
  Let!("\\And",      "\\and");
  Let!("\\leftcite", "\\cite");
  DefMacro!("\\pubnote{}", "\\lx@add@pubnote[role=note]{#1}");

  // These are used as separators WITHIN \author, so...
  def_macro_noop("\\affiliations")?;
  def_macro_noop("\\emails")?;

  // `\author{}` → the shared sectioned-author splitter. The `\lx@ijcai@*`
  // machinery it drives now lives in the engine (base_utilities.rs), so a
  // raw-loaded ijcai derivative (e.g. ttm.sty) that never loads this binding
  // still gets the same split via the `\lx@add@authors` marker-branch.
  // n-th email attaches to the n-th author; affiliations are `\\`-separated.
  DefMacro!("\\author{}", "\\lx@ijcai@authorsplit#1\\affiliations\\done");
});
