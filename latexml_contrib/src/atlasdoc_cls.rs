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
  // and what the cover pages and `\AtlasPaperWebpage` read; each stores as the class does, the last call winning. At
  // `\maketitle` (:1074) the class picks one of four title headers (:991-1006) by its options (:82-91): with
  // `preprint`/`cernpreprint` (64 of 67 sampled papers) the CERN preprint header (:835-856), the journal reference or
  // else "Submitted to: <journal>", the DOI and the preprint number; otherwise with `HEPDATA` the HepData header
  // (:866-893), the preprint number and the arXiv id with its category (:878); with `BOOK` (:894ff; `PAPER`, `CONF` and
  // `PUB` turn it off, :140-192) none of them; otherwise the default header (:746-747), "<\subprepend> <journal>"
  // ("Journal:" by default). The same, as the frontmatter notes, each prefix the note's label. Run 340: nearly every
  // atlasdoc paper calls them (`\AtlasRefCode` 470, `\PreprintIdNumber` 147, `\AtlasJournalRef` 32, `\AtlasVersion` 52;
  // witnesses 2508.20929, 2103.01029, 1904.03536, 2407.10879).
  RawTeX!(
    r"\newcommand*{\ADOC@draftversion}{}\newcommand*{\draftversion}[1]{\renewcommand*{\ADOC@draftversion}{#1}}
\newcommand*{\AtlasVersion}[1]{\renewcommand*{\ADOC@draftversion}{#1}}
\newcommand*{\ADOC@refcode}{}\newcommand*{\AtlasRefCode}[1]{\renewcommand*{\ADOC@refcode}{#1}}
\newcommand*{\AtlasPaperWebpage}{https://atlas.web.cern.ch/Atlas/GROUPS/PHYSICS/PAPERS/\ADOC@refcode}
\newcommand*{\ADOC@atlasnote}{}\newcommand*{\AtlasNote}[1]{\renewcommand*{\ADOC@atlasnote}{#1}}
\newcommand*{\ADOC@journal}{}
\newcommand*{\AtlasJournal}[1]{\renewcommand*{\ADOC@journal}{#1}}\def\journal#1{\renewcommand*{\ADOC@journal}{#1}}
\newcommand*{\ADOC@preprintnumber}{}
\newcommand*{\PreprintIdNumber}[1]{\renewcommand*{\ADOC@preprintnumber}{#1}}
\newcommand*{\ADOC@journalref}{}
\newcommand*{\AtlasJournalRef}[1]{\renewcommand*{\ADOC@journalref}{#1}}
\newcommand*{\ADOC@doi}{}\newcommand*{\AtlasDOI}[1]{\renewcommand*{\ADOC@doi}{#1}}
\newif\iflx@atlas@cernpreprint\newif\iflx@atlas@hepdata\newif\iflx@atlas@book
\def\lx@atlas@titlepage{\iflx@atlas@cernpreprint
  \ifx\ADOC@journalref\@empty\ifx\ADOC@journal\@empty\else
    \lx@add@pubnote[role=journal,name={Submitted to:~}]{\ADOC@journal}\fi
  \else\lx@add@pubnote[role=journalref]{\ADOC@journalref}\fi
  \ifx\ADOC@doi\@empty\else\lx@add@pubnote[role=doi]{\ADOC@doi}\fi
  \ifx\ADOC@preprintnumber\@empty\else\lx@add@pubnote[role=preprint]{\ADOC@preprintnumber}\fi
\else\iflx@atlas@hepdata
  \ifx\ADOC@preprintnumber\@empty\else\lx@add@pubnote[role=preprint]{\ADOC@preprintnumber}\fi
  \ifx\AtlasArXivId\@empty\else\lx@add@pubnote[role=arxiv]{\AtlasArXivId\ \AtlasArXivCat}\fi
\else\iflx@atlas@book\else
  \ifx\ADOC@journal\@empty\else\lx@add@pubnote[role=journal,name={\ADOC@subprepend~}]{\ADOC@journal}\fi
\fi\fi\fi}
\newcommand*{\AtlasArXivId}{}\newcommand*{\AtlasArXivURL}{}
\newcommand*{\arXivId}[1]{\renewcommand*{\AtlasArXivId}{#1}\renewcommand*{\AtlasArXivURL}{https://arXiv.org/abs/#1}}
\newcommand*{\AtlasArXivCat}{[hep-ex]}
\newcommand*{\AtlasHEPDataRecord}{}
\newcommand*{\HepDataRecord}[1]{\renewcommand*{\AtlasHEPDataRecord}{https://www.hepdata.net/record/ins#1}}
\newcommand*{\HEPDataRecord}[1]{\renewcommand*{\AtlasHEPDataRecord}{https://www.hepdata.net/record/ins#1}}
\newcommand*{\ADOC@subprepend}{Journal:}\newcommand*{\subprepend}[1]{\renewcommand*{\ADOC@subprepend}{#1}}"
  );
  // The class's kvoptions (`preprint`, `preprint=true`, `preprint = false`, …; the last setting wins) choose the header:
  // `preprint` or `cernpreprint` the CERN preprint one, which also turns `HEPDATA` off (:82-91).
  let options = ["\\opt@atlasdoc.cls", "\\@classoptionslist"]
    .iter()
    .find_map(|list| {
      lookup_definition(&T_CS!(list))
        .ok()
        .flatten()
        .and_then(|defn| match defn.get_expansion() {
          Some(ExpansionBody::Tokens(body)) => Some(body.to_string()),
          _ => None,
        })
    });
  let setting = |key: &str| {
    options
      .as_deref()
      .unwrap_or("")
      .split(',')
      .fold(false, |set, option| {
        let option: String = option.chars().filter(|c| !c.is_whitespace()).collect();
        match option.split_once('=') {
          None if option == key => true,
          Some((name, value)) if name == key => value == "true",
          _ => set,
        }
      })
  };
  let cern_preprint = setting("preprint") || setting("cernpreprint");
  if cern_preprint {
    RawTeX!(r"\lx@atlas@cernpreprinttrue");
  } else if setting("HEPDATA") {
    RawTeX!(r"\lx@atlas@hepdatatrue");
  } else if setting("BOOK") && !["PAPER", "CONF", "PUB"].iter().any(|kind| setting(kind)) {
    RawTeX!(r"\lx@atlas@booktrue");
  }
  AddToMacro!("\\lx@maketitle@body", "\\lx@atlas@titlepage");
  // :304-305 `\AtlasDate` (the title page's date, `\today` by default) as the document's date.
  DefMacro!("\\AtlasDate{}", "\\date{#1}");
  // The cover-page commands (the class's own, :339ff, for the `coverpage` option): the analysis team, editors,
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
