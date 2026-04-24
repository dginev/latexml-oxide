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
  DefMacro!("\\pnode[]", "");
  DefMacro!("\\cnode[]{}", "");
  DefMacro!("\\Cnode[]{}", "");
  DefMacro!("\\circlenode[]{}{}", "#3");
  DefMacro!("\\ovalnode[]{}{}", "#3");
  DefMacro!("\\fnode OptionalMatch:* []{}", "");
  DefMacro!("\\dotnode OptionalMatch:* []{}", "");
  DefMacro!("\\trinode[]{}{}", "#3");
  DefMacro!("\\dianode[]{}{}", "#3");

  // Connection macros — Perl L130-350
  DefMacro!("\\ncline OptionalMatch:* []{}{}", "");
  DefMacro!("\\pcline OptionalMatch:* []{}{}", "");
  DefMacro!("\\nccurve OptionalMatch:* []{}{}", "");
  DefMacro!("\\pccurve OptionalMatch:* []{}{}", "");
  DefMacro!("\\ncbar OptionalMatch:* []{}{}", "");
  DefMacro!("\\ncdiag OptionalMatch:* []{}{}", "");
  DefMacro!("\\ncangle OptionalMatch:* []{}{}", "");
  DefMacro!("\\ncangles OptionalMatch:* []{}{}", "");
  DefMacro!("\\ncloop OptionalMatch:* []{}{}", "");
  DefMacro!("\\ncarc OptionalMatch:* []{}{}", "");
  DefMacro!("\\pcbar OptionalMatch:* []{}{}", "");
  DefMacro!("\\pcdiag OptionalMatch:* []{}{}", "");
  DefMacro!("\\pcangle OptionalMatch:* []{}{}", "");
  DefMacro!("\\pcangles OptionalMatch:* []{}{}", "");
  DefMacro!("\\pcloop OptionalMatch:* []{}{}", "");
  DefMacro!("\\pcarc OptionalMatch:* []{}{}", "");

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
  DefMacro!("\\Aput OptionalMatch:* []", "");
  DefMacro!("\\Bput OptionalMatch:* []", "");

  // Box macros — Perl L460-520
  DefMacro!("\\psmatrix", "");
  DefMacro!("\\endpsmatrix", "");
  DefMacro!("\\psrowalign{}", "");
  DefMacro!("\\pscolalign{}", "");
  DefMacro!("\\ncdot", "");

  // Perl L224-232: \cnodeput OptionalMatch:* [] OptionalBracketed ZeroPSCoord {} {}
  // builds a \rput{\circlenode{#5}{#6}} expansion. For LaTeXML DVI-only
  // binding, reduce to placing the body.
  DefMacro!("\\cnodeput OptionalMatch:* [] [] {}{}{}", "#6");
  // Perl L334-339: \ncdiagg {} {} — node diagonal connection; DVI-only no-op.
  DefMacro!("\\ncdiagg OptionalMatch:* []{}{}", "");
  // Perl L341-346: \pcdiagg — same shape.
  DefMacro!("\\pcdiagg OptionalMatch:* []{}{}", "");
});
