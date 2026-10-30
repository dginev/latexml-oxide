//! pos.sty (SISSA Medialab's PoS template, v2.1 2018/11/13 and v2.2 2021/09/21, loaded under `article`): the papers
//! ship the package (it is not in TeX Live), which is read raw, its title page made by the frontmatter. pos.sty keeps
//! that page in stores only its own `\maketitle` prints (:450-530), which the kernel's locked `\maketitle` refuses: the
//! authors in expl3 sequences (:88-127 `\RenewDocumentCommand\author{som}`, which the kernel's locked `\author` refused
//! too, so `\author*{Javier M. Duarte}` read `*` as the name and the name as an affiliation), the affiliations and
//! e-mail addresses in token registers (:411-430), the conference in a box (:364-369). Every one is the frontmatter's
//! here, defined before the copy is read, whose `\newcommand`s of the same names are then ignored as those of any name
//! LaTeXML defines (and its `\NewDocumentCommand\onbehalf`, :249). Perl has no binding: it does not read the package,
//! keeps an empty name and raises an error for each `\affiliation` and `\emailAdd` (KNOWN_PERL_ERRORS #1198).
//! Witnesses 2409.20413, 2512.14592, 2602.22379 (OXIDIZED_DESIGN_DIVERGENCES #904).
use crate::{
  package::{jheppub_sty::jhep_contact_markup, pos_cls::pos_title_page_fields},
  prelude::*,
  shipped_class::{class_code, class_code_argument, shipped_file_text},
};

/// The text of the speaker's footnote in the paper's copy: the argument of the `\footnotetext` that `\printAuthors`
/// sets under the speaker's mark (:228-230 `\renewcommand\thefootnote{$\speakerFlag$}\footnotetext{\speakerText}`),
/// which v2.1 and early v2.2 copies write out ("Speaker"; 2107.12913's "Presenter"); `\speakerText` (:85, "Speaker",
/// which a paper may renew) where the copy names it, or has none.
fn speaker_text() -> String {
  const MARK: &str = "\\renewcommand\\thefootnote{$\\speakerFlag$}";
  shipped_file_text("pos", "sty")
    .and_then(|text| {
      let code = class_code(&text);
      class_code_argument(&code[code.find(MARK)? + MARK.len()..], "\\footnotetext")
    })
    .unwrap_or_else(|| s!("\\speakerText"))
}

#[rustfmt::skip]
LoadDefinitions!({
  // :88-127 `\author*[marks]{name}`: one author per call (a name list in one is an author each, as jheppub's), its
  // optional marks the labels of the affiliations it has (`\affiliation[a]{…}`), the star the speaker, whom the title
  // page marks `\speakerFlag` (:84 `\ast`) and names in a footnote, "∗ Speaker" (:226-233): that footnote is the
  // speaker's note, labelled `lx@pos@speaker`, which every starred author requests beside its marks (one footnote
  // under every star; given once). A label of its own, as the star is a mark: an unlabelled note would be one of the
  // notes the kernel deals out one per labelled author, in order, when they are as many — `\author[a]{Ann Lee}
  // \author*[b]{Bob Kim}\onbehalf{…}` gave Ann Lee the "Speaker" note and Bob Kim the on-behalf line.
  DefMacro!("\\author OptionalMatch:* []{}", sub[(star, marks, name)] {
    let author = if star.is_some() { "\\lx@pos@speaker@author" } else { "\\lx@pos@author" };
    Ok(Invocation!(T_CS!(author), vec![marks, Some(name)]))
  });
  DefMacro!("\\lx@pos@author[]{}", "\\lx@add@authors@append[annotations={#1}]{#2}");
  DefMacro!("\\lx@pos@speaker@author[]{}",
    "\\if\\relax\\detokenize{#1}\\relax\\expandafter\\@firstoftwo\\else\\expandafter\\@secondoftwo\\fi\
     {\\lx@pos@author[lx@pos@speaker]{#2}}{\\lx@pos@author[#1,lx@pos@speaker]{#2}}\\lx@pos@speaker@note");
  DefMacro!("\\lx@pos@speaker@note",
    "\\lx@add@contact[role=note,label={lx@pos@speaker}]{\\lx@pos@speaker}\\global\\let\\lx@pos@speaker@note\\relax");
  // `\affiliation[label]{…}` (:411-419), `\emailAdd{…}` (:421-430, the "E-mail:" line under the affiliations, each
  // address after the first after a comma), `\note[mark]{…}` (:380-400, a footnote's text inside an `\author`) and
  // `\dedicated{…}` (:359-360): jheppub.sty's commands, to the same effect.
  jhep_contact_markup()?;
  // `\ShortTitle{…}` and `\FullConference{…}` (:362-369): PoS.cls's.
  pos_title_page_fields()?;
  // `\onbehalf{for the X Collaboration}` (:249-260): the line `\printAuthors` sets after the names (:223), "Javier M.
  // Duarte∗ on behalf of the ALICE, ATLAS, CMS, and LHCb Collaborations" (2409.20413; `\onbehalf*` sets it below
  // them): a note of the author given last before it, after whose name it is printed (2308.05475, 2507.06616 give it
  // after their author lists).
  DefMacro!("\\onbehalf OptionalMatch:* {}", "\\lx@add@contact[role=note]{#2}");
  // `\notes{…}` (:357): the text the title page prints under the abstract (:506), with no label
  // (FIDELITY_CHARTER §4, user ruling 2026-10-30; OXIDIZED_DESIGN_DIVERGENCES #921).
  DefMacro!("\\notes{}", "\\lx@add@pubnote[role=note,name={}]{#1}");
  // :356 `\renewcommand{\abstract}[1]{\gdef\@abstract{#1}}`: the abstract is one argument, as jheppub.sty's.
  AssignValue!("\\abstract:redefined@nargs" => Stored::Number(Number::new(1)), Some(Scope::Global));
  RawTeX!(&s!("\\def\\lx@pos@speaker{{{}}}", speaker_text()));
  if shipped_file_text("pos", "sty").is_some() {
    InputDefinitions!("pos", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  } else {
    // (no copy: the packages pos.sty:21-50 loads)
    RequirePackage!("amsthm");
    RequirePackage!("amsmath");
    RequirePackage!("graphicx");
    RequirePackage!("natbib", options => vec![s!("numbers"), s!("sort&compress")]);
    RequirePackage!("caption", options => vec![s!("font=small"), s!("labelfont=bf")]);
    RequirePackage!("xcolor");
    RequirePackage!("hyperref");
  }
  RawTeX!(r"\providecommand{\speakerText}{Speaker}\providecommand{\speakerFlag}{\ast}");
});
