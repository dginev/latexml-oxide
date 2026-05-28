//! pst-node.sty — PSTricks node connections (DVI-only)
//! Perl: pst-node.sty.ltxml — 557 lines
//! Node definition and connection macros for PSTricks diagrams.
//! DVI-only: all node/connection commands produce no visible output.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("pstricks");

  // Perl pst-node.sty.ltxml L91-107: register node-connection keyvals on
  // the shared `pstricks` keyval group. Perl types PSAngle / PSDimension /
  // PSDimDim / Float aren't currently registered Rust types; register with
  // the untyped placeholder ("") since Rust pst-node stubs all connection
  // commands to no-ops (DVI-only), so no consumer actually coerces these.
  // Documents that switch on the key-presence (`\@ifundefined{KV@pstricks@
  // angle@default}`) get the right answer.
  for key in ["angle", "angleA", "angleB",
              "arcangle", "arcangleA", "arcangleB",
              "nodesep", "nodesepA", "nodesepB",
              "offset", "arm", "armA", "armB",
              "ncurv", "loopsize", "radius", "framesize"] {
    DefKeyVal!("pstricks", key, "");
  }

  // Node definition macros — Perl L30-120. `\rnode` / `\pnode` etc. are
  // Perl DefConstructor with coordinate-reading closures; Rust stubs them
  // as DefMacro passthrough of the content (#3). DP-audit flags the kind
  // flip. Structural — pst-node nodes need coordinate+position readers
  // (same PSDim/PSAngle family as pstricks_support, WISDOM #41 parameter
  // gap). Rust's passthrough preserves the label text in output; the
  // geometric node-graph is simply not emitted. Not an `\edef` site.
  //
  // Intentional divergence (WISDOM #44 class: blocked-on-parameter-type):
  // ALL node + connection stubs below share this single root cause —
  // missing PSDim/PSAngle/PSDimDim/Float types — so the DefConstructor
  // → DefMacro kind flips are a single-cluster waiver. Porting the
  // PS* parameter-type family closes every rnode/Rnode/pnode/cnode/
  // circlenode/ovalnode/trinode/dianode + nc*/pc* connection entry at
  // once. Audit currently flags rnode+pnode explicitly.
  DefMacro!("\\rnode[]{}{}", "#3");
  DefMacro!("\\Rnode[]{}{}", "#3");
  def_macro_noop("\\pnode[]")?;
  // PSTricks node macros take a `(coord)` argument, not `[...]`. Perl
  // pst-node.sty.ltxml: \cnode = 'ZeroPSCoord {PSDimension} {}'
  // (coord, radius, name); \Cnode = 'ZeroPSCoord {}' (coord, name).
  // The old `[]{}` signatures left the `(x,y)` coord unconsumed, so
  // `\Cnode(1,1){000}` leaked "1,1)" as body text. Consume the optional
  // `[par]`, the `(coord)`, then the braced args (mirrors the
  // `\cnodeput ... [] () {} {}` pattern in pst_all_sty.rs).
  def_macro_noop("\\cnode OptionalMatch:* [] () {} {}")?;
  def_macro_noop("\\Cnode OptionalMatch:* [] () {}")?;
  DefMacro!("\\circlenode[]{}{}", "#3");
  DefMacro!("\\ovalnode[]{}{}", "#3");
  def_macro_noop("\\fnode OptionalMatch:* []{}")?;
  def_macro_noop("\\dotnode OptionalMatch:* []{}")?;
  DefMacro!("\\trinode[]{}{}", "#3");
  DefMacro!("\\dianode[]{}{}", "#3");

  // Connection macros — Perl L130-350
  def_macro_noop("\\ncline OptionalMatch:* []{}{}")?;
  def_macro_noop("\\pcline OptionalMatch:* []{}{}")?;
  def_macro_noop("\\nccurve OptionalMatch:* []{}{}")?;
  def_macro_noop("\\pccurve OptionalMatch:* []{}{}")?;
  def_macro_noop("\\ncbar OptionalMatch:* []{}{}")?;
  def_macro_noop("\\ncdiag OptionalMatch:* []{}{}")?;
  def_macro_noop("\\ncangle OptionalMatch:* []{}{}")?;
  def_macro_noop("\\ncangles OptionalMatch:* []{}{}")?;
  def_macro_noop("\\ncloop OptionalMatch:* []{}{}")?;
  def_macro_noop("\\ncarc OptionalMatch:* []{}{}")?;
  def_macro_noop("\\pcbar OptionalMatch:* []{}{}")?;
  def_macro_noop("\\pcdiag OptionalMatch:* []{}{}")?;
  def_macro_noop("\\pcangle OptionalMatch:* []{}{}")?;
  def_macro_noop("\\pcangles OptionalMatch:* []{}{}")?;
  def_macro_noop("\\pcloop OptionalMatch:* []{}{}")?;
  def_macro_noop("\\pcarc OptionalMatch:* []{}{}")?;
  // `\nccircle{node}{radius}` — Perl pst-node.sty.ltxml `DefPSConstructor`.
  // We stub it to empty here; the actual arc rendering is PSTricks-DVI-only
  // and not ported. Witness: 1304.4491 (stage 14 RUST-REGRESSION).
  def_macro_noop("\\nccircle OptionalMatch:* []{}{}")?;
  def_macro_noop("\\pccircle OptionalMatch:* []{}{}")?;
  // `\ncdiagg{node}{node}` (extended diagonal) and `\pcdiagg` — also stubs.
  def_macro_noop("\\ncdiagg OptionalMatch:* []{}{}")?;
  def_macro_noop("\\pcdiagg OptionalMatch:* []{}{}")?;

  // Label macros — Perl L360-450
  DefMacro!("\\naput OptionalMatch:* []{}", "#2");
  DefMacro!("\\nbput OptionalMatch:* []{}", "#2");
  DefMacro!("\\ncput OptionalMatch:* []{}", "#2");
  DefMacro!("\\nput OptionalMatch:* []{}{}", "#3");
  DefMacro!("\\lput OptionalMatch:* []{}{}", "#3");
  DefMacro!("\\aput OptionalMatch:* []{}", "#2");
  DefMacro!("\\bput OptionalMatch:* []{}", "#2");
  DefMacro!("\\mput OptionalMatch:* {}", "#1");
  DefMacro!("\\Lput OptionalMatch:* []{}{}", "#3");
  DefMacro!("\\Mput OptionalMatch:* {}", "#1");
  DefMacro!("\\Rput OptionalMatch:* []{}{}", "#3");

  // Perl pst-node.sty.ltxml L504-538: \Aput / \Bput — capitalized node-label
  // macros. Perl signature is `OptionalMatch:* []` and the full path forwards
  // through \Aput@start + \put@end label-put infrastructure (computeLabelPos,
  // finishLabelPut, _psActiveTransform). Rust lacks that graphics pipeline —
  // pst-node is stubbed DVI-only. Match the Perl signature exactly and
  // expand to empty; any label written after (`\Aput[sep]{text}`) is
  // emitted by normal expansion, matching the drop-args-pass-through
  // behavior of sibling stubs \aput/\bput.
  def_macro_noop("\\Aput OptionalMatch:* []")?;
  def_macro_noop("\\Bput OptionalMatch:* []")?;

  // Box macros — Perl L460-520
  def_macro_noop("\\psmatrix")?;
  def_macro_noop("\\endpsmatrix")?;
  def_macro_noop("\\psrowalign{}")?;
  def_macro_noop("\\pscolalign{}")?;
  def_macro_noop("\\ncdot")?;

  // Perl L224-232: \cnodeput OptionalMatch:* [] OptionalBracketed ZeroPSCoord {} {}
  // builds a \rput{\circlenode{#5}{#6}} expansion. For LaTeXML DVI-only
  // binding, reduce to placing the body.
  DefMacro!("\\cnodeput OptionalMatch:* [] [] {}{}{}", "#6");
  // Perl L334-339: \ncdiagg {} {} — node diagonal connection; DVI-only no-op.
  def_macro_noop("\\ncdiagg OptionalMatch:* []{}{}")?;
  // Perl L341-346: \pcdiagg — same shape.
  def_macro_noop("\\pcdiagg OptionalMatch:* []{}{}")?;
});
