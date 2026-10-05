//! Stub for LIPIcs class (Dagstuhl Leibniz International Proceedings).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // lipics-v2021.cls:514-1098 loads these packages, in this order. The binding stands in for the class, so they load
  // here or not at all: `\newcolumntype` (array, 2609.13401), `{subfigure}` (subcaption, 2609.10114), the `{CCSXML}`
  // block the class excludes (comment, :861-862; its `_`s were math errors, 2609.13485), `\multirow`, `tabularx`,
  // `threeparttable`, `listings`, soul with the class's `\textsolittle` (:540-541)… Font, PDF and page-layout packages
  // (microtype, lmodern, fontawesome5, hyperxmp, totpages, colorprofiles) are left out, and so are the class's MnSymbol
  // block (:534-538) and its options to amsmath (`tbtags,fleqn`).
  // Not xcolor at class time: the class loads `color` and takes xcolor at the document's start only when the paper
  // has not (:543-546, below), so a paper's own `\usepackage[table]{xcolor}` decides its options (colortbl's
  // `\rowcolor`, array `m{}` columns). (Perl has no lipics binding: OmniBus with `maybeRequireDependencies`,
  // Package.pm:2732, 2776-2794, which regex-scans the shipped class and so loads xcolor at class time.)
  RequirePackage!("fontenc", options => vec![s!("T1")]);
  RequirePackage!("textcomp");
  RequirePackage!("eucal", options => vec![s!("mathscr")]);
  RequirePackage!("amssymb");
  RequirePackage!("soul");
  RawTeX!(
    r"\sodef\textsolittle{}{.12em}{.5em\@plus.08em\@minus.06em}{.4em\@plus.275em\@minus.183em}"
  );
  RequirePackage!("color");
  // :542-553: at the document's start, xcolor if the paper has not loaded it (so a paper's own `[table]` stands) and the
  // class's named colours, which papers use (`lipicsYellow`, 2609.13401).
  RawTeX!(
    r"\AtBeginDocument{\@ifpackageloaded{xcolor}{}{\RequirePackage{xcolor}}\definecolor{darkgray}{rgb}{0.31,0.31,0.33}\definecolor[named]{lipicsGray}{rgb}{0.31,0.31,0.33}\definecolor[named]{lipicsBulletGray}{rgb}{0.60,0.60,0.61}\definecolor[named]{lipicsLineGray}{rgb}{0.51,0.50,0.52}\definecolor[named]{lipicsLightGray}{rgb}{0.85,0.85,0.86}\definecolor[named]{lipicsYellow}{rgb}{0.99,0.78,0.07}}"
  );
  RequirePackage!("babel");
  RequirePackage!("amsmath");
  RequirePackage!("enumerate");
  RequirePackage!("graphicx");
  RequirePackage!("array");
  RequirePackage!("multirow");
  RequirePackage!("tabularx");
  RequirePackage!("threeparttable", options => vec![s!("online")]);
  RequirePackage!("listings");
  RequirePackage!("lineno", options => vec![s!("left"), s!("mathlines")]);
  RequirePackage!("hyperref");
  // :707-709, the caption setup the class gives its figures (passed for fidelity: the caption binding stores these keys
  // and reads none of them).
  RequirePackage!("caption", options => vec![
    s!("labelsep=space"),
    s!("singlelinecheck=false"),
    String::from("font={up,small}"),
    String::from("labelfont={sf,bf}"),
    s!("listof=false"),
  ]);
  RequirePackage!("rotating", options => vec![s!("figuresright")]);
  RequirePackage!("subcaption");
  RequirePackage!("xstring");
  RequirePackage!("comment");
  RawTeX!(r"\excludecomment{CCSXML}");
  RequirePackage!("amsthm");
  // :193/:1015-1016: the `thm-restate` documentclass OPTION (`\DeclareOption{thm-restate}{\let\usethmrestate\relax}` →
  // `\ifx\usethmrestate\relax\RequirePackage{thm-restate}\fi`) loads thm-restate, providing the `restatable`
  // environment. The binding does not process class options; thm-restate's `restatable` is self-contained and harmless
  // when unused, so it loads unconditionally. Witness 2211.04601 (`\documentclass[...,thm-restate]{lipics-v2021}`).
  RequirePackage!("thm-restate");
  // lipics-v2021.cls:1113 `\RequirePackage[capitalise,noabbrev]{cleveref}` (guarded by
  // the `cleveref` class option). Without it `\cref`/`\Cref` come out Error:undefined — Perl (no lipics binding:
  // OmniBus's dependency scan of the shipped class) gets cleveref. Must load AFTER hyperref. Witness 2606.01187.
  RequirePackage!("cleveref", options => vec!["capitalise".to_string(), "noabbrev".to_string()]);

  // LIPIcs frontmatter — preserve author content as ltx:note
  // frontmatter entries with role markers.
  // lipics-v2021.cls L919: \newcommand{\relatedversiondetails}[3][]{...\textit{#2}:
  // \href{#3}{...}...} — a "Related Version" line (type + URL, with optional
  // linktext=/cite= keyval). Was undefined (Perl defines it). Preserve the core
  // (type + linked URL); the optional keyval is dropped. Witness 2311.17226.
  DefMacro!(
    "\\relatedversiondetails[]{}{}",
    "\\@add@frontmatter{ltx:note}[role=related-version]{\\textit{#2}: \\href{#3}{#3}}"
  );
  DefMacro!(
    "\\Copyright{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1}"
  );
  def_macro_noop("\\CopyrightDetails")?;
  DefMacro!(
    "\\authorrunning{}",
    "\\@add@frontmatter{ltx:note}[role=runningauthor]{#1}"
  );
  DefMacro!(
    "\\titlerunning{}",
    "\\@add@frontmatter{ltx:note}[role=runningtitle]{#1}"
  );
  DefMacro!(
    "\\funding{}",
    "\\@add@frontmatter{ltx:note}[role=funding]{#1}"
  );
  DefMacro!(
    "\\fundingAgency{}",
    "\\@add@frontmatter{ltx:note}[role=funding-agency]{#1}"
  );
  DefMacro!(
    "\\authorcredit{}",
    "\\@add@frontmatter{ltx:note}[role=authorcredit]{#1}"
  );
  def_macro_noop("\\nolinenumbers")?;
  DefMacro!(
    "\\category{}",
    "\\@add@frontmatter{ltx:note}[role=category]{#1}"
  );
  DefMacro!(
    "\\related{}",
    "\\@add@frontmatter{ltx:note}[role=related]{#1}"
  );
  DefMacro!(
    "\\relatedversion{}",
    "\\@add@frontmatter{ltx:note}[role=relatedversion]{#1}"
  );
  DefMacro!(
    "\\supplement{}",
    "\\@add@frontmatter{ltx:note}[role=supplement]{#1}"
  );
  DefMacro!(
    "\\supplementdetails[]{}{}",
    "\\@add@frontmatter{ltx:note}[role=supplement]{#2: #3}"
  );
  // \acknowledgements{text} — render as structural ltx:acknowledgements
  // (post-processors map to canonical role/styling).
  DefConstructor!(
    "\\acknowledgements{}",
    "<ltx:acknowledgements>#1</ltx:acknowledgements>"
  );
  DefMacro!(
    "\\ccsdesc[]{}",
    "\\@add@frontmatter{ltx:classification}[scheme=ccs]{#2}"
  );
  DefMacro!(
    "\\subjclass[]{}",
    "\\@add@frontmatter{ltx:classification}[scheme=AMS]{#2}"
  );
  DefMacro!(
    "\\keywords{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}"
  );
  DefMacro!("\\event{}", "\\@add@frontmatter{ltx:note}[role=event]{#1}");
  DefMacro!(
    "\\EventEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors]{#1}"
  );
  DefMacro!(
    "\\EventLongTitle{}",
    "\\@add@frontmatter{ltx:note}[role=event-title]{#1}"
  );
  DefMacro!(
    "\\EventShortTitle{}",
    "\\@add@frontmatter{ltx:note}[role=event-shorttitle]{#1}"
  );
  DefMacro!(
    "\\EventAcronym{}",
    "\\@add@frontmatter{ltx:note}[role=event-acronym]{#1}"
  );
  DefMacro!(
    "\\EventYear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}"
  );
  DefMacro!(
    "\\EventDate{}",
    "\\@add@frontmatter{ltx:note}[role=event-date]{#1}"
  );
  DefMacro!(
    "\\EventLocation{}",
    "\\@add@frontmatter{ltx:note}[role=event-location]{#1}"
  );
  // EventLogo wraps \includegraphics or visual content; preserve.
  DefMacro!(
    "\\EventLogo{}",
    "\\@add@frontmatter{ltx:note}[role=event-logo]{#1}"
  );
  DefMacro!(
    "\\SeriesVolume{}",
    "\\@add@frontmatter{ltx:note}[role=series-volume]{#1}"
  );
  DefMacro!(
    "\\ArticleNo{}",
    "\\@add@frontmatter{ltx:note}[role=articleno]{#1}"
  );
  // LIPIcs L739: \EventNoEds{N} sets editor count.
  def_macro_noop("\\EventNoEds{}")?;
  // LIPIcs L860: \hideLIPIcs sets \@hideLIPIcs to suppress the
  // article-number/page header. No-op in XML. Witness 2502.11299 +6.
  def_macro_noop("\\hideLIPIcs")?;
  // \headers{left}{right} — LIPIcs running-header alias used by
  // some templates. Round-34 surpass-Perl: preserve as ltx:note so
  // the author-typed text isn't dropped.
  DefMacro!(
    "\\headers{}{}",
    "\\@add@frontmatter{ltx:note}[role=runningheads]{#1 / #2}"
  );

  // LIPIcs L1158-1234: theorem-like environments.
  RawTeX!(
    r"\newtheorem{theorem}{Theorem}
\newtheorem{lemma}[theorem]{Lemma}
\newtheorem{corollary}[theorem]{Corollary}
\newtheorem{proposition}[theorem]{Proposition}
\newtheorem{definition}[theorem]{Definition}
\newtheorem{observation}[theorem]{Observation}
\newtheorem{remark}[theorem]{Remark}
\newtheorem{example}[theorem]{Example}
\newtheorem{claim}[theorem]{Claim}
\newtheorem{conjecture}[theorem]{Conjecture}"
  );
});
