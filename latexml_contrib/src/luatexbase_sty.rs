use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // luatexbase.sty — LuaTeX allocation/compat layer (catcode tables,
  // attributes, lua callbacks). Was a Warn-stub; TL manuals loading it via
  // themes/emoji packages then hit `undefined:\initcatcodetable` etc.
  // (sweep-16 tail: 8 bundles). The allocation surface is engine
  // bookkeeping with no XML content — absorb faithfully-shaped.
  //
  // The catcode-table / attribute / lua-function allocators are the lualatex
  // FORMAT surface (latex.ltx:896-1058), provided by the `luatex` profile
  // (latexml_sty/mod.rs). luatexbase.sty:20/293 adds only its own wrappers.
  def_macro_noop("\\RequireLuaModule []{}")?;
  RawTeX!(r"\let\luatexbase@ensure@primitive\@gobble");
  // Under the pdfTeX-model default (no `luatex` option) nothing above
  // exists; absorb the allocator surface so a document that loads
  // luatexbase anyway keeps going.
  RawTeX!(r"\ifx\newattribute\@undefined
    \def\newcatcodetable#1{}\def\initcatcodetable#1{}\def\savecatcodetable#1{}\def\catcodetable#1{}
    \def\newattribute#1{}\def\setattribute#1#2{}\def\unsetattribute#1{}
    \def\newwhatsit#1{}\def\newluabytecode#1{}\def\newluachunkname#1{}\def\newluafunction#1{}
    \def\newluacmd#1{\def#1{}}\def\newprotectedluacmd#1{\def#1{}}
  \fi");
  // Callback management (lua-side) — absorb.
  def_macro_noop("\\luatexbase@directlua{}")?;
});
