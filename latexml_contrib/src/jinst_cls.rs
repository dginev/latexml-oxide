//! The JHEP class family as shipped with papers and not in TeX Live: `JINST` (Journal of Instrumentation) and `CECS`,
//! both JHEP3-derived (`\auto@maketitle` title page, `\abstract@cs`, `\Jrece@cs`). One binding serves them, the union
//! of their frontmatter commands; `JHEP`, `JHEP2` and `JHEP3` themselves have the Perl-ported emulating bindings
//! (latexml_package `jhep_cls.rs`), whose routing this follows.
use latexml_package::prelude::*;

/// Interpret the shipped class `name` raw (as the document asked for it, a path-prefixed copy included:
/// [`crate::shipped_class::requested_shipped_class`]), then route the frontmatter it stores for its own title page to the
/// frontmatter, as the JHEP binding does (Perl JHEP.cls.ltxml):
/// - `\received`, `\revised`, `\accepted`, `\published` (JINST.cls:764-774) and the copyright dates (`\JINSTcopydate`
///   :777, `\JHEPcopydate`, `\CECScopydate`) are frontmatter dates, after the class's own setter has run for the flags
///   its `\AtBeginDocument` checks in `published` mode (:1592-1607);
/// - `\keywords` (:826-832, a catcode-changing reader storing `\@keywords`) is the frontmatter's keywords, its flag set;
/// - `\dedicated`, `\preprint`, `\conference` are publication notes and `\email` (:798) the author's email contact, as
///   the JHEP binding has them (JHEP.cls.ltxml:43);
/// - the frontmatter is made (the kernel `\maketitle`) where `\auto@maketitle` typesets the title page at
///   `\begin{document}`: it then kills the commands that page used (:488-506: `\global\let\email\@gobble`,
///   `\thanks`, `\author`, `\title`, …), and the queued `\author` blocks, digested any later, lost their emails
///   (0704.3706, gr-qc0303113, 41 of 2,918 papers);
/// - `\@maketitle` (:462-560), the title page `\auto@maketitle` typesets at `\begin{document}`, is empty: everything it
///   lays out is in the frontmatter, and it typeset a second title, author block and abstract in the body (2601.17161,
///   0704.3706, 0801.2206, 0805.3984); the rest of `\auto@maketitle` (footnote style, the commands it kills after the
///   title) runs as the class has it. The class's own `\maketitle` only warns that it is ignored (:461), and is not
///   deposited.
///
/// The `\author` and `\abstract` checks of `\AtBeginDocument` read flags the kernel's locked `\author`/`\abstract` set
/// by replaying the class's own (OXIDIZED_DESIGN #482). A paper whose source lacks the class falls to OmniBus.
pub(crate) fn jhep_family_class(name: &str) -> Result<()> {
  let Some(class) = crate::shipped_class::requested_shipped_class(name)? else {
    LoadClass!("OmniBus");
    return Ok(());
  };
  InputDefinitions!(&class, noltxml => true, extension => Some(Cow::Borrowed("cls")));
  // a class only named like the family (the prefix fallback's `JINSTfoo`) is left as it is: without the JHEP title
  // page (`\auto@maketitle`), nothing it stores is laid out only there
  if !IsDefined!(&T_CS!("\\auto@maketitle")) {
    return Ok(());
  }
  for (cs, frontmatter) in [
    (
      "received",
      "\\lx@add@date[role=received,name={\\receivedname}]{#1}",
    ),
    (
      "revised",
      "\\lx@add@date[role=revised,name={\\revisedname}]{#1}",
    ),
    (
      "accepted",
      "\\lx@add@date[role=accepted,name={\\acceptedname}]{#1}",
    ),
    (
      "published",
      "\\lx@add@date[role=published,name={\\publishedname}]{#1}",
    ),
    ("JINSTcopydate", "\\lx@add@date[role=copydate]{#1}"),
    ("JHEPcopydate", "\\lx@add@date[role=copydate]{#1}"),
    ("CECScopydate", "\\lx@add@date[role=copydate]{#1}"),
  ] {
    route_after_class_setter(cs, frontmatter)?;
  }
  DefMacro!(
    "\\keywords{}",
    "\\lx@add@keywords[name={\\keywordsname~}]{#1}\\global\\csname @keywordstrue\\endcsname"
  );
  DefMacro!("\\email Semiverbatim", "\\lx@add@email{#1}");
  // The frontmatter is made where the class typesets its title page: `\auto@maketitle` then kills the commands that
  // page has used (`\global\let\email\@gobble`, `\thanks`, `\author`, …), which the queued `\author` blocks,
  // digested any later, still need.
  Let!("\\lx@jhep@auto@maketitle", "\\auto@maketitle");
  RawTeX!(r"\def\auto@maketitle{\maketitle\lx@jhep@auto@maketitle}");
  DefMacro!("\\dedicated{}", "\\lx@add@pubnote[role=dedication]{#1}");
  DefMacro!("\\preprint{}", "\\lx@add@pubnote[role=preprint]{#1}");
  DefMacro!("\\conference{}", "\\lx@add@pubnote[role=conference]{#1}");
  Let!("\\@maketitle", "\\@empty");
  Let!("\\lx@dropped@maketitle", "\\relax");
  Ok(())
}

/// `\<cs>{…}` as the frontmatter's `frontmatter` (its `#1` the argument), after the class's own `\<cs>` where it has one.
fn route_after_class_setter(cs: &str, frontmatter: &str) -> Result<()> {
  let token = T_CS!(s!("\\{cs}"));
  if IsDefined!(&token) {
    let class_setter = s!("\\lx@jhep@class@{cs}");
    Let!(&T_CS!(&class_setter), token);
    RawTeX!(&s!(
      "\\def\\{cs}#1{{\\csname lx@jhep@class@{cs}\\endcsname{{#1}}{frontmatter}}}"
    ));
  } else {
    RawTeX!(&s!("\\def\\{cs}#1{{{frontmatter}}}"));
  }
  Ok(())
}

LoadDefinitions!({
  jhep_family_class("JINST")?;
});
