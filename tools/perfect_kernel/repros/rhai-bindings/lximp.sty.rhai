// The binding for imperative_constructor_drops_space_red.tex: \lxa a template constructor, \lxb an imperative one.
DefConstructor("\\lxa{}", "<ltx:text class='a'>#1</ltx:text>");
DefConstructor("\\lxb{}", |document, x| {
  document.openElement("ltx:text", #{class: "b"}); document.absorb(x); document.closeElement("ltx:text"); });
