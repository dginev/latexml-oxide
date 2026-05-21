use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "l3draw.sty",
    "l3draw.sty is not fully supported — drawings render as empty groups."
  );
  // l3draw uses LaTeX3 \draw_begin: / \draw_end: bracketed drawing groups
  // populated by \draw_path_*:n functions. We can't render the vector
  // drawing without an actual PDF/SVG backend, but we can stub the
  // primitives as no-ops so papers that wrap an icon in `\draw_begin:
  // ... \draw_end:` don't fail with undefined-CS cascades. Effect:
  // drawing is silently dropped; surrounding text/math continues
  // cleanly. Witness: arXiv-2503.08256v1 (popets paper with icon
  // drawings inside section labels).
  //
  // Covers the surface area observed in recent corpus papers. Add more
  // entries here if new \draw_*:... functions surface as undefined.
  def_macro_noop("\\draw_begin:")?;
  def_macro_noop("\\draw_end:")?;
  def_macro_noop("\\draw_baseline:n{}")?;
  def_macro_noop("\\draw_path_moveto:n{}")?;
  def_macro_noop("\\draw_path_lineto:n{}")?;
  def_macro_noop("\\draw_path_arc:nnn{}{}{}")?;
  def_macro_noop("\\draw_path_close:")?;
  def_macro_noop("\\draw_path_circle:nn{}{}")?;
  def_macro_noop("\\draw_path_use_clear:n{}")?;
  def_macro_noop("\\draw_path_use:n{}")?;
  def_macro_noop("\\draw_path_rectangle:nn{}{}")?;
  def_macro_noop("\\draw_path_ellipse:nnn{}{}{}")?;
  def_macro_noop("\\draw_path_grid:nnnn{}{}{}{}")?;
  def_macro_noop("\\draw_linewidth:n{}")?;
  def_macro_noop("\\draw_dash_pattern:nn{}{}")?;
  def_macro_noop("\\draw_color:n{}")?;
  def_macro_noop("\\draw_color_fill:n{}")?;
  def_macro_noop("\\draw_color_stroke:n{}")?;
  def_macro_noop("\\draw_transform_shift:n{}")?;
  def_macro_noop("\\draw_transform_scale:n{}")?;
  def_macro_noop("\\draw_transform_rotate:n{}")?;
  def_macro_noop("\\draw_transform_xshift:n{}")?;
  def_macro_noop("\\draw_transform_yshift:n{}")?;
  def_macro_noop("\\draw_scope_begin:")?;
  def_macro_noop("\\draw_scope_end:")?;
});
