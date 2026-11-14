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
  RawTeX!(
    r"\def\lx@fmf@keep#1{\expandafter\fmf@keep\lx@fmf@size{#1}}
\def\lx@fmf@keepstar#1{\expandafter\fmf@keepstar\lx@fmf@size{#1}}"
  );
  // In the graph `\fmfkeep{<name>}` keeps it for `\fmfreuse{<name>}` (feynmf.sty:182/:196, feynmp.sty:156):
  // `\fmf@keep`/`\fmf@keepstar` (feynmf.sty:230-237) record the graph `\begin{fmfgraph}(w,h)\end{fmfgraph}`,
  // which the raw `\fmfreuse` (:239-250) sets again. Outside a graph it stays the package's
  // `\fmf@nokeep` error (:226-229), which every kept graph raised (witnesses hep-lat0210017,
  // hep-th9805098, 1411.4662, hep-ph0606300).
  DefEnvironment!("{fmfgraph} Match:( Until:, Until:)",
    "<ltx:note role='feynman-diagram'>(Feynman diagram, #2x#3)</ltx:note>",
    after_digest_begin => sub[whatsit] { fmf_keep_in_graph(whatsit, "\\lx@fmf@keep") },
    mode => "internal_vertical");
  DefEnvironment!("{fmfgraph*} Match:( Until:, Until:)",
    "<ltx:note role='feynman-diagram'>(Feynman diagram, #2x#3)</ltx:note>",
    after_digest_begin => sub[whatsit] { fmf_keep_in_graph(whatsit, "\\lx@fmf@keepstar") },
    mode => "internal_vertical");
  // {fmffile}{name} - wraps a Feynman-diagram session. Render as no-op
  // env (the diagrams inside are rendered by {fmfgraph}/{fmfgraph*}).
  // The argument is the METAFONT/MetaPost file's name (feynmp.sty:97-109 `\def\thefmffile{#1}`, `\openout`), never
  // typeset: `{sde_feynman}` raised "Script _ can only appear in math mode" (1906.06341, 2512.17695; OXIDIZED_DESIGN
  // #551). Guard `perfect_kernel_batch65m::file_arguments_are_not_typeset`.
  // feynmf.sty:97 `\def\thefmffile{#1}` for the environment, which `\fmfkeep` records and `\fmfreuse` reads
  // back (`\fmf@k:f:<name>`, :233/:240); left `\relax` (:142), the reuse defined `\thefmffile` as itself.
  DefEnvironment!("{fmffile} Semiverbatim", "#body",
    after_digest_begin => sub[whatsit] {
      let name = whatsit.get_arg(1).map(|a| a.to_string()).unwrap_or_default();
      def_macro(T_CS!("\\thefmffile"), None, Tokens::new(ExplodeText!(name.trim())), None)?;
    },
    mode => "internal_vertical");
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

/// The graph's `\fmfkeep` (feynmf.sty:182 `\def\fmfkeep##1{\fmf@keep{#1}{#2}{##1}}`): `keeper` (`\lx@fmf@keep` or
/// `\lx@fmf@keepstar`, which pass the graph's `{<width>}{<height>}`, `\lx@fmf@size`), local to the graph's group.
fn fmf_keep_in_graph(whatsit: &mut Whatsit, keeper: &str) -> Result<()> {
  let mut size = vec![T_BEGIN!()];
  if let Some(width) = whatsit.get_arg(2) {
    size.extend(width.revert()?.unlist());
  }
  size.extend([T_END!(), T_BEGIN!()]);
  if let Some(height) = whatsit.get_arg(3) {
    size.extend(height.revert()?.unlist());
  }
  size.push(T_END!());
  def_macro(T_CS!("\\lx@fmf@size"), None, Tokens::new(size), None)?;
  let_i(&T_CS!("\\fmfkeep"), &T_CS!(keeper), None);
  Ok(())
}
