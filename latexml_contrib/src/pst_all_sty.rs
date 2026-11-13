//! `pst-all.sty` — the PSTricks bundle loader.
//!
//! pst-all.sty:17-32 is nothing but a `\RequirePackage` chain; the former
//! stub loaded pstricks + pst-node only (behind a `missing_file` warning) and
//! no-op'd the pst-node connections, so `\multido`, `\psplot`, `\pstree`…
//! were undefined for every pst-all document (K1 step 3 pass two, batch
//! 56an). Now the real chain: the bundled packages that have bindings use
//! them (pstricks, pst-plot, pst-node, pst-grad, multido); the rest raw-load
//! on top of the pstricks binding, exactly as pstricks-add.tex:27 already
//! raw-inputs pst-node.tex (the psmatrix/dsptricks path of batch 56af).
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("pstricks");
  RequirePackage!("pst-plot");
  RequirePackage!("pst-node");
  RequirePackage!("pst-tree");
  RequirePackage!("pst-grad");
  RequirePackage!("pst-coil");
  RequirePackage!("pst-text");
  RequirePackage!("pst-3d");
  RequirePackage!("pst-eps");
  RequirePackage!("pst-fill", options => vec!["tiling".to_string()]);
  RequirePackage!("pstricks-add");
  RequirePackage!("multido");
});
