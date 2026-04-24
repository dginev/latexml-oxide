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
  // Rust's \alt{}{}/\only/\onslide/\temporal/\pause take the
  // "always-true" branch (first arg, or body) — faithful to what a
  // reader expects from beamer slides printed as a continuous
  // document. See Perl L793-834 for the full overlay/pause machinery.
  DefMacro!("\\only{}", "#1");
  DefMacro!("\\onslide", "");
  DefMacro!("\\temporal{}{}{}", "#2");
  DefMacro!("\\pause", "");
  DefMacro!("\\alt{}{}", "#1");

  // Perl beamer.cls.ltxml L796-798: \uncover / \visible / \invisible
  // dispatch via \alt to semantic inline-block markers. The markers at
  // L718-737 emit <ltx:inline-block class='ltx_visible'> / _invisible /
  // _uncovered / _covered / _alert wrappers. Without the markers,
  // post-processors stripping overlays can't distinguish visible text
  // from what would have been hidden. Perl TODO at L716 notes the
  // classes aren't yet consumed downstream, but shipping them preserves
  // the semantic hook for future CSS theming.
  DefMacro!("\\visible",   "\\alt{\\beamer@visible}{\\beamer@invisible}");
  DefMacro!("\\uncover",   "\\alt{\\beamer@uncovered}{\\beamer@covered}");
  DefMacro!("\\invisible", "\\alt{\\beamer@invisible}{\\beamer@visible}");

  DefMacro!("\\beamer@visible{}",   "\\beamer@visible@begin{#1}\\beamer@visible@end");
  DefConstructor!("\\beamer@visible@begin", "<ltx:inline-block class='ltx_visible'>");
  DefConstructor!("\\beamer@visible@end",   "</ltx:inline-block>");

  DefMacro!("\\beamer@invisible{}", "\\beamer@invisible@begin{#1}\\beamer@invisible@end");
  DefConstructor!("\\beamer@invisible@begin", "<ltx:inline-block class='ltx_invisible'>");
  DefConstructor!("\\beamer@invisible@end",   "</ltx:inline-block>");

  DefMacro!("\\beamer@uncovered{}", "\\beamer@uncovered@begin{#1}\\beamer@uncovered@end");
  DefConstructor!("\\beamer@uncovered@begin", "<ltx:inline-block class='ltx_uncovered'>");
  DefConstructor!("\\beamer@uncovered@end",   "</ltx:inline-block>");

  DefMacro!("\\beamer@covered{}", "\\beamer@covered@begin{#1}\\beamer@covered@end");
  DefConstructor!("\\beamer@covered@begin", "<ltx:inline-block class='ltx_covered'>");
  DefConstructor!("\\beamer@covered@end",   "</ltx:inline-block>");

  DefMacro!("\\beamer@alerted{}", "\\beamer@alerted@begin{#1}\\beamer@alerted@end");
  DefConstructor!("\\beamer@alerted@begin", "<ltx:inline-block class='ltx_alert'>");
  DefConstructor!("\\beamer@alerted@end",   "</ltx:inline-block>");

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
  DefMacro!("\\insertshortpart[]", "");
  DefMacro!("\\insertshortsubtitle[]", "");
  DefMacro!("\\inserttotalframenumber", "");
  DefMacro!("\\insertmainframenumber", "");
  DefMacro!("\\insertappendixframenumber", "");

  // Perl L1013-1045 beamerTODO navigation + page-range \insert*s.
  // All are stomach-time no-ops under Rust's continuous-document
  // rendering (beamer's slide-tracking state machine is not ported).
  // Shipping the stubs prevents undefined-CS errors in beamer themes
  // that reference them via `\setbeamertemplate{footline}` bodies.
  DefMacro!("\\insertnavigation{}", "");
  DefMacro!("\\insertsectionnavigation{}", "");
  DefMacro!("\\insertsectionnavigationhorizontal{}{}{}", "");
  DefMacro!("\\insertsubsectionnavigation{}", "");
  DefMacro!("\\insertsubsectionnavigationhorizontal{}{}{}", "");
  DefMacro!("\\insertverticalnavigation{}", "");
  DefMacro!("\\insertsubsection", "");
  DefMacro!("\\insertsubsubsection", "");
  DefMacro!("\\insertframestartpage", "");
  DefMacro!("\\insertframeendpage", "");
  DefMacro!("\\insertsubsectionstartpage", "");
  DefMacro!("\\insertsubsectionendpage", "");
  DefMacro!("\\insertsectionstartpage", "");
  DefMacro!("\\insertsectionendpage", "");
  DefMacro!("\\insertpartstartpage", "");
  DefMacro!("\\insertpartendpage", "");
  DefMacro!("\\insertpresentationstartpage", "");
  DefMacro!("\\insertpresentationendpage", "");
  DefMacro!("\\insertappendixstartpage", "");
  DefMacro!("\\insertappendixendpage", "");
  DefMacro!("\\insertdocumentstartpage", "");
  DefMacro!("\\insertdocumentendpage", "");

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
  // Perl L493-495: \presentation / \article / \common route to
  // \mode<…>. Since the Rust \mode dispatcher is already a no-op for
  // all overlay modes, the three become empty stubs. Including them
  // keeps preamble-level `\mode<all>` equivalents (from example
  // beamer style files) from throwing undefined-CS errors.
  DefMacro!("\\presentation", "");
  DefMacro!("\\common",       "");
  // `\article` would clash with LaTeX `\article` docclass naming in
  // principle, but LaTeXML's catcode + class-file routing keeps the
  // control sequence distinct from the class name string. Perl ships
  // this alias unconditionally.
  DefMacro!("\\article",      "");

  // Perl L414-416: beamer TODO CSes (expand to warnings under Perl;
  // Rust matches by absorbing args and emitting nothing — same
  // behaviour for slide-order rendering without the beamerTODO warning.
  DefMacro!("\\jobnamebeamerversion{}",   "");
  DefMacro!("\\includeslide{}",           "");
  DefMacro!("\\setjobnamebeamerversion",  "");

  // Misc commands
  // Perl beamer.cls.ltxml L810-813 wraps `\alert{…}` in `\alertenv` which
  // threads through `\beamer@alerted@begin` / `@end` — the semantic
  // inline-block marker ported above. Route through that instead of a
  // bare \textbf so downstream CSS can style ltx_alert independently of
  // bold, and authoring `\setbeamercolor{alerted text}{…}` can be
  // honoured later without re-porting the call site.
  DefMacro!("\\alert OptionalMatch:<> {}",
    "\\beamer@alerted@begin{#2}\\beamer@alerted@end");
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
