//! currfile.sty — current input-file name/path/ext (Rust-only binding; Perl
//! has none and falls through to `missing_file`).
//!
//! The real package is loaded raw: it is pure kernel-level TeX (kvoptions +
//! filehook) and defines `\currfilename` = `\jobname.tex` for the main
//! file, plus the `\ifcurrfilename{name}{yes}{no}` 3-argument macro family
//! (currfile.sty:322-335, via `\currfile@if` :337-347). An earlier stub
//! shape here defined that family as `DefConditional!` — a TeX `\if…`
//! whose `\else`/`\fi` never arrives, so `\ifcurrfilename{x}{yes}{no}`
//! swallowed the rest of the document as an `\iffalse` body (witness: TL
//! doc corpus currfile/currfile, pythontex/pythontex — pythontex.sty:32
//! `\RequirePackage{currfile}`). Complete support over stubs.
//!
//! Known limit: the per-`\input` push/pop (currfile.sty:68-73 hooks
//! `\filehook@every@atbegin`/`@atend`, fired from LaTeX's `file/before`
//! hooks in filehook-2020.sty) does not fire in this engine, which has no
//! `file/*` hook layer (`tex_file_io.rs`, `\CurrentFile` note). Inside a
//! `\input` file the macros keep reporting the main file.
use crate::prelude::*;

LoadDefinitions!({
  // Real currfile.sty L30 `\RequirePackage{filehook}` — go through the
  // binding (it locks `\filehook@cmp` and registers the versioned
  // `filehook-2020.sty` sub-file as an INTERPRETABLE source) so the raw
  // currfile.sty finds `\filehook@prefixwarg` & co. defined.
  RequirePackage!("filehook");
  InputDefinitions!("currfile", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
