// A rewrite whose replacement reads a register whose getter calls Fatal() (the repro rewrite_fatal_keeps_the_document.tex).
DefRegister("\\lxrwf", Dimension("0pt"), #{ getter: || { Fatal("unexpected", "Internal", "lxrwf gives up"); Dimension("1pt") } });
DefConstructor("\\rwv{}", "<ltx:text class=\"victim\">#1</ltx:text>");
DefRewrite(#{ select: "descendant-or-self::ltx:text[@class='victim']" }, |document, nodes| {
  document.openElement("ltx:text");
  document.absorbString("[" + ToString(LookupRegister("\\lxrwf")) + "]");
  document.closeElement("ltx:text");
});
