use latexml_package::prelude::*;

LoadDefinitions!({
  // svg-extract.sty is the export-companion to svg.sty; it forwards
  // `\includesvg` to svg.sty and adds PDF/EPS conversion bookkeeping
  // we don't need. Just load svg.sty so `\includesvg` resolves.
  // Witness 2504.08550 (`\usepackage{svg-extract}` + `\includesvg{…}`).
  RequirePackage!("svg");
});
