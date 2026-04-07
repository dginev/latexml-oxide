//! smfart.cls — Société Mathématique de France article class
//! No Perl binding exists — Perl uses OmniBus (generic article fallback).
//! smfart.cls is based on AMS classes (requires amsgen, amsfonts, amsmath).
//! We load amsart as base, which provides the core AMS document structure.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Load amsart as the base class — provides AMS document structure,
  // title/author/abstract environments, theorem-like environments, etc.
  load_class("amsart", Vec::new(), Tokens!())?;

  // smfart-specific commands that papers may use.
  // These are simplified stubs matching the raw TeX definitions.
  // \alttitle — alternate title (French/English toggle)
  DefMacro!("\\alttitle{}", "");
  // \altauthor — alternate author listing
  DefMacro!("\\altauthor{}", "");
  // \dedicatory — dedication line
  DefMacro!("\\dedicatory{}", "\\par\\noindent\\itshape #1\\par");
  // \keywords — keywords (often used via \subjclass in AMS)
  DefMacro!("\\keywords{}", "");
  // \altabstract environment — alternate language abstract
  DefEnvironment!("{altabstract}", "#body");
  // \resu environment — French resumé (abstract)
  DefEnvironment!("{resu}", "#body");
  // \smfbymark — SMF "by" connector for author lists
  DefMacro!("\\smfbymark", " ");
  // \altmaketitle — alternate maketitle (no-op, \maketitle handles it)
  DefMacro!("\\altmaketitle", "");
  // \smf@org@trivlist — smfart saves/restores the trivlist environment.
  // In raw TeX smfart.cls, this is defined via \let\smf@org@trivlist\trivlist.
  // The raw TeX loading sometimes fails before this let, leaving it undefined.
  Let!("\\smf@org@trivlist", "\\trivlist");
  Let!("\\smf@org@endtrivlist", "\\endtrivlist");
});
