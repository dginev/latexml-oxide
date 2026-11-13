//! bfhlayout.sty (bfh-ci, Bern University of Applied Sciences): the package is loaded raw, and the title page's foot —
//! `\department` and `\institute` (`\titlefooterleft`, bfhlayout.sty:735-753) and `\titlefooterright` (:755-756),
//! printed only by the title page's scrlayer footer layer (`title.BFH.footer`), a page style LaTeXML never typesets —
//! becomes frontmatter notes (roles `department`, `institute`, `titlefooter`) after the class's print site,
//! `\maketitle`, with the values set then (the demos set them after `\begin{document}`). A store the kernel already
//! handed to the authors as an affiliation there (`\department`/`\institute` read by a raw class's `\@maketitle`,
//! frontmatter_stores.rs; DEMO-BFHThesis) is not repeated. User ruling 2026-10-03: the department is frontmatter, as
//! a letter's sender (OXIDIZED_DESIGN_DIVERGENCES #412). SHARED: Perl with raw classes drops them alike. Residual: a
//! document's own `\titlefooterleft{…}`, replacing the department and institute there, is not read. Witnesses
//! bfh-ci DEMO-BFHFactsheet, DEMO-BFHProjektProposal, DEMO-BFHThesis. Guard
//! `perfect_kernel_batch61::bfh_title_footer_is_frontmatter`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("bfhlayout", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  DefMacro!("\\lx@bfh@frontmatter", sub[_args] {
    let mut notes = String::from(r"\global\let\lx@bfh@frontmatter\relax");
    for (store, role) in [("department", "department"), ("institute", "institute"), ("titlefooterright", "titlefooter")] {
      if !lookup_bool(&s!("lx_store_handed_{store}")) {
        notes.push_str(&s!(r"\ifx\@{store}\@empty\else\lx@add@frontmatter{{ltx:note}}[role={role}]{{\@{store}}}\fi"));
      }
    }
    Ok(TokenizeInternal!(TeXString::assembled(notes)))
  });
  RawTeX!(
    r"\AtBeginDocument{\let\lx@bfh@maketitle\maketitle\def\maketitle{\lx@bfh@maketitle\lx@bfh@frontmatter}}"
  );
});
