//! Surpass #230: a graphic named WITH its extension that the search paths
//! do not hold is asked of kpsewhich as-is. Perl (Util/Image.pm:49-53) asks
//! kpsewhich only for extensionless names with `.png`/`.pdf` appended, so a
//! package-shipped asset referenced by its full name (`openmoji-color-all.pdf`
//! and the other icon galleries: ~27,700 corpus figures) never resolved.
//! The whole `<graphics>` element is pinned with its texmf candidate path
//! normalized (it is host-relative).

#[test]
fn extensioned_texmf_graphic_is_a_candidate() {
  if !latexml::util::test::kpse_has("example-image-a.pdf") {
    return; // no such asset on this TeX Live
  }
  let tex = "\\documentclass{article}\n\\usepackage{graphicx}\n\\begin{document}\n\\includegraphics{example-image-a.pdf}\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  let g = latexml::util::test::xml_element(&xml, "graphics", &[]).unwrap_or_default();
  let re = regex::Regex::new(r#"candidates="[^"]*/example-image-a\.pdf""#).unwrap();
  assert!(
    re.is_match(&g),
    "no texmf candidate on the graphic: {g}\n{xml}"
  );
  let normalized = re.replace(&g, "candidates=\"<texmf>/example-image-a.pdf\"");
  assert_eq!(
    normalized,
    "<graphics candidates=\"<texmf>/example-image-a.pdf\" cssstyle=\"width:32.120em; height:24.090em\" graphic=\"example-image-a.pdf\" xml:id=\"p1.g1\"/>",
    "{xml}"
  );
}
