use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: doublespace.sty.ltxml
  // Mostly no-op for LaTeXML, except for two ignorable environments
  // Same vertical-mode fix as setspace_sty.rs — paragraph-container envs
  // must keep BOUND_MODE vertical so `$$` inside enters display math.
  DefEnvironment!("{singlespace}",  "#body", mode => "internal_vertical");
  DefEnvironment!("{doublespace}",  "#body", mode => "internal_vertical");
  DefEnvironment!("{spacing}{}",    "#body", mode => "internal_vertical");
  // doublespace.sty:66 (the copy quant-ph0004056 ships; TeX Live has none). Witnesses physics0003041,
  // quant-ph0005132. Guard `perfect_kernel_batchclustfix3::doublespace_setstretch`.
  DefMacro!("\\setstretch{}", "\\renewcommand{\\baselinestretch}{#1}");
});
