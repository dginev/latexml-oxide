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
/// sibling class: route `\author`/`\affiliation`/`\contribution` through
/// LaTeXML's author-annotation / contact-label plan (so the superscript marks
/// link each author to its institutions/notes) and the rest to
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
  // The class's `\author[mark]{name}` / `\affiliation[mark]{inst}` /
  // `\contribution[mark]{text}` link authors to institutions/notes by superscript
  // MARK (`\author[1,2,*]{…}` cites `\affiliation[1]{…}` and `\contribution[*]{…}`).
  // Route the marks through LaTeXML's author-annotation / contact-label plan — the
  // authblk.sty.ltxml idiom (`\lx@add@creator[annotations={#1}]` +
  // `\lx@add@contact[role,annotate,label={#1}]`), whose `relocate_annotations`
  // step attaches each contact to the creators that cite its mark. Dropping the
  // mark (the old `\lx@add@author{#2}` + detached `ltx:note`) lost every
  // author↔institution association (arXiv/html_feedback#1396 and the fairmeta
  // family #662/#3512/#4707/#4971/#5035/#5466). The annotation/label mechanism is
  // byte-identical to Perl LaTeXML (verified via the shared `\lx@add@creator`/
  // `\lx@add@contact` primitives); on the fairmeta papers themselves Rust
  // surpasses Perl, which has no binding and mangles them under OmniBus. An
  // uncited mark has no target and is dropped — parity with Perl's plan; the
  // marks in real papers are always cited. `\author` must NOT expand to `\author`
  // (that re-matches this macro and recurses) — use `\lx@add@creator`.
  DefMacro!(
    "\\author[]{}",
    "\\lx@add@creator[role=author,annotations={#1}]{#2}"
  );
  DefMacro!(
    "\\affiliation[]{}",
    "\\lx@add@contact[role=affiliation,annotate={\\ifx.#1.new\\else 1\\fi},label={#1}]{#2}"
  );
  DefMacro!(
    "\\contribution[]{}",
    "\\lx@add@contact[role=contribution,annotate={\\ifx.#1.new\\else 1\\fi},label={#1}]{#2}"
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
