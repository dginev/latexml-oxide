use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // titleps.sty (titlesec bundle) — page-style declaration front-end.
  // Running heads/feet are purely presentational (no HTML counterpart), so
  // the page-style definition family swallows its bodies, matching the
  // scrlayer-scrpage precedent. The `\sethead`/`\setfoot` calls live INSIDE
  // those bodies (titleps.sty L330-343 `\ttl@pagestyle`), so swallowing the
  // body also disposes of them; top-level forms are noop'd for direct use.
  //
  // Witness: ufrgscca-abnt.sty L507-529 (`\renewpagestyle{plain}…` with
  // `\sethead[\ifthechapter{\sffamily\thepage}{}]…`) — perfect-kernel corpus,
  // ufrgscca manual 6 errors.
  DefMacro!("\\newpagestyle{}[]{}", "");
  DefMacro!("\\renewpagestyle{}[]{}", "");
  // titleps.sty L424-433: three bracket groups then three braced fields.
  DefMacro!("\\sethead[][][]{}{}{}", "");
  DefMacro!("\\setfoot[][][]{}{}{}", "");
  // titleps.sty L480-488.
  DefMacro!("\\widenhead OptionalMatch:* [][]{}{}", "");
  def_macro_noop("\\headrule")?;
  def_macro_noop("\\footrule")?;
  DefMacro!("\\setheadrule{}", "");
  DefMacro!("\\setfootrule{}", "");
  DefMacro!("\\makeheadrule", "");
  DefMacro!("\\makefootrule", "");
  // titleps.sty L192-208: `\ifthe<level>{then}{else}` — true when the level's
  // running mark is non-empty. Outside a real page-style context there is no
  // mark; take the else branch.
  DefMacro!("\\ifthepart{}{}", "#2");
  DefMacro!("\\ifthechapter{}{}", "#2");
  DefMacro!("\\ifthesection{}{}", "#2");
  DefMacro!("\\ifthesubsection{}{}", "#2");
  DefMacro!("\\ifthesubsubsection{}{}", "#2");
  DefMacro!("\\iftheparagraph{}{}", "#2");
  DefMacro!("\\ifthesubparagraph{}{}", "#2");
  DefMacro!("\\ifthepage{}{}", "#1");
  // \settitlemarks{level,…} — mark wiring, presentational.
  DefMacro!("\\settitlemarks OptionalMatch:* []{}", "");
  def_macro_noop("\\bottitlemarks")?;
  def_macro_noop("\\toptitlemarks")?;
  def_macro_noop("\\firsttitlemarks")?;
  def_macro_noop("\\nexttoptitlemarks")?;
  def_macro_noop("\\outertitlemarks")?;
  def_macro_noop("\\innertitlemarks")?;
});
