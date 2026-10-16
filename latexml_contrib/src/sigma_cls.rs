//! Stub for sigma.cls (SIGMA journal — Symmetry, Integrability, Geometry).
use latexml_package::prelude::*;

use crate::shipped_class::{class_code_contains, shipped_class_text};

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");
  // sigma.cls:20-28 (0902.2977's copy, every one of the 20 sampled) `\RequirePackage{amsthm}`, `{amsmath}`,
  // `{latexsym}`, `{amssymb}`, `{amscd}`, `{epsfig}`, `{graphics}`, `{ifthen}`, `\usepackage{color}`: papers draw
  // commutative diagrams in `{CD}` (20 of run 353 with it undefined: 0902.2977, 0907.2394, 0910.5929) and colour with
  // `\color` (12) without loading them.
  RequirePackage!("latexsym");
  RequirePackage!("amscd");
  RequirePackage!("epsfig");
  RequirePackage!("graphics");
  RequirePackage!("ifthen");
  RequirePackage!("color");

  // SIGMA frontmatter — preserve author content.
  def_macro_noop("\\FirstPageHeading")?;
  DefMacro!(
    "\\ShortArticleName{}",
    "\\@add@frontmatter{ltx:note}[role=shorttitle]{#1}"
  );
  // \ArticleName{title} → \title{...}, \Author{name} → \author{...}.
  DefMacro!("\\ArticleName{}", "\\title{#1}");
  DefMacro!("\\Author{}", "\\author{#1}");
  DefMacro!(
    "\\AuthorNameForHeading{}",
    "\\@add@frontmatter{ltx:note}[role=runningauthor]{#1}"
  );
  DefMacro!(
    "\\Address{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#1}"
  );
  DefMacro!("\\EmailD{}", "\\@add@frontmatter{ltx:note}[role=email]{#1}");
  // \Email{addr} — alternate variant in newer sigma.cls templates.
  // Witness 2306.12539, 2307.06355 (both fail with undefined:\\Email).
  DefMacro!("\\Email{}", "\\@add@frontmatter{ltx:note}[role=email]{#1}");
  // sigma.cls:77-83 (0705.3628's v3.0 and 1404.0720's v1.7 copies; v1.8, 2609.01661, keeps `\EmailD`, `\URLaddress`,
  // `\URLaddressD`): the e-mail and URL lines under a doubly marked (`DD`) or a starred (`Marked`) author, each printing
  // nothing for an empty argument (`\ifthenelse{\equal{#1}{}}{}{…}`; 66 papers of the inventory with `\EmailDD`
  // undefined, 14 with `\URLaddressDD`: 0705.3628, 0707.2869, 0711.1671), and v1.8's `\mail{x}`, a `mailto:` link
  // written inside `\EmailD` (sigma.cls:80 of 2609.01661; 2 of the 211 sampled copies), defined where the paper's copy
  // has it, so a paper whose copy does not can define its own.
  RawTeX!(
    r"\def\lx@sigma@contact#1#2{\def\lx@sigma@arg{#2}\ifx\lx@sigma@arg\@empty\else\@add@frontmatter{ltx:note}[role=#1]{#2}\fi}
\def\EmailDD{\lx@sigma@contact{email}}\def\EmailMarked{\lx@sigma@contact{email}}
\def\URLaddressDD{\lx@sigma@contact{url}}\def\URLaddressMarked{\lx@sigma@contact{url}}"
  );
  let shipped = shipped_class_text("sigma")?;
  if shipped
    .as_deref()
    .is_some_and(|text| class_code_contains(text, "\\newcommand{\\mail}"))
  {
    RawTeX!(r"\def\mail#1{\href{mailto:#1}{#1}}");
  }
  DefMacro!(
    "\\URLaddressD{}",
    "\\@add@frontmatter{ltx:note}[role=url]{#1}"
  );
  // \URLaddress{url} — newer sigma.cls variant (without trailing D).
  DefMacro!(
    "\\URLaddress{}",
    "\\@add@frontmatter{ltx:note}[role=url]{#1}"
  );
  DefMacro!(
    "\\ArticleDates{}",
    "\\@add@frontmatter{ltx:note}[role=dates]{#1}"
  );
  // sigma.cls custom frontmatter macros for abstract / keywords /
  // classification + last-page sentinel.
  DefMacro!("\\Abstract{}", "\\begin{abstract}#1\\end{abstract}");
  DefMacro!(
    "\\Keywords{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}"
  );
  DefMacro!(
    "\\Classification{}",
    "\\@add@frontmatter{ltx:classification}[scheme=AMS]{#1}"
  );
  def_macro_noop("\\LastPageEnding")?;
  // sigma.cls:161-178 (0705.3628's copy): the class's theorem set, each with its own counter, `{definition}`,
  // `{example}`, `{remark}`, `{note}` in amsthm's definition style, and an unnumbered starred twin of each
  // (`{theorem*}` 20 papers of the inventory, `{remark*}` 12, `{example*}` 6: 1301.4541, 1404.0720, 1601.01181). The
  // class sets them in a group, which ends the style; amsthm defines the environments globally (amsthm.sty:81), so they
  // outlive it. Without them OmniBus's fallback set numbered them by section on a shared counter. The starred twins and
  // `{conjecture}` came with v1.2 (208 of the 211 sampled copies, and a paper without a copy); the v1.0/1.1 copies
  // (nlin0510055's :154-161, nlin0511055, nlin0512032) define the rest.
  if shipped
    .as_deref()
    .is_none_or(|text| class_code_contains(text, "\\newtheorem*{theorem*}"))
  {
    RawTeX!(
      r"\newtheorem{theorem}{Theorem}\newtheorem*{theorem*}{Theorem}
\newtheorem{lemma}{Lemma}\newtheorem*{lemma*}{Lemma}
\newtheorem{corollary}{Corollary}\newtheorem*{corollary*}{Corollary}
\newtheorem{proposition}{Proposition}\newtheorem*{proposition*}{Proposition}
\newtheorem{conjecture}{Conjecture}\newtheorem*{conjecture*}{Conjecture}
{\theoremstyle{definition}\newtheorem{definition}{Definition}\newtheorem*{definition*}{Definition}
\newtheorem{example}{Example}\newtheorem*{example*}{Example}
\newtheorem{remark}{Remark}\newtheorem*{remark*}{Remark}
\newtheorem{note}{Note}\newtheorem*{note*}{Note}}"
    );
  } else {
    RawTeX!(
      r"\newtheorem{theorem}{Theorem}\newtheorem{lemma}{Lemma}\newtheorem{corollary}{Corollary}
\newtheorem{proposition}{Proposition}
{\theoremstyle{definition}\newtheorem{definition}{Definition}\newtheorem{example}{Example}
\newtheorem{remark}{Remark}\newtheorem{note}{Note}}"
    );
  }
  // sigma.cls:182-191 (v3.0 on: 0705.3628, 1404.0720): `\tsep{dim}` and `\bsep{dim}`, invisible struts that add
  // `dim` above or below a table row (104 and 83 papers of the inventory: 0706.0314, 0708.0957).
  RawTeX!(
    r"\def\tsep#1{{\@tempdima=\ht\strutbox\advance\@tempdima #1\vrule height \@tempdima depth 0pt width 0pt\nobreak\hspace{0pt}}}
\def\bsep#1{{\@tempdima=\dp\strutbox \advance\@tempdima #1\nobreak\hspace{0pt}\vrule height 0pt depth \@tempdima width 0pt}}"
  );
  // SIGMA authors use \orcid for the ORCID identifier. Route through the kernel
  // `\lx@add@orcid` → `ltx:contact[role=orcid]` → clickable orcid.org link
  // (vs a bare dagger note). html_feedback#6571.
  DefMacro!("\\orcid{}", "\\lx@add@orcid{#1}");
});
