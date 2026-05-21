//! Stub for bytedance_seed.cls (ByteDance Seed paper template).
//!
//! User-bundled template with author/affiliation/contribution-list
//! helpers built on \addtolist. Our raw-load fails on the
//! \addtolist[5] (5-arg, optional-arg-first) signature; provide
//! content-preserving stubs so the metadata reaches XML output.
//!
//! Witness: 2503.04598 (Seed-1.5 thinking paper).
use latexml_package::prelude::*;


LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("graphicx");
  RequirePackage!("hyperref");
  RequirePackage!("booktabs");
  RequirePackage!("etoolbox");

  // Author/affiliation/contribution lists — preserve as ltx:note.
  def_macro_noop("\\authorlist")?;
  def_macro_noop("\\affiliationlist")?;
  def_macro_noop("\\contributionlist")?;
  def_macro_noop("\\checkdatalist")?;
  // \author[mark]{name} — emit name as author.
  DefMacro!("\\author[]{}", "\\author{#2}");
  // \affiliation[mark]{text} — emit affiliation note.
  DefMacro!("\\affiliation[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}");
  // \contribution[mark]{text} — emit contribution note.
  DefMacro!("\\contribution[]{}",
    "\\@add@frontmatter{ltx:note}[role=contribution]{#2}");
  // \checkdata[label]{value} — emit as keyed note (label: value).
  DefMacro!("\\checkdata[]{}",
    "\\@add@frontmatter{ltx:note}[role=#1]{#2}");
  // \correspondence{text} — emit corresponding-author note.
  DefMacro!("\\correspondence{}",
    "\\@add@frontmatter{ltx:note}[role=correspondence]{#1}");
  // \beginappendix — bytedance uses this in place of \appendix.
  DefMacro!("\\beginappendix", "\\appendix");
  DefMacro!("\\seedblue{}", "{\\color[HTML]{2E5AA8}\\textbf{#1}}");
  DefMacro!("\\nm{}", "#1");
  DefMacro!("\\citeas{}", "\\cite{#1}");
});
