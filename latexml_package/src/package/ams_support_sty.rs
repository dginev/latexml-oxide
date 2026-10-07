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

#[rustfmt::skip]
LoadDefinitions!({
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
  RequirePackage!("amsthm");
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

  DefMacro!("\\title[]{}",
    "\\gdef\\@shorttitle{#1}\\gdef\\@title{#2}\\ifx.#1.\\else\\lx@add@toctitle{#1}\\fi\\lx@add@title{#2}");

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
  DefMacro!("\\address[]{}", "\\lx@add@address[name={#1}]{#2}");
  DefMacro!("\\curraddr[]{}",
    "\\lx@add@contact[role=current_address,name={\\curraddrname\\ifx.#1.\\else{, #1}\\fi:\\ }]{#2}");
  DefMacro!("\\email[]{}",
    "\\lx@add@email[name={\\emailaddrname\\ifx.#1.\\else{, #1}~\\fi:\\ }]{#2}");
  DefMacro!("\\urladdr[]{}",
    "\\lx@add@url[name={\\urladdrname\\ifx.#1.\\else{, #1}~\\fi:\\ }]{#2}");
  DefMacro!("\\dedicatory{}",   "\\lx@add@contact[role=dedicatory]{#1}");
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
  DefMacro!("\\translator[]{}", "\\lx@add@translator[name={\\translname~}]{#2}");
  DefMacro!("\\keywords{}",     "\\lx@add@keywords[name={\\keywordsname:~}]{#1}");

  // \thanks{} ( == ack, not latex's \thanks, not in author)
  // make a throwaway optional argument available for OmniBus use
  DefMacro!("\\thanks[]{}",
    "\\lx@add@pubnote[role=thanks,name={\\@ifundefined{thanksname}{}{\\thanksname}}]{#2}");

  // Non-standard but makes it easier to create bindings for variations on AMS classes;
  // just redefine this macro
  DefMacro!("\\@subjclassyear", None, "1991");

  DefMacro!("\\subjclassname", None,
    "\\textup{\\@subjclassyear} Mathematics Subject Classification");
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
  DefMacro!("\\subjclass[Default:\\@subjclassyear]{}",
    "\\ifx.#1.\\else\\xdef\\@subjclassyear{#1}\\fi\
     \\lx@add@classification[scheme={#1 Mathematics Subject Classification},name={\\subjclassname:~}]{#2}");

  DefMacro!("\\copyrightinfo{}{}", "\\lx@add@copyright{#1, #2}");

  def_macro_noop("\\pagespan{}{}")?; // ?
  DefMacro!("\\PII{}",  "\\lx@add@classification[scheme=PII]{#1}");
  DefMacro!("\\ISSN{}", "\\lx@add@classification[scheme=ISSN]{#1}");

  DefMacro!("\\currentvolume", None, "");
  DefMacro!("\\currentissue", None, "");
  DefMacro!("\\currentmonth", None, "");
  DefMacro!("\\currentyear", None, "");
  DefMacro!("\\volinfo", None, "");
  DefMacro!("\\issueinfo{}{}{}{}",
    "\\def\\currentvolume{#1}\\def\\currentissue{#2}\\def\\currentmonth{#3}\\def\\currentyear{#4}\\def\\volinfo{Volume \\currentvolume, Number \\number0\\currentissue, \\currentmonth\\ \\currentyear}\\lx@add@pubnote[role=volume]{\\volinfo}");

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

  DefMacro!("\\format@title@figure{}", "\\lx@tag[][. ]{\\lx@fnum@@{figure}}#1");
  DefMacro!("\\format@title@table{}", "\\lx@tag[][. ]{\\lx@fnum@@{table}}#1");

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

  // \newswitch, \setFalse, \setTrue — complex sub closures, stubbed as no-ops
  def_macro_noop("\\newswitch[]{}")?;
  def_macro_noop("\\setFalse{}")?;
  def_macro_noop("\\setTrue{}")?;

  // funny control structures, using above switches
  // \except
  // \for
  // \forany

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
