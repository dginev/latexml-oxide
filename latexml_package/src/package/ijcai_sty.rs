use crate::prelude::*;

/// This binding's frontmatter conventions (the observer's, OXIDIZED_DESIGN_DIVERGENCES #837), declared at its load
/// (`declare_frontmatter_conventions`, user ruling 2026-10-28: each binding declares its own):
/// the `\emails` line under the affiliations (ijcai19.sty:127-130), a class's address list: placed by spelling, else
/// by author order, else the block (`owner=sequence`, OD #908; 2105.02711, 1905.13566, 2308.08563).
pub const FRONTMATTER_CONVENTIONS: FmConventions = FmConventions {
  name: "ijcai",
  binding: "ijcai.sty",
  email_owner: FmEmailOwner::SpelledElseOrder,
  cite: "ijcai.sty.ltxml:50-53 (`\\emails`, `labelseq=author`); OD #908 (2105.02711, 1905.13566, 2308.08563)",
  written: false,
  ..FM_KERNEL_CONVENTIONS
};

#[rustfmt::skip]
LoadDefinitions!({
  // (this binding's frontmatter conventions, first: OXIDIZED_DESIGN_DIVERGENCES #837)
  declare_frontmatter_conventions(&FRONTMATTER_CONVENTIONS);
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
  // ijcai24.sty:62 `\pubnote` sets its text in the running head, no label (FIDELITY_CHARTER §4, user ruling 2026-10-30; OXIDIZED_DESIGN_DIVERGENCES #921) (2406.14201).
  DefMacro!("\\pubnote{}", "\\lx@add@pubnote[role=note,name={}]{#1}");

  // These are used as separators WITHIN \author, so...
  def_macro_noop("\\affiliations")?;
  def_macro_noop("\\emails")?;

  // `\author{}` → the shared sectioned-author splitter. The `\lx@ijcai@*`
  // machinery it drives now lives in the engine (base_utilities.rs), so a
  // raw-loaded ijcai derivative (e.g. ttm.sty) that never loads this binding
  // still gets the same split via the `\lx@add@authors` marker-branch.
  // Affiliations are `\\`-separated; the `\emails` list is placed as a class's address list (OD #908).
  DefMacro!("\\author{}", "\\lx@ijcai@authorsplit#1\\affiliations\\done");
});
