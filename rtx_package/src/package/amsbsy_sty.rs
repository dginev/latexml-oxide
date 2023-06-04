//**********************************************************************
// See amsldoc
//**********************************************************************
use crate::package::*;
LoadDefinitions!(state, {

  RequirePackage!("amsgen");

  DefConstructor!("\\boldsymbol{}", "#1", bounded => true, require_math => true, font => { forcebold => true });

  // I think the intent is that you use \pmb to get bold, but your current font doesn't supply
  // a bold for those glyphs.  Since we're just forcing the glyph to be bold,
  // \pmb is probably \boldsymbol. or... ??
  Let!("\\pmb", "\\boldsymbol");

});