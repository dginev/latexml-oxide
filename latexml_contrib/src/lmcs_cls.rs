//! Stub for lmcs.cls (Logical Methods in Computer Science journal class).
//!
//! lmcs.cls extends amsart for the LMCS journal. The raw cls relies on
//! pgfmath + tikz + XeTeXLinkBox for ORCID-logo rendering, which fails
//! mid-load in our system; consequently `\lmcsdoi`, `\lmcsheading`,
//! `\lmcsorcid` (the publication-metadata macros every LMCS paper uses
//! in the preamble) end up undefined. Provide content-preserving stubs.
//! Witness 2305.14448, 2305.19985.
use latexml_package::prelude::*;


LoadDefinitions!({
  LoadClass!("amsart");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
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
  DefMacro!("\\lmcsdoi{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=lmcs-doi]{Volume #1, Issue #2, Paper #3}");
  // \lmcsheading{vol}{issue}{year}{pages}{subm}{publ}{rev}{spec_iss}{title}
  // The raw cls signature is 7-args but with optional/positional variations.
  // We don't reproduce the running-header layout — just discard, since the
  // metadata is already captured by \lmcsdoi.
  def_macro_noop("\\lmcsheading{}{}{}{}")?;
  // \lmcsorcid{orcid-id} — render as a plain link rather than the
  // tikz/XeTeXLinkBox logo construction.
  DefMacro!("\\lmcsorcid{}",
    "\\href{https://orcid.org/#1}{ORCID:#1}");

  // Section-numbering and shortauthors/shorttitle helpers used by raw
  // cls header layout. Stub as no-op or pass-through.
  def_macro_noop("\\shorttitle{}")?;
  def_macro_noop("\\shortauthors{}")?;

  // `\dOi` placeholder produced by the raw cls when no \lmcsdoi was
  // declared. Stub to empty so it doesn't appear as red error text.
  def_macro_noop("\\dOi")?;
});
