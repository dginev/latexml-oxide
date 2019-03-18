use crate::package::*;

LoadDefinitions!(state, {
  //======================================================================
  DefPrimitive!("\\cal", None);
  // TODO:  font => { family => 'caligraphic', series => 'medium', shape => 'upright' });

  // In principle, <ltx:emph> is a nice markup for emphasized.
  // Unfortunately, TeX really just treats it as a font switch.
  // Something like:  \em et.al. \rm more stuff
  // works in TeX, but in our case, since there is no explicit {},
  // the <ltx:emph> stays open!  Ugh!
  // This could still be made to work, but merge font would
  // need to look at any open <ltx:emph>, and then somehow close it!
  DefPrimitive!("\\em", None,
  before_digest => {
    let font = LookupFont!().unwrap();
    let shape = font.get_shape().unwrap_or(&Cow::Borrowed(""));
    let shapevariant = if shape == "italic" { "normal" } else { "italic" };
    AssignValue!("font", font.merge(fontmap!(shape => shapevariant)), Some(Scope::Local));
  });

  // Change math font while still in text!
  DefPrimitive!("\\boldmath", None);
  // TODO:
  // beforeDigest => sub { AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => 1),
  // 'local'); }, forbidMath => 1);
  DefPrimitive!("\\unboldmath", None);
  // TODO:
  // beforeDigest => sub { AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => 0),
  // 'local'); }, forbidMath => 1);
});
