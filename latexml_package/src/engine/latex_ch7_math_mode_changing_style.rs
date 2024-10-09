use crate::prelude::*;
//======================================================================
// C.7.8 Changing Style
//======================================================================
// For Math style changes, we record the current font, which is then merged
// into the Whatsit's created for letters, etc.  The merging depends on
// the type of letter, greek, symbol, etc.
// Apparently, with the normal TeX setup, these fonts don't really merge,
// rather they override all of family, series and shape.
LoadDefinitions!({
  DefConstructor!("\\mathrm{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "serif", series => "medium", shape => "upright"});
  DefConstructor!("\\mathit{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {shape => "italic", family => "serif", series => "medium"});
  DefConstructor!("\\mathbf{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {series => "bold", family => "serif", shape => "upright"});
  DefConstructor!("\\mathsf{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "sansserif", series => "medium", shape => "upright"});
  DefConstructor!("\\mathtt{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "typewriter", series => "medium", shape => "upright"});
  DefConstructor!("\\mathcal{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "caligraphic", series => "medium", shape => "upright"});
  DefConstructor!("\\mathscr{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "script", series => "medium", shape => "upright"});
  DefConstructor!("\\mathnormal{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "math", shape => "italic", series => "medium"});

  DefMacro!("\\fontsubfuzz", ".4pt");
  DefMacro!("\\oldstylenums", "");

  DefPrimitive!("\\operator@font", None,
    font => {family => "serif", series => "medium", shape => "upright"});
});
