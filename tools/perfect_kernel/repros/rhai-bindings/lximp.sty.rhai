// The binding for constructor_opening_the_paragraph_keeps_the_space.tex: \lxa a template constructor that opens the
// paragraph (so it enters horizontal mode), \lxb an imperative one.
DefConstructor("\\lxa{}", "<ltx:text class='a'>#1</ltx:text>", #{enterHorizontal: true});
DefConstructor("\\lxb{}", |document, x| {
  document.openElement("ltx:text", #{class: "b"}); document.absorb(x); document.closeElement("ltx:text"); });
