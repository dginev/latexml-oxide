//! Binding for appolb.cls (Acta Physica Polonica B; not in TeX Live, and the papers that use it do not ship it, so
//! without a binding the class is OmniBus's; OXIDIZED_DESIGN_DIVERGENCES #719).
use latexml_package::prelude::*;

LoadDefinitions!({
  // appolb.cls (v3.5, 2210.03396's copy) is a class of its own, not on article.cls: OmniBus serves its frontmatter
  // (`\title`, `\author`, `\address`, `\thanks`, `{abstract}`), as it did with no binding.
  LoadClass!("OmniBus");
  // appolb.cls:322 `\def\PACS#1{\par\noindent\strut\kern18pt{\PACSsize\rm PACS numbers: #1}\par}`, the classification
  // after the abstract (13 papers of the int19 census with `\PACS` undefined: 0712.3606, 1007.2277, 1101.3125,
  // 1108.2782, 1605.06550, nucl-th0308018, physics0501160); :321 `\received`, the date line.
  DefMacro!(
    "\\PACS{}",
    "\\lx@add@classification[scheme=pacs,name={PACS numbers: }]{#1}"
  );
  DefMacro!("\\received{}", "\\date{#1}");
  // appolb.cls:578-582 `\eqsec`, which the class's template invites ("uncomment this line to get equations numbered by
  // (sec.num)"): equations numbered within sections.
  RawTeX!(
    r"\def\eqsec{\def\theequation{\thesection.\@arabic\c@equation}\@addtoreset{equation}{section}}"
  );
  // appolb.cls:299, :304 `\headtitle`, `\headauthor`, the running head's title and authors (hep-ph0403117), as the
  // running-head notes of other journal bindings; :1019-1033 its text abbreviations (`\ie`: hep-ph0403117), each
  // followed by a space unless punctuation follows (`\comsp`); :1059 `\e`, the upright e.
  DefMacro!(
    "\\headtitle{}",
    "\\@add@frontmatter{ltx:note}[role=runningtitle]{#1}"
  );
  DefMacro!(
    "\\headauthor{}",
    "\\@add@frontmatter{ltx:note}[role=runningauthor]{#1}"
  );
  RawTeX!(
    r"\def\comsp{\@ifnextchar,\relax{\@ifnextchar\ \relax{\@ifnextchar:\relax{\@ifnextchar:\relax{\ }}}}}
\DeclareRobustCommand{\ie}{{\it i.e.}\comsp}\DeclareRobustCommand{\eg}{{\it e.g.}\comsp}
\DeclareRobustCommand{\cf}{{\it cf.}\comsp}\DeclareRobustCommand{\etal}{{\it et~al.}\comsp}
\DeclareRobustCommand{\ibid}{{\it ibid.}\comsp}"
  );
  DefMath!("\\e", "e", font => { shape => "upright" });
  // appolb.cls:1036-1048 `\lambdabar`, a barred lambda (U+019B, as newtxmath's).
  DefMath!("\\lambdabar", "\u{019B}", role => "ID");
  // The rest of the class's public commands, as the class defines them, since the binding also stands in for a copy a
  // paper ships (0801.2427, cond-mat0205657): :83 `\ifRedakcja` (the `reda` option's editorial mode, whose `redakcja`
  // package is the journal's own), :278-282 `\preprint` with its `\ifPreprint` and `\prepNo` (the preprint number the
  // title page sets in place of the journal heading; here a preprint note, as revtex's `\preprint`), :283-285 the
  // `\volume` and `\No` counts and `\pageno` (the journal heading's), :311 `\WERS`, :1019 `\anymode`, :1053-1057
  // `\half`/`\Half` (LaTeX's `\frac` form), :1061 `\Tr`, :1063 `\Eq`, :1065-1066 `\seteqprefix`/`\reseteqprefix`.
  // Guard `perfect_kernel_batch71a::appolb_class_commands`.
  RawTeX!(
    r"\newif\ifRedakcja \Redakcjafalse
\newif\ifPreprint \Preprintfalse
\def\prepNo{}
\long\def\preprint#1{\Preprinttrue\long\gdef\prepNo{#1}\lx@add@pubnote[role=preprint]{#1}}
\newcount\volume \volume=0
\let\pageno=\c@page
\newcount\No \No=0
\let\WERS\MakeUppercase
\def\anymode#1{\ifmmode #1\else $#1$\fi}
\def\half{{\textstyle\frac12}}
\def\Half{\frac12}
\def\Tr{\mathop{\operator@font Tr}\nolimits}
\def\Eq#1{Eq. (\ref{#1})}
\def\seteqprefix#1{\renewcommand{\p@equation}{#1}}
\def\reseteqprefix#1{\setcounter{equation}{0}\renewcommand{\p@equation}{#1}}"
  );
});
