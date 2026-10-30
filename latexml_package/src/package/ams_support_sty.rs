use crate::prelude::*;

/// amsart.cls:350 — 10pt is the default size: `\@typesizes`, the sizes `\@xsetfontsize` picks from, unless a size
/// option already set them.
#[rustfmt::skip]
pub fn ams_size_default() -> Result<()> {
  raw_tex(r"\@ifundefined{@typesizes}{\def\@typesizes{\or{5}{6}\or{6}{7}\or{7}{8}\or{8}{10}\or{9}{11}\or{10}{12}\or{\@xipt}{13}\or{\@xiipt}{14}\or{\@xivpt}{17}\or{\@xviipt}{20}\or{\@xxpt}{24}}}{}")
}

/// amsart.cls/amsbook.cls/amsproc.cls:258-296 (the three classes' option blocks are the same): each size option sets
/// `\@mainsize`, `\@ptsize` and `\@typesizes`. Declared by the class bindings, as the classes declare them, so a class
/// loaded with a size (`\LoadClass[12pt]{amsart}`) sets it; ams_support declares them as no-ops (Perl
/// ams_support.sty.ltxml:31-39), as a package's `ProcessOptions` would run the document's size option again after the
/// class settled its own (and over a non-AMS class's `\@ptsize`).
#[rustfmt::skip]
pub fn declare_ams_size_options() -> Result<()> {
  // amsart.cls:169-170
  raw_tex(r"\providecommand{\@mainsize}{10}\providecommand{\@ptsize}{0}")?;
  ams_size_default()?;
  DeclareOption!("8pt", "\\def\\@mainsize{8}\\def\\@ptsize{8}\\def\\@typesizes{\\or{5}{6}\\or{5}{6}\\or{5}{6}\\or{6}{7}\\or{7}{8}\\or{8}{10}\\or{9}{11}\\or{10}{12}\\or{\\@xipt}{13}\\or{\\@xiipt}{14}\\or{\\@xivpt}{17}}");
  DeclareOption!("9pt", "\\def\\@mainsize{9}\\def\\@ptsize{9}\\def\\@typesizes{\\or{5}{6}\\or{5}{6}\\or{6}{7}\\or{7}{8}\\or{8}{10}\\or{9}{11}\\or{10}{12}\\or{\\@xipt}{13}\\or{\\@xiipt}{14}\\or{\\@xivpt}{17}\\or{\\@xviipt}{20}}");
  DeclareOption!("10pt", "\\def\\@mainsize{10}\\def\\@ptsize{0}\\def\\@typesizes{\\or{5}{6}\\or{6}{7}\\or{7}{8}\\or{8}{10}\\or{9}{11}\\or{10}{12}\\or{\\@xipt}{13}\\or{\\@xiipt}{14}\\or{\\@xivpt}{17}\\or{\\@xviipt}{20}\\or{\\@xxpt}{24}}");
  DeclareOption!("11pt", "\\def\\@mainsize{11}\\def\\@ptsize{1}\\def\\@typesizes{\\or{6}{7}\\or{7}{8}\\or{8}{10}\\or{9}{11}\\or{10}{12}\\or{\\@xipt}{13}\\or{\\@xiipt}{14}\\or{\\@xivpt}{17}\\or{\\@xviipt}{20}\\or{\\@xxpt}{24}\\or{\\@xxvpt}{30}}");
  DeclareOption!("12pt", "\\def\\@mainsize{12}\\def\\@ptsize{2}\\def\\@typesizes{\\or{7}{8}\\or{8}{10}\\or{9}{11}\\or{10}{12}\\or{\\@xipt}{13}\\or{\\@xiipt}{14}\\or{\\@xivpt}{17}\\or{\\@xviipt}{20}\\or{\\@xxpt}{24}\\or{\\@xxvpt}{30}\\or{\\@xxvpt}{30}}");
  Ok(())
}

/// This binding's frontmatter conventions (the observer's, OXIDIZED_DESIGN_DIVERGENCES #837), declared at its load
/// (`declare_frontmatter_conventions`, user ruling 2026-10-28: each binding declares its own):
/// amsart: `\and` prints "and", in an `\address` too (`\address{Einstein Institute … \and Rutgers}`, 1712.02568).
/// Reaches a binding only while its `\author` at the call is still this binding's (the key is
/// `ledger::author_binding`): ams_core's amsart, amsproc and amsbook, amsaddr. Not lmcs: its own `\author` calls
/// `\lx@add@author`, so the observer records no author block (2306.08266, 2108.06944, 2402.09138). OmniBus loads this
/// binding lazily — its `\subjclass` autoload (omnibus_cls.rs:820), or the ambient `\curraddr` one (tex.rs:314) — so it
/// is reached only when that load comes before `\author`; otherwise OmniBus is inst_support's.
pub const FRONTMATTER_CONVENTIONS: FmConventions = FmConventions {
  name: "ams",
  binding: "ams_support.sty",
  and_word: true,
  // (the second family: 0.38 % unclassified over 2,903 census papers, a seeded classified sample 33 of 34 right, none
  // wrong — SYNC "FM S2b next family: ams_support", int110l)
  written: true,
  cite: "amsart.cls:599 `\\def\\and`; Perl ams_support.sty.ltxml:94 `\\lx@author@conj`; 1712.02568",
  ..FM_KERNEL_CONVENTIONS
};

#[rustfmt::skip]
LoadDefinitions!({
  // (this binding's frontmatter conventions, first: OXIDIZED_DESIGN_DIVERGENCES #837)
  declare_frontmatter_conventions(&FRONTMATTER_CONVENTIONS);
  // Perl: ams_support.sty.ltxml — common support for AMS document classes

  //======================================================================
  // Document structure.

  Let!("\\@xp", "\\expandafter");
  Let!("\\@nx", "\\noexpand");

  // None of the options are vital, I think; deferred.
  // [though loading an unwanted amsfonts (noamsfonts) could be an issue]
  for option in [
    "a4paper", "letterpaper", "landscape", "portrait",
    "oneside", "twoside", "draft", "final", "e-only",
    "titlepage", "notitlepage",
    "openright", "openany", "onecolumn", "twocolumn",
    "nomath", "noamsfonts", "psamsfonts",
    "leqno", "reqno", "centertags", "tbtags", "fleqn",
    "8pt", "9pt", "10pt", "11pt", "12pt",
    "makeidx",
  ].iter() {
    DeclareOption!(*option, None);
  }
  // amsart.cls:169-170, 181-192, 213-219: the size machinery a class built on amsart redefines its size commands with
  // (m2an.cls:241 `\renewcommand\normalsize{\@xsetfontsize\normalsize 6\@adjustvertspacing…}`; 2609.37833, undefined
  // `\@xsetfontsize` then Fatal PushbackLimit). `\@xsetfontsize\cs N` sets `\cs` at the Nth of `\@typesizes`' sizes
  // (normalsize is 6), which the class's size options set (:258-296, `declare_ams_size_options`; 10pt the default, :350).
  RawTeX!(r"\chardef\@currsizeindex=6
\def\@xsetfontsize#1#2{\chardef\@currsizeindex#2\relax
  \edef\@tempa{\@nx\@setfontsize\@nx#1\@xp\ifcase\@xp\@currsizeindex\@typesizes\else{99}{99}\fi}\@tempa}
\def\@adjustvertspacing{\bigskipamount.7\baselineskip plus.7\baselineskip
  \medskipamount\bigskipamount \divide\medskipamount\tw@
  \smallskipamount\medskipamount \divide\smallskipamount\tw@
  \abovedisplayskip\medskipamount \belowdisplayskip \abovedisplayskip
  \abovedisplayshortskip\abovedisplayskip \advance\abovedisplayshortskip-1\abovedisplayskip
  \belowdisplayshortskip\abovedisplayshortskip \advance\belowdisplayshortskip 1\smallskipamount
  \jot\baselineskip \divide\jot 4 \relax}");
  ams_size_default()?;
  ProcessOptions!();

  //======================================================================
  // Font size commands:

  DefPrimitive!("\\larger",  None, font => { scale => 1.2 });
  // amsart.cls `\smaller` is relative (`\larger[-1]`), as `\larger`; Perl's absolute `size => 1/1.2`
  // (ams_support.sty.ltxml:46, KNOWN_PERL_ERRORS #404) gave a 0.83pt font and, once size switches set
  // the leading (58c), a 1pt `\baselineskip`.
  DefPrimitive!("\\smaller", None, font => { scale => 0.8333333333333334 }); // 1/1.2

  // \@xsetfontize
  DefPrimitive!("\\TINY", None, font => { size => 3 });
  DefPrimitive!("\\Tiny", None, font => { size => 4 });
  Let!("\\SMALL", "\\scriptsize");
  Let!("\\Small", "\\footnotesize");
  DefPrimitive!("\\HUGE", None, font => { size => 29.8 });
  Let!("\\upn", "\\textup");

  //======================================================================
  // Sec. 3. The Preamble
  // Included packages
  // amsmath, amsthm,
  // amsfonts (unless noamsfonts)

  RequirePackage!("amsmath");
  // Perl ams_support.sty.ltxml:23 — `RequirePackage('amstex') if LookupValue('2.09_COMPATIBILITY')`.
  // 2.09_COMPATIBILITY is set by `\documentstyle` in tex_job.rs's compat
  // shim. Legacy AMS papers (e.g. alg-geom/9208004, alg-geom/9202004)
  // use `\documentstyle[12pt,verbatim]{amsart}` and rely on the AmS-TeX
  // `\Sb` / `\Sp` substack environments which are only defined by the
  // amstex binding.
  if lookup_bool("2.09_COMPATIBILITY") {
    RequirePackage!("amstex");
  }
  // amsart.cls:41-42 (amsproc.cls, amsbook.cls alike) marks amsthm.sty loaded (`\ver@amsthm.sty`), its theorem code
  // being the class's own; here amsthm's binding stands for it. A document that wants amsthm.sty itself unmarks it
  // (`\expandafter\let\csname ver@amsthm.sty\endcsname\relax`, then `\let\proof\relax…\usepackage{amsthm}`), which
  // the loader honours as LaTeX's `\@ifl@aded` does for this mark only (`amsthm.sty:class_marked`,
  // `state::unmarked_by_document`; user ruling 2026-10-14: no general loader `\ver@` rule; 1510.08833, 2106.07418).
  RequirePackage!("amsthm");
  AssignValue!("amsthm.sty:class_marked" => true, Some(Scope::Global));
  RequirePackage!("amsfonts");
  RequirePackage!("makeidx");

  // Useful packages:
  // amssymb,
  // amsmidx for multiple-indexes,
  // graphicx,
  // longtable,
  // upref makes references upcase?, upright?
  // xypic,

  //======================================================================
  // Sec. 4. Top Matter
  // FrontMatter:
  DefMacro!("\\shorttitle{}", "\\lx@add@toctitle{#1}");
  // Author / address fields — preserve as ltx:note so the strings
  // reach the XML output instead of being gobbled (content-
  // preserving). These are typically the short-form variants
  // already covered by \author / \address from the main flow, but
  // when authors set them explicitly the values are still real
  // metadata.
  DefMacro!("\\shortauthor{}",
    "\\lx@add@frontmatter{ltx:note}[role=shortauthor]{#1}");
  DefMacro!("\\authors{}",
    "\\lx@add@frontmatter{ltx:note}[role=authors]{#1}");
  // Perl `ams_support.sty.ltxml` L82: `DefMacro('\shortauthors{}', Tokens())`
  // — gobble (redundant running head). Match Perl; preserving it errored on a
  // literal `&` in the running head (catcode-4 `&` → stray-`&`). See 0709.4236
  // and aas_support_sty.rs.
  def_macro_noop("\\shortauthors{}")?;
  DefMacro!("\\addresses{}",
    "\\lx@add@frontmatter{ltx:note}[role=addresses]{#1}");
  // The AMS classes replace these setters with amsart's storage (`amsart_author_storage`);
  // without an AMS class (the `\curraddr`/`\subjclass` autoloads) `\author` stores nothing.
  RawTeX!(r"\def\lx@ams@addto@authors#1#2{}");
  DefMacro!("\\publname{}",
    "\\lx@add@frontmatter{ltx:note}[role=publication]{#1}");

  // 68t: a call inside the document's `{titlepage}` also runs a refused printing `\title` there (sect05.rs
  // `titlepage_class_call`, as revtex4_support's: gr-qc9404034).
  // The class's refused `\title` runs first for its side effects, as under the kernel's (sect05.rs `\title`,
  // OXIDIZED_DESIGN #482: the cvpr template's `\thetitle` under acmart, 2506.01799).
  // amsart.cls:457 (amsbook.cls:436, amsproc.cls:435) `\renewcommand*{\title}`, :551-568 `\def\dedicatory#1`,
  // `\def\keywords#1`, `\def\translator#1`, :555 `\newcommand*\subjclass`: none is `\long`, so a paragraph's end in
  // the argument ends the call, "Paragraph ended before \subjclass was complete" (tex.web §396), and the paragraphs
  // after it stay the document's. A `\subjclass{…: 65N12, 74G65` whose `}` never comes took the rest of the document
  // into the classification, every `\label` with it (2212.11488, 81 of 153 references dangling). Guard
  // `perfect_kernel_batch79m::an_unclosed_subjclass_ends_at_the_paragraph`.
  DefMacro!("\\title[]{}",
    "\\lx@titlepage@class@call{title}{#1}{#2}{\\lx@replay@class@setter{title}{#1}{#2}\\gdef\\@shorttitle{#1}\\gdef\\@title{#2}\\ifx.#1.\\else\\lx@add@toctitle{#1}\\fi\\lx@add@title{#2}}",
    short => true);

  DefMacro!("\\lx@author@sep", ",\\ ");
  DefMacro!("\\lx@author@conj", "\\ and\\ ");   // \@@and

  // \author[shortname]{name} Use one \author per author
  // followed by whatever contact information applies to that author.
  // What to do with shortauthor ?  (Perl PR #2767)
  // amsart's `\author` is `\@dblarg`'d (amsart.cls:460-477): an absent short name is the full
  // one, an explicit `[]` gives none.
  DefMacro!("\\author[]{}", sub[(short, name)] {
    let stored = short.clone().unwrap_or_else(|| name.clone());
    Ok(Invocation!(
      T_CS!("\\lx@ams@author"),
      vec![Some(short.unwrap_or_default()), Some(name), Some(stored)]
    ))
  });
  DefMacro!("\\lx@ams@author{}{}{}",
    "\\def\\@shortauthor{#1}\\def\\@author{#2}\\lx@ams@addto@authors{#3}{#2}\\lx@add@authors@append{#2}");

  DefMacro!("\\datename", None, "\\textit{Date}:");

  DefMacro!("\\@commby", "Communicated by");
  DefMacro!("\\curraddrname", "{\\itshape Current address}");
  DefMacro!("\\emailaddrname", "{\\itshape Email address}");
  DefMacro!("\\urladdrname", "{\\itshape URL}");
  DefMacro!("\\translname", "Translated by");
  DefMacro!("\\keywordsname", None, "Key words and phrases");

  // Various frontmatter creator contact information; attaches to previous \author
  DefMacro!("\\contrib[]{}", "\\lx@add@creator[role=contributor, name={#1}]{#2}");
  DefMacro!("\\commby{}",    "\\lx@add@creator[role=communicator,name={\\@commby~}]{#1}");
  DefMacro!("\\address[Default:\\lx@contact@address@name]{}", "\\lx@add@address[name={#1}]{#2}");
  DefMacro!("\\curraddr[]{}",
    "\\lx@add@contact[role=current_address,name={\\curraddrname\\ifx.#1.\\else{, #1}\\fi:\\ }]{#2}");
  // amsart.cls:538-546 (amsproc.cls:516-524, amsbook.cls:486-494) set the address in `\ttfamily`: an
  // `ann\string_able@a.edu` reads '_' (2101.10462, 2501.04035).
  DefMacro!("\\email[]{}",
    "\\lx@add@email[name={\\emailaddrname\\ifx.#1.\\else{, #1}~\\fi:\\ }]{\\lx@address@urlfont{#2}}");
  DefMacro!("\\urladdr[]{}",
    "\\lx@add@url[name={\\urladdrname\\ifx.#1.\\else{, #1}~\\fi:\\ }]{\\lx@address@urlfont{#2}}");
  DefMacro!("\\lx@ams@dedicatory{}", "\\lx@add@contact[role=dedicatory]{#1}", short => true);
  DefMacro!("\\dedicatory", sub[_args] { post_title_topmatter("\\lx@ams@dedicatory") });
  // amsart's raw `\maketitle` internals, reached when a derivative class
  // redefines `\maketitle`/`\@maketitle` on top of `\LoadClass{amsart}`
  // (resphilosophica.cls:323 `\author@andify\authors`, :358 `\ifx\@empty
  // \@dedicatory`, :259/:364 `\@setabstract`): amsart.cls:803 `\author@andify`
  // (an and-joiner over the captured authors), :552 `\let\@dedicatory\@empty`,
  // :856 `\@setabstract` (typesets the captured abstract). The frontmatter
  // is already emitted by the `\lx@add@*` capture at the declaration sites,
  // so these run inert (RUST-ONLY: Perl's amsart path never reaches the raw
  // layout); the AMS classes replace `\author@andify` with amsart's joiner over
  // their author storage (`amsart_author_storage`). Guard:
  // `perfect_kernel_batch54::amsart_maketitle_internals_are_defined`.
  Let!("\\@dedicatory", "\\@empty");
  DefMacro!("\\author@andify{}", "");
  DefMacro!("\\@setabstract", "");
  DefMacro!("\\dateposted{}",   "\\lx@add@date[role=posted]{#1}");
  DefMacro!("\\translator[]{}", "\\lx@add@translator[name={\\translname~}]{#2}", short => true);
  // (each records that the binding's own setter ran: `amsart_class_internals`'s hand-over of a document's own store)
  DefMacro!(
    "\\lx@ams@keywords{}",
    "\\global\\let\\lx@ams@keywords@given\\@empty\\lx@add@keywords[name={\\keywordsname:~}]{#1}",
    short => true
  );
  DefMacro!("\\keywords", sub[_args] { post_title_topmatter("\\lx@ams@keywords") });

  // \thanks{} ( == ack, not latex's \thanks, not in author)
  // make a throwaway optional argument available for OmniBus use
  // amsart.cls:836 `\@setthanks` prints each `\thanks` in the first-page footnote with no label: a class with no
  // `\thanksname` names it by the role's default, none unless its binding sets one (resphilosophica's
  // "Acknowledgments"; FIDELITY_CHARTER §4, user ruling 2026-10-30; OXIDIZED_DESIGN_DIVERGENCES #921).
  DefMacro!("\\thanks[]{}",
    "\\lx@add@pubnote[role=thanks,name={\\@ifundefined{thanksname}{\\lx@pubnote@thanks@name}{\\thanksname}}]{#2}");

  // Non-standard but makes it easier to create bindings for variations on AMS classes;
  // just redefine this macro
  DefMacro!("\\@subjclassyear", None, "1991");

  DefMacro!("\\subjclassname", None,
    "\\textup{\\@subjclassyear} Mathematics Subject Classification");
  // amsart.cls:781-789 (amsproc.cls, amsbook.cls alike): the name of each MSC edition, which classes built on amsart
  // read: quantumdust.cls:278-286 re-dresses `\subjclass` to warn `\ClassWarning{\@classname}` on an
  // `\@ifundefined{subjclassname@#1}` edition (2606.20943). `\@classname` itself is the class's own (amsart.cls:58-60),
  // set in each class binding.
  RawTeX!(r"\@namedef{subjclassname@1991}{\textup{1991} Mathematics Subject Classification}
\@namedef{subjclassname@2000}{\textup{2000} Mathematics Subject Classification}
\@namedef{subjclassname@2010}{\textup{2010} Mathematics Subject Classification}
\@namedef{subjclassname@2020}{\textup{2020} Mathematics Subject Classification}");
  // Perl ams_support.sty.ltxml L141-144: pure expansion macro. Translate
  // `[Default:\@subjclassyear]` to `[\@subjclassyear]` (default-fill of
  // empty optional arg) and inline the `\ifx.#1.\else\xdef…\fi` guard so
  // the body tokens (`#2`) are passed straight through to
  // `\@add@frontmatter` without a Rust-side `to_string` round-trip. The
  // earlier Rust `\lx@subjclass@{}{}` reified `#2` to a string, which
  // mangled `\sc AMS` into `\scAMS` (the trailing-space-after-CS rule
  // doesn't survive `tokenize_internal`-after-`to_string`). Driver paper:
  // arXiv:1902.09816 (`\subjclass{{\sc AMS Subject Classification:} ...}`).
  // Perl ams_support.sty.ltxml L141-144 — strict translation:
  // `[Default:\@subjclassyear]` provides `\@subjclassyear`-expansion as
  // the Optional default when the user omits `[...]`. The `\ifx.#1.`
  // guard updates the global year only when the user supplied a non-CS
  // value. Body tokens (`#2`) pass straight through to
  // `\@add@frontmatter` — no Rust-side `to_string` round-trip (which
  // mangled `\sc AMS` into `\scAMS` by losing the trailing-space-after-CS
  // rule). Driver paper: arXiv:1902.09816
  // (`\subjclass{{\sc AMS Subject Classification:} 06B05}`).
  DefMacro!("\\subjclass", sub[_args] { post_title_topmatter("\\lx@ams@subjclass") });
  DefMacro!("\\lx@ams@subjclass[Default:\\@subjclassyear]{}",
    "\\ifx.#1.\\else\\xdef\\@subjclassyear{#1}\\fi\\global\\let\\lx@ams@subjclass@given\\@empty\
     \\lx@add@classification[scheme={#1 Mathematics Subject Classification},name={\\subjclassname:~}]{#2}",
    short => true);

  // amsart.cls:723-725 `\@setcopyright` prints the copyright only with a year (`\ifx\@empty\copyrightyear\else`; and
  // only with a series logo, `\ifx\@empty\@serieslogo`, which the frontmatter keeps as metadata): a class's
  // `\copyrightinfo{}{}` default (jgcc.cls:397; 2309.03819, 2401.15504) adds none. Perl ams_support.sty.ltxml:139 adds
  // ", " (KNOWN_PERL_ERRORS #959). The one empty-year test, the amsart variant's too (`amsart_class_internals`).
  DefMacro!("\\lx@ams@add@copyright{}{}", "\\if\\relax\\detokenize{#1}\\relax\\else\\lx@add@copyright{#1, #2}\\fi");
  DefMacro!("\\copyrightinfo{}{}", "\\lx@ams@add@copyright{#1}{#2}");

  def_macro_noop("\\pagespan{}{}")?; // ?
  DefMacro!("\\PII{}",  "\\lx@add@classification[scheme=PII]{#1}");
  DefMacro!("\\ISSN{}", "\\lx@add@classification[scheme=ISSN]{#1}");

  DefMacro!("\\currentvolume", None, "");
  DefMacro!("\\currentissue", None, "");
  DefMacro!("\\currentmonth", None, "");
  DefMacro!("\\currentyear", None, "");
  DefMacro!("\\volinfo", None, "");
  // amsart.cls:121-143, :680: the first page prints `\volinfo` ("Volume 12, Number 3, March 2024") alone — no label
  // (FIDELITY_CHARTER §4, user ruling 2026-10-30; OXIDIZED_DESIGN_DIVERGENCES #921).
  DefMacro!("\\issueinfo{}{}{}{}",
    "\\def\\currentvolume{#1}\\def\\currentissue{#2}\\def\\currentmonth{#3}\\def\\currentyear{#4}\\def\\volinfo{Volume \\currentvolume, Number \\number0\\currentissue, \\currentmonth\\ \\currentyear}\\lx@add@pubnote[role=volume,name={}]{\\volinfo}");

  // abstract otherwise defined in LaTeX.pool
  DefMacro!("\\abstractname", None, "\\textsc{Abstract}");

  //======================================================================
  // Sec. 5. Document Body

  // Mostly normal LaTeX

  // For multiple indexes:
  // \usepackage{amsmidex}
  // \makeindex{name of index file}
  // \makeindex{name of index file}
  //
  // \index{name of index}{index term}   ...
  // \Printindex{name of index}{title of index} ...

  DefMacro!("\\format@title@abstract{}", "#1. ");
  DefMacro!("\\format@title@section{}", "\\lx@tag[][.\\space]{\\thesection}#1");
  DefMacro!("\\format@title@subsection{}", "\\lx@tag[][.\\space]{\\thesubsection}#1");
  DefMacro!("\\format@title@subsubsection{}", "\\lx@tag[][.\\space]{\\thesubsubsection}#1");

  DefMacro!("\\format@title@description{}", "\\lx@tag[][:\\space]{#1}");
  DefMacro!("\\descriptionlabel{}", "\\normalfont\\bfseries #1:\\space");

  //======================================================================
  // Sec 6. Floating objects: Figures and tables
  // Normal LaTeX

  // For compatibility — Perl ams_support.sty.ltxml L194-200.
  // When 2.09_COMPATIBILITY is set (via \documentstyle), define the
  // LaTeX-2.09-era `pf` / `pf*` environment aliases for `proof`.
  // Sandbox paper 0802.1100 (and similar 2.09-style submissions) uses
  // `\begin{pf}` which isn't in modern amsart; this restores the alias.
  //
  // PERL-FAITHFUL: Perl ONLY provides the `pf` env alias in 2.09 mode.
  // Modern amsart papers that use `\newcommand{\pf}{...}` (e.g.
  // Pfaffian operator) AFTER `\begin{document}` rely on `\pf` being
  // undefined at that point. Pre-providing it via `\AtBeginDocument`
  // (our previous behavior) caused `is_definable_latex` to refuse
  // the user's redefinition, leaving `\pf` as `\begin{@proof}` —
  // which then expanded in `$\pf$` math context and triggered
  // `\itshape`/`\not@math@alphabet@@` cascades (witness 1102.0135,
  // ~100 errors via `\itdefault invalid in math mode` →
  // `\lx@end@inline@math` mode-mismatch loop).
  //
  // Trade-off vs Perl: papers that genuinely use `\begin{pf}` for
  // amsart's proof-alias env will emit one "undefined macro {pf}"
  // error. Perl emits the same error (verified on minimal repro;
  // Perl reports "Conversion complete: 1 error; 1 undefined
  // macro[{pf}]"). Removing our preemptive `\AtBeginDocument` block
  // makes Rust match Perl exactly on both cases.
  if lookup_bool("2.09_COMPATIBILITY") {
    DefMacro!("\\defaultfont", "\\normalfont");
    DefMacro!("\\rom", "\\textup");
    raw_tex(
      "\\newenvironment{pf}{\\begin{@proof}}{\\end{@proof}}\
       \\newenvironment{pf*}[1]{\\begin{@proof}[#1]}{\\end{@proof}}"
    )?;
  }

  // (The AMS classes' caption format, `\format@title@figure`/`\format@title@table` in Perl's binding here,
  // ams_support.sty.ltxml:192-193, is the classes' own: `ams_class_caption_format`.)

  // Excersise environments ??:
  // xca "must be defined with \theoremstyle{definition} and \newtheorem ???
  // xcb only for monographs, at end of chapter

  //======================================================================
  // Sec 7. Bibliographic References
  // \bibliographicstyle{}  amsplain or amsalpha
  // \bibliography{bibfile}
  // Normal LaTeX

  DefMacro!("\\bysame", " by same author");
  DefMacro!("\\bibsetup", None, "");

  //======================================================================
  // Sec 8 Monograph Formatting:

  // TOC's should be built by latexml... ?
  def_macro_noop("\\tocpart{}{}{}")?;
  def_macro_noop("\\tocchapter{}{}{}")?;
  def_macro_noop("\\tocsection{}{}{}")?;
  def_macro_noop("\\tocsubsection{}{}{}")?;
  def_macro_noop("\\tocsubsubsection{}{}{}")?;
  def_macro_noop("\\tocparagraph{}{}{}")?;
  def_macro_noop("\\tocsubparagraph{}{}{}")?;
  def_macro_noop("\\tocappendix{}{}{}")?;
  DefMacro!("\\contentsnamefont", None, "\\scshape");

  DefMacro!("\\labelenumi", None, "(\\theenumi)");
  DefMacro!("\\labelenumii", None, "(\\theenumii)");
  DefMacro!("\\labelenumiii", None, "(\\theenumiii)");
  DefMacro!("\\labelenumiv", None, "(\\theenumiv)");

  DefRegister!("\\normaltopskip"    => Glue!("10pt"));
  DefRegister!("\\linespacing"      => Dimension::from_str("1pt")?);
  DefRegister!("\\normalparindent"  => Dimension::from_str("12pt")?);
  DefRegister!("\\abovecaptionskip" => Glue!("12pt"));
  DefRegister!("\\belowcaptionskip" => Glue!("12pt"));
  DefRegister!("\\captionindent"    => Glue!("3pc"));
  DefPrimitive!("\\nonbreakingspace", "\u{00A0}");
  DefMacro!("\\fullwidthdisplay", None, "");
  DefRegister!("\\listisep" => Glue::new(0));

  DefMacro!("\\calclayout", None, "");
  DefMacro!("\\indentlabel", None, "");

  //======================================================================
  DefMacro!("\\@True", None, "00");
  DefMacro!("\\@False", None, "01");

  // amsart.cls:63-74 (amsproc.cls, amsbook.cls alike): `\newswitch{<name>}` makes `\?@<name>` the `\@False` (or the
  // optional `[True]`) its tests read, `\setFalse`/`\setTrue` set it. Perl implements them (ams_support.sty.ltxml:
  // 254-258); the earlier port stubbed them as no-ops, so a `\setTrue{toc}` set nothing. The class lines verbatim. The
  // tests over them, `\except`, `\for` and `\forany`, are the classes' own (`amsart_class_internals`).
  RawTeX!(r"\newcommand\newswitch[2][False]{%
  \expandafter\@ifdefinable\csname ?@#2\endcsname{%
    \global\expandafter\let\csname ?@#2\expandafter\endcsname
      \csname @#1\endcsname
  }%
}
\newcommand{\setFalse}[1]{%
  \expandafter\let\csname ?@#1\endcsname\@False
}
\newcommand{\setTrue}[1]{%
  \expandafter\let\csname ?@#1\endcsname\@True
}");

  DefMacro!("\\Mc", None, "Mc");

  // Generated comma and "and" separated lists...
  // \andify, \xandlist, \nxandlist

  //======================================================================

  // \URLhref{url} — hyperref-style URL reference (Round-34 surpass-
  // Perl: was gobbled). Route through our \URL → \@ams@url chain.
  DefMacro!("\\URLhref{}", "\\URL{#1}");
  // \URL — complex catcode manipulation, stubbed as simple macro
  // that delegates to \@ams@url to get the href attribute set (Perl L282-294).
  DefMacro!("\\URL{}", "\\@ams@url{#1}");
  DefConstructor!("\\@ams@url {}",
    "<ltx:ref href='#href'>#1</ltx:ref>",
    properties => sub[args] {
      let url_str = args[0].as_ref().map(|t| t.to_string()).unwrap_or_default();
      Ok(stored_map!("href" => clean_url(&url_str)))
    });

  DefMacro!("\\MR{}", "MR #1");
  // \MRhref{label} — Math Reviews link; preserve as note (the link
  // target encodes the MR id which is genuine reference metadata).
  DefMacro!("\\MRhref{}", "\\lx@add@frontmatter{ltx:note}[role=mr-ref]{#1}");
  // amsbook.cls:1779 / amsart `\markleft{}` — a running-head mark, ignored
  // like `\markright`/`\markboth` (Author_Handbook_Memo).
  def_primitive_noop("\\markleft{}")?;
});

/// amsart's author storage (amscls/amsart.cls:460-477, amsbook.cls the same): `\authors`,
/// `\shortauthors` and `\addresses` are parameterless macros, empty until `\author[#1]{#2}` adds
/// its names, joined by `\and`; a document's `\maketitle` or running head prints them. Perl's
/// ams_support reads an argument (ams_support.sty.ltxml:82-84), gobbling the next token: "Written
/// by \authors\ (…)" lost its text, and 2605.03453's `\authors` inside a tabular swallowed its
/// `\end` (KNOWN_PERL_ERRORS #366). amsart's `\maketitle` keeps `\and` (amsart.cls:599-621), which
/// the kernel title code clears as article's does, so the class empties that step
/// (`\lx@maketitle@clear@and`): a document's own `\renewcommand\and` survives the title too.
/// `\addresses` stays empty: LaTeXML builds the addresses from `\address`. The one-argument setters
/// stay for the classes that call them (aas_support gobbles 0709.4236's `\shortauthors{…&…}`, an
/// aastex paper); here a setter-style call typesets its text, as in pdflatex. Not modelled: amsart's
/// `\maketitle` rewrites `\shortauthors` for the running head (`\andify`, or the short title when
/// empty, amsart.cls:604-606).
pub fn amsart_author_storage() -> Result<()> {
  RequirePackage!("amsgen");
  Let!("\\authors", "\\@empty");
  Let!("\\shortauthors", "\\@empty");
  Let!("\\addresses", "\\@empty");
  RawTeX!(
    r"\def\lx@ams@addto@authors#1#2{%
  \ifx\@empty\authors\gdef\authors{#2}\else\g@addto@macro\authors{\and#2}\fi
  \@ifnotempty{#1}{\ifx\@empty\shortauthors\gdef\shortauthors{#1}\else\g@addto@macro\shortauthors{\and#1}\fi}}
\let\lx@maketitle@clear@and\relax"
  );
  // amsart's list joiners over that storage (amsart.cls:580-598, :803-807; its `\newcommand`s as
  // `\long\def`, the class having no earlier definition to guard): a derived
  // class prints `\authors` through them (resphilosophica.cls:323 `\author@andify\authors`:
  // "A, B, and C"), where ams_support's inert `\author@andify` left the names run together.
  RawTeX!(
    r"\long\def\xandlist#1#2#3#4{\@andlista{{#1}{#2}{#3}}#4\and\and}
\def\@andlista#1#2\and#3\and{\@andlistc{#2}\@ifnotempty{#3}{%
  \@andlistb#1{#3}}}
\def\@andlistb#1#2#3#4#5\and{%
  \@ifempty{#5}{%
    \@andlistc{#2#4}%
  }{%
    \@andlistc{#1#4}\@andlistb{#1}{#3}{#3}{#5}%
  }}
\let\@andlistc\@iden
\long\def\nxandlist#1#2#3#4{%
  \def\@andlistc##1{\toks@\@xp{\the\toks@##1}}%
  \toks@{\toks@\@emptytoks \@andlista{{#1}{#2}{#3}}}%
  \the\@xp\toks@#4\and\and
  \edef#4{\the\toks@}%
  \let\@andlistc\@iden}
\def\@@and{and}
\long\def\andify{%
  \nxandlist{\unskip, }{\unskip{} \@@and~}{\unskip, \@@and~}}
\def\author@andify{%
  \nxandlist {\unskip ,\penalty-1 \space\ignorespaces}%
    {\unskip {} \@@and~}%
    {\unskip ,\penalty-2 \space \@@and~}%
}"
  );
  Ok(())
}

/// amsart's `\uppercasenonmath` (amscls/amsart.cls:405-426; amsproc.cls:383-404 and
/// amsbook.cls:384-405 identical): uppercases a macro's text in place, leaving its `$…$` and
/// `\(…\)` math as is, for the class's own title code and derived classes' titles and running
/// heads. Neither Perl binding ports it (ams_core.cls.ltxml, ams_support.sty.ltxml), so it was
/// undefined. The class lines verbatim, each `\newcommand` as `\long\def` (the class has no earlier
/// definition to guard); `\Mc`'s `\providecommand` is already in `ams_support`. Not ported: the
/// textcase switch (amsart.cls:427-430, `\altucnm` in place of `\uppercasenonmath` when
/// `\MakeTextUppercase` is defined), whose `\edef#1{\the\toks@}` after `\MakeTextUppercase{\toks@{#1}}`
/// empties the title here (RED repro sectioning-frontmatter/amsart_uppercasenonmath_textcase_keeps_the_title).
pub fn amsart_uppercase_nonmath() -> Result<()> {
  RequirePackage!("amsgen");
  RawTeX!(
    r"\long\def\uppercasenonmath#1{\toks@\@emptytoks
  \@xp\@skipmath\@xp\@empty#1$$%
  \edef#1{{\@nx\protect\@nx\@upprep\the\toks@}}%
}
\long\def\@upprep{%
  \spaceskip1.3\fontdimen2\font plus1.3\fontdimen3\font
  \upchars@}
\long\def\upchars@{%
  \def\ss{SS}\def\i{I}\def\j{J}\def\ae{\AE}\def\oe{\OE}%
  \def\o{\O}\def\aa{\AA}\def\l{\L}\def\Mc{M{\scshape c}}}
\long\def\@skipmath#1$#2${%
  \@xskipmath#1\(\)%
  \@ifnotempty{#2}{\toks@\@xp{\the\toks@$#2$}\@skipmath\@empty}}%
\long\def\@xskipmath#1\(#2\){%
  \uppercase{\toks@\@xp\@xp\@xp{\@xp\the\@xp\toks@#1}}%
  \@ifnotempty{#2}{\toks@\@xp{\the\toks@\(#2\)}\@xskipmath\@empty}}%
"
  );
  Ok(())
}

/// amsart.cls:1386-1410 (amsbook.cls:1282, amsproc.cls:1322) `\@makecaption`: "#1.\enspace #2" for every caption type, a
/// newfloat type's and a listing's as a figure's (`\lx@makecaption`, latex_constructs_rust_only.rs), the `\enspace` a
/// space here; and the fonts it sets them in (amsart.cls:1354-1355), whose `\@captionheadfont` tells caption the class is
/// an AMS one (caption_sty.rs `CAPTION_DEFAULT_LABELSEP`, caption3.sty:1138). The classes' own, so their bindings call
/// it (ams_core_cls.rs, amsbook_cls.rs; lmcs and acmart through amsart), not ams_support, which amsaddr and the
/// `\curraddr`/`\subjclass` autoloads bring to any class: under article the captions keep the kernel's "Figure 1:",
/// "Listing 1:", and under caption "Program 1:" (pdflatex), where Perl's ams_support.sty.ltxml:192-193 gives figures
/// and tables ". " there too (KNOWN_PERL_ERRORS #1140). Guards
/// `perfect_kernel_batch78y::{amsart_newfloat_caption_takes_the_class_format, caption_*, amsaddr_*}`.
pub fn ams_class_caption_format() -> Result<()> {
  DefMacro!("\\@captionheadfont", "\\scshape");
  DefMacro!("\\@captionfont", "\\normalfont");
  DefMacro!("\\lx@makecaption{}{}", "\\lx@tag[][. ]{\\lx@fnum@@{#1}}#2");
  DefMacro!("\\format@title@figure{}", "\\lx@makecaption{figure}{#1}");
  DefMacro!("\\format@title@table{}", "\\lx@makecaption{table}{#1}");
  Ok(())
}

/// The amsart class internals that documents and `.ldf` files call directly (amscls/amsart.cls; amsproc.cls and
/// amsbook.cls carry the same lines), for the three class bindings (ams_core_cls.rs), not the ams_support loads an
/// autoload makes in other classes. The class lines verbatim, a `\newcommand` as `\def`:
/// - :75-96 `\except`, `\for`, `\forany` over the `\newswitch` switches, with the switches the class declares,
///   `runhead` (:305) and `toc`/`lof`/`lot` (:1143-1145): `\section{\for{toc}{<short>}\except{toc}{<long>}}` keeps the
///   long title in the body (run-347 inventory 89 + 73 papers; 1304.2934, 1601.05128, 1912.08145).
/// - :554, :564 `\@keywords` and `\@subjclass`, empty: `\keywords` and `\subjclass` give the frontmatter, so a
///   document's own print of them (2510.22858 `\ifx\@keywords\@empty\else…`) prints nothing twice, as `\@dedicatory`;
///   :114 `\@dateposted` and :710-711 `\@PII` alike (memo-l.cls:116-171's abstract-page notes read them: 2412.15928).
///   A document's own `\subjclass` or `\keywords` that stores them as amsart's does (amsart.cls:553-564; 2004.06211:12
///   `\renewcommand*\subjclass[2][2000]{\def\@subjclass{#2}…}`) has them printed by amsart's `\maketitle` (:657-658, a
///   first-page footnote): the `\maketitle` hands a non-empty one to the frontmatter, as the binding's own setter would
///   (`\lx@ams@stored@topmatter`, at `\lx@maketitle@begin`), unless that setter ran (`\lx@ams@subjclass@given`,
///   `\lx@ams@keywords@given`), so a paper using the class's commands is unchanged, nor when the K11 reroute owns a raw
///   class's `\keywords` store (`\iflx@ams@keywords@rerouted`; a class wrapping acmart, which loads amsart). Perl
///   loses them (OXIDIZED_DESIGN_DIVERGENCES #902; 1009.1929 too).
/// - :687-689 `\copyrightinfo{0000}{(copyright holder)}`, the defaults `\copyrightyear`/`\copyrightholder` (2403.17965).
/// - :837, :856-864 `\abstractbox` and `\@setabstract`/`\@setabstracta`, which set a box an abstract-like environment
///   of the document's fills (1512.02361 and 2602.22909 `{abstracts}`, 2303.07306 and 2201.00740 `{acknowledgements}`,
///   all amsart's own `{abstract}` code under another name): `\maketitle` sets it and makes `\@setabstract` `\relax`,
///   so one after the title is set where it ends (`\ifx\@setabstract\relax \@setabstracta \fi`; the acknowledgements).
///   The box `\maketitle` sets is the title page's abstract, as amsart's `{abstract}` fills it: the frontmatter abstract
///   (`<ltx:abstract>`, 1512.02361's English and French abstracts). The kernel `{abstract}` gives the frontmatter and
///   leaves the box void. Before, `\@setabstract` was an empty macro and the box was never set (1512.02361 kept its text
///   only through an Error's recovery, with a stray "=").
/// - :1189-1218, :1240-1241 the contents-line code, `\@tocwrite`/`\@tocwriteb` (2004.14791's own `\chapter`, 1805.00244)
///   and `\l@figure`/`\l@table` (magyar.ldf:1696-1697 patches them; 2503.14638).
/// - :1244 `\@defaultbiblabelstyle`, :1262 `\bibliofont` (2105.14892, 2312.03234).
///
/// Not ported: :320-322 the section names, empty until `\appendix` (1504.08082 `\markboth{\sectionname}…`, a running
/// head): the refnum composition reads a defined `\<counter>name` as the unit's name (base_utilities.rs
/// `\lx@refnum@compose`), so an empty one put a space before every section number.
///
/// Perl's binding defines none of them (ams_core.cls.ltxml, ams_support.sty.ltxml), KNOWN_PERL_ERRORS #721.
pub fn amsart_class_internals() -> Result<()> {
  RequirePackage!("amsgen");
  // Inside a derived class's title-page deposit (`\lx@deposit@maketitle`, sect05.rs) the frontmatter carries the
  // abstract and the class's abstract box reads given and empty (`\lx@dropped@env@stores`): its `\@setabstract`
  // (resphilosophica.cls:259) sets nothing there, where the empty box made an empty paragraph.
  DefConditional!("\\iflx@ams@depositing", {
    lookup_bool("lx_depositing_frontmatter_fields")
  });
  // A raw class's own `\keywords` store is the K11 reroute's (frontmatter_stores.rs `reroute_raw_class_stores`, which
  // records its store name), handed on as the class sets it: not handed over again at the `\maketitle`.
  DefConditional!("\\iflx@ams@keywords@rerouted", {
    lookup_value("lx_store_cs_keywords").is_some_and(|v| !matches!(v, Stored::None))
  });
  RawTeX!(
    r"\newswitch{}
\DeclareRobustCommand{\except}[1]{%
  \if\csname ?@#1\endcsname \expandafter\@gobble
  \else \expandafter\@firstofone
  \fi
}
\DeclareRobustCommand{\for}[1]{%
  \if\csname ?@#1\endcsname \expandafter\@firstofone
  \else \expandafter\@gobble
  \fi
}
\DeclareRobustCommand{\forany}[1]{%
  \csname for@any@01\endcsname#1,?,\@nil
}
\@namedef{for@any@\@False}#1,{%
  \csname for@any@%
    \csname ?@\zap@space#1 \@empty\endcsname
  \endcsname
}
\@namedef{?@?}{x}
\@namedef{for@any@\@True}#1\@nil#2{#2}
\def\for@any@x{\@car\@gobble}
\newswitch{runhead}
\newswitch{toc}
\newswitch{lof}
\newswitch{lot}
\let\@keywords=\@empty
\let\@subjclass=\@empty
\let\lx@ams@keywords@given\relax
\let\lx@ams@subjclass@given\relax
\def\lx@ams@stored@topmatter{%
  \ifx\@subjclass\@empty\else\ifx\@subjclass\relax\else\ifx\lx@ams@subjclass@given\@empty\else
    \expandafter\lx@ams@stored@subjclass\expandafter{\@subjclass}%
    \global\let\@subjclass\@empty\fi\fi\fi
  \ifx\@keywords\@empty\else\ifx\@keywords\relax\else\ifx\lx@ams@keywords@given\@empty\else
    \iflx@ams@keywords@rerouted\else
    \expandafter\lx@ams@stored@keywords\expandafter{\@keywords}%
    \global\let\@keywords\@empty\fi\fi\fi\fi}
\def\lx@ams@stored@subjclass#1{\expandafter\lx@ams@stored@subjclass@\expandafter{\subjclassname}{#1}}
\def\lx@ams@stored@subjclass@#1#2{\lx@add@classification[scheme={#1},name={#1:~}]{#2}}
\def\lx@ams@stored@keywords#1{\lx@add@keywords[name={\keywordsname:~}]{#1}}
\let\@dateposted\@empty
\let\@PII\@empty
\def\copyrightyear{0000}\def\copyrightholder{(copyright holder)}
\newbox\abstractbox
\def\@setabstract{\iflx@ams@depositing\else\ifvoid\abstractbox\else
  \lx@add@frontmatter@now{ltx:abstract}{\box\abstractbox}\fi\fi
  \global\let\@setabstract\relax}
\def\@setabstracta{%
  \iflx@ams@depositing\else
  \ifvoid\abstractbox
  \else
    \skip@20\p@ \advance\skip@-\lastskip
    \advance\skip@-\baselineskip \vskip\skip@
    \box\abstractbox
    \prevdepth\z@ % because \abstractbox is a vtop
  \fi\fi
}
\def\@tochangmeasure#1{\sbox\z@{#1}%
  \ifdim\wd\z@>\csname r@tocindent\@toclevel\endcsname\relax
    \@xp\xdef\csname r@tocindent\@toclevel\endcsname{\the\wd\z@}%
  \fi
}
\def\@toclevel{0}
\def\@tocline#1#2#3#4#5#6#7{\relax
  \ifnum #1>\c@tocdepth % then omit
  \else
    \par \addpenalty\@secpenalty\addvspace{#2}%
    \begingroup \hyphenpenalty\@M
    \@ifempty{#4}{%
      \@tempdima\csname r@tocindent\number#1\endcsname\relax
    }{%
      \@tempdima#4\relax
    }%
    \parindent\z@ \leftskip#3\relax \advance\leftskip\@tempdima\relax
    \rightskip\@pnumwidth plus4em \parfillskip-\@pnumwidth
    #5\leavevmode\hskip-\@tempdima #6\nobreak\relax
    \hfil\hbox to\@pnumwidth{\@tocpagenum{#7}}\par
    \nobreak
    \endgroup
  \fi}
\def\@tocpagenum#1{\hss{\mdseries #1}}
\def\@tocwrite#1{\@xp\@tocwriteb\csname toc#1\endcsname{#1}}
\def\@tocwriteb#1#2#3{%
  \begingroup
    \def\@tocline##1##2##3##4##5##6{%
      \ifnum##1>\c@tocdepth
      \else \sbox\z@{##5\let\indentlabel\@tochangmeasure##6}\fi}%
    \csname l@#2\endcsname{#1{\csname#2name\endcsname}{\@secnumber}{}}%
  \endgroup
  \addcontentsline{toc}{#2}%
    {\protect#1{\csname#2name\endcsname}{\@secnumber}{#3}}}
\def\l@figure{\@tocline{0}{3pt plus2pt}{0pt}{1.5pc}{}}
\let\l@table=\l@figure
\def\@defaultbiblabelstyle#1{#1.}
\def\bibliofont{\footnotesize}"
  );
  // :685-689: `\copyrightinfo` sets the two as well as the copyright.
  DefMacro!(
    "\\copyrightinfo{}{}",
    "\\def\\copyrightyear{#1}\\@ifnotempty{#2}{\\def\\copyrightholder{#2}}\\lx@ams@add@copyright{#1}{#2}"
  );
  // amsart.cls:657-658: `\maketitle` prints the subject classification and keywords the document stored itself.
  AddToMacro!("\\lx@maketitle@begin", "\\lx@ams@stored@topmatter");
  // amsart.cls:645: `\@maketitle` runs `\@setabstract` after the title (here `\lx@maketitle@body`, sect05.rs).
  AddToMacro!("\\lx@maketitle@body", "\\@setabstract");
  Ok(())
}

/// amsart.cls:620-629 `\@cleartopmattertags`: once its `\maketitle` has run, amsart lets `\keywords`, `\subjclass`,
/// `\dedicatory` (and the other topmatter commands) be `\relax`, so pdflatex typesets a later `\keywords{…}` as the
/// text it is and never reads it as an argument. The binding goes on reading it, for its frontmatter markup (ruling
/// 2026-10-25), except where that short argument would run away — a `\par` (tex.web §392 `par_token`) or the end of the
/// look-ahead's input before its closing brace, which pdflatex never meets: there the call is pdflatex's `\relax`, and the
/// group is read as text (2204.07665: `\keywords{\small{ … }` a brace short after a first `\maketitle`; "Paragraph
/// ended before `\keywords` was complete", the two labels inside it lost). After the title our cleanup has let
/// `\maketitle` be `\relax` too, as amsart's does. Guard `perfect_kernel_batch80a::amsart_post_title_keywords_one_brace_short`.
fn post_title_topmatter(setter: &str) -> Result<Tokens> {
  let title_done = x_equals(&T_CS!("\\maketitle"), &T_CS!("\\relax"));
  if title_done && short_argument_runs_away() {
    return Ok(Tokens!());
  }
  Ok(Tokens!(T_CS!(setter)))
}

/// Whether the argument ahead — an optional `[…]`, then a braced group — meets a `\par` before its group closes, or
/// is still open where the look-ahead ends (a bounded peek: nothing is read).
fn short_argument_runs_away() -> bool {
  let mut tokens = peek_upcoming_tokens(120, 48_000, 12_000)
    .filter(|t| t.get_catcode() != Catcode::SPACE)
    .peekable();
  let par = T_CS!("\\par");
  if tokens.peek() == Some(&T_OTHER!("[")) {
    // the optional argument, to its `]` outside braces
    let mut depth = 0usize;
    loop {
      match tokens.next() {
        None => return false,
        Some(t) if t == par => return false,
        Some(t) if t.get_catcode() == Catcode::BEGIN => depth += 1,
        Some(t) if t.get_catcode() == Catcode::END => depth = depth.saturating_sub(1),
        Some(t) if depth == 0 && t == T_OTHER!("]") => break,
        Some(_) => {},
      }
    }
  }
  if !tokens
    .next()
    .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
  {
    return false;
  }
  let mut depth = 1usize;
  for t in tokens {
    if t == par {
      return true;
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 {
          return false;
        }
      },
      _ => {},
    }
  }
  true
}
