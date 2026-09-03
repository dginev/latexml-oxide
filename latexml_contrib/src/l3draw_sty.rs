use latexml_package::prelude::*;

// l3draw (l3experimental): a binding REPLACES the raw file, so it must carry
// the FULL public surface — the former subset (with a stale
// `\draw_linewidth:n`; the API is `\draw_set_linewidth:n`) left
// `\draw_set_linewidth:n`, `\draw_path_scope_*` and the content-bearing
// `\draw_box_use:N`/`\draw_coffin_use:Nnn` undefined (circledtext.sty:25,
// tabular2.sty:12, suanpan-l3; witness arXiv-2503.08256v1). Absorb tier:
// path/state/transform/layer functions are drawing geometry with no XML
// meaning and are no-ops; `\draw_point_*` answer a neutral point;
// `\draw_box_use:N` (l3draw.sty:40) and `\draw_coffin_use:Nnn` (:98 →
// `\coffin_typeset:Nnnnn`) carry the actual CONTENT (the circled text, the
// table cells) and typeset it. The list is generated from l3draw.sty's
// `\draw_*` names (TL2025). Guard:
// `perfect_kernel_batch54::l3draw_surface_keeps_box_content`.
LoadDefinitions!({
  // Bodies name their l3 targets via `\\csname` — a `DefMacro!` body is
  // tokenized under standard catcodes, where `_`/`:` would split the name.
  def_macro_noop("\\draw_begin:")?;
  DefMacro!("\\draw_box_use:N Token", "\\csname box_use:N\\endcsname #1");
  DefMacro!(
    "\\draw_box_use:Nn Token {}",
    "\\csname box_use:N\\endcsname #1"
  );
  DefMacro!(
    "\\draw_coffin_use:Nnn Token {} {}",
    "\\csname coffin_typeset:Nnnnn\\endcsname #1{#2}{#3}{0pt}{0pt}"
  );
  DefMacro!(
    "\\draw_coffin_use:Nnnn Token {} {} {}",
    "\\csname coffin_typeset:Nnnnn\\endcsname #1{#2}{#3}{0pt}{0pt}"
  );
  def_macro_noop("\\draw_end:")?;
  def_macro_noop("\\draw_layer_begin:n {}")?;
  def_macro_noop("\\draw_layer_end:")?;
  def_macro_noop("\\draw_layer_new:n {}")?;
  def_macro_noop("\\draw_path_arc:nnn {} {} {}")?;
  def_macro_noop("\\draw_path_arc:nnnn {} {} {} {}")?;
  def_macro_noop("\\draw_path_arc_axes:nnnn {} {} {} {}")?;
  def_macro_noop("\\draw_path_canvas_curveto:nnn {} {} {}")?;
  def_macro_noop("\\draw_path_canvas_lineto:n {}")?;
  def_macro_noop("\\draw_path_canvas_moveto:n {}")?;
  def_macro_noop("\\draw_path_circle:nn {} {}")?;
  def_macro_noop("\\draw_path_close:")?;
  def_macro_noop("\\draw_path_corner_arc:nn {} {}")?;
  def_macro_noop("\\draw_path_curveto:nn {} {}")?;
  def_macro_noop("\\draw_path_curveto:nnn {} {} {}")?;
  def_macro_noop("\\draw_path_ellipse:nnn {} {} {}")?;
  def_macro_noop("\\draw_path_grid:nnnn {} {} {} {}")?;
  def_macro_noop("\\draw_path_lineto:n {}")?;
  def_macro_noop("\\draw_path_moveto:n {}")?;
  def_macro_noop("\\draw_path_rectangle:nn {} {}")?;
  def_macro_noop("\\draw_path_rectangle_corners:nn {} {}")?;
  def_macro_noop("\\draw_path_replace_bb:")?;
  def_macro_noop("\\draw_path_scope_begin:")?;
  def_macro_noop("\\draw_path_scope_end:")?;
  def_macro_noop("\\draw_path_use:n {}")?;
  def_macro_noop("\\draw_path_use_clear:n {}")?;
  DefMacro!("\\draw_point:n {}", "0pt,0pt");
  DefMacro!(
    "\\draw_point_interpolate_arcaxes:nnnnnn {} {} {} {} {} {}",
    "0pt,0pt"
  );
  DefMacro!(
    "\\draw_point_interpolate_curve:nnnnnn {} {} {} {} {} {}",
    "0pt,0pt"
  );
  DefMacro!("\\draw_point_interpolate_distance:nnn {} {} {}", "0pt,0pt");
  DefMacro!("\\draw_point_interpolate_line:nnn {} {} {}", "0pt,0pt");
  DefMacro!(
    "\\draw_point_intersect_circles:nnnnn {} {} {} {} {}",
    "0pt,0pt"
  );
  DefMacro!(
    "\\draw_point_intersect_line_circle:nnnnn {} {} {} {} {}",
    "0pt,0pt"
  );
  DefMacro!("\\draw_point_intersect_lines:nnnn {} {} {} {}", "0pt,0pt");
  DefMacro!("\\draw_point_polar:nn {} {}", "0pt,0pt");
  DefMacro!("\\draw_point_polar:nnn {} {} {}", "0pt,0pt");
  DefMacro!("\\draw_point_transform:n {}", "0pt,0pt");
  DefMacro!("\\draw_point_unit_vector:n {}", "0pt,0pt");
  DefMacro!("\\draw_point_vec:nn {} {}", "0pt,0pt");
  DefMacro!("\\draw_point_vec:nnn {} {} {}", "0pt,0pt");
  DefMacro!("\\draw_point_vec_polar:nn {} {}", "0pt,0pt");
  DefMacro!("\\draw_point_vec_polar:nnn {} {} {}", "0pt,0pt");
  def_macro_noop("\\draw_scope_begin:")?;
  def_macro_noop("\\draw_scope_end:")?;
  def_macro_noop("\\draw_set_baseline:n {}")?;
  def_macro_noop("\\draw_set_cap_butt:")?;
  def_macro_noop("\\draw_set_cap_rectangle:")?;
  def_macro_noop("\\draw_set_cap_round:")?;
  def_macro_noop("\\draw_set_dash_pattern:nn {} {}")?;
  def_macro_noop("\\draw_set_evenodd_rule:")?;
  def_macro_noop("\\draw_set_join_bevel:")?;
  def_macro_noop("\\draw_set_join_miter:")?;
  def_macro_noop("\\draw_set_join_round:")?;
  def_macro_noop("\\draw_set_linewidth:n {}")?;
  def_macro_noop("\\draw_set_miterlimit:n {}")?;
  def_macro_noop("\\draw_set_nonzero_rule:")?;
  def_macro_noop("\\draw_suspend_begin:")?;
  def_macro_noop("\\draw_suspend_end:")?;
  def_macro_noop("\\draw_transform_matrix:nnnn {} {} {} {}")?;
  def_macro_noop("\\draw_transform_matrix_absolute:nnnn {} {} {} {}")?;
  def_macro_noop("\\draw_transform_matrix_invert:")?;
  def_macro_noop("\\draw_transform_matrix_reset:")?;
  def_macro_noop("\\draw_transform_rotate:n {}")?;
  def_macro_noop("\\draw_transform_scale:n {}")?;
  def_macro_noop("\\draw_transform_shift:n {}")?;
  def_macro_noop("\\draw_transform_shift_absolute:n {}")?;
  def_macro_noop("\\draw_transform_shift_invert:")?;
  def_macro_noop("\\draw_transform_shift_reset:")?;
  def_macro_noop("\\draw_transform_triangle:nnn {} {} {}")?;
  def_macro_noop("\\draw_transform_xscale:n {}")?;
  def_macro_noop("\\draw_transform_xshift:n {}")?;
  def_macro_noop("\\draw_transform_xslant:n {}")?;
  def_macro_noop("\\draw_transform_yscale:n {}")?;
  def_macro_noop("\\draw_transform_yshift:n {}")?;
  def_macro_noop("\\draw_transform_yslant:n {}")?;
  def_macro_noop("\\draw_xvec:n {}")?;
  def_macro_noop("\\draw_yvec:n {}")?;
  def_macro_noop("\\draw_zvec:n {}")?;
});
