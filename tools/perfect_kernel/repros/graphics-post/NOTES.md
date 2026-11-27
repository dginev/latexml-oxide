# graphics-post: a converted graphic's file against its display size

- `svg_root_display_size_picture_1605.07806.tex`, `svg_root_display_size_figure.tex` (GREEN, SVGSCALE): RUST-ONLY (Perl
  has no SVG path; it scales its raster, `Util/Image.pm` L382-389). The `<object>`'s `width`/`height` are presentational
  hints, so a stylesheet's `width:auto` (ar5iv.css) showed the SVG root's own size. Fix: `Graphics::size_svg`
  sets the root to the display size and each size gets its own file, the first the stem unless converted in place
  (OXIDIZED_DESIGN_DIVERGENCES #1034). Guard
  `latexml_post graphics::tests::a_sized_vector_render_carries_its_display_size`.
- `raster_render_keeps_its_density_size_red.tex` (RED, post-merge): the PNG path renders at the 120-dpi density and does
  not resample to the size (Perl does), so ar5iv.css shows the PNG at the density's size. Fix site: the raster branch of
  `Graphics::process`'s worker (a resample after `transform_raster_inplace`), or the stylesheet.
- `copied_svg_source_keeps_its_own_size_red.tex` (RED, post-merge; files `svgsrc.pdf`, `svgsrc.svg`): where an author's
  SVG sits beside the PDF, LaTeXML prefers the SVG and copies it, so its root keeps its own size while the object
  carries the requested one. pdfTeX takes the PDF. Fix site: the copy branch (size the copied root as
  `Graphics::size_svg` does a render's). Our own `<stem>.svg` on an in-place re-run is GREEN: a candidate the
  destination's cache records as our output ranks last (`Graphics::our_output`, OD #1157 (7); guard
  `graphics::tests::a_figure_converted_in_place_again_reads_the_authors_source`).
- `inplace_rerun_sizes_by_our_raster_red.tex` (RED, post-merge; files `epsonly.eps`, `epsonly-eps-converted-to.pdf` for pdflatex; convert it twice in place): the
  TeX stage lists our own `epsonly.png` (a declared extension) before the EPS and sizes the natural box from its pixels
  (260.172pt for 216.81pt), base and head alike. The post side reads the EPS again (`Graphics::our_output`). Fix site:
  `latexml_core` `image_candidates`/`first_natural_size`, which do not see the destination's cache. An SVG render never
  leaks there: `.svg` is no declared extension, so the PDF sorts first (`in_graphics_type_order`).
