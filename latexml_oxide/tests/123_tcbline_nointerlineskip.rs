//! Regression test: the height measurement of a `\tcbline`-segmented tcolorbox
//! must not over-count the paragraph BEFORE the rule.
//!
//! `\tcbline` (tcolorbox's `tcbskins` segmentation, `\tcbline@`) brackets its
//! dashed-rule picture with `{\parskip\z@\par\nointerlineskip}` on each side.
//! The `\par` there fires INSIDE a `{...}` group. Rust digests a text-mode
//! `{...}` into a localized box-list (a `List`), so the grouped `\par` repacks
//! only that empty inner list — the outer paragraph's characters ("xxx") stayed
//! LOOSE (unpacked). Perl's bare `{...}` flows flat, so its enclosing `\par`
//! packs them; Perl never sees the loose run. `compute_boxes_size` then counted
//! each loose glyph as its own vertical line, adding one `\baselineskip` per
//! extra glyph: a `xxx\tcbline yyy` box measured 111.03 where Perl is 77.82.
//! That inflated tcolorbox height and left an oversized gap after the dashed
//! rule (arXiv:2605.29955 Figure 1, the Lean code card).
//!
//! Ground truth (box viewBox height, `xxx\tcbline yyy` @ width=100pt):
//!   pdflatex 65.79 · Perl 77.82 · Rust (pre-fix) 111.03.
//! The fix (`tex_box.rs` `{` primitive, "R2"): when a text-mode `{...}` group
//! breaks a paragraph (its digested body resumed vertical mode via `\par`), the
//! `{` primitive repacks the outer paragraph's loose run at DIGESTION — so the
//! box-list matches Perl's flat-digestion structure (a packed paragraph `List`,
//! not loose chars) and `compute_boxes_size` measures the pure Perl way. Result:
//! 77.82, an exact Perl match. (Perl's own 12pt excess over pdflatex is a shared
//! limitation, not this test's target.)
//!
//! The coalescing itself is OXIDIZED_DESIGN #100. NOTE: it does NOT address box
//! content overflowing the drawn frame when the CLIENT browser substitutes a
//! wider/taller monospace than cmtt10 — that is the TeX-realm-vs-SVG font
//! impedance mismatch (shapes frozen at TeX-font metrics; only the
//! foreignObject text reflows). That is #99's known residual / WISDOM #47.
//!
//! Conditional: needs the kernel dump + tcolorbox in the host TeX tree.

use latexml::util::test::{convert_fixture, dump_available, kpse_has};

fn first_box_viewbox_height(xml: &str) -> f64 {
  // The tcolorbox root <svg:svg> is the first picture; its viewBox is
  // "minx miny W H" — take H (the 4th number). Skip the nested tcbline
  // sub-picture by taking the FIRST svg:svg (the outer box).
  let at = xml.find("<svg:svg").expect("no <svg:svg> box in output");
  let tag = &xml[at..xml[at..].find('>').expect("unterminated svg:svg") + at];
  let vb = tag.split("viewBox=\"").nth(1).expect("no viewBox");
  let vb = &vb[..vb.find('"').unwrap()];
  vb.split_whitespace()
    .nth(3)
    .and_then(|s| s.parse::<f64>().ok())
    .expect("could not parse viewBox height")
}

const TEX: &str = "literal:\\documentclass{article}\n\
  \\usepackage[most]{tcolorbox}\n\
  \\begin{document}\n\
  \\begin{tcolorbox}[width=100pt]xxx\\tcbline yyy\\end{tcolorbox}\n\
  \\end{document}\n";

#[test]
fn tcbline_box_not_over_measured() {
  if !(dump_available() && kpse_has("tcolorbox.sty")) {
    eprintln!("skipping: needs kernel dump + tcolorbox.sty in the host TeX tree");
    return;
  }
  let r = convert_fixture(TEX);
  let xml = r.result.expect("conversion produced no result");

  let h = first_box_viewbox_height(&xml);
  // pdflatex 65.79 / Perl 77.82 ground truth; the pre-fix Rust value is 111.03.
  // The paragraph before the rule must pack to one line, not one-per-glyph.
  assert!(
    h < 90.0,
    "\\tcbline box viewBox height {h} is over-measured (pdflatex 65.8 / Perl 77.8; \
     pre-fix Rust 111.0) — the paragraph before the rule was not repacked at \
     digestion, so its characters count as one vertical line each.\n{xml}",
  );
  // Sanity floor: it must still contain the rule + two lines (not collapse).
  assert!(
    h > 55.0,
    "\\tcbline box height {h} collapsed below the rule+content floor.\n{xml}",
  );
}
