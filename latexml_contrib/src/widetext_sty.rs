//! Stub for widetext.sty (revtex-style wide-text environment).
//!
//! widetext.sty provides {widetext} env which renders body across
//! the full page width in two-column mode. For XML output the column
//! distinction doesn't matter; render body transparently.
//!
//! Witness: 2503.01409 (mnras paper with `\usepackage{widetext}` +
//! `\begin{widetext}...\end{widetext}`).
use latexml_package::prelude::*;

LoadDefinitions!({
  DefEnvironment!("{widetext}", "#body");
  // widetext.sty L56-57: `\IfFileExists{cuted.sty}{\RequirePackage{cuted}}`.
  // cuted.sty L176/189 defines `\strip`/`\endstrip` as a `{strip}` env
  // for full-page-width blocks at top/bottom in two-column mode. Our
  // widetext binding bypasses raw load, so cuted's strip env doesn't
  // register. Render body transparently (no column distinction in
  // HTML/XML). Witness 2403.05215 (SAJ paper using widetext + saj.sty).
  DefEnvironment!("{strip}", "#body");
});
