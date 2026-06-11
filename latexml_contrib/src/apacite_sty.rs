//! Stub for apacite.sty (APA citation/bibliography style).
//!
//! Apacite defines a large family of APAref* macros driven by its .bbl
//! generator. We don't implement APA formatting; pass-through the
//! content-bearing args (authors, titles) and gobble the rest.
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("natbib");
  // apacite-generated .bbl entries routinely contain `\url{...}` and
  // `\doi{10.x/...}` even when the user's main .tex doesn't load url /
  // doi. Pre-load url (defines \url) and stub \doi as a no-op-wrapped
  // href so the bbl pass doesn't error out. Witness 2205.09172 (cogsci
  // article with apacite-formatted main.bbl).
  RequirePackage!("url");
  DefMacro!("\\doi Semiverbatim", "doi:#1");

  // Core APAref* set (apacite.sty L1257-2243). Render as the
  // content-bearing argument so titles / authors survive in the XML.
  def_macro_noop("\\APACinsertmetastar{}")?;
  DefMacro!("\\APACrefatitle{}{}", "#2");
  DefMacro!("\\APACrefbtitle{}{}", "#2");
  DefMacro!("\\APACrefYear{}", "(#1)");
  DefMacro!("\\APACrefYearMonthDay{}{}{}", "(#1)");
  DefMacro!("\\APACjournalVolNumPages{}{}{}{}", "#1 #2 #3 #4");
  def_macro_identity("\\APAChowpublished{}")?;
  DefMacro!("\\APACaddressPublisher{}{}", "#1: #2");
  DefMacro!("\\APACaddressInstitution{}{}", "#1: #2");
  DefMacro!("\\APACexlab{}{}", "#2");
  // \APACmonth{name} — month text (was gobbled). Pass through inline.
  def_macro_identity("\\APACmonth{}")?;
  def_macro_identity("\\APACrefnote{}")?;
  DefMacro!("\\APAhyperref{}{}", "#2");
  def_macro_noop("\\PrintBackRefs{}")?;
  def_macro_noop("\\CurrentBib")?;
  def_macro_noop("\\bibcomputersoftwaremanual{}{}{}")?;

  // APAref* environments
  DefEnvironment!("{APACrefauthors}", "#body");
  DefEnvironment!("{APACrefURL}", "#body");
  DefEnvironment!("{APACrefDOI}", "#body");

  // Additional APAC* macros (apacite ships many).
  DefMacro!("\\APACyear{}", "(#1)");
  DefMacro!("\\APACciteatitle{}{}", "#2");
  DefMacro!("\\APACcitebtitle{}{}", "#2");
  DefMacro!("\\APACrefaetitle{}{}", "#2");
  DefMacro!("\\APACrefbetitle{}{}", "#2");
  DefMacro!("\\APACbVolEdTR{}{}", "#2");
  DefMacro!("\\APACbVolEdTRpgs{}{}{}", "#3");
  DefMacro!("\\APACaddressInstitutionEqAuth{}{}", "#1: #2");
  DefMacro!("\\APACaddressPublisherEqAuth{}{}", "#1: #2");
  DefMacro!("\\APACaddressSchool{}{}", "#1: #2");
  DefMacro!("\\APACtypeAddressSchool{}{}{}", "#3");
  def_macro_noop("\\APACmetastar")?;
  DefMacro!("\\APACorigyearnote{}", "(#1)");
  def_macro_identity("\\APACorigjournalnote{}")?;
  def_macro_identity("\\APACorigbooknote{}")?;
  DefMacro!("\\APACorigED", "Ed.");
  DefMacro!("\\APACorigEDS", "Eds.");
  def_macro_identity("\\APACstd{}")?;
  def_macro_noop("\\APACSortNoop{}")?;
  def_macro_noop("\\APACmetaprenote")?;
  def_macro_noop("\\APACrefauthstyle{}")?;
  def_macro_noop("\\APACbibcite{}")?;

  // apacite citation forms (apacite.sty L328+). Delegate to natbib's
  // \cite which we wrapped in natbib_sty.rs. Forms:
  //   \citeA[pre][post]{key} — author-only ("Smith")
  //   \citeauthor[pre][post]{key} — author-only (alternate spelling)
  //   \citeNP[pre][post]{key} — citation without parens
  //   \citeyearNP[pre][post]{key} — year-only without parens
  // Witness 2407.14158, 2407.18402, 2407.16770 (apacite-using papers).
  DefMacro!("\\citeA[][] Semiverbatim", "\\citet[#1][#2]{#3}");
  DefMacro!("\\citeNP[][] Semiverbatim", "\\citealp[#1][#2]{#3}");
  DefMacro!("\\citeyearNP[][] Semiverbatim", "\\citeyear[#1][#2]{#3}");
  // apacite "short" cite family (apacite.sty L277-401, the CLASSIC block —
  // distinct from the `\citet`/`\citep` defined only under the `natbibemu`
  // option at L587+). These are abbreviated-author variants of
  // \cite/\citeA/\citeNP/\citeauthor: apacite shortens long author lists to
  // "et al." sooner, but the reference resolves identically, so we delegate
  // to the matching natbib command (same approximation as `\citeNP` above).
  //   \shortcite       — parenthetical          → \citep
  //   \shortciteA      — textual                → \citet
  //   \shortciteNP     — no parentheses         → \citealp
  //   \shortciteauthor — author-only            → \citeauthor
  // Witness 1606.03620 (`\shortciteNP`, via apacdoc's `\DSMshortciteNP`).
  DefMacro!("\\shortcite[][] Semiverbatim", "\\citep[#1][#2]{#3}");
  DefMacro!("\\shortciteA[][] Semiverbatim", "\\citet[#1][#2]{#3}");
  DefMacro!("\\shortciteNP[][] Semiverbatim", "\\citealp[#1][#2]{#3}");
  DefMacro!("\\shortciteauthor[][] Semiverbatim", "\\citeauthor{#3}");
  def_macro_noop("\\APACrestorebibitem")?;
  def_macro_noop("\\APACemindex{}")?;
  def_macro_noop("\\APACltxemindex{}")?;
  def_macro_noop("\\APACtocindex{}")?;
  def_macro_noop("\\APACstdindex{}")?;
  def_macro_noop("\\APACurlBreaks")?;

  // Short-form helpers (apacite L1300+: \BBA, \BCnt, \BPGS, etc.)
  // \BBA = `\BBAA` = `\&` (escaped ampersand, NOT alignment `&`).
  // apacite.sty L2123: `\newcommand{\BBAA}{\&}`. Using plain `&` would
  // emit a catcode-ALIGN cell separator inside the .bbl, triggering
  // 19+ "Stray alignment \"&\"" errors (witness 2205.09172).
  DefMacro!("\\BBA", "\\&");
  // \BBAA: same as \BBA — the underlying glyph macro.
  DefMacro!("\\BBAA", "\\&");
  def_macro_noop("\\BBCQ")?;
  def_macro_noop("\\BBOQ")?;
  DefMacro!("\\BPBI", ".");
  DefMacro!("\\BHBI", "-");
  def_macro_noop("\\BDBL")?;
  def_macro_noop("\\BCBT")?;
  def_macro_noop("\\BCBL")?;
  def_macro_identity("\\BCnt{}")?;
  def_macro_identity("\\BPGS{}")?;
  def_macro_identity("\\BVOL{}")?;
  DefMacro!("\\BOthers{}", "et al.");
  // apacite.sty L2082: `\newcommand{\BOthersPeriod}[1]{et al.\hbox{}}` —
  // "et al." variant ending the author list with a period. Used in
  // .bbl `\APACrefauthors` blocks. Witness 2005.03899 (apacite bibstyle).
  DefMacro!("\\BOthersPeriod{}", "et al.");
  DefMacro!("\\BEDS", "Eds.");
  DefMacro!("\\BED", "Ed.");
  DefMacro!("\\BIn", "In");
  // apacite.sty technical-report citation short forms. Witness 2205.05718.
  DefMacro!("\\BTR", "Tech.\\ Rep.");
  DefMacro!("\\BNUM", "No.");
  DefMacro!("\\BNUMS", "Nos.");
  DefMacro!("\\BTRANS", "Trans.");
  DefMacro!("\\BTRANSS", "Trans.");
  DefMacro!("\\BTRANSL", "trans.");
  // \BPG = singular page (vs \BPGS = pages). Witness 2205.09172 (cogsci
  // article + apacite .bbl): "\BPGS\ 1173--1182" plural already worked,
  // but single-page entries use "\BPG\ N". apacite.sty L1336.
  DefMacro!("\\BPG{}", "p.\\ #1");
  // Remaining apacite bibliography abbreviation text-macros (apacite.sty
  // L2018-2075). Perl ships no apacite binding and raw-loads apacite.sty, so
  // it gets ALL of these; our hand-built binding was missing many, surfacing
  // one-undefined-macro-at-a-time as `.bbl` entries used them. Port the full
  // text-abbreviation set verbatim from apacite.sty (the `\hbox{}` in the
  // originals is a no-op spacing hack we drop). NB `\BEd` (lowercase, "ed.")
  // is DISTINCT from the already-defined `\BED` ("Ed.", editor). Witness
  // 2106.02003 (`\PrintOrdinal{3}\ \BEd` in an apacite `main.bbl`).
  DefMacro!("\\BEd", "ed."); // edition (apacite L2037)
  DefMacro!("\\BVOLS", "Vols."); // volumes
  DefMacro!("\\BCHAP", "chap."); // chapter
  DefMacro!("\\BCHAPS", "chap."); // chapters
  DefMacro!("\\BCHAIR", "Chair");
  DefMacro!("\\BCHAIRS", "Chairs");
  DefMacro!("\\BIP", "in press");
  DefMacro!("\\Bby", "by");
  DefMacro!("\\BMTh", "Master's thesis");
  DefMacro!("\\BUMTh", "Unpublished master's thesis");
  DefMacro!("\\BPhD", "Doctoral dissertation");
  DefMacro!("\\BUPhD", "Unpublished doctoral dissertation");
  DefMacro!("\\BAuthor", "Author");
  DefMacro!("\\BOWP", "Original work published");
  DefMacro!("\\BREPR", "Reprinted from");
  DefMacro!("\\BAvailFrom", "Available from\\ ");
  DefMacro!("\\BRetrievedFrom", "Retrieved from\\ ");
  DefMacro!("\\BMsgPostedTo", "Message posted to\\ ");
  DefMacro!("\\BRetrieved{}", "Retrieved #1");
  DefMacro!("\\BBOP", "("); // bibliography open paren
  DefMacro!("\\BBCP", ")"); // bibliography close paren
  // \PrintOrdinal{N} → "Nth" ordinal (1st/2nd/3rd/4th…11th/12th/13th/Nth).
  // Ported verbatim from apacite.sty L2098-2138 (pure TeX; uses \count@,
  // \afterassignment, \ifcase, last-digit recursion). Used in `.bbl` edition
  // fields, e.g. `\PrintOrdinal{3}` → "3rd".
  RawTeX!(
    r"%
\let\@xp\expandafter
\newcommand{\PrintOrdinal}[1]{%
    \afterassignment\print@ordinal
    \count@ 0#1\relax\@nil}
\def\print@ordinal#1#2\@nil{%
    \ifx\relax#1\relax
        \ifnum\count@>\z@ \CardinalNumeric\count@ \else ??th\fi
    \else
        \ifnum \count@>\z@ \number\count@ \fi #1#2\relax
    \fi}
\newcommand{\CardinalNumeric}[1]{%
    \number#1\relax
    \if \ifnum#1<14 \ifnum#1>\thr@@ T\else F\fi \else F\fi Tth%
    \else \@xp\keep@last@digit\@xp#1\number#1\relax
        \ifcase#1th\or st\or nd\or rd\else th\fi
    \fi}
\def\keep@last@digit#1#2{%
    \ifx\relax#2\@xp\@gobbletwo \else #1=#2\relax \fi \keep@last@digit#1}"
  );
  // \shortciteA[pre][post]{key} — author-only short cite (apacite
  // citation form). Delegate to natbib's \citet (author-name cite).
  DefMacro!("\\shortciteA[][] Semiverbatim", "\\citet[#1][#2]{#3}");
  // \shortciteauthor[pre][post]{key} — short form of \citeauthor.
  DefMacro!(
    "\\shortciteauthor[][] Semiverbatim",
    "\\citeauthor[#1][#2]{#3}"
  );
  // apacite.sty L1448-1452 allocates \bibleftmargin et al via
  // `\newskip{\bibleftmargin}` and friends, then main.tex bodies use
  // `\setlength{\bibleftmargin}{...}` and `\setlength{\bibindent}{-\bibleftmargin}`.
  // We don't raw-load apacite.sty, so these must be defined as glue
  // registers here so \setlength's Variable parameter resolves.
  // Witness 2205.09172 (cogsci paper, "Error:expected:<variable>").
  DefRegister!("\\bibleftmargin" => Glue!("2.5em"));
  DefRegister!("\\bibindent"     => Glue::new(0));
  DefRegister!("\\bibparsep"     => Glue::new(0));
  DefRegister!("\\bibitemsep"    => Glue::new(0));
  DefRegister!("\\biblabelsep"   => Glue::new(0));
});
