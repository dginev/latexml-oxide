//! Shared helper: the frontmatter interface common to the "addtolist ML
//! meta-class" family (`fairmeta` / `selfevolagent` / `openmoss` and future
//! siblings) — pre-print classes whose `\author`/`\affiliation`/`\contribution`/
//! `\correspondence`/`\abstract`/`\beginappendix` commands are defined on an
//! `\addtolist` accumulator in the class BODY. An unknown `.cls` body is not
//! raw-loaded (OmniBus extracts dependencies only), so every one of those is
//! `Error:undefined` without a binding. Mirrors the shared-helper pattern of
//! [`crate::discard_env`].
use latexml_package::prelude::*;

/// Install the identical, order-independent frontmatter routing shared by every
/// sibling class: route `\author` to the creator sink and the rest to
/// `\@add@frontmatter`/`\lx@add@abstract` so they reach `<ltx:document>`
/// frontmatter. Call once from inside the class's `LoadDefinitions!` block.
///
/// The per-class parts stay in each `*_cls.rs`: the (order-sensitive) dependency
/// list, the colour palette, and the class-specific labeled field — `\metadata`
/// vs `\checkdata`, and whether its label routes to a `role` attribute
/// (attribute-safe labels, `fairmeta`) or to note content (arbitrary-markup
/// labels, `selfevolagent`/`openmoss`).
pub fn install_meta_class_frontmatter() -> Result<()> {
  // Accumulator lists → no-ops; the `\@add@frontmatter` sink accumulates.
  def_macro_noop("\\authorlist")?;
  def_macro_noop("\\affiliationlist")?;
  def_macro_noop("\\contributionlist")?;
  // `\author[mark]{name}` → the creator sink (NOT `\author` — that re-matches
  // this macro and recurses); the leading affiliation mark #1 is dropped.
  DefMacro!("\\author[]{}", "\\lx@add@author{#2}");
  DefMacro!(
    "\\affiliation[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}"
  );
  DefMacro!(
    "\\contribution[]{}",
    "\\@add@frontmatter{ltx:note}[role=contribution]{#2}"
  );
  DefMacro!(
    "\\correspondence{}",
    "\\@add@frontmatter{ltx:note}[role=correspondence]{#1}"
  );
  DefMacro!("\\abstract{}", "\\lx@add@abstract{#1}");
  DefMacro!("\\email{}", "\\href{mailto:#1}{\\texttt{#1}}");
  DefMacro!("\\beginappendix", "\\appendix");
  Ok(())
}
