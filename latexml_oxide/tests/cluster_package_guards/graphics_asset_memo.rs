//! Batch 56ci: the on-disk image size reads are memoized per path for a
//! conversion (pdfTeX reads an image file's box once and shares the
//! XObject across every `\includegraphics` of it). hwemoji's manual
//! includes its 6.7 MB, 3,677-page `hwemoji-assets.pdf` ~7,800 times
//! (`\hwemoji@insert`, hwemoji.sty:11) — three re-reads per inclusion timed
//! out at 420 s once divergence #230 let the texmf asset resolve. The guard
//! runs 3,600 inclusions through the binary under the helper's 110 s
//! `--timeout`: the pre-fix binary re-read the asset ~42 ms per inclusion
//! (~150 s, a timeout Fatal), the memoized one finishes in ~20 s; every
//! graphic resolves.

#[test]
fn repeated_inclusions_of_one_asset_read_it_once() {
  if !latexml::util::test::kpse_has("hwemoji-assets.pdf") {
    return; // no such asset on this TeX Live
  }
  let body: String = (1..=3600)
    .map(|k| {
      format!(
        "\\includegraphics[page={},height=1em]{{hwemoji-assets.pdf}}",
        k % 3677 + 1
      )
    })
    .collect::<Vec<_>>()
    .join("");
  let tex = format!(
    "\\documentclass{{article}}\n\\usepackage{{graphicx}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"
  );
  let (stderr, xml) = super::perfect_kernel_batch46::convert_args(&tex, &[]);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert_eq!(xml.matches("<graphics ").count(), 3600, "{xml}");
  let first = latexml::util::test::xml_element(&xml, "graphics", &[]).unwrap_or_default();
  let re = regex::Regex::new(r#"candidates="[^"]*/hwemoji-assets\.pdf""#).unwrap();
  assert!(
    re.is_match(&first),
    "asset not resolved as a candidate: {first}"
  );
}
