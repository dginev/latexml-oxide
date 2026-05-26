//! figcaps.sty (from the `preprint` bundle) — defers figure
//! captions to the end of the document so that figures can be
//! placed at the end of a preprint.
//!
//! figcaps.sty starts with `\@ifundefined{chapter}{}{\PackageError
//! {figcaps}{`figcaps' may only be used with article-like classes}}`.
//! Our engine seemingly defines `\chapter` even when loading
//! article-like REVTeX (witness arXiv:1912.07260: revtex4-style
//! preprint). The error fires, but raw-load continues; we then
//! also touch caption-handling code that doesn't behave usefully
//! in HTML (we put captions inline with figures).
//!
//! Perl LaTeXML has no figcaps binding (INCLUDE_STYLES=false
//! skips), single missing-binding warning.
//!
//! Match Perl: stub as a no-op shell. Caption-deferral has no
//! HTML equivalent; the kernel `\figure`/`\table` envs already
//! emit captions inline.
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "figcaps.sty",
    "figcaps.sty is a no-op stub — caption-deferral is a print-only concern with no HTML equivalent."
  );
  // figcaps's user toggles — accept and discard.
  def_macro_noop("\\figcapson")?;
  def_macro_noop("\\figcapsoff")?;
  def_macro_noop("\\figmarkon")?;
  def_macro_noop("\\figmarkoff")?;
});
