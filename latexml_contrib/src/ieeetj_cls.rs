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

  // Corresponding-author byline and author/funding note — preserve as notes, each keeping its last value as the
  // class's `\gdef` does.
  DefMacro!(
    "\\corresp{}",
    "\\lx@clear@frontmatter{ltx:note}[role=corresponding]\\lx@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
  DefMacro!(
    "\\authornote{}",
    "\\lx@clear@frontmatter{ltx:note}[role=note]\\lx@add@frontmatter{ltx:note}[role=note]{#1}"
  );
  // `\affil{…}` — a numbered affiliation block. inst_support already defines
  // `\affil`; keep that. (Listed here only for the record.)

  // Editorial dates and the DOI, as the first page prints them (ieeetj.cls:3439-3440 "Received …; revised …;
  // accepted …; Date of publication …; date of current version …", "Digital Object Identifier 10.1109/…"; a
  // template's "XX Month, XXXX" placeholders included, as pdflatex prints them); gobbling them as template noise
  // dropped real dates and DOIs too.
  DefMacro!("\\receiveddate{}", "\\lx@add@date[role=received]{#1}");
  DefMacro!("\\reviseddate{}", "\\lx@add@date[role=revised]{#1}");
  DefMacro!("\\accepteddate{}", "\\lx@add@date[role=accepted]{#1}");
  DefMacro!("\\publisheddate{}", "\\lx@add@date[role=published]{#1}");
  DefMacro!("\\currentdate{}", "\\lx@add@date[role=current]{#1}");
  // An empty `\doiinfo{}` (the class's own default) prints no DOI line (ieeetj.cls:3440 `\ifx\@doiinfo\@empty`; 2609.27083).
  DefMacro!(
    "\\doiinfo{}",
    "\\lx@clear@frontmatter{ltx:pubnote}[role=doi]\\def\\lx@ieee@doi{#1}\\ifx\\lx@ieee@doi\\@empty\\else\\lx@add@pubnote[role=doi]{10.1109/#1}\\fi"
  );
  // The names the first page prints them with (ieeetj.cls:3439; Perl looks a date's name up as `\lx@date@<role>@name`).
  DefMacro!("\\lx@date@published@name", "Date of publication~");
  DefMacro!("\\lx@date@current@name", "Date of current version~");
  def_macro_noop("\\history{}")?;
  def_macro_noop("\\articletype{}")?;
  // Author-supplied funding acknowledgement — preserve rather than gobble.
  DefMacro!(
    "\\fundingtext{}",
    "\\lx@add@frontmatter{ltx:note}[role=funding]{#1}"
  );
});
