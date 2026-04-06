//! beamer.cls — Minimal stubs for beamer presentation class
//! Perl: beamer.cls.ltxml (1364 lines)
//!
//! Provides enough definitions for the beamer test to pass without loading
//! the raw beamer.cls (which exceeds the 5M token limit). Full beamer
//! support requires porting the complete Perl binding.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Load article.cls as the base class (beamer builds on article).
  // Don't load raw beamer.cls — its expansion chains exceed the token limit.
  RequirePackage!("article");

  // Frame environment — the core beamer construct.
  // Absorbs optional overlay spec and optional title/subtitle args.
  // Perl: DefEnvironment('{frame}[][]', '<ltx:slide...>...</ltx:slide>');
  DefEnvironment!("{frame}[][]",
    "<ltx:subsection _noautoclose='1'>#body</ltx:subsection>");

  // Overlay specification commands — stub as no-ops
  DefMacro!("\\visible{}", "#1");
  DefMacro!("\\uncover{}", "#1");
  DefMacro!("\\invisible{}", "#1");
  DefMacro!("\\only{}", "#1");
  DefMacro!("\\onslide", "");
  DefMacro!("\\temporal{}{}{}", "#2");
  DefMacro!("\\pause", "");
  DefMacro!("\\alt{}{}", "#1");

  // Frame structure
  DefMacro!("\\frametitle OptionalMatch:<> []{}",
    "\\par\\textbf{#3}\\par");
  DefMacro!("\\framesubtitle OptionalMatch:<> {}", "");

  // Insert counters
  DefMacro!("\\insertframenumber", "");
  DefMacro!("\\insertslidenumber", "");
  DefMacro!("\\insertpagenumber", "");
  DefMacro!("\\insertoverlaynumber", "");

  // Overlay environments
  DefEnvironment!("{onlyenv}", "#body");
  DefEnvironment!("{altenv}{}{}{}{}", "#body");
  DefEnvironment!("{alertenv}", "#body");
  DefEnvironment!("{uncoverenv}", "#body");
  DefEnvironment!("{actionenv}", "#body");
  DefEnvironment!("{visibleenv}", "#body");
  DefEnvironment!("{invisibleenv}", "#body");
  DefEnvironment!("{overlayarea}{}{}", "#body");
  DefEnvironment!("{overprint}", "#body");

  // Block environments — Perl L1189 beamerbaseblocks.sty
  DefEnvironment!("{block} OptionalMatch:<> {}",
    "<ltx:theorem class='ltx_theorem_block'><ltx:title class='ltx_runin'>#2</ltx:title>#body</ltx:theorem>");
  DefEnvironment!("{alertblock} OptionalMatch:<> {}",
    "<ltx:theorem class='ltx_theorem_alertblock'><ltx:title class='ltx_runin'>#2</ltx:title>#body</ltx:theorem>");
  DefEnvironment!("{exampleblock} OptionalMatch:<> {}",
    "<ltx:theorem class='ltx_theorem_exampleblock'><ltx:title class='ltx_runin'>#2</ltx:title>#body</ltx:theorem>");

  // Columns environment — Perl L1230-1240 beamerbaseboxes.sty
  DefEnvironment!("{columns} OptionalMatch:<> []", "#body");
  DefEnvironment!("{column} OptionalMatch:<> {}", "#body");
  DefMacro!("\\column OptionalMatch:<> {}", "");

  // Title page macros — Perl L1010-1035
  DefMacro!("\\institute OptionalMatch:<> []{}", "\\@add@frontmatter{ltx:creator}{\\@@@affiliation{#3}}");
  DefMacro!("\\logo{}", "");
  DefMacro!("\\titlegraphic{}", "");
  DefMacro!("\\titlepage", "\\maketitle");
  DefMacro!("\\insertauthor", "");
  DefMacro!("\\inserttitle", "");
  DefMacro!("\\insertsubtitle", "");
  DefMacro!("\\insertdate", "");
  DefMacro!("\\insertinstitute", "");
  DefMacro!("\\insertshortauthor[]", "");
  DefMacro!("\\insertshortdate[]", "");
  DefMacro!("\\insertshortinstitute[]", "");
  DefMacro!("\\insertshorttitle[]", "");
  DefMacro!("\\inserttotalframenumber", "");

  // Theme commands — Perl L1246-1253
  DefMacro!("\\usetheme[]{}", "");
  DefMacro!("\\usecolortheme[]{}", "");
  DefMacro!("\\usefonttheme[]{}", "");
  DefMacro!("\\useinnertheme[]{}", "");
  DefMacro!("\\useoutertheme[]{}", "");
  DefMacro!("\\setbeamertemplate{}{}", "");
  DefMacro!("\\setbeamercolor{}{}", "");
  DefMacro!("\\setbeamerfont{}{}", "");
  DefMacro!("\\setbeamersize{}", "");
  DefMacro!("\\setbeamercovered{}", "");
  DefMacro!("\\addtobeamertemplate{}{}{}", "");
  DefMacro!("\\defbeamertemplate OptionalMatch:* {}{}{}", "");

  // Navigation/footline/headline — no-ops
  DefMacro!("\\beamertemplatenavigationsymbolsempty", "");
  DefMacro!("\\setbeamercolor*{}{}", "");
  DefMacro!("\\hypersetup{}", "");

  // Beamer list environments — Perl L1160-1179
  DefEnvironment!("{itemize} OptionalMatch:<>",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    mode => "internal_vertical", locked => true);
  DefEnvironment!("{enumerate} OptionalMatch:<> []",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    mode => "internal_vertical");
  DefEnvironment!("{description} OptionalMatch:<>",
    "<ltx:description xml:id='#id'>#body</ltx:description>",
    mode => "internal_vertical", locked => true);

  // Theorems — Perl L1193-1230
  RequirePackage!("amsthm");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RawTeX!(r#"
\newtheorem{theorem}{Theorem}
\newtheorem{corollary}[theorem]{Corollary}
\newtheorem{fact}[theorem]{Fact}
\newtheorem{lemma}[theorem]{Lemma}
\newtheorem{problem}[theorem]{Problem}
\newtheorem{solution}[theorem]{Solution}
\newtheorem{definition}[theorem]{Definition}
\newtheorem{definitions}[theorem]{Definitions}
\newtheorem{example}[theorem]{Example}
\newtheorem{examples}[theorem]{Examples}
"#);
  DefMacro!("\\pushQED{}", "");
  DefMacro!("\\popQED", "");
  DefMacro!("\\qedhere", "");

  // Mode commands — Perl L448-460
  DefMacro!("\\mode OptionalMatch:* {}", "");
  DefMacro!("\\mode<>{}", "");

  // Misc commands
  DefMacro!("\\alert OptionalMatch:<> {}", "\\textbf{#2}");
  DefMacro!("\\structure OptionalMatch:<> {}", "#2");
  DefMacro!("\\emph OptionalMatch:<> {}", "\\textit{#2}");
  DefMacro!("\\AtBeginSection[]{}", "");
  DefMacro!("\\AtBeginSubsection[]{}", "");
  DefMacro!("\\AtBeginPart[]{}", "");
  DefMacro!("\\lecture{}{}", "");
  DefMacro!("\\againframe OptionalMatch:<> []{}", "");
  DefMacro!("\\appendix", "");
  DefMacro!("\\note OptionalMatch:<> []{}", "");
  DefMacro!("\\beamerdefaultoverlayspecification{}", "");

  // Translation stubs
  DefMacro!("\\translate{}", "#1");

  // Color-related
  DefMacro!("\\usebeamercolor OptionalMatch:* {}", "");

  // Hyperlink
  DefMacro!("\\hyperlink{}{}", "#2");
  DefMacro!("\\hypertarget{}{}", "#2");
});
