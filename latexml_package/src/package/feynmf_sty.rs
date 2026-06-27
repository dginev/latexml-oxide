//! feynmf.sty — Feynman diagrams with MetaFont
//! Perl: feynmf.sty.ltxml — loads the raw sty with noltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("feynmf", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // feynmf {fmfgraph}/{fmfgraph*} environments: 2-arg-on-begin
  // `(width,height)` followed by Feynman diagram body. Real package
  // emits a Metafont diagram. For HTML rendering we drop the graphics
  // body (no Metafont in our pipeline) but preserve the env so the
  // surrounding equation/figure context still parses cleanly. Witness
  // 2309.07343 (15 errors all from {fmfgraph*} undefined).
  DefEnvironment!("{fmfgraph}{}{}",
    "<ltx:note role='feynman-diagram'>(Feynman diagram, #1x#2)</ltx:note>",
    mode => "internal_vertical");
  DefEnvironment!("{fmfgraph*}{}{}",
    "<ltx:note role='feynman-diagram'>(Feynman diagram, #1x#2)</ltx:note>",
    mode => "internal_vertical");
  // {fmffile}{name} - wraps a Feynman-diagram session. Render as no-op
  // env (the diagrams inside are rendered by {fmfgraph}/{fmfgraph*}).
  DefEnvironment!("{fmffile}{}", "#body", mode => "internal_vertical");
  feynmf_diagram_stubs()?;
});

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
