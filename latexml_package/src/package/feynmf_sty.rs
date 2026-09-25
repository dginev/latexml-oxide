//! feynmf.sty — Feynman diagrams with MetaFont
//! Perl: feynmf.sty.ltxml — loads the raw sty with noltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("feynmf", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  feynmf_graph_environments()?;
  feynmf_diagram_stubs()?;
});

/// feynmf/feynmp `{fmfgraph}`/`{fmfgraph*}` environments: a `(width,height)`
/// pair (feynmf.sty:180 `\def\fmfgraph(#1,#2)`, :193 `fmfgraph*`) followed by
/// the diagram body. The real package emits a Metafont/MetaPost diagram. For
/// HTML rendering we drop the graphics body (neither runs in our pipeline) but
/// preserve the env so the surrounding equation/figure context still parses
/// cleanly. Witness 2309.07343 (15 errors all from {fmfgraph*} undefined).
/// The pair is `Match:( Until:, Until:)`, as `\put` reads its: the former
/// `{}{}` took `(` and `3` of `(30,20)`, so the note read
/// "(Feynman diagram, (x3)". Guard:
/// `binding_singletons_56::fmfgraph_reads_its_size_pair`.
pub(crate) fn feynmf_graph_environments() -> Result<()> {
  DefEnvironment!("{fmfgraph} Match:( Until:, Until:)",
    "<ltx:note role='feynman-diagram'>(Feynman diagram, #2x#3)</ltx:note>",
    mode => "internal_vertical");
  DefEnvironment!("{fmfgraph*} Match:( Until:, Until:)",
    "<ltx:note role='feynman-diagram'>(Feynman diagram, #2x#3)</ltx:note>",
    mode => "internal_vertical");
  // {fmffile}{name} - wraps a Feynman-diagram session. Render as no-op
  // env (the diagrams inside are rendered by {fmfgraph}/{fmfgraph*}).
  DefEnvironment!("{fmffile}{}", "#body", mode => "internal_vertical");
  Ok(())
}

/// Diagram-content macros used inside `{fmfgraph}`/`{fmfgraph*}` (shared by the
/// `feynmf` and `feynmp` packages — feynmp is the MetaPost/PDF variant with the
/// SAME user macros). We don't render the diagrams (no MetaFont/MetaPost in our
/// pipeline), so — like Perl, which raw-loads the real `.sty` — we absorb the
/// macros' args as no-ops so the surrounding context parses cleanly instead of
/// digesting their bodies (e.g. a `label=$$` would otherwise cascade into
/// `expected:$` display-math errors; witness 1003.1620 feynmp, Rust 28 / Perl 0).
pub(crate) fn feynmf_diagram_stubs() -> Result<()> {
  def_macro_noop("\\fmf{}{}")?;
  def_macro_noop("\\fmfv{}{}")?;
  def_macro_noop("\\fmfset{}{}")?;
  def_macro_noop("\\fmflabel{}{}")?;
  // Vertex-placement and decoration macros (1-arg vertex lists / 0-arg).
  def_macro_noop("\\fmfleft{}")?;
  def_macro_noop("\\fmfright{}")?;
  def_macro_noop("\\fmftop{}")?;
  def_macro_noop("\\fmfbottom{}")?;
  def_macro_noop("\\fmfsurround{}")?;
  def_macro_noop("\\fmfdot{}")?;
  def_macro_noop("\\fmfblob{}{}")?;
  def_macro_noop("\\fmffreeze")?;
  def_macro_noop("\\fmfcmd{}")?;
  def_macro_noop("\\fmfpen{}")?;
  Ok(())
}
