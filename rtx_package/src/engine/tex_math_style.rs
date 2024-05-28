use crate::prelude::*;

LoadDefinitions!({
  // TODO:
  // #======================================================================
  // # \choose & friends, also need VERY special argument handling

  // # After digesting the \choose (or whatever), grab the previous and following material
  // # and store as args in the whatsit.

  // # Increment the mathstyle stored in any boxes & whatsits.
  // # The tricky part is to know when NOT to increment!
  // # \displaystyle, constructors that set their own specific style,...
  // # And, any collateral adjustments that had been done in digestion depending on mathstyle
  // # WONT be adjusted!
  // # We don't have a clear API to find the displayable Boxes within;
  // # and we don't have a good handle on grouping...


  // DefMacro('\choose',
  //   '\lx@generalized@over{\choose}{meaning=binomial,thickness=0pt,left=\@left(,right=\@right)}');
  // DefMacro('\brace',
  //   '\lx@generalized@over{\brace}{thickness=0pt,left=\@left\{,right=\@right\}}');
  // DefMacro('\brack',
  //   '\lx@generalized@over{\brack}{thickness=0pt,left=\@left[,right=\@right]}');

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
  //  beforeDigest => sub { AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => 1),
  // 'local'); }, forbidMath => 1);
  DefPrimitive!("\\unboldmath", None);
  // TODO:
  // beforeDigest => sub { AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => 0),
  // 'local'); }, forbidMath => 1);
});
