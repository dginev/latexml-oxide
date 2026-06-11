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
  DefMacro!("\\publname{}",
    "\\lx@add@frontmatter{ltx:note}[role=publication]{#1}");

  DefMacro!("\\title[]{}",
    "\\gdef\\@shorttitle{#1}\\gdef\\@title{#2}\\ifx.#1.\\else\\lx@add@toctitle{#1}\\fi\\lx@add@title{#2}");

  DefMacro!("\\lx@author@sep", ",\\ ");
  DefMacro!("\\lx@author@conj", "\\ and\\ ");   // \@@and

  // \author[shortname]{name} Use one \author per author
  // followed by whatever contact information applies to that author.
  // What to do with shortauthor ?  (Perl PR #2767)
  DefMacro!("\\author[]{}",
    "\\def\\@shortauthor{#1}\\def\\@author{#2}\\lx@add@author{#2}");

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
});
