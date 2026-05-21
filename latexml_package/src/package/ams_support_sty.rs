use crate::prelude::*;

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
  ProcessOptions!();

  //======================================================================
  // Font size commands:

  DefPrimitive!("\\larger",  None, font => { scale => 1.2 });
  DefPrimitive!("\\smaller", None, font => { size => 0.8333333333333334 }); // 1/1.2

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
  if state::lookup_bool("2.09_COMPATIBILITY") {
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
  DefMacro!("\\shorttitle{}", "\\@add@frontmatter{ltx:toctitle}{#1}");
  // Author / address fields — preserve as ltx:note so the strings
  // reach the XML output instead of being gobbled (content-
  // preserving). These are typically the short-form variants
  // already covered by \author / \address from the main flow, but
  // when authors set them explicitly the values are still real
  // metadata.
  DefMacro!("\\shortauthor{}",
    "\\@add@frontmatter{ltx:note}[role=shortauthor]{#1}");
  DefMacro!("\\authors{}",
    "\\@add@frontmatter{ltx:note}[role=authors]{#1}");
  DefMacro!("\\shortauthors{}",
    "\\@add@frontmatter{ltx:note}[role=shortauthors]{#1}");
  DefMacro!("\\addresses{}",
    "\\@add@frontmatter{ltx:note}[role=addresses]{#1}");
  DefMacro!("\\publname{}",
    "\\@add@frontmatter{ltx:note}[role=publication]{#1}");

  DefMacro!("\\title[]{}",
    "\\if.#1.\\else\\def\\shorttitle{#1}\\@add@frontmatter{ltx:toctitle}{#1}\\fi\\@add@frontmatter{ltx:title}{#2}");

  DefMacro!("\\lx@author@sep", ",\\ ");
  DefMacro!("\\lx@author@conj", "\\ and\\ ");   // \@@and

  DefMacro!("\\author[]{}",
    "\\if.#1.\\else\\def\\shortauthor{#1}\\fi\\def\\@author{#2}\\lx@author{#2}");

  DefMacro!("\\datename", None, "\\textit{Date}:");

  DefMacro!("\\contrib[]{}",
    "\\@add@frontmatter{ltx:creator}[role=contributor]{\\@personname{#2}}");

  DefMacro!("\\commby{}",
    "\\@add@frontmatter{ltx:creator}[role=communicator]{\\@personname{#1}}");

  DefConstructor!("\\@@@address{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\address[]{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#2}}");

  DefConstructor!("\\@@@curraddr{}", "^ <ltx:contact role='current_address'>#1</ltx:contact>");
  DefMacro!("\\curraddr{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@curraddr{#1}}");

  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email[]{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#2}}");

  DefConstructor!("\\@@@urladdr{}", "^ <ltx:contact role='url'>#1</ltx:contact>");
  DefMacro!("\\urladdr{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@urladdr{#1}}");

  DefConstructor!("\\@@@dedicatory{}", "^ <ltx:contact role='dedicatory'>#1</ltx:contact>");
  DefMacro!("\\dedicatory{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@dedicatory{#1}}");

  // \date{}
  DefMacro!("\\dateposted{}", "\\@add@frontmatter{ltx:date}[role=posted]{#1}");

  // \thanks{} ( == ack, not latex's \thanks, not in author)
  // make a throwaway optional argument available for OmniBus use
  DefMacro!("\\thanks[]{}",
    "\\@add@frontmatter{ltx:acknowledgements}[name={\\@ifundefined{thanksname}{}{\\thanksname}}]{#2}");

  DefMacro!("\\translator[]{}",
    "\\@add@frontmatter{ltx:creator}[role=translator]{\\@personname{#2}}");

  DefMacro!("\\keywordsname", None, "Key words and phrases");
  DefMacro!("\\keywords{}",
    "\\@add@frontmatter{ltx:keywords}[name={\\keywordsname}]{#1}");

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
     \\@add@frontmatter{ltx:classification}[scheme={#1 Mathematics Subject Classification},name={\\subjclassname}]{#2}");

  DefMacro!("\\copyrightinfo{}{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{\\copyright #1: #2}");

  def_macro_noop("\\pagespan{}{}")?; // ?
  DefMacro!("\\PII{}",
    "\\@add@frontmatter{ltx:classification}[scheme=PII]{#1}");
  DefMacro!("\\ISSN{}",
    "\\@add@frontmatter{ltx:classification}[scheme=ISSN]{#1}");

  DefMacro!("\\currentvolume", None, "");
  DefMacro!("\\currentissue", None, "");
  DefMacro!("\\currentmonth", None, "");
  DefMacro!("\\currentyear", None, "");
  DefMacro!("\\volinfo", None, "");
  DefMacro!("\\issueinfo{}{}{}{}",
    "\\def\\currentvolume{#1}\\def\\currentissue{#2}\\def\\currentmonth{#3}\\def\\currentyear{#4}\\def\\volinfo{Volume \\currentvolume, Number \\number0\\currentissue, \\currentmonth\\ \\currentyear}\\@add@frontmatter{ltx:note}[role=volume-info]{\\volinfo}");

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
  if lookup_bool("2.09_COMPATIBILITY") {
    DefMacro!("\\defaultfont", "\\normalfont");
    DefMacro!("\\rom", "\\textup");
  }
  // amsart.cls L1922 (modern), amsproc/amsbook also: `\newenvironment{pf}
  // {\@newpf[\proofname]}{\popQED\endtrivlist}` — same as {proof} but
  // shorter name. Provide as a proof alias so any
  // amsart/amsproc/amsbook user calling \begin{pf}...\end{pf} resolves
  // cleanly. Witness 14 papers with cas-sc, amsart, AMS-derived classes.
  //
  // CRUCIAL: defer to AtBeginDocument, and guard with `\@ifundefined{pf}`.
  // Without the deferral, our `\newenvironment{pf}` runs at
  // `\documentclass{amsart}` time — BEFORE the user's preamble macro
  // `\newcommand{\pf}{\operatorname{pf}}` (Pfaffian). Our engine then
  // refuses the user's redefinition, and downstream `\pf(U)` in display
  // math expands to `\begin{@proof}` instead of `\operatorname{pf}` →
  // cascading XMath-in-title / equation-in-XMath malformedness. Witness:
  // 2306.17599 (Pfaffian paper, amsart class — was 622 errors, now 0).
  //
  // `\newenvironment` is a primitive — needs digestion, not just expansion.
  // We package the deferred raw_tex into `\AtBeginDocument` so it fires
  // after the entire user preamble has run.
  // Witness 2308.01739 (amsart): paper defines `\def\endpf{\hfill $\Box$
  // \vskip0.5cm}` as a manual proof terminator macro. The original
  // `\@ifundefined{pf}` guard ONLY checked for `\pf` (env-begin); if the
  // user had defined ONLY `\endpf` (env-end-shaped CS), our guard still
  // passed and `\newenvironment{pf}` OVERWROTE the user's `\endpf` with
  // env-end machinery (`\end{@proof}`), then standalone `\endpf` calls
  // tried to pop a `{@proof}` env that was never opened → "Attempt to
  // close a group that switched to mode internal_vertical" cascades.
  // Add a symmetric `\@ifundefined{endpf}` guard so a pre-defined
  // user `\endpf` blocks our redefinition.
  stomach::raw_tex(
    "\\AtBeginDocument{\
       \\@ifundefined{pf}{\\@ifundefined{endpf}{\\newenvironment{pf}{\\begin{@proof}}{\\end{@proof}}}{}}{}\
       \\@ifundefined{pf*}{\\@ifundefined{endpf*}{\\newenvironment{pf*}[1]{\\begin{@proof}[##1]}{\\end{@proof}}}{}}{}}"
  )?;

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
      Ok(stored_map!("href" => common::cleaners::clean_url(&url_str)))
    });

  DefMacro!("\\MR{}", "MR #1");
  // \MRhref{label} — Math Reviews link; preserve as note (the link
  // target encodes the MR id which is genuine reference metadata).
  DefMacro!("\\MRhref{}", "\\@add@frontmatter{ltx:note}[role=mr-ref]{#1}");
});
