use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl scalefnt.sty.ltxml L20-22:
  //   DefPrimitive('\scalefont{}', sub { MergeFont(scale => ToString($scale)); });
  // Takes a float scale factor and multiplies the current font size. The
  // Float parameter type delivers an f64 directly; fontmap! handles the
  // Option wrap. Previous stub was a no-op DefMacro — user-visible size
  // changes (e.g. `\scalefont{0.8}`) silently dropped.
  DefPrimitive!("\\scalefont Float", sub[(scale)] {
    merge_font(fontmap!(scale => scale.value_f64()));
    Ok(Vec::new())
  });
});
