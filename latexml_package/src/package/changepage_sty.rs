//! changepage.sty — page-layout adjustments (mostly no-op in LaTeXML)
//!
//! Provides {adjustwidth} / {adjustwidth*} envs that change left/right margins
//! inside a block. We can't reflow the page, but we MUST keep BOUND_MODE
//! vertical so that `$$` / paragraph-level math inside the env behaves
//! normally. Without this, raw-load of changepage.sty defines the envs as
//! restricted_horizontal (Package.pm default), and any `$$...$$` inside fires
//! a chain of `Error:unexpected:_` / `_:^` because the `$` handler refuses to
//! enter display math (tex_math.rs:447, `BOUND_MODE.ends_with("vertical")`).
//! Witness: 2305.09826 (\begin{adjustwidth}{-2.25in}{0cm} $$...$$).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefEnvironment!("{adjustwidth}{}{}",  "#body", mode => "internal_vertical");
  DefEnvironment!("{adjustwidth*}{}{}", "#body", mode => "internal_vertical");

  // Length-setting macros: no layout effect, just silence
  DefMacro!("\\changetext{}{}{}{}{}", None);
  DefMacro!("\\changepage{}{}{}{}{}{}{}{}{}", None);
});
