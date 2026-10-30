//! jinstpub.sty — JINST's SISSA/IOP publication style (2015/08/05 v.1.0 and 2020/09/18 v.1.1638, shipped with the
//! papers, not in TeX Live): jheppub's frontmatter — the accumulating `\author[affil]{name}`, `\affiliation[label]{…}`,
//! `\emailAdd{…}` and `\keywords`/`\abstract` stores its own `\maketitle` lays out (jinstpub.sty:60-150, 176-252) —
//! with JINST's extras, as jcappub_sty gives JCAP's. Read raw, the affiliations were an itemize, the e-mail a
//! paragraph and the author's `\note{Corresponding author.}` text in the personname (2006.05543, 2303.10224; 23 of
//! the 30,000-paper sample). The older JINST class (`JINST.cls`, JHEP3-derived) has its own binding,
//! latexml_contrib jinst_cls.rs. Neither Perl nor Rust had a jinstpub binding (so it is a contrib binding).
use latexml_package::prelude::*;

/// This binding's frontmatter conventions (the observer's, OXIDIZED_DESIGN_DIVERGENCES #837), declared at its load
/// (`declare_frontmatter_conventions`, user ruling 2026-10-28: each binding declares its own):
/// jheppub's `\emailAdd` line: placed by spelling, else by author order
pub const FRONTMATTER_CONVENTIONS: FmConventions = FmConventions {
  name: "jinstpub",
  binding: "jinstpub.sty",
  email_owner: FmEmailOwner::SpelledElseOrder,
  cite: "OD #801, #805 (jinstpub's placement: steps 1 and 4)",
  written: false,
  ..FM_KERNEL_CONVENTIONS
};

#[rustfmt::skip]
LoadDefinitions!({
  // (this binding's frontmatter conventions, first: OXIDIZED_DESIGN_DIVERGENCES #837)
  declare_frontmatter_conventions(&FRONTMATTER_CONVENTIONS);
  // jinstpub.sty:32-56: its packages in its order — the newtx text and math fonts (amsthm before newtxmath) before
  // jheppub's amsmath, hyperref and natbib —, then the `wrapfigure` environment and T1
  RequirePackage!("newtxtext");
  RequirePackage!("amsthm");
  RequirePackage!("newtxmath");
  jheppub_sty::load_definitions()?;
  RequirePackage!("wrapfig");
  RequirePackage!("fontenc", options => vec![s!("T1")]);
  // :35
  DefMacro!("\\mathsterling", "\\mbox{\\textsterling}");
  // :100
  DefMacro!("\\jname", "JINST");
  // :133-137 `\emailAdd` lists the addresses, which `\maketitle` prints as one "E-mail:" line under the affiliations
  // (:219-224), each `\href{mailto:#1}{\ttfamily\small #1}` (:138): jheppub's binding, which places an address with
  // the author it spells, else with the n-th author when the list is the authors' in order, else with the group's only
  // author or the author block (OXIDIZED_DESIGN_DIVERGENCES #801, #805; 2311.16063's bross@fnal.gov is Alan D.
  // Bross's, author 2, not Andrea Bersani's; 2307.12991, 1910.06438, 2311.00870, 2605.30709; an `\emailAdd{a@x, b@y}`
  // list an address each: 2211.05026, 2409.03065, 2112.00370).
  // :22 the first page's header line, "Prepared for submission to JINST", printed first by `\maketitle` (:180), is
  // page furniture; a paper's own (its copy's `\gdef\@fpheader{…}`, read after this binding, or its preamble's) is
  // author content, its report number or publication status: a pubnote (2103.06391 "Prepared for submission to JINST
  // (FERMILAB-PUB-21-079-ND-SCD)", 2205.10526 "JINST-060P-0522, accepted for publication, 11/6/2022", 2311.06155)
  RawTeX!(r"\def\lx@jinst@stockheader{Prepared for submission to JINST}
\def\lx@jinst@header#1{\lx@add@pubnote[role=preprint]{#1}}
\AtBeginDocument{\@ifundefined{@fpheader}{}{\ifx\@fpheader\lx@jinst@stockheader\else\ifx\@fpheader\@empty\else%
\expandafter\lx@jinst@header\expandafter{\@fpheader}\fi\fi}}");
  // (:91-98 `\newcommand\note[2][]`, an optional mark for the footnote: jheppub's `\note`, as above)
  // :145-147 `\newcommand*\collaboration[2][t]`: the optional placement (t, b, c) of the title page, warned about
  RawTeX!(r"\renewcommand*\collaboration[2][t]{\lx@add@pubnote[role=collaboration]{#2}%
\if b#1 \PackageWarningNoLine{\jname}{option b}\fi\if c#1 \PackageWarningNoLine{\jname}{option c}\fi}");
  // (:241-249 `\maketitle` ends with the table of contents unless `\notoc`, :81: jheppub's binding, whose title page
  // does the same, drops it and makes `\notoc` a no-op, as Perl's jheppub.sty.ltxml:97; the family keeps one rule, a
  // SYNC_STATUS row awaiting a ruling)
  // :335 the equations numbered within sections, "(2.1)"
  RawTeX!(r"\numberwithin{equation}{section}");
  // :362-378 the bold label and its period, "**Figure 1.** …"
  caption_sty::class_makecaption_label("\\bfseries", ". ", false)?;
  // :384-387 the reference shorthands
  RawTeX!(r"\newcommand\erratum[4][ibid.\ ]{\emph{Erratum #1}{\bf #2} (#3) #4}
\newcommand\addendum[4][ibid.\ ]{\emph{Addendum #1}{\bf #2} (#3) #4}
\newcommand\ibid[3]{\emph{ibid.\ }{\bf #1} (#2) #3}
\newcommand\pos[1]{\href{http://pos.sissa.it/cgi-bin/reader/contribution.cgi?id=#1}{\tt #1}}");
});
