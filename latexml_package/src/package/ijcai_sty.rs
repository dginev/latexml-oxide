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

  DefMacro!("\\author{}", "\\lx@ijcai@authorsplit#1\\affiliations\\done");
  DefMacro!("\\lx@ijcai@authorsplit Until:\\affiliations Until:\\done",
    "\\lx@add@authors{#1}\\ifx.#2.\\else\\lx@ijcai@affilsplit#2\\emails\\affiliations\\done\\fi");
  DefMacro!("\\lx@ijcai@affilsplit  Until:\\emails Until:\\affiliations Until:\\done",
    "\\ifx.#1.\\else\\expandafter\\lx@ijcai@affiliations\\expandafter{\\lx@strip@braces{#1}}\\fi\\ifx.#2.\\else\\expandafter\\lx@ijcai@emails\\expandafter{\\lx@strip@braces{#2}}\\fi");
  DefMacro!("\\lx@ijcai@affiliations{}", "\\lx@add@affiliations[labelseq=author]{#1}");

  // emails is expected WITHIN \author.
  // Multiple affiliations separated by ,
  // the n-th email is attached to n-th author
  DefMacro!("\\lx@ijcai@emails{}",
    "\\lx@clear@frontmatter{ltx:contact}[role=email]\\lx@splitting{\\lx@ijcai@email}{,}{#1}");
  DefMacro!("\\lx@ijcai@email{}", "\\lx@add@email[labelseq=author]{#1}");
});
