use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: etex.sty.ltxml
  LoadPool!("eTeX");
  // etex.sty register-allocator macros (etex.sty L332-348). Real defs
  // use `\et@xglob`/`\et@xloc` to allocate from extended register
  // pools (Numbers 256+ for count/dimen/etc.). For our purposes the
  // semantic is "allocate a new register"; forward to LaTeX's
  // `\newcount`/`\newdimen`/etc. which already exist.
  //
  // Glob* variants allocate globally; loc* variants locally.
  // In LaTeXML's flat-state model these are effectively equivalent.
  //
  // Witness: arXiv:2506.16610 / .16657 / .20642 (papers via etex.sty
  // raw-load + linegoal.sty / etextools / similar). Rust 2 → 0
  // expected, beating Perl=3.
  DefMacro!("\\globcount", "\\newcount");
  DefMacro!("\\loccount", "\\newcount");
  DefMacro!("\\globdimen", "\\newdimen");
  DefMacro!("\\locdimen", "\\newdimen");
  DefMacro!("\\globskip", "\\newskip");
  DefMacro!("\\locskip", "\\newskip");
  DefMacro!("\\globmuskip", "\\newmuskip");
  DefMacro!("\\locmuskip", "\\newmuskip");
  DefMacro!("\\globbox", "\\newbox");
  DefMacro!("\\locbox", "\\newbox");
  DefMacro!("\\globtoks", "\\newtoks");
  DefMacro!("\\loctoks", "\\newtoks");
  DefMacro!("\\globmarks", "\\newmarks");
  DefMacro!("\\locmarks", "\\newmarks");
});
