use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // luatexbase.sty — LuaTeX allocation/compat layer (catcode tables,
  // attributes, lua callbacks). Was a Warn-stub; TL manuals loading it via
  // themes/emoji packages then hit `undefined:\initcatcodetable` etc.
  // (sweep-16 tail: 8 bundles). The allocation surface is engine
  // bookkeeping with no XML content — absorb faithfully-shaped.
  //
  // Catcode tables (luatexbase manual §2.3; ltluatex.dtx).
  def_macro_noop("\\newcatcodetable DefToken")?;
  def_macro_noop("\\initcatcodetable DefToken")?;
  def_macro_noop("\\savecatcodetable DefToken")?;
  def_macro_noop("\\setcatcodetable DefToken")?;
  def_macro_noop("\\catcodetable DefToken")?;
  // Attribute / whatsit / bytecode allocators.
  def_macro_noop("\\newattribute DefToken")?;
  def_macro_noop("\\newwhatsit DefToken")?;
  def_macro_noop("\\newluatexregister DefToken")?;
  def_macro_noop("\\newluabytecode DefToken")?;
  def_macro_noop("\\newluachunkname DefToken")?;
  def_macro_noop("\\newluafunction DefToken")?;
  // \newluacmd / \newprotectedluacmd {\cs}{lua body}: define \cs to run lua.
  // Under the pdfTeX-model default the lua body is inert — define the CS as
  // a noop so later calls are absorbed; under the luatex profile \directlua
  // exists and a full run-through would go via the bridge (out of scope
  // here — these allocator-defined commands are formatting hooks in the
  // witnesses: asmeconf, biblatex-gost, greek-fontenc).
  DefPrimitive!("\\newluacmd DefToken {}", sub[(cs, _body)] {
    let _ = def_macro(cs, None, ExpansionBody::Tokens(Tokens!()), None);
  });
  DefPrimitive!("\\newprotectedluacmd DefToken {}", sub[(cs, _body)] {
    let _ = def_macro(cs, None, ExpansionBody::Tokens(Tokens!()), None);
  });
  // Callback management (lua-side) — absorb.
  def_macro_noop("\\luatexbase@directlua{}")?;
});
