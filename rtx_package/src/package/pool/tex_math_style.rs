use crate::package::*;

LoadDefinitions!(state, {
  //======================================================================
  DefPrimitiveI!("\\cal", noprimitive!());
  // TODO:  font => { family => 'caligraphic', series => 'medium', shape => 'upright' });

  // In principle, <ltx:emph> is a nice markup for emphasized.
  // Unfortunately, TeX really just treats it as a font switch.
  // Something like:  \em et.al. \rm more stuff
  // works in TeX, but in our case, since there is no explicit {},
  // the <ltx:emph> stays open!  Ugh!
  // This could still be made to work, but merge font would
  // need to look at any open <ltx:emph>, and then somehow close it!
  DefPrimitiveI!("\\em", noprimitive!(),
  before_digest => beforeproc!(_stomach, state, {
    let font = state.lookup_font().unwrap();
    let shape = font.get_shape().unwrap_or(Cow::Borrowed(""));
    let shapevariant = if shape == "italic" { "normal" } else { "italic" };
    state.assign_value("font", font.merge(&fontmap!(shape => shapevariant)), Some(Scope::Local));
  }));

  // Change math font while still in text!
  DefPrimitiveI!("\\boldmath", noprimitive!());
  // TODO:
  // beforeDigest => sub { AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => 1),
  // 'local'); }, forbidMath => 1);
  DefPrimitiveI!("\\unboldmath", noprimitive!());
  // TODO:
  // beforeDigest => sub { AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => 0),
  // 'local'); }, forbidMath => 1);
});
