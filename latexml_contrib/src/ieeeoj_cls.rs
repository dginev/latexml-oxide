//! Stub for IEEEoj.cls (IEEE Open Journal template).
//!
//! IEEEoj is IEEEtran-derived but uses authblk-style `\author{}`/`\affil{}` for
//! the author/affiliation block. It was previously dispatched to
//! `ieeeaerospace_cls` (IEEEtran + `\acknowledgments`), which omits authblk — so
//! `\affil` was undefined while Perl defines it. Give IEEEoj its OWN binding:
//! ieeeaerospace's provisions (IEEEtran base + `\acknowledgments`) PLUS authblk
//! (which supplies `\affil`/`\affilmark`/`\author`), mirroring the sibling
//! `ieeeojcsys_cls`. Kept separate from `ieeeaerospace_cls` so the IEEE Aerospace
//! conference papers (which use IEEEtran's own author style) are unaffected by
//! authblk's `\author` redefinition. Witness 2203.03906.
//!
//! NOTE: the OJ-family siblings IEEEapm/IEEEtai (still on ieeeaerospace) likely
//! share the same `\affil` gap; route them here too once witnessed.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("IEEEtran");
  RequirePackage!("authblk");
  // Mirror ieeeaerospace_cls: IEEE OJ/Aerospace templates define a no-arg
  // `\acknowledgments` opening an unnumbered "Acknowledgments" section.
  DefMacro!("\\acknowledgments", "\\section*{Acknowledgments}");
  // IEEEoj.cls:3447-3451, 3713-3717, 4877: the class's front-matter setters — the author note and the
  // corresponding-author line as notes, the dates and the DOI as the first page prints them (:3457-3458 "Received …;
  // revised …; accepted …; Date of publication …; date of current version …", "Digital Object Identifier 10.1109/…"),
  // a template's "XX Month, XXXX" placeholders included, as pdflatex prints them (2609.01380, 05811, 11359).
  // Each setter keeps its last value, as the class's `\gdef` does (the dates' `\lx@add@date` clears its role too).
  DefMacro!(
    "\\authornote{}",
    "\\lx@clear@frontmatter{ltx:note}[role=note]\\lx@add@frontmatter{ltx:note}[role=note]{#1}"
  );
  DefMacro!(
    "\\corresp{}",
    "\\lx@clear@frontmatter{ltx:note}[role=corresponding]\\lx@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
  DefMacro!("\\receiveddate{}", "\\lx@add@date[role=received]{#1}");
  DefMacro!("\\reviseddate{}", "\\lx@add@date[role=revised]{#1}");
  DefMacro!("\\accepteddate{}", "\\lx@add@date[role=accepted]{#1}");
  DefMacro!("\\publisheddate{}", "\\lx@add@date[role=published]{#1}");
  DefMacro!("\\currentdate{}", "\\lx@add@date[role=current]{#1}");
  // An empty `\doiinfo{}` (the class's own default) prints no DOI line (IEEEoj.cls:3458 `\ifx\@doiinfo\@empty`; 2609.27083).
  DefMacro!(
    "\\doiinfo{}",
    "\\lx@clear@frontmatter{ltx:pubnote}[role=doi]\\def\\lx@ieee@doi{#1}\\ifx\\lx@ieee@doi\\@empty\\else\\lx@add@pubnote[role=doi]{10.1109/#1}\\fi"
  );
  // The names the first page prints them with (IEEEoj.cls:3457; Perl looks a date's name up as `\lx@date@<role>@name`).
  DefMacro!("\\lx@date@published@name", "Date of publication~");
  DefMacro!("\\lx@date@current@name", "Date of current version~");
});
