use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // AAS LaTeX 2pp4 — early-1990s 2-column AAS preprint style.
  //
  // Old AAS-family option: `\documentstyle[aas2pp4]{article}`. Shares
  // the API with aaspp/aasms (`\affil`, `\acknowledgments`, `\keywords`,
  // `\reference`, `\references`, etc.), all bound in
  // `aas_support_sty.rs`. No separate upstream `aas2pp4.sty.ltxml`
  // exists in LaTeXML or ar5iv-bindings; route through the shared
  // aas_support layer in the same shape as `aaspp_sty.rs`.
  //
  // Driver paper from sandbox_failures_181_html: astro-ph9610252
  // (`\documentstyle[aas2pp4]{article}`).
  RequirePackage!("aas_support");
  RequirePackage!("epsf");
});
