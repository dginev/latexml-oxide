/// Perl: icml_support.sty.ltxml — Support for ICML conference styles
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("times");
  RequirePackage!("fancyhdr");
  RequirePackage!("color");
  // ICML 2024/2025 templates use xcolor's \colorlet for callout colors;
  // load xcolor eagerly. Pre-load with [dvipsnames, table] so that
  // user `\usepackage[dvipsnames, table]{xcolor}` doesn't silently
  // option-clash and leave colortbl/dvipsnam.def unloaded. Witness
  // 2405.18180 (icml2025).
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);
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
  // \icmlsetsymbol{name}{symbol} — creates `\<name>` macro expanding
  // to <symbol> for use in author lists. Real icml2024.sty L100-ish:
  //   `\def\icmlsetsymbol#1#2{\expandafter\def\csname #1\endcsname{#2}}`
  // Previous stub gobbled both args without defining the CS, breaking
  // `\icmlauthor{...}{equal,affil,icmlWorkDone}` which later references
  // `\icmlWorkDone` in `\printAffiliationsAndNotice`. Witness 2310.06430.
  DefMacro!("\\icmlsetsymbol{}{}",
    "\\expandafter\\def\\csname #1\\endcsname{#2}");

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
  // ICML 2024: simpler `\printAffiliations` no-arg form (newer template
  // emits the affiliation list inline without trailing notice). Witness
  // 2310.18127.
  def_macro_noop("\\printAffiliations")?;
  DefMacro!("\\icmlEqualContribution", "Equal contribution");
  // ICML 2023 (icml2023.sty L526): per-paper "equal advising" marker.
  // Witness 2306.01153.
  DefMacro!("\\icmlEqualAdvising", "Equal advising");
  // ICML 2025: extended marker for joint first + senior authorship.
  // Witness: 2503.15703 (icml2025.sty L534).
  DefMacro!("\\icmlEqualContributionAndSenior",
    "\\textsuperscript{*}Equal contribution, \
     \\textsuperscript{\\char`\u{2020}}Equal senior authorship");
  // ICML 2024 introduced `\icmlEqualSeniorContribution` — senior-only
  // joint authorship marker (no joint-first). Witness 2305.xxxxx in
  // wp4 (2 papers in stage 1).
  DefMacro!("\\icmlEqualSeniorContribution",
    "\\textsuperscript{\\char`\u{2020}}Equal senior contribution");
  DefMacro!("\\icmlkeywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");

  // Random extra bits
  def_macro_noop("\\abovestrut{}")?;
  def_macro_noop("\\belowstrut{}")?;
  def_macro_noop("\\abovespace")?;
  def_macro_noop("\\aroundspace")?;
  def_macro_noop("\\belowspace")?;
  def_macro_noop("\\icmlruler{}")?;
});
