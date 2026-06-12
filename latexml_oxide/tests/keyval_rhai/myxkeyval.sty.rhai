// Faithful port of Perl t/keyval/myxkeyval.sty.ltxml.
RequirePackage("xkeyval");

DefKeyVal("scenario", "role", "", "", #{ prefix: "myxkeyval" });
DefKeyVal("scenario", "country", "", "", #{ prefix: "myxkeyval", kind: "command" });
DefKeyVal("scenario", "color", "", "", #{ prefix: "myxkeyval", kind: "choice", choices: ["red", "yellow", "green"] });
DefKeyVal("scenario", "cross", "", "", #{ prefix: "myxkeyval", kind: "boolean" });

// \scenario: imperative body (mirrors the prior compiled binding). Note the
// `role` cell intentionally shows the `cross` value, as in the Perl original.
DefConstructor("\\scenario RequiredKeyVals:myxkeyval|scenario", |document, kv| {
  document.openElement("ltx:para"); document.setAttribute("class", "scenario");
  document.openElement("ltx:text"); document.setAttribute("class", "role");
  document.absorbString(GetKeyVal(kv, "cross"));
  document.closeElement("ltx:text");
  document.openElement("ltx:text"); document.setAttribute("class", "country");
  document.absorbString(GetKeyVal(kv, "country"));
  document.closeElement("ltx:text");
  document.openElement("ltx:text"); document.setAttribute("class", "color");
  document.absorbString(GetKeyVal(kv, "color"));
  document.closeElement("ltx:text");
  document.openElement("ltx:text"); document.setAttribute("class", "cross");
  document.absorbString(GetKeyVal(kv, "cross"));
  document.closeElement("ltx:text");
  document.closeElement("ltx:para");
});
