//! Stub for AAMAS conference class.
//!
//! aamas.cls is an ACM-style class. Reuse acmart's macros for all the
//! ACM frontmatter (\setcopyright, \acmConference, \acmDOI, etc.) and
//! treat content like amsart.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("acmart");
  // aamas.cls 2026 (2602.10598's copy :1435-1436, 65 of the 69 copies papers ship): `\doi{SUFFIX}` sets the paper's
  // DOI under the IFAAMAS prefix, `10.65109/SUFFIX`, which the ACM reference line prints (:1903, :2492; `\acmDOI` stays
  // unchanged, the template says). A preamble `\doi` with a suffix is the paper's DOI note (65 papers: 2602.03799,
  // 2602.05679, 2602.06599); elsewhere, or empty, it only sets `\@doi`, as the class does.
  RawTeX!(r"\def\@doi{10.65109/}");
  DefMacro!("\\doi{}", sub[(suffix)] {
    let mut out = TokenizeInternal!("\\def\\@doi{10.65109/").unlist();
    out.extend(suffix.unlist_ref().iter().copied());
    out.push(T_END!());
    if lookup_bool("inPreamble") && !suffix.unlist_ref().iter().all(|t| *t == T_SPACE!()) {
      out.extend(TokenizeInternal!("\\lx@add@pubnote[role=doi]{10.65109/").unlist());
      out.extend(suffix.unlist());
      out.push(T_END!());
    }
    Ok(Tokens::new(out))
  });
});
