use crate::prelude::*;

// isorot.sty — the predecessor of rotating.sty with the same environment and
// command names (`sideways`, `turn`, `rotate`, `sidewaysfigure(*)`,
// `sidewaystable(*)`, `rotcaption`, `landscape`), so the binding is rotating's
// plus isorot's own surface. Bound rather than raw because isorot's raw
// `\@xrotfloat` (isorot.sty:139-147, 223-226) builds the float as an `lrbox`
// + `minipage` capture, inside which `\caption`'s float-up (`^^`,
// `float_to_element`) finds no float and errors "`<ltx:caption>` isn't
// allowed in `<ltx:block>`" (isorot/rotman; Perl identical, pdflatex clean) —
// the same reason rotating.sty.ltxml binds the float environments. Guard:
// `perfect_kernel_batch54::isorot_sideways_float_holds_its_caption`.
#[rustfmt::skip]
LoadDefinitions!({
  // isorot.sty:21-22 tracing options; the rest pass to graphics (:23).
  DeclareOption!("errorshow", None);
  DeclareOption!("debugshow", None);
  DeclareOption!("figuresleft", None);
  DeclareOption!("figuresright", None);
  DeclareOption!("clockwise", None);
  DeclareOption!("counterclockwise", None);
  ProcessOptions!();
  RequirePackage!("rotating");
  RequirePackage!("lscape");
  // isorot.sty:35 `\rotdriver{name}` inputs a dvi→PS driver `.def`; :53/:56
  // `\clockwise`/`\counterclockwise` and :45/:49 `\figuresleft`/
  // `\figuresright` set the rotation sense (rotating's options); :81
  // `\rotatedirection{}`; :218 `\rotcapfont` the rotated-caption font;
  // :257-258 `\controtcaption` = a continued rotated caption.
  def_macro_noop("\\rotdriver{}")?;
  def_macro_noop("\\clockwise")?;
  def_macro_noop("\\counterclockwise")?;
  def_macro_noop("\\figuresleft")?;
  def_macro_noop("\\figuresright")?;
  def_macro_noop("\\rotatedirection{}")?;
  def_macro_noop("\\rotcapfont")?;
  Let!("\\controtcaption", "\\rotcaption");
});
