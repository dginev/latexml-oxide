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
  // luatexbase.sty:288-300: the LuaTeX-0.x compatibility spellings are plain
  // `\let` aliases of the modern primitives (`:295 \let\luatexattributedef
  // \attributedef`) — categorically different from the engine-detection
  // probes (`\directlua`, `\luatexversion`) this project never defines.
  // luatexja's ltj-base.sty:30 uses `\luatexattributedef` (13 CJK docs
  // under the luatex profile once the l3sys identity took the luatex branch).
  RawTeX!(r"\ifx\attributedef\@undefined\else
    \let\luatexattributedef\attributedef \let\luatexattribute\attribute
    \let\luatexcatcodetable\catcodetable \let\luatexluaescapestring\luaescapestring
    \let\luatexlatelua\latelua \let\luatexoutputbox\outputbox
    \let\luatexscantextokens\scantextokens
    \let\newluatexattribute\newattribute \let\setluatexattribute\setattribute
    \let\unsetluatexattribute\unsetattribute \let\newluatexcatcodetable\newcatcodetable
    \let\setluatexcatcodetable\catcodetable
  \fi");
  // Callback management (lua-side) — absorb.
  def_macro_noop("\\luatexbase@directlua{}")?;
  // Catcode range helpers (luatexbase.sty:51 \let\SetCatcodeRange\@setrangecatcode):
  // the kernel-level `\@setrangecatcode` loop (latex_constructs_rust_only.rs) serves.
  Let!("\\SetCatcodeRange", "\\@setrangecatcode");
  Let!("\\setcatcoderange", "\\@setrangecatcode");
});
