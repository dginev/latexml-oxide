//! ieeetj.cls — IEEE Transactions journal template (author-bundled, ~4900 lines;
//! not raw-loaded). The class is IEEEtran V1.7a copied inline (its header,
//! ieeetj.cls:209) plus its own `\affil{…}` store (:4856-4858, numbered
//! affiliations after a flat `\author` list), so it is IEEEtran with that
//! `\affil` (authblk's would attach every affiliation to every author). As an
//! OmniBus stub, IEEEtran's `\ifCLASSOPTION…`
//! conditionals, `{IEEEkeywords}`, `\IEEEPARstart` and `\IEEEpubidadjcol` were
//! undefined (arXiv 2605.01773). The class's late-defined journal-metadata
//! frontmatter macros are bound below; they leaked as literal text before
//! (witness 2405.01673, 2603.04284 → `\corresp \authornote \receiveddate …`).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("IEEEtran");
  // The author list is flat, with `\\` as a line break inside it (2405.01673);
  // inst_support's `\author` reads it that way, as under OmniBus.
  RequirePackage!("inst_support");
  // ieeetj.cls:4858 `\affil{text}` stores one numbered affiliation block.
  DefMacro!("\\affil{}", "\\lx@add@affiliation{#1}");

  // Corresponding-author byline and author/funding note — preserve as notes.
  DefMacro!(
    "\\corresp{}",
    "\\lx@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
  DefMacro!(
    "\\authornote{}",
    "\\lx@add@frontmatter{ltx:note}[role=note]{#1}"
  );
  // `\affil{…}` — a numbered affiliation block. inst_support already defines
  // `\affil`; keep that. (Listed here only for the record.)

  // Editorial dates and identifiers. IEEE templates ship these as unfilled
  // "XX Month, XXXX" / "XXXX.2022.1234567" placeholders; rendering them would
  // surface template noise, so gobble them (they leaked as raw text before).
  def_macro_noop("\\receiveddate{}")?;
  def_macro_noop("\\reviseddate{}")?;
  def_macro_noop("\\accepteddate{}")?;
  def_macro_noop("\\publisheddate{}")?;
  def_macro_noop("\\currentdate{}")?;
  def_macro_noop("\\doiinfo{}")?;
  def_macro_noop("\\history{}")?;
  def_macro_noop("\\articletype{}")?;
  // Author-supplied funding acknowledgement — preserve rather than gobble.
  DefMacro!(
    "\\fundingtext{}",
    "\\lx@add@frontmatter{ltx:note}[role=funding]{#1}"
  );
});
