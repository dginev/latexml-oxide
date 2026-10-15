//! Stub for oup-authoring-template.cls (Oxford University Press journal template).
//!
//! User-bundled OUP class with many metadata setters and font/layout
//! configuration. Provide content-preserving stubs for the user-facing
//! metadata so author content reaches the XML output.
//!
//! Witness: 2503.21884 (\\journaltitle, \\DOI, \\access, ...).
use latexml_package::prelude::*;

use crate::shipped_class::{class_code_contains, shipped_class_text};

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("graphicx");
  RequirePackage!("hyperref");
  RequirePackage!("array");
  RequirePackage!("caption");
  // oup.cls:137 `\RequirePackage[figuresright]{rotating}` — `sidewaystable`
  // must exist for its `\caption` (oup-authoring-template). Guard:
  // `perfect_kernel_batch54::class_stubs_require_their_float_packages`.
  RequirePackage!("rotating");

  // OUP-specific metadata setters — preserve content.
  DefMacro!(
    "\\journaltitle{}",
    "\\@add@frontmatter{ltx:note}[role=journal]{#1}"
  );
  DefMacro!("\\DOI{}", "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!(
    "\\access{}",
    "\\@add@frontmatter{ltx:note}[role=access]{#1}"
  );
  DefMacro!(
    "\\copyrightyear{}",
    "\\@add@frontmatter{ltx:note}[role=copyright-year]{#1}"
  );
  DefMacro!(
    "\\copyrightstatement{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1}"
  );
  DefMacro!("\\pubyear{}", "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!(
    "\\appnotes{}",
    "\\@add@frontmatter{ltx:note}[role=appnotes]{#1}"
  );
  // Line numbers cite the vendor TL2025 tree (/usr/local/texlive/2025), not
  // the distro copy. oup-authoring-template.cls:810 `\def\authormark##1{\gdef\leftmark{##1}}`:
  // a running-head setter, no output — the earlier `\textsuperscript{#1}`
  // typeset "Author Name et al." as a leading `<para>` before the title.
  DefMacro!("\\authormark{}", "");
  // cls:1145 `\def\corresp{\@@corresp}` → `\newcommand{\@@corresp}[2][]`: an
  // optional mark then the text; the one-arg stub took `[` as the argument
  // and leaked `$\ast$]Corresponding author…` into the body.
  DefMacro!(
    "\\corresp[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#2}"
  );
  DefMacro!(
    "\\firstpage{}",
    "\\@add@frontmatter{ltx:note}[role=firstpage]{#1}"
  );
  DefMacro!(
    "\\titlemark{}",
    "\\@add@frontmatter{ltx:note}[role=titlemark]{#1}"
  );
  DefMacro!(
    "\\runninghead{}",
    "\\@add@frontmatter{ltx:note}[role=runninghead]{#1}"
  );
  // cls:1085-1087 `\def\received#1#2#3`: a day/month-switch/year triple
  // (`\myswitch`, cls:1059, maps the month number to its name when the raw
  // class is loaded; the number stands in otherwise).
  DefMacro!(
    "\\received{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=received]{#1 \\ifdefined\\myswitch\\myswitch{#2}\\else#2\\fi\\ #3}"
  );
  // cls:1085-1087 `\def\revised#1#2#3`: a day/month-switch/year triple.
  DefMacro!(
    "\\revised{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=revised]{#1 \\ifdefined\\myswitch\\myswitch{#2}\\else#2\\fi\\ #3}"
  );
  // cls:1085-1087 `\def\accepted#1#2#3`: a day/month-switch/year triple.
  DefMacro!(
    "\\accepted{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=accepted]{#1 \\ifdefined\\myswitch\\myswitch{#2}\\else#2\\fi\\ #3}"
  );
  DefMacro!(
    "\\editor{}",
    "\\@add@frontmatter{ltx:note}[role=editor]{#1}"
  );
  DefMacro!(
    "\\address[]{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#2}"
  );
  // oup-authoring-template.cls:39 `\newif\if@numbib \@numbibtrue` (2407.13283's copy; TL 2025's :39 the same), the
  // numbered-references switch a paper sets (3 papers of the inventory).
  RawTeX!(r"\newif\if@numbib\@numbibtrue");
  // oup-authoring-template.cls:1054 (TL 2025; 2312.04714's copy :1034) `\vol{N}`, the volume the running head
  // prints (45 papers of the inventory: 2104.15079, 2204.11994, 2206.08262).
  DefMacro!("\\vol{}", "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  // :1117-1119 `\boxedtext{text}`, a boxed statement under the abstract, headed "Key Messages" (TL 2025, 14 of the
  // sampled copies) or "Significance statement" (2312.04714's :1107, 7 copies; 18 papers: 2306.17210); the newer
  // copies take the heading as a first argument, `\boxedtext{heading}{text}` (2604.25257's :1271-1272, 44 of the 65
  // sampled copies). The heading leads the note's text, bold, as the box prints it.
  let shipped = shipped_class_text("oup-authoring-template")?;
  let has = |needle: &str| {
    shipped
      .as_deref()
      .is_some_and(|text| class_code_contains(text, needle))
  };
  if has("\\newcommand{\\@@boxedtext}[2]") {
    RawTeX!(r"\def\boxedtext#1#2{\@add@frontmatter{ltx:note}[role=boxedtext]{\textbf{#1} #2}}");
  } else {
    let heading = if has("\\textbf{Significance statement}\\par#1") {
      "Significance statement"
    } else {
      "Key Messages"
    };
    raw_tex(&format!(
      r"\def\boxedtext#1{{\@add@frontmatter{{ltx:note}}[role=boxedtext]{{\textbf{{{heading}}} #1}}}}"
    ))?;
  }
  // :143-150 the sans-serif font switches the class's headings use, which papers call (`\sffamilyfont`,
  // `\sffamilyfontbold`: 2505.18036).
  RawTeX!(
    r"\def\sffamilyfont{\sffamily}\def\sffamilyfontitalic{\sffamily\itshape\selectfont}
\def\sffamilyfontbold{\sffamily\bfseries\selectfont}\def\sffamilyfontbolditalic{\sffamily\bfseries\itshape\selectfont}
\def\sffamilyfontcn{\sffamily\fontseries{m}\fontshape{n}\selectfont}\def\sffamilyfontcnitalic{\sffamily\fontseries{m}\fontshape{it}\selectfont}
\def\sffamilyfontcnbold{\sffamily\bfseries\selectfont}\def\sffamilyfontcnbolditalic{\sffamily\fontseries{b}\fontshape{it}\selectfont}"
  );
  // :1676-1720 `\secsize`, `\subsecsize`, `\subsubsecsize`, the heading fonts, bold in each of the class's styles (2
  // papers: 2309.05250); their sizes and colours are print layout.
  RawTeX!(r"\def\secsize{\bfseries}\def\subsecsize{\bfseries}\def\subsubsecsize{\bfseries}");
  // Commands newer copies add (TL 2025's, of 2022, has neither): defined where the paper's copy has them, so a paper
  // whose copy does not can define its own. 2602.07066's copy :1232 `\otherabstract[heading]{text}`, a further
  // abstract under its own heading (5 papers), and :1935 `\figalttext[]{text}`, a figure's alternative text, which
  // the class drops (4 papers).
  if let Some(text) = shipped.as_deref() {
    if class_code_contains(text, "\\newcommand\\otherabstract[2]") {
      RawTeX!(
        r"\newcommand\otherabstract[2][Other abstract heading]{\@add@frontmatter{ltx:abstract}[name={#1}]{#2}}"
      );
    }
    if class_code_contains(text, "\\newcommand\\figalttext[2]") {
      def_macro_noop("\\figalttext[]{}")?;
    }
  }
  // SetCrop/SetTrim/SetBleed are layout configs — gobble.
  def_macro_noop("\\SetCrop{}{}")?;
  def_macro_noop("\\SetTrim{}{}")?;
  def_macro_noop("\\SetBleed{}{}")?;
  // oup-authoring-template.cls:2733 `\DeclareRobustCommand\ORCID[1]{…\href
  // {https://orcid.org/#1}{\XeTeXLinkBox{…\pic{orcidlogo}…}}}`: the link with
  // the id as its text (the logo picture is out of scope).
  DefMacro!("\\ORCID{}", "\\href{https://orcid.org/#1}{#1}");

  // oup-authoring-template.cls, TL 2025's copy :2423-2435 (2102.02669's :2390-2400): required packages;
  // `\RequirePackage{multirow}` (TL 2025 :2426, 2102.02669 :2391; 39 papers of run 353 with `\multirow` undefined:
  // 2102.02669, 2104.08961, 2206.08262), `{url}` (:2428, :2393), `{mathrsfs}` (:2430, :2395).
  RequirePackage!("multirow");
  RequirePackage!("url");
  RequirePackage!("mathrsfs");
  RequirePackage!("booktabs");
  RequirePackage!("algorithm");
  RequirePackage!("algorithmicx");
  RequirePackage!("algpseudocode");
  RequirePackage!("listings");
  RequirePackage!("appendix");

  // Table rules and notes
  Let!("\\botrule", "\\bottomrule");
  DefMacro!("\\tablenotes[]", "\\begin{itemize}");
  DefMacro!("\\endtablenotes", "\\end{itemize}");

  // Organization address fields (oup-authoring-template.cls:1120-1127)
  DefMacro!("\\orgdiv{}", "#1");
  DefMacro!("\\orgname{}", "#1");
  DefMacro!("\\orgaddress{}", "#1");
  DefMacro!("\\street{}", "#1");
  DefMacro!("\\postcode{}", "#1");
  DefMacro!("\\state{}", "#1");
  DefMacro!("\\country{}", "#1");

  // Environments (oup-authoring-template.cls:2098, 2303)
  DefEnvironment!(
    "{unlist}",
    "<ltx:itemize class='unlist'>#body</ltx:itemize>"
  );
  DefEnvironment!(
    "{biography}{}{}",
    "<ltx:note role='biography'>#1 #2#body</ltx:note>",
    before_digest => {
      DefMacro!("\\author{}", "{\\bfseries #1\\enspace}");
    },
    mode => "internal_vertical"
  );

  // Predefined theorem styles (oup-authoring-template.cls:2370-2410)
  RequirePackage!("amsthm");
  Let!("\\th@thmstyleone", "\\th@plain");
  Let!("\\th@thmstyletwo", "\\th@plain");
  Let!("\\th@thmstylethree", "\\th@definition");
  Let!("\\th@thmstylefour", "\\th@remark");
});
