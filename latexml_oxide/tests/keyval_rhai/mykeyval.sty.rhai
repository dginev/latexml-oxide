// Faithful port of Perl t/keyval/mykeyval.sty.ltxml.
RequirePackage("keyval");

DefKeyVal("foo", "path", "Semiverbatim");
DefKeyVal("foo", "height", "");
DefKeyVal("foo", "width", "");

DefConstructor("\\KVsimple OptionalKeyVals:foo", "<ltx:graphics graphic='none' options='#1'/>");
DefConstructor("\\KVcomplex OptionalKeyVals:foo",
  "<ltx:graphics graphic='none' imagewidth='&GetKeyVal(#1,width)' imageheight='&GetKeyVal(#1,height)'/>");
DefEnvironment("{KVenv} OptionalKeyVals:foo",
  "<ltx:text width='&GetKeyVal(#1,width)' height='&GetKeyVal(#1,height)'>#body</ltx:text>");

// KVstruct: imperative body. The Perl template uses a `?cond(...)` conditional
// with nested ltx:text; the prior compiled binding instead used a closure with
// inline text (two consecutive nested ltx:text children triggered a libxml
// double-free), so we mirror that closure to match the fixture's output.
DefEnvironment("{KVstruct} OptionalKeyVals:foo", |document, kv| {
  document.openElement("ltx:text");
  let w = GetKeyVal(kv, "width");
  if w != "" && w != "false" { document.absorbString("Width: " + w); }
  document.absorbString("&");
  let h = GetKeyVal(kv, "height");
  if h != "" && h != "false" { document.absorbString("Height: " + h); }
  document.absorbProperty("body");
  document.closeElement("ltx:text");
});

DefConstructor("\\KVauto RequiredKeyVals:foo", "<ltx:text>#1</ltx:text>");
