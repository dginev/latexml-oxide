//! atlasdoc.cls — the ATLAS Collaboration paper/note class (author-bundled, very
//! large; not raw-loaded). A full binding (hundreds of physics-notation macros,
//! dozens of bundled `atlas*.sty`) is out of scope. This binds only the
//! FRONT-MATTER macros that otherwise leak as literal text — the reported issue
//! (witness 2508.20929 → `\AtlasTitle \AtlasAbstract` shown raw, no title/abstract).
//!
//! The ATLAS author list is `\input` in the document BODY (a `flushleft`
//! `{\Large The ATLAS Collaboration}` + `\AtlasOrcid[orcid]{Name}` list), not in
//! the pre-`\maketitle` frontmatter, so those names render as the collaboration
//! block in the body rather than as title-page creators; bind `\AtlasOrcid` so
//! the names show cleanly instead of leaking the macro.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");

  // atlasdoc.cls L296: `\newcommand{\AtlasTitle}[1]{\title{#1}…}`.
  DefMacro!("\\AtlasTitle{}", "\\title{#1}");
  // atlasdoc.cls L360: `\AtlasAbstract{#1}` stashes the abstract; surface it as
  // the document abstract.
  DefMacro!("\\AtlasAbstract{}", "\\lx@add@abstract{#1}");
  // atlasdoc.cls L515: `\NewDocumentCommand \AtlasOrcid { o m }` — optional ORCID
  // + author name. Render the name (the trailing `$^{…}$` affiliation marker in
  // the source follows naturally); drop the ORCID decoration.
  DefMacro!("\\AtlasOrcid[]{}", "#2");
  // atlasdoc.cls (v15, 2504.04805's copy :286-345) the document-data setters, which store what the title page prints
  // (:835-856: the journal reference, or "Submitted to: <journal>", the DOI, the preprint number and the date) and
  // what the cover pages and `\AtlasPaperWebpage` read. Each stores as the class does; the printed ones also become
  // the frontmatter notes. Run 340: nearly every atlasdoc paper calls them (`\AtlasRefCode` 470, `\PreprintIdNumber`
  // 147, `\AtlasJournalRef` 32, `\AtlasVersion` 52; witnesses 2508.20929, 2103.01029, 1904.03536, 2407.10879).
  RawTeX!(
    r"\newcommand*{\ADOC@draftversion}{}\newcommand*{\draftversion}[1]{\renewcommand*{\ADOC@draftversion}{#1}}
\newcommand*{\AtlasVersion}[1]{\renewcommand*{\ADOC@draftversion}{#1}}
\newcommand*{\ADOC@refcode}{}\newcommand*{\AtlasRefCode}[1]{\renewcommand*{\ADOC@refcode}{#1}}
\newcommand*{\AtlasPaperWebpage}{https://atlas.web.cern.ch/Atlas/GROUPS/PHYSICS/PAPERS/\ADOC@refcode}
\newcommand*{\ADOC@atlasnote}{}\newcommand*{\AtlasNote}[1]{\renewcommand*{\ADOC@atlasnote}{#1}}
\newcommand*{\ADOC@journal}{}
\newcommand*{\AtlasJournal}[1]{\renewcommand*{\ADOC@journal}{#1}\lx@add@pubnote[role=journal]{Submitted to: #1}}
\newcommand*{\ADOC@preprintnumber}{}
\newcommand*{\PreprintIdNumber}[1]{\renewcommand*{\ADOC@preprintnumber}{#1}\lx@add@pubnote[role=preprint]{#1}}
\newcommand*{\ADOC@journalref}{}
\newcommand*{\AtlasJournalRef}[1]{\renewcommand*{\ADOC@journalref}{#1}\lx@add@pubnote[role=journalref]{#1}}
\newcommand*{\ADOC@doi}{}\newcommand*{\AtlasDOI}[1]{\renewcommand*{\ADOC@doi}{#1}\lx@add@pubnote[role=doi]{#1}}
\newcommand*{\AtlasArXivId}{}\newcommand*{\AtlasArXivURL}{}
\newcommand*{\arXivId}[1]{\renewcommand*{\AtlasArXivId}{#1}\renewcommand*{\AtlasArXivURL}{https://arXiv.org/abs/#1}}
\newcommand*{\AtlasArXivCat}{[hep-ex]}
\newcommand*{\AtlasHEPDataRecord}{}
\newcommand*{\HepDataRecord}[1]{\renewcommand*{\AtlasHEPDataRecord}{https://www.hepdata.net/record/ins#1}}
\newcommand*{\HEPDataRecord}[1]{\renewcommand*{\AtlasHEPDataRecord}{https://www.hepdata.net/record/ins#1}}
\newcommand*{\ADOC@subprepend}{Journal:}\newcommand*{\subprepend}[1]{\renewcommand*{\ADOC@subprepend}{#1}}"
  );
  // :304-305 `\AtlasDate` (the title page's date, `\today` by default) as the document's date.
  DefMacro!("\\AtlasDate{}", "\\date{#1}");
  // The cover-page commands (atlascover.sty, which the class loads for `coverpage`; :575): the analysis team, editors,
  // editorial board, supporting notes and comment deadline print on the internal cover page only (run 340: 14 papers).
  for cover in [
    "\\AtlasCoverSupportingNote{}{}",
    "\\AtlasCoverAnalysisTeam{}",
    "\\AtlasCoverEditor{}{}",
    "\\AtlasCoverEdBoardMember{}",
    "\\AtlasCoverReaderMember{}",
    "\\AtlasCoverEgroupAnalysisTeam{}",
    "\\AtlasCoverEgroupEditors{}",
    "\\AtlasCoverEgroupEdBoard{}",
    "\\AtlasCoverCommentsDeadline{}",
    "\\AtlasCoverTwikiURL{}",
  ] {
    def_macro_noop(cover)?;
  }
});
