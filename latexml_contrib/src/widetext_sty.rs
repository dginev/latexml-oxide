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
});
