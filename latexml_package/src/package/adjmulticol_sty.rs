use crate::prelude::*;

// adjmulticol.sty — multicol with adjusted inner/outer margins. Raw, it
// reaches multicol's page-layout internals (adjmulticol.sty:151 `\mult@@cols`
// = multicol.sty:172, the `\vsplit` column balancer that LaTeXML never
// emulates — the bound `{multicols}` emits pagination markers), so the
// environments are bound the same way and the margins (#2 inner, #3 outer),
// page layout only, are dropped. adjmulticol.sty:110 `\adjmulticols{n}{in}
// {out}`, :196 `\endadjmulticols` → `\endmulticols`; adjmulticol/sample.
// Guard: `perfect_kernel_batch54::adjmulticols_are_pagination_markers`.
#[rustfmt::skip]
LoadDefinitions!({
  DeclareOption!(None, {
    Digest!("\\PassOptionsToPackage{\\CurrentOption}{multicol}")?;
  });
  ProcessOptions!();
  RequirePackage!("multicol");
  DefEnvironment!("{adjmulticols}{}{}{}",
    r###"<ltx:pagination role='start_#1_columns'/>#body<ltx:pagination role='end_#1_columns'/>"###,
    mode => "internal_vertical");
  DefEnvironment!("{adjmulticols*}{}{}{}",
    r###"<ltx:pagination role='start_#1_columns'/>#body<ltx:pagination role='end_#1_columns'/>"###,
    mode => "internal_vertical");
});
