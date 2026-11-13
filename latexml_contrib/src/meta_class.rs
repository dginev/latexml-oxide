//! Shared helper: the frontmatter interface common to the "addtolist ML
//! meta-class" family (`fairmeta` / `selfevolagent` / `openmoss` and future
//! siblings) — pre-print classes whose `\author`/`\affiliation`/`\contribution`/
//! `\correspondence`/`\abstract`/`\beginappendix` commands are defined on an
//! `\addtolist` accumulator in the class BODY. An unknown `.cls` body is not
//! raw-loaded (OmniBus extracts dependencies only), so every one of those is
//! `Error:undefined` without a binding. Mirrors the shared-helper pattern of
//! [`crate::discard_env`].
use latexml_package::{package::authblk_sty::author_names_a_list, prelude::*};

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
  // family #662/#3512/#4707/#4971/#5035/#5466); Perl has no binding and mangles
  // them under OmniBus. An uncited mark has no target and is dropped. An unmarked
  // call naming several authors, as authblk's `\author` reads one, is their list
  // (13 names and their legend in one `\author{}`, 2605.17829).
  DefMacro!("\\author[]{}", sub[(marks, name)] {
    let marks = marks.unwrap_or_default();
    if marks.is_empty() && author_names_a_list(&name)? {
      return Ok(Invocation!(T_CS!("\\lx@add@authors@list"), vec![Some(name)]));
    }
    Ok(Invocation!("\\lx@add@creator[role=author,annotations={#1}]{#2}", vec![Some(marks), Some(name)]))
  });
  // (an unmarked contact is the new authors': `annotate=new` written out, which the queue's reading of a list
  // above reads, as it reads no conditional)
  DefMacro!("\\affiliation[]{}", sub[(mark, text)] { meta_contact("affiliation", mark, text) });
  DefMacro!("\\contribution[]{}", sub[(mark, text)] { meta_contact("contribution", mark, text) });
  DefMacro!(
    "\\correspondence{}",
    "\\@add@frontmatter{ltx:note}[role=correspondence]{#1}"
  );
  DefMacro!("\\abstract{}", "\\lx@add@abstract{#1}");
  DefMacro!("\\email{}", "\\href{mailto:#1}{\\texttt{#1}}");
  DefMacro!("\\beginappendix", "\\appendix");
  Ok(())
}

/// `\lx@add@contact[role=<role>,annotate=…,label={mark}]{text}`: the contact labelled `mark` (`annotate=1`), or, unmarked,
/// the new authors' (`annotate=new`).
fn meta_contact(role: &str, mark: Option<Tokens>, text: Tokens) -> Result<Tokens> {
  let mark = mark.unwrap_or_default();
  let annotate = if mark.is_empty() { "new" } else { "1" };
  let keys = TokenizeInternal!(TeXString::assembled(s!(
    "role={role},annotate={annotate},label="
  )));
  Ok(Invocation!("\\lx@add@contact[#1{#2}]{#3}", vec![
    Some(keys),
    Some(mark),
    Some(text)
  ]))
}
