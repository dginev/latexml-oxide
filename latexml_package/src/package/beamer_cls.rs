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

  // Perl beamer.cls.ltxml L853: DefKeyVal('beamerframe', 'fragile', '', '')
  // — declares `fragile` as a zero-argument key for the beamerframe keyset.
  // Documents using `\begin{frame}[fragile]{Title}` rely on this to parse
  // without "unknown keyval" errors. Frame env's Rust stub doesn't
  // consult keyvals yet, but the declaration itself must load.
  DefKeyVal!("beamerframe", "fragile", "");

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
  def_macro_identity("\\only{}")?;
  def_macro_noop("\\onslide")?;
  DefMacro!("\\temporal{}{}{}", "#2");
  def_macro_noop("\\pause")?;
  def_macro_identity("\\alt{}{}")?;

  // Perl beamer.cls.ltxml L796-798 dispatches \visible/\uncover/
  // \invisible via \alt to the \beamer@{visible,uncovered,…}
  // inline-block markers, but that routing needs the BeamerAngled
  // parameter type + \beamer@ifnextcharospec overlay dispatcher Rust
  // hasn't ported. Keep the body-passthrough stubs for now — the
  // markers below are still defined and usable directly by advanced
  // beamer styles that invoke them without angle-spec preprocessing.
  def_macro_identity("\\visible{}")?;
  def_macro_identity("\\uncover{}")?;
  def_macro_identity("\\invisible{}")?;

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
  def_macro_noop("\\framesubtitle OptionalMatch:<> {}")?;

  // Perl beamer.cls.ltxml L961-963: internal frame title constructors
  // that \frame@ / \beamer@frame@replay invoke to lift title/subtitle
  // onto the enclosing slide element via `^` float-to-parent. Rust
  // \frame stubs as ltx:subsection, so `^<ltx:title>` floats to that.
  // Unported until now, so beamer themes that invoke
  // \beamer@frametitle{...} directly (bypassing \frametitle) hit
  // undefined-CS errors. The three constructors all carry the same
  // float semantics, differing only in element (title vs subtitle)
  // and CSS class.
  DefConstructor!("\\beamer@frametitle{}",
    "^<ltx:title class='ltx_frame_title'>#1</ltx:title>");
  DefConstructor!("\\beamer@frameshorttitle{}",
    "^<ltx:title class='ltx_frame_shorttitle'>#1</ltx:title>");
  DefConstructor!("\\beamer@framesubtitle{}",
    "^<ltx:subtitle class='ltx_frame_subtitle'>#1</ltx:subtitle>");

  // Insert counters
  def_macro_noop("\\insertframenumber")?;
  def_macro_noop("\\insertslidenumber")?;
  def_macro_noop("\\insertpagenumber")?;
  def_macro_noop("\\insertoverlaynumber")?;

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
  def_macro_noop("\\column OptionalMatch:<> {}")?;

  // Title page macros — Perl L1010-1035
  DefMacro!("\\institute OptionalMatch:<> []{}", "\\@add@frontmatter{ltx:creator}{\\@@@affiliation{#3}}");
  // \logo{content} and \titlegraphic{content} typically wrap
  // \includegraphics or similar visual content. Surpass Perl
  // (which doesn't define them) by routing to ltx:note so any
  // \includegraphics inside resolves and the graphic is preserved.
  DefMacro!("\\logo{}", "\\@add@frontmatter{ltx:note}[role=logo]{#1}");
  DefMacro!("\\titlegraphic{}",
    "\\@add@frontmatter{ltx:note}[role=titlegraphic]{#1}");
  DefMacro!("\\titlepage", "\\maketitle");
  def_macro_noop("\\insertauthor")?;
  def_macro_noop("\\inserttitle")?;
  def_macro_noop("\\insertsubtitle")?;
  def_macro_noop("\\insertdate")?;
  def_macro_noop("\\insertinstitute")?;
  def_macro_noop("\\insertshortauthor[]")?;
  def_macro_noop("\\insertshortdate[]")?;
  def_macro_noop("\\insertshortinstitute[]")?;
  def_macro_noop("\\insertshorttitle[]")?;
  def_macro_noop("\\insertshortpart[]")?;
  def_macro_noop("\\insertshortsubtitle[]")?;
  def_macro_noop("\\inserttotalframenumber")?;
  def_macro_noop("\\insertmainframenumber")?;
  def_macro_noop("\\insertappendixframenumber")?;

  // Perl L1013-1045 beamerTODO navigation + page-range \insert*s.
  // All are stomach-time no-ops under Rust's continuous-document
  // rendering (beamer's slide-tracking state machine is not ported).
  // Shipping the stubs prevents undefined-CS errors in beamer themes
  // that reference them via `\setbeamertemplate{footline}` bodies.
  def_macro_noop("\\insertnavigation{}")?;
  def_macro_noop("\\insertsectionnavigation{}")?;
  def_macro_noop("\\insertsectionnavigationhorizontal{}{}{}")?;
  def_macro_noop("\\insertsubsectionnavigation{}")?;
  def_macro_noop("\\insertsubsectionnavigationhorizontal{}{}{}")?;
  def_macro_noop("\\insertverticalnavigation{}")?;
  def_macro_noop("\\insertsubsection")?;
  def_macro_noop("\\insertsubsubsection")?;
  def_macro_noop("\\insertframestartpage")?;
  def_macro_noop("\\insertframeendpage")?;
  def_macro_noop("\\insertsubsectionstartpage")?;
  def_macro_noop("\\insertsubsectionendpage")?;
  def_macro_noop("\\insertsectionstartpage")?;
  def_macro_noop("\\insertsectionendpage")?;
  def_macro_noop("\\insertpartstartpage")?;
  def_macro_noop("\\insertpartendpage")?;
  def_macro_noop("\\insertpresentationstartpage")?;
  def_macro_noop("\\insertpresentationendpage")?;
  def_macro_noop("\\insertappendixstartpage")?;
  def_macro_noop("\\insertappendixendpage")?;
  def_macro_noop("\\insertdocumentstartpage")?;
  def_macro_noop("\\insertdocumentendpage")?;

  // Theme commands — Perl L1246-1253
  def_macro_noop("\\usetheme[]{}")?;
  def_macro_noop("\\usecolortheme[]{}")?;
  def_macro_noop("\\usefonttheme[]{}")?;
  def_macro_noop("\\useinnertheme[]{}")?;
  def_macro_noop("\\useoutertheme[]{}")?;
  def_macro_noop("\\setbeamertemplate{}{}")?;
  def_macro_noop("\\setbeamercolor{}{}")?;
  def_macro_noop("\\setbeamerfont{}{}")?;
  def_macro_noop("\\setbeamersize{}")?;
  def_macro_noop("\\setbeamercovered{}")?;
  def_macro_noop("\\addtobeamertemplate{}{}{}")?;
  def_macro_noop("\\defbeamertemplate OptionalMatch:* {}{}{}")?;

  // Navigation/footline/headline — no-ops
  def_macro_noop("\\beamertemplatenavigationsymbolsempty")?;
  def_macro_noop("\\setbeamercolor*{}{}")?;
  def_macro_noop("\\hypersetup{}")?;

  // Beamer list environments — Perl L1160-1179
  DefEnvironment!("{itemize} OptionalMatch:<>",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    mode => "internal_vertical", locked => true);
  DefEnvironment!("{enumerate} OptionalMatch:<> []",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    mode => "internal_vertical");
  // Perl beamer.cls.ltxml L1174-1179: description's \item[label] renders
  // labels via \makelabel which beamer rebinds to \descriptionlabel
  // (defined in ams_support_sty:188 as bold+space). Same pattern
  // enumitem_sty:444 and ieeetran_cls:287 use.
  DefEnvironment!("{description} OptionalMatch:<>",
    "<ltx:description xml:id='#id'>#body</ltx:description>",
    before_digest => { Let!("\\makelabel", "\\descriptionlabel"); },
    mode => "internal_vertical", locked => true);

  // Theorems — Perl L1193-1230
  RequirePackage!("amsthm");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  // Perl beamer.cls.ltxml L1201-1239: theorem + German-compat envs.
  // `\translate{}` is an identity pass-through in Rust, so bare English
  // names match Perl's expansion.
  RawTeX!(r#"
\newcommand{\ExampleInline}[1]{\translate{Example}: \ignorespaces#1}
\newcommand{\BeispielInline}[1]{Beispiel: \ignorespaces#1}
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
\newtheorem{Beispiel}[theorem]{Beispiel}
\newtheorem{Beispiele}[theorem]{Beispiele}
\newtheorem{Loesung}[theorem]{L\"osung}
\newtheorem{Satz}[theorem]{Satz}
\newtheorem{Folgerung}[theorem]{Folgerung}
\newtheorem{Fakt}[theorem]{Fakt}
\newenvironment{Beweis}{\begin{proof}[Beweis.]}{\end{proof}}
\newenvironment{Lemma}{\begin{lemma}}{\end{lemma}}
\newenvironment{Proof}{\begin{proof}}{\end{proof}}
\newenvironment{Theorem}{\begin{theorem}}{\end{theorem}}
\newenvironment{Problem}{\begin{problem}}{\end{problem}}
\newenvironment{Corollary}{\begin{corollary}}{\end{corollary}}
\newenvironment{Example}{\begin{example}}{\end{example}}
\newenvironment{Examples}{\begin{examples}}{\end{examples}}
\newenvironment{Definition}{\begin{definition}}{\end{definition}}
"#);
  def_macro_noop("\\pushQED{}")?;
  def_macro_noop("\\popQED")?;
  def_macro_noop("\\qedhere")?;

  // Mode commands — Perl L448-460
  def_macro_noop("\\mode OptionalMatch:* {}")?;
  def_macro_noop("\\mode<>{}")?;
  // Perl L493-495: \presentation / \article / \common route to
  // \mode<…>. Since the Rust \mode dispatcher is already a no-op for
  // all overlay modes, the three become empty stubs. Including them
  // keeps preamble-level `\mode<all>` equivalents (from example
  // beamer style files) from throwing undefined-CS errors.
  def_macro_noop("\\presentation")?;
  def_macro_noop("\\common")?;
  // `\article` would clash with LaTeX `\article` docclass naming in
  // principle, but LaTeXML's catcode + class-file routing keeps the
  // control sequence distinct from the class name string. Perl ships
  // this alias unconditionally.
  def_macro_noop("\\article")?;

  // Perl L414-416: beamer TODO CSes (expand to warnings under Perl;
  // Rust matches by absorbing args and emitting nothing — same
  // behaviour for slide-order rendering without the beamerTODO warning.
  def_macro_noop("\\jobnamebeamerversion{}")?;
  def_macro_noop("\\includeslide{}")?;
  def_macro_noop("\\setjobnamebeamerversion")?;

  // Misc commands
  // Perl beamer.cls.ltxml L810-813 wraps \alert in \alertenv which threads
  // through \beamer@alerted@begin/end (inline-block markers defined
  // above). Routing through those requires BeamerAngled overlay parsing
  // (unported), so keep the \textbf fallback — the markers remain defined
  // and usable directly by styles that invoke them without angle-spec.
  DefMacro!("\\alert OptionalMatch:<> {}", "\\textbf{#2}");
  DefMacro!("\\structure OptionalMatch:<> {}", "#2");
  DefMacro!("\\emph OptionalMatch:<> {}", "\\textit{#2}");
  def_macro_noop("\\AtBeginSection[]{}")?;
  def_macro_noop("\\AtBeginSubsection[]{}")?;
  def_macro_noop("\\AtBeginPart[]{}")?;
  // \lecture{title}{shortname} — beamer lecture frontmatter; preserve
  // the title text as ltx:note frontmatter rather than dropping it.
  DefMacro!("\\lecture{}{}",
    "\\@add@frontmatter{ltx:note}[role=lecture]{#1}");
  def_macro_noop("\\againframe OptionalMatch:<> []{}")?;
  def_macro_noop("\\appendix")?;
  def_macro_noop("\\note OptionalMatch:<> []{}")?;
  def_macro_noop("\\beamerdefaultoverlayspecification{}")?;

  // Translation stubs
  def_macro_identity("\\translate{}")?;

  // Color-related
  def_macro_noop("\\usebeamercolor OptionalMatch:* {}")?;

  // Hyperlink
  DefMacro!("\\hyperlink{}{}", "#2");
  DefMacro!("\\hypertarget{}{}", "#2");
});
