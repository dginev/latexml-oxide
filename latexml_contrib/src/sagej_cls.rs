//! Stub for sagej.cls (SAGE journals).
use latexml_package::prelude::*;

use crate::shipped_class::{class_code_contains, class_option_given, shipped_class_text};

LoadDefinitions!({
  LoadClass!("OmniBus");
  // sagej.cls L108-L109 load these math-symbol/font packages:
  //   L108: \RequirePackage{latexsym,ifthen,rotating,calc,textcase,...}
  //   L109: \RequirePackage{amsfonts,amssymb,amsbsy,amsmath,amsthm}
  // Perl ships NO sagej binding → raw-loads the bundled sagej.cls → all of
  // these load, so e.g. `\leqslant` (amssymb), `\mathbb` (amsfonts) and the
  // latexsym glyphs are defined. Our OmniBus binding intercepts the class,
  // so the `\RequirePackage` line never runs — replicate the AMS cluster
  // here (in cls order: amsfonts before amssymb, which depends on it).
  // Pure math-symbol/font packages — no layout side effects. Witness
  // 1802.07225 (`\leqslant` in a paper that loads sagej but not amssymb).
  RequirePackage!("latexsym");
  RequirePackage!("amsfonts");
  RequirePackage!("amssymb");
  RequirePackage!("amsbsy");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // Do NOT eager-load xcolor (Perl ships no sagej binding → OmniBus, no
  // preload). A preloaded xcolor makes a later `\usepackage[table]{xcolor}`
  // a no-op → colortbl/array never load → array `m{}`/`b{}` columns are
  // "Unrecognized tabular template" → "Extra alignment tab". The document
  // loads xcolor with its own options; `\color`/`\definecolor` stay
  // available via hyperref→color. See ifacconf_cls.rs / SYNC_STATUS.
  RequirePackage!("hyperref");
  // sagej templates use \toprule / \midrule / \bottomrule from booktabs.
  // The raw cls relies on the user `\usepackage{booktabs}` but many
  // papers don't load it explicitly. Eager-load. Witness 2309.01813.
  RequirePackage!("booktabs");
  // sagej.cls L108: `\RequirePackage{latexsym,ifthen,rotating,calc,textcase,
  // booktabs,color,endnotes}`. Perl ships no sagej binding → raw-loads the
  // bundled sagej.cls → endnotes loaded → `\endnote`/`\theendnotes` defined.
  // Our binding intercepts sagej.cls (so that `\RequirePackage` never runs),
  // leaving `\endnote` undefined where Perl is clean. Load endnotes to match
  // the real .cls (SAGE papers use `\endnote{…}` for footnote-style notes).
  // Witness 1901.10968 (`\endnote`, `\theendnotes`).
  RequirePackage!("endnotes");
  // The rest of that line (79 of the 80 sampled copies; one loads xcolor for color): ifthen's `\newboolean`/
  // `\setboolean` (2204.02460), rotating's `{sideways}` (2 papers of the sample: 1412.4039), calc, textcase, color.
  RequirePackage!("ifthen");
  RequirePackage!("rotating");
  RequirePackage!("calc");
  RequirePackage!("textcase");
  RequirePackage!("color");

  // sagej frontmatter — preserve author content.
  DefMacro!(
    "\\corrauth{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
  DefMacro!(
    "\\affiliation{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}"
  );
  // sagej.cls:271 `\def\affilnum#1{${}^{\text{{#1}}}$}` — the author's and the affiliation's
  // superscript mark (2609.04585).
  DefMacro!("\\affilnum{}", "\\textsuperscript{#1}");
  DefMacro!(
    "\\runninghead{}",
    "\\@add@frontmatter{ltx:note}[role=runninghead]{#1}"
  );

  // {acks}, {funding}, {dci} envs (sagej L456-470). Use
  // internal_vertical mode so block-level body (paragraphs, lists,
  // funding-statement prose) is accepted — restricted_horizontal
  // default tripped `Attempt to end mode restricted_horizontal in
  // internal_vertical` on multi-paragraph bodies.
  // Each end-matter statement is headed by the class's own text (`\subsection*{\normalsize\sagesf\bfseries
  // <heading>}`), which is the statement's `name`: the XHTML prints `@name` as its visible heading
  // (LaTeXML-structure-xhtml.xsl:174-182, `ltx:acknowledgements`). 2101.10072's sagej.cls:460-470 `{funding}`
  // "Funding", `{dci}` "Declaration of conflicting interests" (every one of the 79 sampled copies); 2505.13309's
  // :460-462 `{credit}` "Author contributions" (3 copies); 2406.06038's :458-461 `{conflict}` "Declaration of
  // conflicting interests"; 2204.02460's :584-586 `{das}` "Data Accessibility Statement"; 2101.10072's :464-466 `{sm}`
  // "Supplemental material" (78 copies; 2503.14633's "Supplemental Material"). `{sm}` 43 papers of the inventory,
  // `{credit}` 3 (1412.4039, 1801.09386); `{acks}` keeps the kernel's acknowledgements heading.
  let shipped = shipped_class_text("sagej")?;
  DefEnvironment!("{acks}", "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{funding}", "<ltx:acknowledgements name='Funding'>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{dci}",
    "<ltx:acknowledgements name='Declaration of conflicting interests'>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{credit}", "<ltx:acknowledgements name='Author contributions'>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{conflict}",
    "<ltx:acknowledgements name='Declaration of conflicting interests'>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{das}", "<ltx:acknowledgements name='Data Accessibility Statement'>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  if shipped
    .as_deref()
    .is_some_and(|text| class_code_contains(text, "\\bfseries Supplemental Material}"))
  {
    DefEnvironment!("{sm}", "<ltx:acknowledgements name='Supplemental Material'>#body</ltx:acknowledgements>",
      mode => "internal_vertical");
  } else {
    DefEnvironment!("{sm}", "<ltx:acknowledgements name='Supplemental material'>#body</ltx:acknowledgements>",
      mode => "internal_vertical");
  }
  // 2101.10072's sagej.cls:118-120, 231, 235, 237: `\sagesf` the class's sans-serif switch (`\rm` under the `PCfour`
  // option; 17 papers: 2101.10072), `{refsize}` and `\titlesize` its type sizes, `\absbox` the abstract's box;
  // :38 `\journalclass` the class's name, printed in its page foot.
  RawTeX!(
    r"\let\sagesf\sf
\newcommand{\refsize}{\fontsize{9}{12pt}\selectfont}
\newcommand{\titlesize}{\fontsize{18.0}{20pt}\selectfont}
\newbox\absbox
\newcommand{\journalclass}{sagej.cls}\newcommand{\journalclassshort}{sagej}"
  );
  if class_option_given("PCfour")? {
    RawTeX!(r"\let\sagesf\rm");
  }
  // 2101.10072's :480-486 `{biog}` and `{biogs}`, the author biographies under the heading "Author biography" or
  // "Author Biographies" (13 and 2 papers: 1801.00336, 1912.06398).
  RawTeX!(
    r"\newenvironment{biog}{\subsection*{Author biography}\begin{refsize}\noindent}{\end{refsize}}
\newenvironment{biogs}{\subsection*{Author Biographies}\begin{refsize}\noindent}{\end{refsize}}"
  );
  // 2401.17393's :246-248 `\Highlights{…}`, the highlights the title page prints with the keywords, and 2309.06697's
  // :257-260 `\twitter{handle}`, a title-page footnote (2 and 1 papers).
  DefMacro!(
    "\\Highlights{}",
    "\\@add@frontmatter{ltx:note}[role=highlights]{#1}"
  );
  DefMacro!(
    "\\twitter{}",
    "\\@add@frontmatter{ltx:note}[role=twitter]{#1}"
  );
});
