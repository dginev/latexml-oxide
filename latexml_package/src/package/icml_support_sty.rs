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
  // Perl gobbles \icmltitlerunning; surpass: it's the running-head
  // variant of the title, genuine author metadata.
  DefMacro!("\\icmltitlerunning{}",
    "\\@add@frontmatter{ltx:toctitle}{#1}");
  DefMacro!("\\icmlsetsymbol{}{}", None);

  DefEnvironment!("{icmlauthorlist}", "#body");

  DefMacro!("\\icmlauthor{}{}", "\\author{#1}");
  DefConstructor!("\\@@@address{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\icmladdress{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#1}}");
  // ICML: \icmlaffiliation{shortname}{full text} maps a short id to
  // an affiliation string used in author list. Preserve as ltx:note.
  DefMacro!("\\icmlaffiliation{}{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1: #2}");
  // \icmlcorrespondingauthor{email}{name} — preserve as ltx:note.
  DefMacro!("\\icmlcorrespondingauthor{}{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding-author]{#2 <#1>}");

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
  // ICML 2025: extended marker for joint first + senior authorship.
  // Witness: 2503.15703 (icml2025.sty L534).
  DefMacro!("\\icmlEqualContributionAndSenior",
    "\\textsuperscript{*}Equal contribution, \
     \\textsuperscript{\\char`\u{2020}}Equal senior authorship");
  DefMacro!("\\icmlkeywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");

  // Random extra bits
  DefMacro!("\\abovestrut{}", None);
  DefMacro!("\\belowstrut{}", None);
  DefMacro!("\\abovespace", None);
  DefMacro!("\\aroundspace", None);
  DefMacro!("\\belowspace", None);
  DefMacro!("\\icmlruler{}", None);
});
