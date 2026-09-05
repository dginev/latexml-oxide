//! No-op stub for showexpl.sty.
//!
//! showexpl.sty (TL) builds the `LTXexample` environment and
//! `\LTXinputExample` command (display LaTeX source + its typeset
//! result side by side) on top of `listings`. Its package body uses
//! `\lst@newenvironment{LTXexample}{...}{...\SX@put@code@result}`
//! whose end-group `\xdef\SX@@explpreset{\the\@temptokena,...}` parse
//! tickles a readBalanced `Expected opening '{'` error in our raw-load
//! path, then `\SX@put@code@result` (defined just after, at L208)
//! never registers, cascading.
//!
//! Perl LaTeXML never observes this: its default TEXINPUTS excludes
//! `/usr/share/texlive`, so showexpl.sty is reported as missing-file
//! and skipped, completing cleanly. Verified on arXiv:2002.09910
//! (`\usepackage{...showexpl...}`): Perl emits
//! `Warning:missing_file:showexpl` and 0 errors.
//!
//! Match Perl's effective behavior with a stub that (a) pulls in
//! showexpl's real `\RequirePackage` dependency chain so any
//! transitively-needed macros (listings, varwidth, float) resolve, and
//! (b) no-ops the user-facing showexpl commands. All 15 R-stage papers
//! blocked on `\SX@put@code@result` were checked: NONE invoke
//! `\begin{LTXexample}` or `\LTXinputExample` — they only load the
//! package — so the stub costs no document content.
//!
//! Witnesses (CONVERR_7/CONVERR_3 → OK): 1604.00381, 1606.01035,
//! 1706.03232, 1804.02704, 1804.07221, 1612.01022, 1905.12059,
//! 1706.09226, 1701.01402, 1812.06820, 1801.01025, 1806.10927,
//! 2001.08314, 2002.09910, 1901.08750.

use crate::{
  package::listings_sty::{listings_read_raw_lines, lst_process_display},
  prelude::*,
};

#[rustfmt::skip]
LoadDefinitions!({
  // showexpl.sty L1-13 \RequirePackage chain (attachfile is loaded
  // conditionally via `\IfFileExists`; we load it unconditionally —
  // harmless, has a binding).
  RequirePackage!("listings");
  RequirePackage!("refcount");
  RequirePackage!("varwidth");
  RequirePackage!("float");
  // {LTXexample}[keys] — showexpl's whole point: typeset the SOURCE as a
  // listing AND its RESULT (real showexpl routes the body through listings'
  // write-file layer into \jobname.tmp and \input's it back). Reproduce that
  // semantic directly: capture the body raw, emit the code listing through
  // the listings display engine, then re-tokenize the body so it EXECUTES
  // as the result. `[pos=…]`-style keys arrange the two blocks on the page —
  // presentation. (The TL doc corpus — koma/babel/gauss manuals — uses this
  // env heavily; the earlier noop stub predates OXIDIZED_DESIGN #161, whose
  // DefPlain fix unblocked raw showexpl parsing, and dropped example bodies
  // entirely.)
  DefPrimitive!(T_CS!("\\LTXexample"), None, {
    let _keys = read_optional(None)?;
    // Same group discipline as the built-in lstlisting closure: the display
    // constructor's token stream closes a box group it expects opened here.
    bgroup();
    let text = listings_read_raw_lines("LTXexample");
    // unread stack: LAST unread reads FIRST → reading order is
    // listing, result-body, \end{LTXexample}.
    unread(Tokenize!(TeXString::assembled("\\end{LTXexample}".to_string())));
    unread(Tokenize!(TeXString::assembled(text.clone())));
    unread(Tokens::new(lst_process_display(None, &text)));
  });
  def_macro_noop("\\endLTXexample")?;
  // \LTXinputExample[keys]{file}: same idea from a file — listing + \input.
  DefMacro!("\\LTXinputExample[]{}", "\\lstinputlisting{#2}\\input{#2}");
  def_macro_noop("\\setupSXfiles")?;
  def_macro_noop("\\setupLZfiles")?;
  // showexpl.sty:66-86 load-time state that documents rebuilding
  // `LTXexample` from the internals read back (lshort-german l2kurz.tex:73-100
  // `\edef\x{\endgroup\def\noexpand\SX@codefile{\SX@codefile}…}\x`: with the
  // macros undefined the self-reference `\def\SX@codefile{\SX@codefile}`
  // "expands into itself" 96 times). The counter is showexpl.sty:57.
  RawTeX!(concat!(
    r"\newcommand*\SX@graphicname{}\newcommand*\SX@graphicparam{}",
    r"\newcommand\ResultBox{}\let\ResultBox=\fbox",
    r"\newdimen\ResultBoxSep\ResultBoxSep=\fboxsep\newdimen\ResultBoxRule\ResultBoxRule=\fboxrule",
    r"\newcommand*\SX@pos{}\newcommand*\SX@width{}\newcommand*\SX@hsep{}\newcommand*\SX@vsep{}",
    r"\newcommand*\SX@overhang{}\newcommand*\SX@rframe{}\newcommand\SX@preset{}",
    r"\newcommand*\SX@explpreset{}\newcommand*\SX@@explpreset{}",
    r"\newcommand*\SX@codefile{}\edef\SX@codefile{\jobname.tmp}",
    r"\newcommand*\SX@justification{\raggedright}",
    r"\@ifundefined{c@ltxexample}{\newcounter{ltxexample}}{}",
    // showexpl.sty:58-61,115 — the switches raw dependants skip over
    // (pst-exa.sty:104 `\if@SX@rangeaccept` inside a false `\ifpstexa@swpl`
    // branch: an UNDEFINED conditional is not counted by the skip, tex.web
    // §510, so the nested `\else`/`\fi` desync it — pst-exa-doc). Guard:
    // `perfect_kernel_batch56::showexpl_switches_balance_a_skipped_branch`.
    r"\newif\if@SX@rangeaccept\newif\if@SX@varwidth\newif\if@SX@wide",
    r"\newif\if@SX@attachfile\newif\ifSX@wasodd"
  ));
  // showexpl.sty:208 `\SX@put@code@result`: typeset the listing written to
  // `\SX@codefile` and run it as the result — the binding's own display path.
  DefMacro!("\\SX@put@code@result", "\\lstinputlisting{\\SX@codefile}\\input{\\SX@codefile}");
});
