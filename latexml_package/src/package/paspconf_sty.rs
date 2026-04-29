use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // PASP Conference Series proceedings style.
  //
  // Old AAS-family option: `\documentstyle[paspconf]{article}`. paspconf
  // shares the bulk of its API with aaspp/aasms (`\affil`,
  // `\altaffilmark`, `\altaffiltext`, `\acknowledgments`, `\keywords`,
  // `\reference`, `\references`, `\aap`, `\aj`, `\apj`, `\apjs`,
  // `\mnras`, etc.), all bound in `aas_support_sty.rs`. No separate
  // upstream `paspconf.sty.ltxml` exists in LaTeXML or ar5iv-bindings,
  // so route through the shared aas_support layer in the same shape
  // as `aaspp_sty.rs`.
  //
  // Driver papers from sandbox_failures_181_html: astro-ph9811043,
  // astro-ph9902095, astro-ph9909093 (all `\documentstyle[…paspconf…]
  // {article}`).
  RequirePackage!("aas_support");
  RequirePackage!("epsf");
});
