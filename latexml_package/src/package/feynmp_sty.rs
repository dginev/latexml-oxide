//! feynmp.sty — Feynman diagrams with MetaPost (the PDF-output variant of
//! feynmf; the user-level macros are identical). Perl raw-loads the real
//! feynmp.sty; we mirror feynmf — raw-load when present, then stub the diagram
//! macros (shared `feynmf_diagram_stubs`) so feynmp papers don't cascade into
//! undefined-env / `expected:$` display-math errors (witness 1003.1620:
//! `\fmf{...label=$$}` was Rust 28 / Perl 0 with feynmp unbound).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("feynmp", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Same diagram environments as feynmf (see feynmf_sty.rs): drop the graphics
  // body (no MetaPost in our pipeline) but keep the env so surrounding context
  // parses cleanly.
  DefEnvironment!("{fmfgraph}{}{}",
    "<ltx:note role='feynman-diagram'>(Feynman diagram, #1x#2)</ltx:note>",
    mode => "internal_vertical");
  DefEnvironment!("{fmfgraph*}{}{}",
    "<ltx:note role='feynman-diagram'>(Feynman diagram, #1x#2)</ltx:note>",
    mode => "internal_vertical");
  DefEnvironment!("{fmffile}{}", "#body", mode => "internal_vertical");

  feynmf_sty::feynmf_diagram_stubs()?;
});
