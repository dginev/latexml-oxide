use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl scalefnt.sty.ltxml L20-22:
  //   DefPrimitive('\scalefont{}', sub { MergeFont(scale => ToString($scale)); });
  // Takes a float scale factor and multiplies the current font size. The
  // Float parameter type delivers an f64 directly; fontmap! handles the
  // Option wrap. Previous stub was a no-op DefMacro — user-visible size
  // changes (e.g. `\scalefont{0.8}`) silently dropped.
  // Perl: `\scalefont{}` — read ONE mandatory braced argument.
  // Use `{Float}` so the binding strips the surrounding `{...}` and
  // parses the contents as a float. Bare `Float` consumed only the
  // unbraced numeric prefix and left the literal `{` on the stream,
  // which caused mis-tracked brace depth → runaway gullet pushback
  // (4 GB Vec growth) on subsequent body content. Witness:
  // arXiv:1511.09288 — `\scalefont{0.9}{\hspace{0.2mm}…}` OOMed in
  // a few ms during canvas stage_56 (second-500K).
  DefPrimitive!("\\scalefont {Float}", sub[(scale)] {
    merge_font(fontmap!(scale => scale.value_f64()));
    Ok(Vec::new())
  });
});
