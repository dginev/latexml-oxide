//! Batch 56cc: picture-mode `\makebox`/`\framebox`/`\dashbox` (`\pic@makebox@`)
//! shift their content by the `[pos]` rule for EVERY size pair, including
//! `(0,0)` — Perl's `if ($size)` (latex_constructs.pool.ltxml:5054) has the
//! nonzero-value test commented out on purpose. A zero-size box is the
//! standard picture LABEL idiom, `\put(x,y){\makebox(0,0){…}}`: centred means
//! `translate(-w/2,-(h+d)/2)`; `[l]` keeps x at 0. Rust guarded on a nonzero
//! size and left every label at `translate(0,0)` (bottom-left anchored).
//! Whole `<g class="makebox">` start tags are pinned (Perl: `translate(-7.06,-5.63)`
//! / `translate(0,-5.63)`; the pt strings are this engine's serialization).

#[test]
fn zero_size_makebox_centres_its_content() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\setlength{\\unitlength}{1pt}\n\\begin{picture}(100,50)\n\\put(20,10){\\makebox(0,0){$x^2$}}\n\\end{picture}\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  let g = latexml::util::test::xml_element(&xml, "g", &["class=\"makebox\""]).unwrap_or_default();
  let start = &g[..g.find('>').map_or(g.len(), |i| i + 1)];
  assert_eq!(
    start,
    "<g class=\"makebox\" innerdepth=\"0.0pt\" innerheight=\"8.14003pt\" innerwidth=\"10.2014pt\" transform=\"translate(-7.06,-5.63)\">",
    "{xml}"
  );
}

/// Batch 56ch: `\dashbox{N}(w,h){…}` carries `stroke-dasharray="N"` on its
/// frame rect (Perl latex_constructs.pool.ltxml:5043 `stroke-dasharray='#dash'`,
/// :5076 `dash={N}`); the keyval argument is digested, so its braces are
/// gone by the time the `dash=` value is read, and a `dash={` search found
/// nothing — every dashbox rendered as a solid frame. Whole `<rect>` pinned.
#[test]
fn dashbox_frame_rect_has_its_dash_array() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\setlength{\\unitlength}{1pt}\n\\begin{picture}(60,30)\n\\put(0,0){\\dashbox{2}(30,10){x}}\n\\end{picture}\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "rect",
    &[],
    "<rect fill=\"none\" height=\"10.0pt\" stroke=\"#000000\" stroke-dasharray=\"2.0\" stroke-width=\"0.4\" width=\"30.0pt\" x=\"0\" y=\"0\"/>",
  );
}

#[test]
fn zero_size_makebox_left_position_keeps_x() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\setlength{\\unitlength}{1pt}\n\\begin{picture}(100,50)\n\\put(20,10){\\makebox(0,0)[l]{$x^2$}}\n\\end{picture}\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  let g = latexml::util::test::xml_element(&xml, "g", &["class=\"makebox\""]).unwrap_or_default();
  let start = &g[..g.find('>').map_or(g.len(), |i| i + 1)];
  assert_eq!(
    start,
    "<g class=\"makebox\" innerdepth=\"0.0pt\" innerheight=\"8.14003pt\" innerwidth=\"10.2014pt\" transform=\"translate(0,-5.63)\">",
    "{xml}"
  );
}

/// Batch 56ej: `\makebox[w][s]{…}` (stretch-to-fill) emits schema-valid
/// `align="justified"`, NOT Perl's schema-invalid `stretched` — the align enum
/// is `left|center|right|justified` (LaTeXML-common.rnc:216). `[s]` justifies
/// the inter-word glue to fill the width, which `text-align:justify` (the added
/// `.ltx_align_justified` CSS) renders faithfully. Surpass, OXIDIZED_DESIGN #239.
#[test]
fn makebox_stretch_is_justified() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\noindent\\makebox[3cm][s]{a b c}\\par\n\\makebox[2cm][c]{x}\\par\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  // `[s]` → justified (schema-valid), `[c]` → center; never `stretched`.
  assert!(
    xml.contains("align=\"justified\""),
    "\\makebox[w][s] must emit align=\"justified\"\n{xml}"
  );
  assert!(
    !xml.contains("stretched"),
    "no schema-invalid align=\"stretched\" may be emitted\n{xml}"
  );
}
