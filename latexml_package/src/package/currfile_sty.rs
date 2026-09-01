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
  // currfile.sty:78-85 sanitizes `\@filef@und` BEFORE `\filename@parse`, so
  // under latex.ltx the pieces come out catcode-12 and `\currfile@if`'s
  // `\ifx\@tempa\currfilename` (both sides sanitized) matches. The engine's
  // `\filename@parse` (latex_constructs.rs, Perl pool:980 `ExplodeText`)
  // re-letters its output, so `\ifcurrfilename{main.tex}{yes}{no}` answered
  // "no". Re-sanitize the main-file pieces and rebuild the derived names
  // exactly as `\currfile@set` (:78-86) does — the raw package's own
  // outcome, not a new mechanism. (Kernel-level catcode preservation in
  // `\filename@parse` is a surpass-Perl candidate: PLANS.md P50.)
  RawTeX!(r"\@onelevel@sanitize\currfiledir
\@onelevel@sanitize\currfilebase
\@onelevel@sanitize\currfileext
\xdef\currfilename{\currfilebase\ifx\currfileext\@empty\else.\currfileext\fi}%
\xdef\currfilepath{\currfiledir\currfilename}%
\ifcurrfile@abspath
  \xdef\currfile@stack{{\currfiledir}{\currfilebase}{\currfileext}{\currfileabsdir}}%
\else
  \xdef\currfile@stack{{\currfiledir}{\currfilebase}{\currfileext}}%
\fi
\global\let\currfile@stackinit\currfile@stack");
});
