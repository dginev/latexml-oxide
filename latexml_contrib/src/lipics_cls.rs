//! Stub for LIPIcs class (Dagstuhl Leibniz International Proceedings).
use latexml_package::prelude::*;


LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  // Do NOT eager-load xcolor (Perl ships no lipics binding → OmniBus, no
  // preload). A preloaded xcolor makes a later `\usepackage[table]{xcolor}`
  // a no-op → colortbl/array never load → array `m{}`/`b{}` columns are
  // "Unrecognized tabular template" → "Extra alignment tab". The document
  // loads xcolor with its own options; `\color`/`\definecolor` stay
  // available via hyperref→color. See ifacconf_cls.rs / SYNC_STATUS.
  RequirePackage!("hyperref");

  // LIPIcs frontmatter — preserve author content as ltx:note
  // frontmatter entries with role markers.
  DefMacro!("\\Copyright{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1}");
  def_macro_noop("\\CopyrightDetails")?;
  DefMacro!("\\authorrunning{}",
    "\\@add@frontmatter{ltx:note}[role=runningauthor]{#1}");
  DefMacro!("\\titlerunning{}",
    "\\@add@frontmatter{ltx:note}[role=runningtitle]{#1}");
  DefMacro!("\\funding{}",
    "\\@add@frontmatter{ltx:note}[role=funding]{#1}");
  DefMacro!("\\fundingAgency{}",
    "\\@add@frontmatter{ltx:note}[role=funding-agency]{#1}");
  DefMacro!("\\authorcredit{}",
    "\\@add@frontmatter{ltx:note}[role=authorcredit]{#1}");
  def_macro_noop("\\nolinenumbers")?;
  DefMacro!("\\category{}",
    "\\@add@frontmatter{ltx:note}[role=category]{#1}");
  DefMacro!("\\related{}",
    "\\@add@frontmatter{ltx:note}[role=related]{#1}");
  DefMacro!("\\relatedversion{}",
    "\\@add@frontmatter{ltx:note}[role=relatedversion]{#1}");
  DefMacro!("\\supplement{}",
    "\\@add@frontmatter{ltx:note}[role=supplement]{#1}");
  DefMacro!("\\supplementdetails[]{}{}",
    "\\@add@frontmatter{ltx:note}[role=supplement]{#2: #3}");
  // \acknowledgements{text} — render as structural ltx:acknowledgements
  // (post-processors map to canonical role/styling).
  DefConstructor!("\\acknowledgements{}",
    "<ltx:acknowledgements>#1</ltx:acknowledgements>");
  DefMacro!("\\ccsdesc[]{}",
    "\\@add@frontmatter{ltx:classification}[scheme=ccs]{#2}");
  DefMacro!("\\subjclass[]{}",
    "\\@add@frontmatter{ltx:classification}[scheme=AMS]{#2}");
  DefMacro!("\\keywords{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}");
  DefMacro!("\\event{}",
    "\\@add@frontmatter{ltx:note}[role=event]{#1}");
  DefMacro!("\\EventEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors]{#1}");
  DefMacro!("\\EventLongTitle{}",
    "\\@add@frontmatter{ltx:note}[role=event-title]{#1}");
  DefMacro!("\\EventShortTitle{}",
    "\\@add@frontmatter{ltx:note}[role=event-shorttitle]{#1}");
  DefMacro!("\\EventAcronym{}",
    "\\@add@frontmatter{ltx:note}[role=event-acronym]{#1}");
  DefMacro!("\\EventYear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\EventDate{}",
    "\\@add@frontmatter{ltx:note}[role=event-date]{#1}");
  DefMacro!("\\EventLocation{}",
    "\\@add@frontmatter{ltx:note}[role=event-location]{#1}");
  // EventLogo wraps \includegraphics or visual content; preserve.
  DefMacro!("\\EventLogo{}",
    "\\@add@frontmatter{ltx:note}[role=event-logo]{#1}");
  DefMacro!("\\SeriesVolume{}",
    "\\@add@frontmatter{ltx:note}[role=series-volume]{#1}");
  DefMacro!("\\ArticleNo{}",
    "\\@add@frontmatter{ltx:note}[role=articleno]{#1}");
  // LIPIcs L739: \EventNoEds{N} sets editor count.
  def_macro_noop("\\EventNoEds{}")?;
  // LIPIcs L860: \hideLIPIcs sets \@hideLIPIcs to suppress the
  // article-number/page header. No-op in XML. Witness 2502.11299 +6.
  def_macro_noop("\\hideLIPIcs")?;
  // \headers{left}{right} — LIPIcs running-header alias used by
  // some templates. Round-34 surpass-Perl: preserve as ltx:note so
  // the author-typed text isn't dropped.
  DefMacro!("\\headers{}{}",
    "\\@add@frontmatter{ltx:note}[role=runningheads]{#1 / #2}");

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
