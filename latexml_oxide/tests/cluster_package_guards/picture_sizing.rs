//! An `ltx:picture` that is auto-opened (by a drawing object outside any
//! explicit picture) takes its size from the box that opened it, as Perl's tag
//! `afterClose` does (latex_constructs.pool.ltxml:4943-4950): width, and height
//! plus depth, in px, unless the picture already has them. An unsized
//! picture is drawn by a browser as a 300×150 box, flipped. Batch 56iy; repro
//! `tools/perfect_kernel/repros/graphics-tikz/pspicture_inner_box_picture_size.tex`
//! (pst-flags-doc: 517 unsized pictures; egameps: 162).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

/// Every `<picture>` carries a width and a height; the nested one is 0×0 like
/// the `\psframe` that opened it, as Perl writes it (`width="0" height="0"`),
/// and a `\put(0,0){Hello}` in a paragraph is sized as Perl sizes it
/// (31.13×9.61 px).
#[test]
fn auto_opened_picture_is_sized_from_its_box() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pspicture_inner_box_picture_size.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let pictures: Vec<&str> = xml
    .match_indices("<picture")
    .map(|(i, _)| &xml[i..])
    .collect();
  assert_eq!(pictures.len(), 2, "{xml}");
  for picture in pictures {
    let open = &picture[..picture.find('>').unwrap_or(picture.len())];
    assert!(
      open.contains(" width=\"") && open.contains(" height=\""),
      "unsized: {open}\n{xml}"
    );
  }
  assert_element(
    &xml,
    "picture",
    &[r#"xml:id="p1.pic1.pic1""#],
    r#"<picture height="0" width="0" xml:id="p1.pic1.pic1"><rect fill="none" height="39.37" stroke="black" stroke-width="0.8" width="39.37" x="0" y="0"/></picture>"#,
  );
  let (stderr, xml) = convert(
    "\\documentclass{article}\n\\begin{document}\n\\put(0,0){Hello}\n\\end{document}\n",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let open = xml
    .find("<picture")
    .map(|i| &xml[i..i + xml[i..].find('>').unwrap_or(0)]);
  assert_eq!(
    open,
    Some(r#"<picture height="9.61" width="31.13" xml:id="p1.pic1""#),
    "{xml}"
  );
}
