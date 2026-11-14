//! Stub for AAMAS conference class.
//!
//! aamas.cls is an ACM-style class. Reuse acmart's macros for all the
//! ACM frontmatter (\setcopyright, \acmConference, \acmDOI, etc.) and
//! treat content like amsart.
use latexml_package::prelude::*;

LoadDefinitions!({
  // (the copy read before acmart's load, while `\@currname` is this class)
  let shipped = crate::shipped_class::shipped_class_text("aamas")?;
  // An acmproc-era aamas class — aamas2013, "based on acmproc.cls V1.3", whose authors are `\alignauthor` tabular
  // columns (1401.1086's confSt.cls:342, `\ProvidesClass{aamas2013}`; `\numberofauthors` :333) — is no acmart class:
  // the paper's copy is read raw, as the class it is (acmart's `\author` met its columns: 3-6 Errors and one merged
  // creator, also for a `\documentclass{aamas2013}` whose copy the version fallback answered with this binding). Guard
  // `perfect_kernel_batch77w::acmproc_era_aamas_reads_raw`.
  if shipped
    .as_deref()
    .is_some_and(|text| crate::shipped_class::class_code_contains(text, "\\def\\alignauthor"))
  {
    crate::shipped_class::read_raw(&crate::shipped_class::RawRead {
      shipped: true,
      ..crate::shipped_class::RawRead::new("aamas", "cls")
    })?;
    return Ok(());
  }
  let defines_doi =
    shipped.is_none_or(|text| crate::shipped_class::class_code_contains(&text, "\\def\\doi#1"));
  LoadClass!("acmart");
  // aamas.cls 2026 (2602.10598's copy :1435-1436, 65 of the 69 copies papers ship): `\doi{SUFFIX}` sets the paper's
  // DOI under the IFAAMAS prefix, `10.65109/SUFFIX`, which the ACM reference line prints (:1903, :2492; `\acmDOI` stays
  // unchanged, the template says). A preamble `\doi` with a suffix is the paper's DOI note (65 papers: 2602.03799,
  // 2602.05679, 2602.06599); elsewhere, or empty, it only sets `\@doi`, as the class does. Only that version defines
  // it: the paper's own copy decides (the 2021 v1.78 copies of 2510.21200 and 2602.05888 have none, 2608.28553's has
  // the lines commented out), the newest version without one, so a reference list's own
  // `\providecommand{\doi}[1]{doi: #1}` stands where the class has no `\doi` (its DOIs printed nothing).
  if defines_doi {
    RawTeX!(r"\def\@doi{10.65109/}");
    DefMacro!("\\doi{}", sub[(suffix)] {
      let mut out = TokenizeInternal!("\\def\\@doi{10.65109/").unlist();
      out.extend(suffix.unlist_ref().iter().copied());
      out.push(T_END!());
      if lookup_bool("inPreamble") && !suffix.unlist_ref().iter().all(|t| *t == T_SPACE!()) {
        // (printed as its https://doi.org/ link, aamas.cls:1674, with no label: FIDELITY_CHARTER §4,
        // OXIDIZED_DESIGN_DIVERGENCES #921)
        out.extend(TokenizeInternal!("\\lx@add@pubnote[role=doi,name={}]{10.65109/").unlist());
        out.extend(suffix.unlist());
        out.push(T_END!());
      }
      Ok(Tokens::new(out))
    });
  }
});
