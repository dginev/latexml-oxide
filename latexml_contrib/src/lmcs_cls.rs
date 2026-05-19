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
  RequirePackage!("xcolor");
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
