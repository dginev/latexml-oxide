/// Perl: icml_support.sty.ltxml — Support for ICML conference styles
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("times");
  RequirePackage!("fancyhdr");
  RequirePackage!("color");
  // ICML 2024/2025 templates use xcolor's \colorlet for callout colors;
  // load xcolor eagerly. Witness 2405.18180 (icml2025).
  RequirePackage!("xcolor");
  RequirePackage!("algorithm");
  RequirePackage!("algorithmic");
  RequirePackage!("natbib");

  // Citations
  DefMacro!("\\yrcite Semiverbatim", "\\citeyearpar{#1}");
  DefMacro!("\\cite Semiverbatim", "\\citep{#1}");

  // Frontmatter
  Let!("\\icmltitle", "\\title");
  DefMacro!("\\icmltitlerunning{}", None);
  DefMacro!("\\icmlsetsymbol{}{}", None);

  DefEnvironment!("{icmlauthorlist}", "#body");

  DefMacro!("\\icmlauthor{}{}", "\\author{#1}");
  DefConstructor!("\\@@@address{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\icmladdress{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#1}}");
  DefMacro!("\\icmlaffiliation{}{}", None);
  DefMacro!("\\icmlcorrespondingauthor{}{}", None);

  // \printAffiliationsAndNotice / \printAffiliationsAndWorkNotice emit
  // a re-iteration of the affiliation list + a free-form notice. Since
  // \icmladdress already feeds frontmatter, the affiliation list is
  // captured separately; preserve the notice arg as a ltx:note so the
  // author-supplied "Work done while at X" string survives.
  // Witness: 2502.18679 (icml2025.sty L564).
  DefMacro!("\\printAffiliationsAndNotice{}",
    "\\@add@frontmatter{ltx:note}[role=affiliationnotice]{#1}");
  DefMacro!("\\printAffiliationsAndWorkNotice{}",
    "\\@add@frontmatter{ltx:note}[role=affiliationnotice]{#1}");
  DefMacro!("\\icmlEqualContribution", "Equal contribution");
  DefMacro!("\\icmlkeywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");

  // Random extra bits
  DefMacro!("\\abovestrut{}", None);
  DefMacro!("\\belowstrut{}", None);
  DefMacro!("\\abovespace", None);
  DefMacro!("\\aroundspace", None);
  DefMacro!("\\belowspace", None);
  DefMacro!("\\icmlruler{}", None);
});
