//! Stub for lmcs.cls (Logical Methods in Computer Science journal class).
//!
//! lmcs.cls is NOT distributed in TeX Live (papers bundle it, but the corpus
//! copies don't reach our search path), so Perl LaTeXML — which ships no lmcs
//! binding — emits `Can't find binding for class lmcs (using OmniBus)` and
//! falls back to **OmniBus**. We mirror Perl by loading OmniBus as the base
//! (NOT amsart, which the prior version used): OmniBus supplies the lazy
//! theorem-env autoloads (`\begin{thm}`/`\begin{lem}`/… → `\newtheorem{thm}`
//! /`{lem}`/…), so a paper whose preamble does `\newtheorem{remark}[thm]{…}`
//! and body uses `\begin{thm}`/`\begin{lem}` resolves the shared `thm` counter
//! exactly as Perl does. amsart pre-defines none of those, so Rust hit
//! `undefined:{thm}`/`{lem}`/`\thethm`. Witness 1607.01886 (RUST 3 → 0; 12
//! theorems matching Perl). The raw cls relies on pgfmath/tikz/XeTeXLinkBox for
//! ORCID-logo rendering (fails mid-load), so `\lmcsdoi`/`\lmcsheading`/
//! `\lmcsorcid` get content-preserving stubs below. Witness 2305.14448,
//! 2305.19985, 1607.04128 (graphicx).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  // lmcs.cls L33 `\LoadClass[11pt,reqno]{amsart}`. Perl ships no lmcs binding,
  // so it loads OmniBus AND dependency-scans the raw lmcs.cls, which loads
  // amsart.cls.ltxml → ams_support.sty.ltxml. That is where the amsart-family
  // frontmatter macros (`\urladdr`, `\address`, `\email`, `\curraddr`) come
  // from. Without it, `\urladdr{\url{…}}` was undefined (witness 1709.06170,
  // RUST 1 → 0; Perl loads it via the amsart dep). ams_support is frontmatter
  // only — it does not pre-declare theorem envs, so the OmniBus lazy
  // `\begin{thm}` autoloads that 1607.01886 relies on are untouched.
  RequirePackage!("ams_support");
  RequirePackage!("hyperref");
  // lmcs.cls L24 `\usepackage{helvet,cclicenses}` pulls graphicx in
  // transitively: cclicenses → `\RequirePackage{rotating}` (cclicenses.sty
  // L25) → `\RequirePackage{graphicx}` (rotating.sty L62). Perl ships no lmcs
  // binding, so it raw-loads lmcs.cls and gets graphicx that way; LMCS papers
  // therefore use `\includegraphics` WITHOUT their own `\usepackage{graphicx}`.
  // Our stub intercepts the raw cls (which fails mid-load on its tikz/
  // XeTeXLinkBox ORCID machinery), so the transitive graphicx never loaded
  // and `\includegraphics` was undefined where Perl is clean. Supply it
  // directly. Witness 1607.04128 (`\documentclass{lmcs}`, `\includegraphics`
  // with no explicit graphicx load): RUST 1 → 0.
  RequirePackage!("graphicx");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("enumitem");
  RequirePackage!("etoolbox");

  // LMCS publication metadata. Real macros assign internal counters
  // and set up running headers; for HTML rendering we just preserve
  // the args as named notes.
  DefMacro!(
    "\\lmcsdoi{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=lmcs-doi]{Volume #1, Issue #2, Paper #3}"
  );
  // \lmcsheading{vol}{issue}{year}{pages}{subm}{publ}{rev}{spec_iss}{title}
  // The raw cls signature is 7-args but with optional/positional variations.
  // We don't reproduce the running-header layout — just discard, since the
  // metadata is already captured by \lmcsdoi.
  def_macro_noop("\\lmcsheading{}{}{}{}")?;
  // \lmcsorcid{orcid-id} — render as a plain link rather than the
  // tikz/XeTeXLinkBox logo construction.
  DefMacro!("\\lmcsorcid{}", "\\href{https://orcid.org/#1}{ORCID:#1}");

  // Section-numbering and shortauthors/shorttitle helpers used by raw
  // cls header layout. Stub as no-op or pass-through.
  def_macro_noop("\\shorttitle{}")?;
  def_macro_noop("\\shortauthors{}")?;

  // `\dOi` placeholder produced by the raw cls when no \lmcsdoi was
  // declared. Stub to empty so it doesn't appear as red error text.
  def_macro_noop("\\dOi")?;
});
