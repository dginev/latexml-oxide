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
});
